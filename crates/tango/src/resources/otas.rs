//! `GET /api/otas/` and `GET /api/otidvs/` — Other Transaction Authority
//! award actions and their IDV parents.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

// ============================================================================
// OTAs (/api/otas/)
// ============================================================================

/// Options for [`Client::list_otas`] and [`Client::iterate_otas`].
///
/// Mirrors the Go SDK's `ListOTAsOptions`. All filters are sent as query
/// parameters; empty/None values are omitted.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOTAsOptions {
    // ----- Pagination + shape (shared across every list endpoint) -----
    /// 1-based page number. Mutually exclusive with [`cursor`](Self::cursor).
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100 on most endpoints).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor for cursor-paginated endpoints. Pass the `cursor` field
    /// from the previous [`Page`](crate::Page).
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_OTAS_MINIMAL`](crate::SHAPE_OTAS_MINIMAL)
    /// or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Joiner used between flattened keys when `flat=true`. Defaults to `.` server-side.
    #[builder(into)]
    pub joiner: Option<String>,

    // ----- Identifiers + agencies -----
    /// Awarding agency CGAC code.
    #[builder(into)]
    pub awarding_agency: Option<String>,
    /// Funding agency CGAC code.
    #[builder(into)]
    pub funding_agency: Option<String>,
    /// Procurement Instrument Identifier.
    #[builder(into)]
    pub piid: Option<String>,
    /// Recipient name filter.
    #[builder(into)]
    pub recipient: Option<String>,
    /// Recipient UEI.
    #[builder(into)]
    pub uei: Option<String>,

    // ----- Fiscal-year bounds -----
    /// `fiscal_year` filter.
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,

    // ----- Date bounds -----
    /// Single-day `award_date` filter (ISO `YYYY-MM-DD`).
    #[builder(into)]
    pub award_date: Option<String>,
    /// Lower bound for `award_date` (inclusive).
    #[builder(into)]
    pub award_date_gte: Option<String>,
    /// Upper bound for `award_date` (inclusive).
    #[builder(into)]
    pub award_date_lte: Option<String>,
    /// Lower bound for the OTA's expiration date.
    #[builder(into)]
    pub expiring_gte: Option<String>,
    /// Upper bound for the OTA's expiration date.
    #[builder(into)]
    pub expiring_lte: Option<String>,
    /// Lower bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_gte: Option<String>,
    /// Upper bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_lte: Option<String>,
    /// Lower bound for period-of-performance end date.
    #[builder(into)]
    pub pop_end_date_gte: Option<String>,
    /// Upper bound for period-of-performance end date.
    #[builder(into)]
    pub pop_end_date_lte: Option<String>,

    /// PSC code.
    #[builder(into)]
    pub psc: Option<String>,

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Server-side sort spec (prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListOTAsOptions {
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
        push_opt(&mut q, "joiner", self.joiner.as_deref());
        push_opt(&mut q, "awarding_agency", self.awarding_agency.as_deref());
        push_opt(&mut q, "funding_agency", self.funding_agency.as_deref());
        push_opt(&mut q, "piid", self.piid.as_deref());
        push_opt(&mut q, "recipient", self.recipient.as_deref());
        push_opt(&mut q, "uei", self.uei.as_deref());
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year_gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year_lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "award_date", self.award_date.as_deref());
        push_opt(&mut q, "award_date_gte", self.award_date_gte.as_deref());
        push_opt(&mut q, "award_date_lte", self.award_date_lte.as_deref());
        push_opt(&mut q, "expiring_gte", self.expiring_gte.as_deref());
        push_opt(&mut q, "expiring_lte", self.expiring_lte.as_deref());
        push_opt(
            &mut q,
            "pop_start_date_gte",
            self.pop_start_date_gte.as_deref(),
        );
        push_opt(
            &mut q,
            "pop_start_date_lte",
            self.pop_start_date_lte.as_deref(),
        );
        push_opt(&mut q, "pop_end_date_gte", self.pop_end_date_gte.as_deref());
        push_opt(&mut q, "pop_end_date_lte", self.pop_end_date_lte.as_deref());
        push_opt(&mut q, "psc", self.psc.as_deref());
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

/// Options for [`Client::get_ota`].
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetOTAOptions {
    /// Shape selector. When empty, the server returns its comprehensive default.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetOTAOptions {
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

// ============================================================================
// OTIDVs (/api/otidvs/)
// ============================================================================

/// Options for [`Client::list_otidvs`] and [`Client::iterate_otidvs`].
///
/// Filter shape is identical to [`ListOTAsOptions`] per the sibling SDKs.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOTIDVsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size.
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use
    /// [`SHAPE_OTIDVS_MINIMAL`](crate::SHAPE_OTIDVS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Joiner used between flattened keys when `flat=true`.
    #[builder(into)]
    pub joiner: Option<String>,

    /// Awarding agency CGAC code.
    #[builder(into)]
    pub awarding_agency: Option<String>,
    /// Funding agency CGAC code.
    #[builder(into)]
    pub funding_agency: Option<String>,
    /// Procurement Instrument Identifier.
    #[builder(into)]
    pub piid: Option<String>,
    /// Recipient name filter.
    #[builder(into)]
    pub recipient: Option<String>,
    /// Recipient UEI.
    #[builder(into)]
    pub uei: Option<String>,

    /// `fiscal_year` filter.
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,

    /// Single-day `award_date` filter (ISO `YYYY-MM-DD`).
    #[builder(into)]
    pub award_date: Option<String>,
    /// Lower bound for `award_date`.
    #[builder(into)]
    pub award_date_gte: Option<String>,
    /// Upper bound for `award_date`.
    #[builder(into)]
    pub award_date_lte: Option<String>,
    /// Lower bound for the OTIDV's expiration date.
    #[builder(into)]
    pub expiring_gte: Option<String>,
    /// Upper bound for the OTIDV's expiration date.
    #[builder(into)]
    pub expiring_lte: Option<String>,
    /// Lower bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_gte: Option<String>,
    /// Upper bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_lte: Option<String>,
    /// Lower bound for period-of-performance end date.
    #[builder(into)]
    pub pop_end_date_gte: Option<String>,
    /// Upper bound for period-of-performance end date.
    #[builder(into)]
    pub pop_end_date_lte: Option<String>,

    /// PSC code.
    #[builder(into)]
    pub psc: Option<String>,

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Server-side sort spec (prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListOTIDVsOptions {
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
        push_opt(&mut q, "joiner", self.joiner.as_deref());
        push_opt(&mut q, "awarding_agency", self.awarding_agency.as_deref());
        push_opt(&mut q, "funding_agency", self.funding_agency.as_deref());
        push_opt(&mut q, "piid", self.piid.as_deref());
        push_opt(&mut q, "recipient", self.recipient.as_deref());
        push_opt(&mut q, "uei", self.uei.as_deref());
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year_gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year_lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "award_date", self.award_date.as_deref());
        push_opt(&mut q, "award_date_gte", self.award_date_gte.as_deref());
        push_opt(&mut q, "award_date_lte", self.award_date_lte.as_deref());
        push_opt(&mut q, "expiring_gte", self.expiring_gte.as_deref());
        push_opt(&mut q, "expiring_lte", self.expiring_lte.as_deref());
        push_opt(
            &mut q,
            "pop_start_date_gte",
            self.pop_start_date_gte.as_deref(),
        );
        push_opt(
            &mut q,
            "pop_start_date_lte",
            self.pop_start_date_lte.as_deref(),
        );
        push_opt(&mut q, "pop_end_date_gte", self.pop_end_date_gte.as_deref());
        push_opt(&mut q, "pop_end_date_lte", self.pop_end_date_lte.as_deref());
        push_opt(&mut q, "psc", self.psc.as_deref());
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

/// Options for [`Client::get_otidv`].
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetOTIDVOptions {
    /// Shape selector. When empty, the server returns its comprehensive default.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetOTIDVOptions {
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

// ============================================================================
// OTIDV awards (/api/otidvs/{key}/awards/)
// ============================================================================

/// Options for [`Client::list_otidv_awards`] and [`Client::iterate_otidv_awards`].
///
/// Same filter shape as [`ListOTAsOptions`].
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListOTIDVAwardsOptions {
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

    /// Joiner used between flattened keys when `flat=true`.
    #[builder(into)]
    pub joiner: Option<String>,

    /// Awarding agency CGAC code.
    #[builder(into)]
    pub awarding_agency: Option<String>,
    /// Funding agency CGAC code.
    #[builder(into)]
    pub funding_agency: Option<String>,
    /// Procurement Instrument Identifier.
    #[builder(into)]
    pub piid: Option<String>,
    /// Recipient name filter.
    #[builder(into)]
    pub recipient: Option<String>,
    /// Recipient UEI.
    #[builder(into)]
    pub uei: Option<String>,

    /// `fiscal_year` filter.
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,

    /// Single-day `award_date` filter.
    #[builder(into)]
    pub award_date: Option<String>,
    /// Lower bound for `award_date`.
    #[builder(into)]
    pub award_date_gte: Option<String>,
    /// Upper bound for `award_date`.
    #[builder(into)]
    pub award_date_lte: Option<String>,
    /// Lower bound for the award's expiration date.
    #[builder(into)]
    pub expiring_gte: Option<String>,
    /// Upper bound for the award's expiration date.
    #[builder(into)]
    pub expiring_lte: Option<String>,
    /// Lower bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_gte: Option<String>,
    /// Upper bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_lte: Option<String>,
    /// Lower bound for period-of-performance end date.
    #[builder(into)]
    pub pop_end_date_gte: Option<String>,
    /// Upper bound for period-of-performance end date.
    #[builder(into)]
    pub pop_end_date_lte: Option<String>,

    /// PSC code.
    #[builder(into)]
    pub psc: Option<String>,

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Server-side sort spec.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListOTIDVAwardsOptions {
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
        push_opt(&mut q, "joiner", self.joiner.as_deref());
        push_opt(&mut q, "awarding_agency", self.awarding_agency.as_deref());
        push_opt(&mut q, "funding_agency", self.funding_agency.as_deref());
        push_opt(&mut q, "piid", self.piid.as_deref());
        push_opt(&mut q, "recipient", self.recipient.as_deref());
        push_opt(&mut q, "uei", self.uei.as_deref());
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year_gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year_lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "award_date", self.award_date.as_deref());
        push_opt(&mut q, "award_date_gte", self.award_date_gte.as_deref());
        push_opt(&mut q, "award_date_lte", self.award_date_lte.as_deref());
        push_opt(&mut q, "expiring_gte", self.expiring_gte.as_deref());
        push_opt(&mut q, "expiring_lte", self.expiring_lte.as_deref());
        push_opt(
            &mut q,
            "pop_start_date_gte",
            self.pop_start_date_gte.as_deref(),
        );
        push_opt(
            &mut q,
            "pop_start_date_lte",
            self.pop_start_date_lte.as_deref(),
        );
        push_opt(&mut q, "pop_end_date_gte", self.pop_end_date_gte.as_deref());
        push_opt(&mut q, "pop_end_date_lte", self.pop_end_date_lte.as_deref());
        push_opt(&mut q, "psc", self.psc.as_deref());
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

// ============================================================================
// Client methods
// ============================================================================

impl Client {
    /// `GET /api/otas/` — one page of Other Transaction Authority award actions.
    pub async fn list_otas(&self, opts: ListOTAsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/otas/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/otas/{key}/` — fetch a single OTA by key.
    pub async fn get_ota(&self, key: &str, opts: Option<GetOTAOptions>) -> Result<Record> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "get_ota: key is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/otas/{}/", urlencoding(key));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every OTA matching `opts`.
    pub fn iterate_otas(&self, opts: ListOTAsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_otas(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/otidvs/` — one page of Other Transaction IDV parent records.
    pub async fn list_otidvs(&self, opts: ListOTIDVsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/otidvs/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/otidvs/{key}/` — fetch a single OTIDV by key.
    pub async fn get_otidv(&self, key: &str, opts: Option<GetOTIDVOptions>) -> Result<Record> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "get_otidv: key is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/otidvs/{}/", urlencoding(key));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every OTIDV matching `opts`.
    pub fn iterate_otidvs(&self, opts: ListOTIDVsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_otidvs(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/otidvs/{key}/awards/` — child awards under an OTIDV parent.
    pub async fn list_otidv_awards(
        &self,
        key: &str,
        opts: ListOTIDVAwardsOptions,
    ) -> Result<Page<Record>> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "list_otidv_awards: key is required".into(),
                response: None,
            });
        }
        let q = opts.to_query();
        let path = format!("/api/otidvs/{}/awards/", urlencoding(key));
        let bytes = self.get_bytes(&path, &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every child award under the given OTIDV parent.
    pub fn iterate_otidv_awards(
        &self,
        key: &str,
        opts: ListOTIDVAwardsOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let key = Arc::new(key.to_string());
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            let key = key.clone();
            Box::pin(async move { client.list_otidv_awards(&key, next).await })
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
    fn list_otas_all_filters_emit() {
        let opts = ListOTAsOptions::builder()
            .awarding_agency("9700")
            .funding_agency("9800")
            .piid("FA8650")
            .recipient("Acme")
            .uei("UEI123")
            .fiscal_year("2024")
            .fiscal_year_gte("2020")
            .fiscal_year_lte("2024")
            .award_date("2024-01-01")
            .award_date_gte("2023-01-01")
            .award_date_lte("2024-12-31")
            .expiring_gte("2024-01-01")
            .expiring_lte("2025-12-31")
            .pop_start_date_gte("2024-01-01")
            .pop_start_date_lte("2024-06-30")
            .pop_end_date_gte("2024-07-01")
            .pop_end_date_lte("2024-12-31")
            .psc("D302")
            .search("hypersonics")
            .ordering("-award_date")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "awarding_agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "funding_agency").as_deref(), Some("9800"));
        assert_eq!(get_q(&q, "piid").as_deref(), Some("FA8650"));
        assert_eq!(get_q(&q, "recipient").as_deref(), Some("Acme"));
        assert_eq!(get_q(&q, "uei").as_deref(), Some("UEI123"));
        assert_eq!(get_q(&q, "fiscal_year").as_deref(), Some("2024"));
        assert_eq!(get_q(&q, "fiscal_year_gte").as_deref(), Some("2020"));
        assert_eq!(get_q(&q, "fiscal_year_lte").as_deref(), Some("2024"));
        assert_eq!(get_q(&q, "award_date").as_deref(), Some("2024-01-01"));
        assert_eq!(get_q(&q, "psc").as_deref(), Some("D302"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("hypersonics"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-award_date"));
    }

    #[test]
    fn list_otas_zero_value_omitted() {
        let opts = ListOTAsOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_otas_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xval".to_string());
        let opts = ListOTAsOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert!(q.contains(&("custom_x".into(), "xval".into())));
    }

    #[test]
    fn list_otas_cursor_wins_over_page() {
        let opts = ListOTAsOptions::builder()
            .page(3u32)
            .cursor("abc".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("abc"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[test]
    fn list_otas_shape_and_flat_emit() {
        let opts = ListOTAsOptions::builder()
            .shape(crate::SHAPE_OTAS_MINIMAL)
            .flat(true)
            .flat_lists(true)
            .joiner("__")
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_OTAS_MINIMAL)
        );
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "joiner").as_deref(), Some("__"));
    }

    #[test]
    fn list_otidvs_all_filters_emit() {
        let opts = ListOTIDVsOptions::builder()
            .awarding_agency("9700")
            .piid("FA8650")
            .uei("UEI123")
            .fiscal_year("2024")
            .award_date_gte("2023-01-01")
            .search("hypersonics")
            .ordering("-award_date")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "awarding_agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "piid").as_deref(), Some("FA8650"));
        assert_eq!(get_q(&q, "uei").as_deref(), Some("UEI123"));
        assert_eq!(get_q(&q, "fiscal_year").as_deref(), Some("2024"));
        assert_eq!(get_q(&q, "award_date_gte").as_deref(), Some("2023-01-01"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("hypersonics"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-award_date"));
    }

    #[test]
    fn list_otidv_awards_emits_filters() {
        let opts = ListOTIDVAwardsOptions::builder()
            .awarding_agency("9700")
            .recipient("Acme")
            .search("kw")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "awarding_agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "recipient").as_deref(), Some("Acme"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("kw"));
    }

    #[tokio::test]
    async fn get_ota_validates_empty_key() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_ota("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("key is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_otidv_validates_empty_key() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_otidv("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("key is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_otidv_awards_validates_empty_key() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client
            .list_otidv_awards("", ListOTIDVAwardsOptions::default())
            .await
            .unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("key is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
