//! `GET /api/idvs/` — list and stream Indefinite Delivery Vehicle records.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_idvs`] and [`Client::iterate_idvs`].
///
/// Mirrors the Go SDK's `ListIDVsOptions`. All filters are sent as query
/// parameters; empty/None values are omitted.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListIDVsOptions {
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
    /// Comma-separated field selector. Use a `SHAPE_*` constant or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    // ----- Resource filters -----
    /// Single-day `award_date` filter (ISO `YYYY-MM-DD`).
    #[builder(into)]
    pub award_date: Option<String>,
    /// Lower bound for `award_date` (inclusive).
    #[builder(into)]
    pub award_date_gte: Option<String>,
    /// Upper bound for `award_date` (inclusive).
    #[builder(into)]
    pub award_date_lte: Option<String>,
    /// Awarding agency CGAC code (e.g. `"9700"`).
    #[builder(into)]
    pub awarding_agency: Option<String>,
    /// Funding agency CGAC code.
    #[builder(into)]
    pub funding_agency: Option<String>,
    /// Lower bound for the IDV's expiration date.
    #[builder(into)]
    pub expiring_gte: Option<String>,
    /// Upper bound for the IDV's expiration date.
    #[builder(into)]
    pub expiring_lte: Option<String>,
    /// `fiscal_year` filter.
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,
    /// IDV type code (e.g. `"A"`).
    #[builder(into)]
    pub idv_type: Option<String>,
    /// Lower bound for the IDV's last date to order.
    #[builder(into)]
    pub last_date_to_order_gte: Option<String>,
    /// Upper bound for the IDV's last date to order.
    #[builder(into)]
    pub last_date_to_order_lte: Option<String>,
    /// NAICS code.
    #[builder(into)]
    pub naics: Option<String>,
    /// Server-side sort spec (prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,
    /// Procurement Instrument Identifier filter.
    #[builder(into)]
    pub piid: Option<String>,
    /// Lower bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_gte: Option<String>,
    /// Upper bound for period-of-performance start date.
    #[builder(into)]
    pub pop_start_date_lte: Option<String>,
    /// PSC code.
    #[builder(into)]
    pub psc: Option<String>,
    /// Recipient name filter.
    #[builder(into)]
    pub recipient: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Set-aside filter.
    #[builder(into)]
    pub set_aside: Option<String>,
    /// Solicitation identifier filter.
    #[builder(into)]
    pub solicitation_identifier: Option<String>,
    /// Recipient UEI filter.
    #[builder(into)]
    pub uei: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListIDVsOptions {
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

        push_opt(&mut q, "award_date", self.award_date.as_deref());
        push_opt(&mut q, "award_date_gte", self.award_date_gte.as_deref());
        push_opt(&mut q, "award_date_lte", self.award_date_lte.as_deref());
        push_opt(&mut q, "awarding_agency", self.awarding_agency.as_deref());
        push_opt(&mut q, "funding_agency", self.funding_agency.as_deref());
        push_opt(&mut q, "expiring_gte", self.expiring_gte.as_deref());
        push_opt(&mut q, "expiring_lte", self.expiring_lte.as_deref());
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year_gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year_lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "idv_type", self.idv_type.as_deref());
        push_opt(
            &mut q,
            "last_date_to_order_gte",
            self.last_date_to_order_gte.as_deref(),
        );
        push_opt(
            &mut q,
            "last_date_to_order_lte",
            self.last_date_to_order_lte.as_deref(),
        );
        push_opt(&mut q, "naics", self.naics.as_deref());
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        push_opt(&mut q, "piid", self.piid.as_deref());
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
        push_opt(&mut q, "psc", self.psc.as_deref());
        push_opt(&mut q, "recipient", self.recipient.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "set_aside", self.set_aside.as_deref());
        push_opt(
            &mut q,
            "solicitation_identifier",
            self.solicitation_identifier.as_deref(),
        );
        push_opt(&mut q, "uei", self.uei.as_deref());

        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_idv`].
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetIDVOptions {
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

impl GetIDVOptions {
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
    /// `GET /api/idvs/` — one page of Indefinite Delivery Vehicle records.
    pub async fn list_idvs(&self, opts: ListIDVsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/idvs/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/idvs/{key}/` — fetch a single IDV by key.
    ///
    /// Returns a free-form [`Record`]; deserialize into your own struct via
    /// `serde_json::from_value(Value::Object(record))` if you need typed fields.
    pub async fn get_idv(&self, key: &str, opts: Option<GetIDVOptions>) -> Result<Record> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "get_idv: key is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/idvs/{}/", urlencoding(key));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every IDV record matching `opts`. The stream follows `?cursor=`
    /// (or `?page=` fallback) on the server's `next` URL.
    pub fn iterate_idvs(&self, opts: ListIDVsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_idvs(next).await })
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
    fn list_idvs_all_filters_emit() {
        let opts = ListIDVsOptions::builder()
            .award_date("2024-01-01")
            .awarding_agency("9700")
            .funding_agency("9800")
            .idv_type("A")
            .naics("541512")
            .piid("W15P7T19D0001")
            .psc("D302")
            .recipient("Acme")
            .search("keyword")
            .set_aside("8A")
            .solicitation_identifier("SOL001")
            .uei("UEI12345")
            .ordering("-award_date")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "award_date").as_deref(), Some("2024-01-01"));
        assert_eq!(get_q(&q, "awarding_agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "funding_agency").as_deref(), Some("9800"));
        assert_eq!(get_q(&q, "idv_type").as_deref(), Some("A"));
        assert_eq!(get_q(&q, "naics").as_deref(), Some("541512"));
        assert_eq!(get_q(&q, "piid").as_deref(), Some("W15P7T19D0001"));
        assert_eq!(get_q(&q, "psc").as_deref(), Some("D302"));
        assert_eq!(get_q(&q, "recipient").as_deref(), Some("Acme"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("keyword"));
        assert_eq!(get_q(&q, "set_aside").as_deref(), Some("8A"));
        assert_eq!(
            get_q(&q, "solicitation_identifier").as_deref(),
            Some("SOL001")
        );
        assert_eq!(get_q(&q, "uei").as_deref(), Some("UEI12345"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-award_date"));
    }

    #[test]
    fn list_idvs_zero_value_omitted() {
        let opts = ListIDVsOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_idvs_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xval".to_string());
        let opts = ListIDVsOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert!(q.contains(&("custom_x".into(), "xval".into())));
    }

    #[test]
    fn list_idvs_cursor_wins_over_page() {
        let opts = ListIDVsOptions::builder()
            .page(3u32)
            .cursor("abc".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("abc"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[test]
    fn get_idv_opts_emits_shape_and_flat() {
        let opts = GetIDVOptions::builder()
            .shape("idvs(minimal)")
            .flat(true)
            .flat_lists(true)
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("shape".into(), "idvs(minimal)".into())));
        assert!(q.contains(&("flat".into(), "true".into())));
        assert!(q.contains(&("flat_lists".into(), "true".into())));
    }

    #[tokio::test]
    async fn get_idv_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_idv("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("key"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
