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

/// Options for [`Client::list_budget_accounts`] and [`Client::iterate_budget_accounts`].
///
/// The API rejects an unknown filter name with a 400 rather than ignoring it.
/// The exact, `__gte` and `__lte` range filters on the numeric lifecycle and ratio fields (e.g. `enacted_ba__gte`), and the `__in` multi-value variants, are reachable via the [`extra`](Self::extra) map.
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
    /// Agency code filter (exact).
    #[builder(into)]
    pub agency_code: Option<String>,
    /// Bureau name filter (exact).
    #[builder(into)]
    pub bureau_name: Option<String>,
    /// Case-insensitive substring match on the account title (sent as `account_title__icontains`).
    #[builder(into)]
    pub account_title: Option<String>,
    /// Budget subfunction code filter (exact).
    #[builder(into)]
    pub subfunction_code: Option<String>,
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
        push_opt(&mut q, "bureau_name", self.bureau_name.as_deref());
        push_opt(
            &mut q,
            "account_title__icontains",
            self.account_title.as_deref(),
        );
        push_opt(&mut q, "subfunction_code", self.subfunction_code.as_deref());
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

/// Options for [`Client::get_budget_account_quarters`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct BudgetAccountQuartersOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (default 25, server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Narrow to a single Treasury Account Symbol. Omit to get every TAS that rolls up under the federal account.
    #[builder(into)]
    pub tas: Option<String>,
}

impl BudgetAccountQuartersOptions {
    fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(&mut q, self.page, self.limit, None, None, false, false);
        push_opt(&mut q, "tas", self.tas.as_deref());
        q
    }
}

/// Options for [`Client::get_budget_account_recipients`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct BudgetAccountRecipientsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (default 25, server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Narrow to a single funding office (an organization UUID).
    #[builder(into)]
    pub funding_organization_id: Option<String>,
}

impl BudgetAccountRecipientsOptions {
    fn to_query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        apply_pagination(&mut q, self.page, self.limit, None, None, false, false);
        push_opt(
            &mut q,
            "funding_organization_id",
            self.funding_organization_id.as_deref(),
        );
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

    /// `GET /api/budget/accounts/{id}/` — a single budget-account rollup by its numeric `id` (from a list row's `id` field).
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

    /// `GET /api/budget/accounts/{id}/quarters/` — one row per (TAS, quarter) of obligation and outlay flow for a single account-year.
    ///
    /// Coverage starts at FY2021; earlier years return an empty page.
    /// The envelope also carries `federal_account_symbol` and `fiscal_year`, which [`Page`] does not surface.
    pub async fn get_budget_account_quarters(
        &self,
        id: &str,
        opts: Option<BudgetAccountQuartersOptions>,
    ) -> Result<Page<Record>> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "get_budget_account_quarters: id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/budget/accounts/{}/quarters/", urlencoding(id));
        let bytes = self.get_bytes(&path, &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/budget/accounts/{id}/recipients/` — funding-office x recipient contract flows for a single account-year, largest `contract_obligated` first.
    ///
    /// Contract flows only. Each row carries the resolved `funding_office` and `recipient`, plus a capped `contracts` list; a row that hits the cap sets `contracts_truncated`.
    /// The envelope also carries `federal_account_symbol` and `fiscal_year`, which [`Page`] does not surface.
    pub async fn get_budget_account_recipients(
        &self,
        id: &str,
        opts: Option<BudgetAccountRecipientsOptions>,
    ) -> Result<Page<Record>> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "get_budget_account_recipients: id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
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
            .bureau_name("Operation and Maintenance")
            .account_title("readiness")
            .subfunction_code("051")
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
        assert_eq!(
            get_q(&q, "bureau_name").as_deref(),
            Some("Operation and Maintenance")
        );
        assert_eq!(
            get_q(&q, "account_title__icontains").as_deref(),
            Some("readiness")
        );
        assert_eq!(get_q(&q, "subfunction_code").as_deref(), Some("051"));
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

    #[test]
    fn quarters_options_emit_tas_and_pagination() {
        let q = BudgetAccountQuartersOptions::builder()
            .page(2u32)
            .limit(50u32)
            .tas("097-2020/2021-0100")
            .build()
            .to_query();
        assert_eq!(get_q(&q, "page").as_deref(), Some("2"));
        assert_eq!(get_q(&q, "limit").as_deref(), Some("50"));
        assert_eq!(get_q(&q, "tas").as_deref(), Some("097-2020/2021-0100"));
        assert_eq!(q.len(), 3);
    }

    #[test]
    fn recipients_options_emit_funding_organization_id() {
        let q = BudgetAccountRecipientsOptions::builder()
            .funding_organization_id("0b7e4c1e-0000-4000-8000-000000000000")
            .build()
            .to_query();
        assert_eq!(
            q,
            vec![(
                "funding_organization_id".to_string(),
                "0b7e4c1e-0000-4000-8000-000000000000".to_string()
            )]
        );
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
