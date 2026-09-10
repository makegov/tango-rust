//! `GET /api/sled/opportunities/` and `GET /api/sled/forecasts/` — state, local
//! and education (SLED) procurement.
//!
//! **Beta.** Coverage is partial and grows one jurisdiction at a time. There is
//! no national SLED feed: every jurisdiction publishes on its own portal, and
//! Tango reads them one at a time. A thin per-state result is therefore at least
//! as likely to be a portal Tango does not read as a quiet market —
//! [`Client::get_sled_coverage`] is what resolves that ambiguity.
//!
//! This data does not join to the federal data. There is no UEI, no PIID, no
//! agency-hierarchy key and no NAICS/PSC crosswalk; the `organization(*)` expand
//! here is three strings, not the federal 7-key office payload.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_sled_opportunities`] and
/// [`Client::iterate_sled_opportunities`].
///
/// Two things behave unlike the federal endpoints.
///
/// Leaving both [`status`](Self::status) and [`active`](Self::active) unset
/// returns **open solicitations only** — the API defaults the list to
/// `status=open`, because only about a fifth of the corpus is open and portals
/// drop a closed solicitation rather than restating it. Set `status` explicitly
/// to page the whole corpus. [`Client::get_sled_opportunity`] returns a
/// solicitation whatever its status.
///
/// `status` is Tango-derived liveness, refreshed every fifteen minutes. The
/// portal's own word is served as `source_status`, is frozen at last capture,
/// and is **not** filterable — most of what it calls open already has a passed
/// deadline.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSledOpportunitiesOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use
    /// [`SHAPE_SLED_OPPORTUNITIES_MINIMAL`](crate::SHAPE_SLED_OPPORTUNITIES_MINIMAL)
    /// or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Two-letter state or territory code. Multi-value: `"TX|OK"`.
    #[builder(into)]
    pub state: Option<String>,
    /// Level of government: `"state"`, `"local"`, `"education"`, or
    /// `"unknown"` for the aggregator rows that cannot tell state from local.
    #[builder(into)]
    pub jurisdiction: Option<String>,
    /// `"open"`, `"closed"`, `"awarded"`, `"cancelled"` or `"unknown"`. With
    /// [`active`](Self::active) also unset the API returns open only;
    /// `"unknown"` (standing rosters, dateless RFIs) is hidden by that default
    /// and reachable with `"open|unknown"`.
    #[builder(into)]
    pub status: Option<String>,
    /// Sugar for federal-shaped callers: `true` is `status=open`, `false` is its
    /// complement (so it includes `unknown`). `Option<bool>` so that `false` is
    /// a real filter value rather than an absent one.
    #[builder(into)]
    pub active: Option<bool>,
    /// Substring match on the buyer's published text (min 2 characters). No code
    /// resolution behind it — state agencies have no entry in the federal
    /// organization tree.
    #[builder(into)]
    pub agency: Option<String>,
    /// The number a human would quote. Null on roughly a third of the corpus,
    /// where the portal publishes none.
    #[builder(into)]
    pub solicitation_number: Option<String>,
    /// `"rfp"`, `"ifb"`, `"rfq"`, `"rfi"`, `"itb"`, `"sole_source"`, `"grant"`
    /// or `"other"`. `"null"` (the portal states no type) is a distinct answer
    /// from `"other"` (a type the vocabulary does not recognize).
    #[builder(into)]
    pub solicitation_type: Option<String>,
    /// Whether the solicitation advertises at least one document.
    #[builder(into)]
    pub has_documents: Option<bool>,
    /// Kind of the most recent substantive revision: `"deadline_change"`,
    /// `"status_change"`, `"documents_added"`, `"documents_removed"`,
    /// `"documents_replaced"`, `"title_change"` or `"content_change"`.
    #[builder(into)]
    pub revision_kind: Option<String>,

    /// Exact match within the `naics` category scheme only. Thin on purpose:
    /// scheme tagging is mid-migration, so only a small share of entries are
    /// tagged NAICS. Prefer [`category_code`](Self::category_code) unless you
    /// need scheme precision.
    #[builder(into)]
    pub naics: Option<String>,
    /// Exact match within the `nigp` scheme, the most widely tagged of the four.
    #[builder(into)]
    pub nigp: Option<String>,
    /// Exact match within the `unspsc` scheme.
    #[builder(into)]
    pub unspsc: Option<String>,
    /// Exact match within the `text` scheme, where the code is the portal's own
    /// human label.
    #[builder(into)]
    pub category: Option<String>,
    /// Match a code under ANY scheme, including the untagged pre-migration
    /// strings. The escape hatch when a scheme-specific filter returns less than
    /// you expected.
    #[builder(into)]
    pub category_code: Option<String>,

    /// Lower bound on `posted_date` (ISO `YYYY-MM-DD`, inclusive).
    #[builder(into)]
    pub posted_after: Option<String>,
    /// Upper bound on `posted_date` (inclusive).
    #[builder(into)]
    pub posted_before: Option<String>,
    /// Lower bound on `response_deadline` (inclusive).
    #[builder(into)]
    pub response_deadline_after: Option<String>,
    /// Upper bound on `response_deadline` (inclusive).
    #[builder(into)]
    pub response_deadline_before: Option<String>,
    /// Lower bound on when Tango FIRST OBSERVED the solicitation. The polling
    /// primitive: everything new since your last call.
    #[builder(into)]
    pub first_seen_after: Option<String>,
    /// Upper bound on first observation (inclusive).
    #[builder(into)]
    pub first_seen_before: Option<String>,
    /// Lower bound on when Tango OBSERVED the last substantive change. A scrape
    /// date, not an amendment date: a state portal restates one page in place
    /// and publishes no amendment date, so the resolution is that state's crawl
    /// cadence. The API exposes no upper bound on this field.
    #[builder(into)]
    pub change_seen_after: Option<String>,
    /// Lower bound on when the Tango row last changed (inclusive).
    #[builder(into)]
    pub modified_after: Option<String>,
    /// Upper bound on when the Tango row last changed (inclusive).
    #[builder(into)]
    pub modified_before: Option<String>,

    /// Support filter identifying the source portal's platform family. In no
    /// response shape, and not a stable value.
    #[builder(into)]
    pub platform: Option<String>,
    /// Support filter — the portal's own identifier.
    #[builder(into)]
    pub native_id: Option<String>,
    /// Support filter — Tango's opaque lake key. At most 500 values.
    #[builder(into)]
    pub external_id: Option<String>,

    /// Ranked full-text search over title, agency, identifiers, category labels
    /// and description, widened by the solicitations whose ATTACHMENT text
    /// matched (min 2 characters). A row that matched on its description gains a
    /// `snippet` field; a title-or-agency match carries none.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `rank`, `response_deadline`, `posted_date`, `first_seen_at`,
    /// `last_seen_at`, `last_change_seen_at`, `modified`. `rank` requires a
    /// non-empty [`search`](Self::search).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListSledOpportunitiesOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(
            &mut q,
            self.page,
            self.limit,
            self.cursor.as_deref(),
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
        push_opt(&mut q, "state", self.state.as_deref());
        push_opt(&mut q, "jurisdiction", self.jurisdiction.as_deref());
        push_opt(&mut q, "status", self.status.as_deref());
        push_opt_bool(&mut q, "active", self.active);
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(
            &mut q,
            "solicitation_number",
            self.solicitation_number.as_deref(),
        );
        push_opt(
            &mut q,
            "solicitation_type",
            self.solicitation_type.as_deref(),
        );
        push_opt_bool(&mut q, "has_documents", self.has_documents);
        push_opt(&mut q, "revision_kind", self.revision_kind.as_deref());
        push_opt(&mut q, "naics", self.naics.as_deref());
        push_opt(&mut q, "nigp", self.nigp.as_deref());
        push_opt(&mut q, "unspsc", self.unspsc.as_deref());
        push_opt(&mut q, "category", self.category.as_deref());
        push_opt(&mut q, "category_code", self.category_code.as_deref());
        push_opt(&mut q, "posted_after", self.posted_after.as_deref());
        push_opt(&mut q, "posted_before", self.posted_before.as_deref());
        push_opt(
            &mut q,
            "response_deadline_after",
            self.response_deadline_after.as_deref(),
        );
        push_opt(
            &mut q,
            "response_deadline_before",
            self.response_deadline_before.as_deref(),
        );
        push_opt(&mut q, "first_seen_after", self.first_seen_after.as_deref());
        push_opt(
            &mut q,
            "first_seen_before",
            self.first_seen_before.as_deref(),
        );
        push_opt(
            &mut q,
            "change_seen_after",
            self.change_seen_after.as_deref(),
        );
        push_opt(&mut q, "modified_after", self.modified_after.as_deref());
        push_opt(&mut q, "modified_before", self.modified_before.as_deref());
        push_opt(&mut q, "platform", self.platform.as_deref());
        push_opt(&mut q, "native_id", self.native_id.as_deref());
        push_opt(&mut q, "external_id", self.external_id.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::list_sled_opportunity_revisions`], the nested
/// `/revisions/` route.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSledOpportunityRevisionsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use
    /// [`SHAPE_SLED_REVISIONS_MINIMAL`](crate::SHAPE_SLED_REVISIONS_MINIMAL),
    /// which omits the plan-gated `changes` leaf.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// A revision kind, plus `"enrichment"` — which the `revisions(*)` expand
    /// excludes and this route serves. Multi-value: use `|`.
    #[builder(into)]
    pub kind: Option<String>,
    /// Whether a portal's own amendment marker moved at this emission. True on
    /// about 5% of revisions; everything else is Tango inferring the change from
    /// the diff.
    #[builder(into)]
    pub source_declared: Option<bool>,
    /// Lower bound on `observed_at` (ISO `YYYY-MM-DD`, inclusive).
    #[builder(into)]
    pub observed_after: Option<String>,
    /// Upper bound on `observed_at` (inclusive).
    #[builder(into)]
    pub observed_before: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListSledOpportunityRevisionsOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(
            &mut q,
            self.page,
            self.limit,
            self.cursor.as_deref(),
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
        push_opt(&mut q, "kind", self.kind.as_deref());
        push_opt_bool(&mut q, "source_declared", self.source_declared);
        push_opt(&mut q, "observed_after", self.observed_after.as_deref());
        push_opt(&mut q, "observed_before", self.observed_before.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::list_sled_forecasts`] and
