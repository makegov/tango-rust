//! `GET /api/subawards/` — list and stream subaward records.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, ListOptions};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_subawards`] and [`Client::iterate_subawards`].
///
/// Mirrors `ListSubawardsOptions` in the Go SDK. `ordering` must be
/// `"last_modified_date"` or `"-last_modified_date"`; the server rejects
/// other values (tango#2254).
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSubawardsOptions {
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

    /// Award key filter (typically the prime PIID).
    #[builder(into)]
    pub award_key: Option<String>,
    /// Prime recipient UEI filter.
    #[builder(into)]
    pub prime_uei: Option<String>,
    /// Subrecipient UEI filter.
    #[builder(into)]
    pub sub_uei: Option<String>,
    /// Awarding agency CGAC code.
    #[builder(into)]
    pub awarding_agency: Option<String>,
    /// Funding agency CGAC code.
    #[builder(into)]
    pub funding_agency: Option<String>,
    /// `fiscal_year` filter.
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,
    /// Recipient (sub or prime depending on endpoint allowlist).
    #[builder(into)]
    pub recipient: Option<String>,
    /// Server-side sort spec. Must be `"last_modified_date"` or
    /// `"-last_modified_date"`.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListSubawardsOptions {
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
        push_opt(&mut q, "award_key", self.award_key.as_deref());
        push_opt(&mut q, "prime_uei", self.prime_uei.as_deref());
        push_opt(&mut q, "sub_uei", self.sub_uei.as_deref());
        push_opt(&mut q, "awarding_agency", self.awarding_agency.as_deref());
        push_opt(&mut q, "funding_agency", self.funding_agency.as_deref());
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year_gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year_lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "recipient", self.recipient.as_deref());
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

impl Client {
    /// `GET /api/subawards/` — one page of subaward records.
    pub async fn list_subawards(&self, opts: ListSubawardsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/subawards/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/subawards/{key}/` — a single subaward record.
    pub async fn get_subaward(&self, key: &str, opts: Option<ListOptions>) -> Result<Record> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "get_subaward: key is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/subawards/{}/", urlencoding(key));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every subaward matching `opts`.
    pub fn iterate_subawards(&self, opts: ListSubawardsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_subawards(next).await })
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
    fn subawards_emits_all_filters() {
        let opts = ListSubawardsOptions::builder()
            .award_key("PIID-123")
            .prime_uei("UEI-PRIME")
            .sub_uei("UEI-SUB")
            .awarding_agency("9700")
            .funding_agency("9700")
            .fiscal_year("2024")
            .fiscal_year_gte("2023")
            .fiscal_year_lte("2025")
            .recipient("Acme Inc")
            .ordering("-last_modified_date")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "award_key").as_deref(), Some("PIID-123"));
        assert_eq!(get_q(&q, "prime_uei").as_deref(), Some("UEI-PRIME"));
        assert_eq!(get_q(&q, "sub_uei").as_deref(), Some("UEI-SUB"));
        assert_eq!(get_q(&q, "awarding_agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "funding_agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "fiscal_year").as_deref(), Some("2024"));
        assert_eq!(get_q(&q, "fiscal_year_gte").as_deref(), Some("2023"));
        assert_eq!(get_q(&q, "fiscal_year_lte").as_deref(), Some("2025"));
        assert_eq!(get_q(&q, "recipient").as_deref(), Some("Acme Inc"));
        assert_eq!(
            get_q(&q, "ordering").as_deref(),
            Some("-last_modified_date")
        );
    }

    #[test]
    fn subawards_pagination_emits() {
        let opts = ListSubawardsOptions::builder()
            .page(2u32)
            .limit(50u32)
            .shape(crate::SHAPE_SUBAWARDS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "page").as_deref(), Some("2"));
        assert_eq!(get_q(&q, "limit").as_deref(), Some("50"));
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_SUBAWARDS_MINIMAL)
        );
    }

    #[test]
    fn subawards_extra_map() {
        let mut extra = BTreeMap::new();
        extra.insert("x".to_string(), "y".to_string());
        let opts = ListSubawardsOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "x").as_deref(), Some("y"));
    }

    #[tokio::test]
    async fn get_subaward_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_subaward("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("key")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
