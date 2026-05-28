//! `/api/opportunities/`, `/api/notices/`, `/api/forecasts/`, `/api/grants/`,
//! and the opportunity-attachment semantic-search endpoint.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool, push_opt_u32, ListOptions};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Opportunities
// ---------------------------------------------------------------------------

/// Options for [`Client::list_opportunities`] and
/// [`Client::iterate_opportunities`]. Mirrors `ListOpportunitiesOptions`
/// in the Go SDK.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOpportunitiesOptions {
    // ----- Pagination + shape -----
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    // ----- Filters -----
    /// Tri-state: `Some(true)`/`Some(false)` to filter; `None` to omit.
    #[builder(into)]
    pub active: Option<bool>,
    /// Awarding agency CGAC code.
    #[builder(into)]
    pub agency: Option<String>,
    /// Lower bound for first-notice date (inclusive, ISO `YYYY-MM-DD`).
    #[builder(into)]
    pub first_notice_date_after: Option<String>,
    /// Upper bound for first-notice date (inclusive).
    #[builder(into)]
    pub first_notice_date_before: Option<String>,
    /// Lower bound for last-notice date (inclusive).
    #[builder(into)]
    pub last_notice_date_after: Option<String>,
    /// Upper bound for last-notice date (inclusive).
    #[builder(into)]
    pub last_notice_date_before: Option<String>,
    /// NAICS code filter.
    #[builder(into)]
    pub naics: Option<String>,
    /// Notice-type filter (e.g. `"PRESOL"`).
    #[builder(into)]
    pub notice_type: Option<String>,
    /// Server-side sort spec (prefix with `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,
    /// Place of performance filter (state code, etc.).
    #[builder(into)]
    pub place_of_performance: Option<String>,
    /// PSC code filter.
    #[builder(into)]
    pub psc: Option<String>,
    /// Lower bound for response deadline.
    #[builder(into)]
    pub response_deadline_after: Option<String>,
    /// Upper bound for response deadline.
    #[builder(into)]
    pub response_deadline_before: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Set-aside filter.
    #[builder(into)]
    pub set_aside: Option<String>,
    /// Solicitation number filter.
    #[builder(into)]
    pub solicitation_number: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListOpportunitiesOptions {
    fn to_query(&self) -> Vec<(String, String)> {
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
        push_opt_bool(&mut q, "active", self.active);
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(
            &mut q,
            "first_notice_date_after",
            self.first_notice_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "first_notice_date_before",
            self.first_notice_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "last_notice_date_after",
            self.last_notice_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "last_notice_date_before",
            self.last_notice_date_before.as_deref(),
        );
        push_opt(&mut q, "naics", self.naics.as_deref());
        push_opt(&mut q, "notice_type", self.notice_type.as_deref());
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        push_opt(
            &mut q,
            "place_of_performance",
            self.place_of_performance.as_deref(),
        );
        push_opt(&mut q, "psc", self.psc.as_deref());
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
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "set_aside", self.set_aside.as_deref());
        push_opt(
            &mut q,
            "solicitation_number",
            self.solicitation_number.as_deref(),
        );
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// ---------------------------------------------------------------------------
// Notices
// ---------------------------------------------------------------------------

/// Options for [`Client::list_notices`] and [`Client::iterate_notices`].
/// Mirrors `ListNoticesOptions` in the Go SDK. The server rejects every
/// `?ordering=` value on this endpoint, so the field is intentionally
/// omitted.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListNoticesOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size.
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Tri-state active/inactive filter.
    #[builder(into)]
    pub active: Option<bool>,
    /// Awarding agency CGAC code.
    #[builder(into)]
    pub agency: Option<String>,
    /// NAICS code filter.
    #[builder(into)]
    pub naics: Option<String>,
    /// Notice-type filter.
    #[builder(into)]
    pub notice_type: Option<String>,
    /// Lower bound for posted date (inclusive).
    #[builder(into)]
    pub posted_date_after: Option<String>,
    /// Upper bound for posted date (inclusive).
    #[builder(into)]
    pub posted_date_before: Option<String>,
    /// PSC code filter.
    #[builder(into)]
    pub psc: Option<String>,
    /// Lower bound for response deadline.
    #[builder(into)]
    pub response_deadline_after: Option<String>,
    /// Upper bound for response deadline.
    #[builder(into)]
    pub response_deadline_before: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Set-aside filter.
    #[builder(into)]
    pub set_aside: Option<String>,
    /// Solicitation number filter.
    #[builder(into)]
    pub solicitation_number: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListNoticesOptions {
    fn to_query(&self) -> Vec<(String, String)> {
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
        push_opt_bool(&mut q, "active", self.active);
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "naics", self.naics.as_deref());
        push_opt(&mut q, "notice_type", self.notice_type.as_deref());
        push_opt(
            &mut q,
            "posted_date_after",
            self.posted_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "posted_date_before",
            self.posted_date_before.as_deref(),
        );
        push_opt(&mut q, "psc", self.psc.as_deref());
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
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "set_aside", self.set_aside.as_deref());
        push_opt(
            &mut q,
            "solicitation_number",
            self.solicitation_number.as_deref(),
        );
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// ---------------------------------------------------------------------------
// Forecasts
// ---------------------------------------------------------------------------

/// Options for [`Client::list_forecasts`] and [`Client::iterate_forecasts`].
/// Mirrors `ListForecastsOptions` in the Go SDK.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListForecastsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size.
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Awarding agency CGAC code.
    #[builder(into)]
    pub agency: Option<String>,
    /// Lower bound for award date.
    #[builder(into)]
    pub award_date_after: Option<String>,
    /// Upper bound for award date.
    #[builder(into)]
    pub award_date_before: Option<String>,
    /// `fiscal_year` filter.
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,
    /// Lower bound for last-modified timestamp.
    #[builder(into)]
    pub modified_after: Option<String>,
    /// Upper bound for last-modified timestamp.
    #[builder(into)]
    pub modified_before: Option<String>,
    /// NAICS code filter (exact).
    #[builder(into)]
    pub naics_code: Option<String>,
    /// NAICS prefix filter.
    #[builder(into)]
    pub naics_starts_with: Option<String>,
    /// Server-side sort spec.
    #[builder(into)]
    pub ordering: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Source-system filter (e.g. `"SAM"`).
    #[builder(into)]
    pub source_system: Option<String>,
    /// Status filter (e.g. `"active"`).
    #[builder(into)]
    pub status: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListForecastsOptions {
    fn to_query(&self) -> Vec<(String, String)> {
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
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "award_date_after", self.award_date_after.as_deref());
        push_opt(
            &mut q,
            "award_date_before",
            self.award_date_before.as_deref(),
        );
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year_gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year_lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "modified_after", self.modified_after.as_deref());
        push_opt(&mut q, "modified_before", self.modified_before.as_deref());
        push_opt(&mut q, "naics_code", self.naics_code.as_deref());
        push_opt(
            &mut q,
            "naics_starts_with",
            self.naics_starts_with.as_deref(),
        );
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "source_system", self.source_system.as_deref());
        push_opt(&mut q, "status", self.status.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// ---------------------------------------------------------------------------
// Grants
// ---------------------------------------------------------------------------

/// Options for [`Client::list_grants`] and [`Client::iterate_grants`].
/// Mirrors `ListGrantsOptions` in the Go SDK.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListGrantsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size.
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Awarding agency filter.
    #[builder(into)]
    pub agency: Option<String>,
    /// Applicant-types filter (CSV of grants.gov codes).
    #[builder(into)]
    pub applicant_types: Option<String>,
    /// CFDA number filter.
    #[builder(into)]
    pub cfda_number: Option<String>,
    /// Grant identifier filter (exact match on `grant_id`).
    #[builder(into)]
    pub grant_id: Option<String>,
    /// Funding-categories filter (CSV).
    #[builder(into)]
    pub funding_categories: Option<String>,
    /// Funding-instruments filter (CSV).
    #[builder(into)]
    pub funding_instruments: Option<String>,
    /// Opportunity number filter.
    #[builder(into)]
    pub opportunity_number: Option<String>,
    /// Server-side sort spec.
    #[builder(into)]
    pub ordering: Option<String>,
    /// Lower bound for posted date.
    #[builder(into)]
    pub posted_date_after: Option<String>,
    /// Upper bound for posted date.
    #[builder(into)]
    pub posted_date_before: Option<String>,
    /// Lower bound for response date.
    #[builder(into)]
    pub response_date_after: Option<String>,
    /// Upper bound for response date.
    #[builder(into)]
    pub response_date_before: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Status filter (e.g. `"posted"`).
    #[builder(into)]
    pub status: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListGrantsOptions {
    fn to_query(&self) -> Vec<(String, String)> {
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
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "applicant_types", self.applicant_types.as_deref());
        push_opt(&mut q, "cfda_number", self.cfda_number.as_deref());
        push_opt(&mut q, "grant_id", self.grant_id.as_deref());
        push_opt(
            &mut q,
            "funding_categories",
            self.funding_categories.as_deref(),
        );
        push_opt(
            &mut q,
            "funding_instruments",
            self.funding_instruments.as_deref(),
        );
        push_opt(
            &mut q,
            "opportunity_number",
            self.opportunity_number.as_deref(),
        );
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        push_opt(
            &mut q,
            "posted_date_after",
            self.posted_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "posted_date_before",
            self.posted_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "response_date_after",
            self.response_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "response_date_before",
            self.response_date_before.as_deref(),
        );
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "status", self.status.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

// ---------------------------------------------------------------------------
// Opportunity attachment search
// ---------------------------------------------------------------------------

/// Options for [`Client::search_opportunity_attachments`] — semantic search
/// over the extracted text of opportunity attachments (SOWs, PWSs, J&As).
///
/// `q` is required; an empty `q` causes the call to return
/// [`Error::Validation`] before any network request.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct SearchOpportunityAttachmentsOptions {
    /// Natural-language query. Required.
    #[builder(into)]
    pub q: Option<String>,
    /// Maximum number of matches to return. `None` / `0` means
    /// "use the server default".
    #[builder(into)]
    pub top_k: Option<u32>,
    /// When true, returns the matched attachment text alongside metadata.
    /// Defaults to false to keep responses small.
    #[builder(default)]
    pub include_extracted_text: bool,
}

impl SearchOpportunityAttachmentsOptions {
    fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        push_opt(&mut q, "q", self.q.as_deref());
        push_opt_u32(&mut q, "top_k", self.top_k);
        if self.include_extracted_text {
            q.push(("include_extracted_text".into(), "true".into()));
        }
        q
    }
}