/// [`Client::iterate_sled_forecasts`].
///
/// Forecasts carry no liveness at all — there is no deadline to have passed, so
/// there is no `status` field, no `active` field, and no open-only default.
/// Currency is the caller's call from `estimated_advertisement_date`, which is
/// the START of the published quarter rather than a posting date.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSledForecastsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use
    /// [`SHAPE_SLED_FORECASTS_MINIMAL`](crate::SHAPE_SLED_FORECASTS_MINIMAL)
    /// or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Two-letter state code. Multi-value: use `|`.
    #[builder(into)]
    pub state: Option<String>,
    /// Substring match on the buyer's published text (min 2 characters).
    #[builder(into)]
    pub agency: Option<String>,
    /// The portal's own category word, passed through verbatim.
    #[builder(into)]
    pub procurement_category: Option<String>,
    /// The portal's own method word, passed through verbatim.
    #[builder(into)]
    pub procurement_method: Option<String>,
    /// The number the state expects to award under, where it publishes one in
    /// advance.
    #[builder(into)]
    pub contract_number: Option<String>,
    /// Substring match on the incumbent vendor's name as published. NOT resolved
    /// to a Tango entity.
    #[builder(into)]
    pub incumbent_name: Option<String>,
    /// Lower bound on the estimated advertisement date (inclusive). Remember the
    /// date is a QUARTER START, not a posting date.
    #[builder(into)]
    pub advertisement_after: Option<String>,
    /// Upper bound on the estimated advertisement date (inclusive).
    #[builder(into)]
    pub advertisement_before: Option<String>,
    /// Lower bound on when Tango first observed the forecast (inclusive).
    #[builder(into)]
    pub first_seen_after: Option<String>,
    /// Upper bound on first observation (inclusive).
    #[builder(into)]
    pub first_seen_before: Option<String>,
    /// Lower bound on when the Tango row last changed (inclusive).
    #[builder(into)]
    pub modified_after: Option<String>,
    /// Upper bound on when the Tango row last changed (inclusive).
    #[builder(into)]
    pub modified_before: Option<String>,
    /// Ranked full-text search over title, agency and description (min 2
    /// characters).
    #[builder(into)]
    pub search: Option<String>,
    /// One of `rank`, `estimated_advertisement_date`, `first_seen_at`,
    /// `last_seen_at`, `modified`. `rank` requires a non-empty
    /// [`search`](Self::search).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListSledForecastsOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(
            &mut q,
            self.page,
            self.limit,
            self.cursor.as_deref(),
            self.shape.as_deref(),
            self.flat,
            self.flat_lists,
        );
        push_opt(&mut q, "state", self.state.as_deref());
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(
            &mut q,
            "procurement_category",
            self.procurement_category.as_deref(),
        );
        push_opt(
            &mut q,
            "procurement_method",
            self.procurement_method.as_deref(),
        );
        push_opt(&mut q, "contract_number", self.contract_number.as_deref());
        push_opt(&mut q, "incumbent_name", self.incumbent_name.as_deref());
        push_opt(
            &mut q,
            "advertisement_after",
            self.advertisement_after.as_deref(),
        );
        push_opt(
            &mut q,
            "advertisement_before",
            self.advertisement_before.as_deref(),
        );
        push_opt(&mut q, "first_seen_after", self.first_seen_after.as_deref());
        push_opt(
            &mut q,
            "first_seen_before",
            self.first_seen_before.as_deref(),
        );
        push_opt(&mut q, "modified_after", self.modified_after.as_deref());
        push_opt(&mut q, "modified_before", self.modified_before.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_sled_opportunity`] and
/// [`Client::get_sled_forecast`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetSledOptions {
    /// Shape selector. When empty, the server returns its default detail shape.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetSledOptions {
    pub(crate) fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        push_opt(&mut q, "shape", self.shape.as_deref());
        if self.flat {
            q.push(("flat".into(), "true".into()));
        }
        if self.flat_lists {
            q.push(("flat_lists".into(), "true".into()));
        }
        q
    }
}

impl Client {
    /// `GET /api/sled/opportunities/` — one page of state, local and education
    /// solicitations.
    ///
    /// Leaving both `status` and `active` unset returns open solicitations only.
    /// That default is the API's, and this method deliberately does not
    /// synthesize one: sending `status=open` here would make `active = false`
    /// unreachable, since it is the complement of open rather than an
    /// independent value.
    pub async fn list_sled_opportunities(
        &self,
        opts: ListSledOpportunitiesOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/sled/opportunities/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/sled/opportunities/{opportunity_id}/` — one solicitation,
    /// whatever its status. The open-only default applies to the list endpoint,
    /// not here.
    pub async fn get_sled_opportunity(
        &self,
        opportunity_id: &str,
        opts: Option<GetSledOptions>,
    ) -> Result<Record> {
        if opportunity_id.is_empty() {
            return Err(Error::Validation {
                message: "get_sled_opportunity: opportunity_id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/sled/opportunities/{}/", urlencoding(opportunity_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/sled/opportunities/{opportunity_id}/revisions/` — one
    /// solicitation's observed revision history.
    ///
    /// `observed_at` is the scrape that saw the change, not the date the agency
    /// made it: no state portal emits amendment notices, so `kind` is Tango's
    /// inference from the diff on about 95% of revisions, resolution is that
    /// state's crawl cadence, and history starts when Tango began reading the
    /// jurisdiction rather than when the solicitation was posted.
    ///
    /// Unlike the `revisions(*)` expand, this route serves `enrichment` rows —
    /// Tango's own detail fetch filling in coverage rather than an agency
    /// amendment. Pass `kind = "enrichment"` for only those. The per-field
    /// before/after (`changes`) requires a Small plan; `changed_fields` names
    /// what moved at every plan.
    pub async fn list_sled_opportunity_revisions(
        &self,
        opportunity_id: &str,
        opts: ListSledOpportunityRevisionsOptions,
    ) -> Result<Page<Record>> {
        if opportunity_id.is_empty() {
            return Err(Error::Validation {
                message: "list_sled_opportunity_revisions: opportunity_id is required".into(),
                response: None,
            });
        }
        let q = opts.to_query();
        let path = format!(
            "/api/sled/opportunities/{}/revisions/",
            urlencoding(opportunity_id)
        );
        let bytes = self.get_bytes(&path, &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/sled/opportunities/coverage/` — the per-state coverage rollup.
    ///
    /// Returns corpus totals plus one row per jurisdiction: the total, the count
    /// in each of the five statuses, the jurisdiction levels present, and when a
    /// solicitation there last changed. It answers one question — whether a thin
    /// result for a state is a thin market or a portal Tango does not read.
    ///
    /// Every state row carries all five status buckets whether or not they have
    /// rows, so a total and two buckets never invite subtraction. Takes no
    /// parameters and is neither shaped nor paginated.
    pub async fn get_sled_coverage(&self) -> Result<Record> {
        self.get_json::<Record>("/api/sled/opportunities/coverage/", &[])
            .await
    }

    /// `GET /api/sled/forecasts/` — one page of planned state procurements.
    pub async fn list_sled_forecasts(
        &self,
        opts: ListSledForecastsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/sled/forecasts/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/sled/forecasts/{forecast_id}/` — one forecast.
    pub async fn get_sled_forecast(
        &self,
        forecast_id: &str,
        opts: Option<GetSledOptions>,
    ) -> Result<Record> {
        if forecast_id.is_empty() {
            return Err(Error::Validation {
                message: "get_sled_forecast: forecast_id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/sled/forecasts/{}/", urlencoding(forecast_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every SLED solicitation matching `opts`.
    pub fn iterate_sled_opportunities(
        &self,
        opts: ListSledOpportunitiesOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_sled_opportunities(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// Stream every SLED forecast matching `opts`.
    pub fn iterate_sled_forecasts(&self, opts: ListSledForecastsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_sled_forecasts(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn list_sled_opportunities_all_filters_emit() {
        let opts = ListSledOpportunitiesOptions::builder()
            .state("TX|OK")
            .jurisdiction("local|education")
            .status("open|unknown")
            .active(true)
            .agency("Texas Commission")
            .solicitation_number("RFP-2026-001")
            .solicitation_type("rfp")
            .has_documents(true)
            .revision_kind("deadline_change")
            .naics("541620")
            .nigp("962-47")
            .unspsc("77101500")
            .category("Environmental Services")
            .category_code("541620")
            .posted_after("2026-01-01")
            .posted_before("2026-12-31")
            .response_deadline_after("2026-09-01")
            .response_deadline_before("2026-10-01")
            .first_seen_after("2026-09-01")
            .first_seen_before("2026-09-30")
            .change_seen_after("2026-09-05")
            .modified_after("2026-09-01")
            .modified_before("2026-09-30")
            .platform("custom")
            .native_id("abc-123")
            .external_id("tx:custom:abc-123")
            .search("environmental mitigation")
            .ordering("response_deadline")
            .build();
        let q = opts.to_query();
        for (key, want) in [
            ("state", "TX|OK"),
            ("jurisdiction", "local|education"),
            ("status", "open|unknown"),
            ("active", "true"),
            ("agency", "Texas Commission"),
            ("solicitation_number", "RFP-2026-001"),
            ("solicitation_type", "rfp"),
            ("has_documents", "true"),
            ("revision_kind", "deadline_change"),
            ("naics", "541620"),
            ("nigp", "962-47"),
            ("unspsc", "77101500"),
            ("category", "Environmental Services"),
            ("category_code", "541620"),
            ("posted_after", "2026-01-01"),
            ("posted_before", "2026-12-31"),
            ("response_deadline_after", "2026-09-01"),
            ("response_deadline_before", "2026-10-01"),
            ("first_seen_after", "2026-09-01"),
            ("first_seen_before", "2026-09-30"),
            ("change_seen_after", "2026-09-05"),
            ("modified_after", "2026-09-01"),
            ("modified_before", "2026-09-30"),
            ("platform", "custom"),
            ("native_id", "abc-123"),
            ("external_id", "tx:custom:abc-123"),
            ("search", "environmental mitigation"),
            ("ordering", "response_deadline"),
        ] {
            assert_eq!(get_q(&q, key).as_deref(), Some(want), "param {key}");
        }
    }

    /// The open-only default is the API's. Synthesizing `status=open` here would
    /// make `active = false` unreachable, since it is the complement of open.
    #[test]
    fn no_liveness_filter_is_sent_when_none_requested() {
        let opts = ListSledOpportunitiesOptions::builder().state("TX").build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "state").as_deref(), Some("TX"));
        assert_eq!(get_q(&q, "status"), None);
        assert_eq!(get_q(&q, "active"), None);
    }

    #[test]
    fn active_false_is_sent_not_dropped() {
        let opts = ListSledOpportunitiesOptions::builder()
            .active(false)
            .build();
        assert_eq!(get_q(&opts.to_query(), "active").as_deref(), Some("false"));
    }

    #[test]
    fn has_documents_false_is_sent_not_dropped() {
        let opts = ListSledOpportunitiesOptions::builder()
            .has_documents(false)
            .build();
        assert_eq!(
            get_q(&opts.to_query(), "has_documents").as_deref(),
            Some("false")
        );
    }

    /// `status` is Tango-derived liveness; the portal's frozen word is served but
    /// not filterable.
    #[test]
    fn source_status_is_not_a_filter() {
        let opts = ListSledOpportunitiesOptions::builder()
            .status("closed")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "status").as_deref(), Some("closed"));
        assert_eq!(get_q(&q, "source_status"), None);
    }

    #[test]
    fn list_sled_opportunities_zero_value_omitted() {
        let opts = ListSledOpportunitiesOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_sled_opportunities_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("verbose".to_string(), "true".to_string());
        let opts = ListSledOpportunitiesOptions::builder().extra(extra).build();
        assert!(opts.to_query().contains(&("verbose".into(), "true".into())));
    }

    #[test]
    fn revision_filters_emit() {
        let opts = ListSledOpportunityRevisionsOptions::builder()
            .kind("deadline_change")
            .source_declared(true)
            .observed_after("2026-09-01")
            .observed_before("2026-09-30")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "kind").as_deref(), Some("deadline_change"));
        assert_eq!(get_q(&q, "source_declared").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "observed_after").as_deref(), Some("2026-09-01"));
        assert_eq!(get_q(&q, "observed_before").as_deref(), Some("2026-09-30"));
    }

    /// The `revisions(*)` expand excludes enrichment rows; this route is how a
    /// caller reaches them.
    #[test]
    fn revision_kind_reaches_enrichment() {
        let opts = ListSledOpportunityRevisionsOptions::builder()
            .kind("enrichment")
            .build();
        assert_eq!(
            get_q(&opts.to_query(), "kind").as_deref(),
            Some("enrichment")
        );
    }

    /// `changes` needs a Small plan, so naming it in the suggested revision shape
    /// would 403 a Free caller on a field they never asked to gate.
    #[test]
    fn revisions_shape_omits_plan_gated_changes() {
        let shape = crate::SHAPE_SLED_REVISIONS_MINIMAL;
        assert!(
            !shape.split(',').any(|f| f == "changes"),
            "SHAPE_SLED_REVISIONS_MINIMAL must not name the Small-gated `changes` leaf: {shape}"
        );
        assert!(
            shape.contains("changed_fields"),
            "SHAPE_SLED_REVISIONS_MINIMAL should carry `changed_fields`, which every plan can read: {shape}"
        );
    }

    #[test]
    fn list_sled_forecasts_filters_emit() {
        let opts = ListSledForecastsOptions::builder()
            .state("MD")
            .agency("Department of Transportation")
            .procurement_category("Services")
            .procurement_method("Competitive Sealed Proposals")
            .contract_number("K-2026-001")
            .incumbent_name("Acme")
            .advertisement_after("2026-10-01")
            .advertisement_before("2027-03-31")
            .first_seen_after("2026-09-01")
            .first_seen_before("2026-09-30")
            .modified_after("2026-09-01")
            .modified_before("2026-09-30")
            .search("data center")
            .ordering("estimated_advertisement_date")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "state").as_deref(), Some("MD"));
        assert_eq!(
            get_q(&q, "procurement_method").as_deref(),
            Some("Competitive Sealed Proposals")
        );
        assert_eq!(
            get_q(&q, "advertisement_after").as_deref(),
            Some("2026-10-01")
        );
        assert_eq!(get_q(&q, "search").as_deref(), Some("data center"));
        // A forecast has no deadline, so there is no liveness filter to send.
        assert_eq!(get_q(&q, "status"), None);
        assert_eq!(get_q(&q, "active"), None);
    }

    #[test]
    fn get_sled_options_emit() {
        let opts = GetSledOptions::builder()
            .shape(crate::SHAPE_SLED_OPPORTUNITIES_COMPREHENSIVE)
            .flat(true)
            .flat_lists(true)
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_SLED_OPPORTUNITIES_COMPREHENSIVE)
        );
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists").as_deref(), Some("true"));
    }

    #[test]
    fn cursor_wins_over_page() {
        let opts = ListSledOpportunitiesOptions::builder()
            .page(2u32)
            .cursor("xyz".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("xyz"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[tokio::test]
    async fn get_sled_opportunity_validates_empty_id() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_sled_opportunity("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("opportunity_id is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_sled_opportunity_revisions_validates_empty_id() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client
            .list_sled_opportunity_revisions("", ListSledOpportunityRevisionsOptions::default())
            .await
            .unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("opportunity_id is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_sled_forecast_validates_empty_id() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_sled_forecast("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("forecast_id is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
