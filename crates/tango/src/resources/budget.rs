//! `/api/budget/accounts/` — federal-account x fiscal-year budget rollups.
//!
//! One row per `(federal_account_symbol, fiscal_year)` covering the full
//! budget lifecycle (requested → enacted → apportioned → obligated →
//! outlayed), pre-computed ratios + trends, the contract/assistance/unlinked
//! breakdown, and request-vs-actual contract spend. The schema is wide
//! (~63 fields) and shape-driven, so every method returns the untyped
//! [`Record`] map like the other resource families.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, ListOptions};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_budget_accounts`] and
/// [`Client::iterate_budget_accounts`]. Mirrors `ListBudgetAccountsOptions`
/// in the Go SDK.
///
/// The full `__gte` / `__lte` range-filter set (the 26 numeric metrics on the
/// underlying FilterSet) is reachable via the [`extra`](Self::extra) map.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListBudgetAccountsOptions {
    // ----- Pagination + shape -----
    /// 1-based page number. Mutually exclusive with [`cursor`](Self::cursor).
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use
    /// [`SHAPE_BUDGET_ACCOUNTS_MINIMAL`](crate::SHAPE_BUDGET_ACCOUNTS_MINIMAL)
    /// or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    // ----- Resource filters -----
    /// Federal account symbol filter (exact, e.g. `"097-0100"`).
    #[builder(into)]
    pub federal_account_symbol: Option<String>,
    /// `fiscal_year` filter (exact).
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year` (inclusive).
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year` (inclusive).
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,
    /// Awarding/funding agency CGAC code filter (exact).
    #[builder(into)]
    pub agency_code: Option<String>,
    /// Bureau of Economic Analysis category filter (exact).
    #[builder(into)]
    pub bea_category: Option<String>,
    /// On/off-budget flag filter (exact).
    #[builder(into)]
    pub on_off_budget: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Server-side sort spec (prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct
    /// (e.g. the `*_gte` / `*_lte` range filters on the numeric metrics).
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListBudgetAccountsOptions {
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
        push_opt(
            &mut q,
            "federal_account_symbol",
            self.federal_account_symbol.as_deref(),
        );
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year__gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year__lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "agency_code", self.agency_code.as_deref());
        push_opt(&mut q, "bea_category", self.bea_category.as_deref());
        push_opt(&mut q, "on_off_budget", self.on_off_budget.as_deref());
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

impl Client {
    /// `GET /api/budget/accounts/` — one page of budget-account rollups.
    pub async fn list_budget_accounts(
        &self,
        opts: ListBudgetAccountsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/budget/accounts/", &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every budget-account rollup matching `opts`.
    pub fn iterate_budget_accounts(&self, opts: ListBudgetAccountsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_budget_accounts(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/budget/accounts/{id}/` — a single budget-account rollup.
    pub async fn get_budget_account(&self, id: &str, opts: Option<ListOptions>) -> Result<Record> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "get_budget_account: id is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/budget/accounts/{}/", urlencoding(id));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/budget/accounts/{id}/quarters/` — quarterly lifecycle detail
    /// for a single account-year.
    pub async fn get_budget_account_quarters(
        &self,
        id: &str,
        opts: Option<ListOptions>,
    ) -> Result<Page<Record>> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "get_budget_account_quarters: id is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/budget/accounts/{}/quarters/", urlencoding(id));
        let bytes = self.get_bytes(&path, &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/budget/accounts/{id}/recipients/` — funding-office x recipient
    /// contract-flow detail for a single account-year. The response envelope
    /// carries extra keys (`federal_account_symbol`, `fiscal_year`) alongside
    /// the standard pagination fields, so callers should navigate the returned
    /// [`Record`] structure directly.
    pub async fn get_budget_account_recipients(
        &self,
        id: &str,
        opts: Option<ListOptions>,
    ) -> Result<Page<Record>> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "get_budget_account_recipients: id is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/budget/accounts/{}/recipients/", urlencoding(id));
        let bytes = self.get_bytes(&path, &q).await?;
        Page::decode(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn options_emit_all_filters() {
        let opts = ListBudgetAccountsOptions::builder()
            .federal_account_symbol("097-0100")
            .fiscal_year("2024")
            .fiscal_year_gte("2020")
            .fiscal_year_lte("2025")
            .agency_code("9700")
            .bea_category("discretionary")
            .on_off_budget("on")
            .search("operations")
            .ordering("-enacted_ba")
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "federal_account_symbol").as_deref(),
            Some("097-0100")
        );
        assert_eq!(get_q(&q, "fiscal_year").as_deref(), Some("2024"));
        assert_eq!(get_q(&q, "fiscal_year__gte").as_deref(), Some("2020"));
        assert_eq!(get_q(&q, "fiscal_year__lte").as_deref(), Some("2025"));
        assert_eq!(get_q(&q, "agency_code").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "bea_category").as_deref(), Some("discretionary"));
        assert_eq!(get_q(&q, "on_off_budget").as_deref(), Some("on"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("operations"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-enacted_ba"));
    }

    #[test]
    fn pagination_and_shape_emit() {
        let opts = ListBudgetAccountsOptions::builder()
            .page(2u32)
            .limit(50u32)
            .shape(crate::SHAPE_BUDGET_ACCOUNTS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "page").as_deref(), Some("2"));
        assert_eq!(get_q(&q, "limit").as_deref(), Some("50"));
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_BUDGET_ACCOUNTS_MINIMAL)
        );
    }

    #[test]
    fn extra_forwards_range_filters() {
        let mut extra = BTreeMap::new();
        extra.insert("enacted_ba__gte".to_string(), "1000000".to_string());
        let opts = ListBudgetAccountsOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "enacted_ba__gte").as_deref(), Some("1000000"));
    }

    #[tokio::test]
    async fn get_budget_account_empty_id_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_budget_account("", None)
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("id")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_budget_account_quarters_empty_id_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_budget_account_quarters("", None)
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("id")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_budget_account_recipients_empty_id_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_budget_account_recipients("", None)
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("id")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