// ---------------------------------------------------------------------------
// Client methods
// ---------------------------------------------------------------------------

impl Client {
    /// `GET /api/opportunities/` — one page of opportunity records.
    pub async fn list_opportunities(&self, opts: ListOpportunitiesOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/opportunities/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every opportunity matching `opts`.
    pub fn iterate_opportunities(&self, opts: ListOpportunitiesOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_opportunities(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/notices/` — one page of notice records.
    pub async fn list_notices(&self, opts: ListNoticesOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/notices/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every notice matching `opts`.
    pub fn iterate_notices(&self, opts: ListNoticesOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_notices(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/forecasts/` — one page of forecast records.
    pub async fn list_forecasts(&self, opts: ListForecastsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/forecasts/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every forecast matching `opts`.
    pub fn iterate_forecasts(&self, opts: ListForecastsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_forecasts(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/grants/` — one page of grant records.
    pub async fn list_grants(&self, opts: ListGrantsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/grants/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every grant matching `opts`.
    pub fn iterate_grants(&self, opts: ListGrantsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_grants(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/opportunities/{opportunity_id}/` — a single opportunity.
    pub async fn get_opportunity(
        &self,
        opportunity_id: &str,
        opts: Option<ListOptions>,
    ) -> Result<Record> {
        if opportunity_id.is_empty() {
            return Err(Error::Validation {
                message: "get_opportunity: opportunity_id is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/opportunities/{}/", urlencoding(opportunity_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/notices/{notice_id}/` — a single notice.
    pub async fn get_notice(&self, notice_id: &str, opts: Option<ListOptions>) -> Result<Record> {
        if notice_id.is_empty() {
            return Err(Error::Validation {
                message: "get_notice: notice_id is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/notices/{}/", urlencoding(notice_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/forecasts/{id}/` — a single procurement forecast.
    pub async fn get_forecast(&self, id: &str, opts: Option<ListOptions>) -> Result<Record> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "get_forecast: id is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/forecasts/{}/", urlencoding(id));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/grants/{grant_id}/` — a single grant opportunity.
    pub async fn get_grant(&self, grant_id: &str, opts: Option<ListOptions>) -> Result<Record> {
        if grant_id.is_empty() {
            return Err(Error::Validation {
                message: "get_grant: grant_id is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/grants/{}/", urlencoding(grant_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/opportunities/attachment-search/` — semantic search over
    /// the extracted text of opportunity attachments (SOWs, PWSs, J&As).
    ///
    /// Returns [`Error::Validation`] when `opts.q` is missing or empty.
    pub async fn search_opportunity_attachments(
        &self,
        opts: SearchOpportunityAttachmentsOptions,
    ) -> Result<Record> {
        if opts.q.as_deref().filter(|s| !s.is_empty()).is_none() {
            return Err(Error::Validation {
                message: "search_opportunity_attachments: q is required".into(),
                response: None,
            });
        }
        let q = opts.to_query();
        self.get_json::<Record>("/api/opportunities/attachment-search/", &q)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn opportunities_emits_all_string_filters() {
        let opts = ListOpportunitiesOptions::builder()
            .active(true)
            .agency("9700")
            .first_notice_date_after("2024-01-01")
            .last_notice_date_before("2024-12-31")
            .naics("541512")
            .notice_type("PRESOL")
            .ordering("-response_deadline")
            .place_of_performance("VA")
            .psc("D302")
            .response_deadline_after("2024-01-15")
            .response_deadline_before("2024-03-31")
            .search("cloud computing")
            .set_aside("8A")
            .solicitation_number("W15P7T24R0001")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "active").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "agency").as_deref(), Some("9700"));
        assert_eq!(
            get_q(&q, "first_notice_date_after").as_deref(),
            Some("2024-01-01")
        );
        assert_eq!(
            get_q(&q, "last_notice_date_before").as_deref(),
            Some("2024-12-31")
        );
        assert_eq!(get_q(&q, "naics").as_deref(), Some("541512"));
        assert_eq!(get_q(&q, "notice_type").as_deref(), Some("PRESOL"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-response_deadline"));
        assert_eq!(get_q(&q, "place_of_performance").as_deref(), Some("VA"));
        assert_eq!(get_q(&q, "psc").as_deref(), Some("D302"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("cloud computing"));
        assert_eq!(get_q(&q, "set_aside").as_deref(), Some("8A"));
        assert_eq!(
            get_q(&q, "solicitation_number").as_deref(),
            Some("W15P7T24R0001")
        );
    }

    #[test]
    fn opportunities_active_false_emits_false() {
        let opts = ListOpportunitiesOptions::builder().active(false).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "active").as_deref(), Some("false"));
    }

    #[test]
    fn opportunities_active_none_omits() {
        let opts = ListOpportunitiesOptions::default();
        let q = opts.to_query();
        assert!(!q.iter().any(|(k, _)| k == "active"));
    }

    #[test]
    fn notices_emits_filters_without_ordering_field() {
        let opts = ListNoticesOptions::builder()
            .active(true)
            .agency("9700")
            .naics("541512")
            .notice_type("AWARD")
            .posted_date_after("2024-01-01")
            .posted_date_before("2024-12-31")
            .search("cybersecurity")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "active").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "notice_type").as_deref(), Some("AWARD"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("cybersecurity"));
    }

    #[test]
    fn forecasts_emits_all_filters() {
        let opts = ListForecastsOptions::builder()
            .agency("9700")
            .fiscal_year("2024")
            .fiscal_year_gte("2023")
            .fiscal_year_lte("2025")
            .naics_code("541512")
            .naics_starts_with("5415")
            .ordering("award_date")
            .source_system("SAM")
            .status("active")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "fiscal_year").as_deref(), Some("2024"));
        assert_eq!(get_q(&q, "naics_starts_with").as_deref(), Some("5415"));
        assert_eq!(get_q(&q, "source_system").as_deref(), Some("SAM"));
        assert_eq!(get_q(&q, "status").as_deref(), Some("active"));
    }

    #[test]
    fn grants_emits_all_filters() {
        let opts = ListGrantsOptions::builder()
            .agency("9700")
            .applicant_types("11")
            .cfda_number("10.001")
            .grant_id("GRANT-123")
            .funding_categories("AR")
            .funding_instruments("G")
            .opportunity_number("OPP-001")
            .ordering("response_date")
            .posted_date_after("2024-01-01")
            .response_date_before("2024-11-30")
            .search("environment")
            .status("posted")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "applicant_types").as_deref(), Some("11"));
        assert_eq!(get_q(&q, "cfda_number").as_deref(), Some("10.001"));
        assert_eq!(get_q(&q, "grant_id").as_deref(), Some("GRANT-123"));
        assert_eq!(get_q(&q, "funding_categories").as_deref(), Some("AR"));
        assert_eq!(get_q(&q, "funding_instruments").as_deref(), Some("G"));
        assert_eq!(get_q(&q, "opportunity_number").as_deref(), Some("OPP-001"));
        assert_eq!(get_q(&q, "status").as_deref(), Some("posted"));
    }

    #[tokio::test]
    async fn get_grant_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_grant("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("grant_id")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_opportunity_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_opportunity("", None)
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("opportunity_id")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_forecast_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_forecast("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("id")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_notice_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_notice("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("notice_id")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn attachment_search_emits_all_flags() {
        let opts = SearchOpportunityAttachmentsOptions::builder()
            .q("statement of work cloud migration")
            .top_k(5u32)
            .include_extracted_text(true)
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "q").as_deref(),
            Some("statement of work cloud migration")
        );
        assert_eq!(get_q(&q, "top_k").as_deref(), Some("5"));
        assert_eq!(get_q(&q, "include_extracted_text").as_deref(), Some("true"));
    }

    #[test]
    fn attachment_search_top_k_zero_omitted() {
        let opts = SearchOpportunityAttachmentsOptions::builder()
            .q("test query")
            .top_k(0u32)
            .build();
        let q = opts.to_query();
        assert!(!q.iter().any(|(k, _)| k == "top_k"));
    }

    #[test]
    fn attachment_search_extracted_text_omitted_when_false() {
        let opts = SearchOpportunityAttachmentsOptions::builder()
            .q("test")
            .build();
        let q = opts.to_query();
        assert!(!q.iter().any(|(k, _)| k == "include_extracted_text"));
    }

    #[tokio::test]
    async fn search_opportunity_attachments_empty_q_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .search_opportunity_attachments(SearchOpportunityAttachmentsOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains('q'));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
