//! `GET /api/contracts/` — list and stream federal contract records.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, first_non_empty, push_opt, ListOptions};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::resources::entity_subresources::EntitySubresourceOptions;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_contracts`] and [`Client::iterate_contracts`].
///
/// SDK-friendly aliases (`naics_code`, `psc_code`, `recipient_name`,
/// `recipient_uei`, `set_aside_type`) map to the canonical API names —
/// passing both prefers the SDK alias to mirror the Node and Python SDKs.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListContractsOptions {
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
    /// Comma-separated field selector. Use a [`SHAPE_*`](crate::shapes)
    /// constant or roll your own.
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
    /// `award_type` filter (e.g. `"BPA Call"`).
    #[builder(into)]
    pub award_type: Option<String>,

    /// `fiscal_year` filter (accepts `"2024"`, `"FY24"`, or range expressions).
    #[builder(into)]
    pub fiscal_year: Option<String>,
    /// Lower bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_gte: Option<String>,
    /// Upper bound for `fiscal_year`.
    #[builder(into)]
    pub fiscal_year_lte: Option<String>,

    /// Lower bound for `obligated` (dollars).
    #[builder(into)]
    pub obligated_gte: Option<String>,
    /// Upper bound for `obligated` (dollars).
    #[builder(into)]
    pub obligated_lte: Option<String>,

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
    /// Lower bound for contract expiration date.
    #[builder(into)]
    pub expiring_gte: Option<String>,
    /// Upper bound for contract expiration date.
    #[builder(into)]
    pub expiring_lte: Option<String>,

    /// Awarding agency CGAC code (e.g. `"9700"`).
    #[builder(into)]
    pub awarding_agency: Option<String>,
    /// Funding agency CGAC code.
    #[builder(into)]
    pub funding_agency: Option<String>,
    /// Procurement Instrument Identifier filter.
    #[builder(into)]
    pub piid: Option<String>,
    /// Solicitation identifier filter.
    #[builder(into)]
    pub solicitation_identifier: Option<String>,
    /// NAICS code (canonical name).
    #[builder(into)]
    pub naics: Option<String>,
    /// PSC code (canonical name).
    #[builder(into)]
    pub psc: Option<String>,
    /// Recipient name filter (canonical name).
    #[builder(into)]
    pub recipient: Option<String>,
    /// Recipient UEI filter (canonical name).
    #[builder(into)]
    pub uei: Option<String>,
    /// Set-aside filter (canonical name).
    #[builder(into)]
    pub set_aside: Option<String>,

    // SDK-friendly aliases mirroring Node/Python:
    /// SDK-friendly alias for [`naics`](Self::naics).
    #[builder(into)]
    pub naics_code: Option<String>,
    /// SDK-friendly alias for [`psc`](Self::psc).
    #[builder(into)]
    pub psc_code: Option<String>,
    /// SDK-friendly alias for [`recipient`](Self::recipient).
    #[builder(into)]
    pub recipient_name: Option<String>,
    /// SDK-friendly alias for [`uei`](Self::uei).
    #[builder(into)]
    pub recipient_uei: Option<String>,
    /// SDK-friendly alias for [`set_aside`](Self::set_aside).
    #[builder(into)]
    pub set_aside_type: Option<String>,

    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// SDK-friendly alias for [`search`](Self::search).
    #[builder(into)]
    pub keyword: Option<String>,
    /// Server-side sort spec (e.g. `"obligated"`, prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,
    /// Sort field — combined with [`order`](Self::order) into `ordering`.
    #[builder(into)]
    pub sort: Option<String>,
    /// `"asc"` or `"desc"`. Only meaningful when `sort` is set.
    #[builder(into)]
    pub order: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListContractsOptions {
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

        push_opt(&mut q, "award_date", self.award_date.as_deref());
        push_opt(&mut q, "award_date_gte", self.award_date_gte.as_deref());
        push_opt(&mut q, "award_date_lte", self.award_date_lte.as_deref());
        push_opt(&mut q, "award_type", self.award_type.as_deref());
        push_opt(&mut q, "fiscal_year", self.fiscal_year.as_deref());
        push_opt(&mut q, "fiscal_year_gte", self.fiscal_year_gte.as_deref());
        push_opt(&mut q, "fiscal_year_lte", self.fiscal_year_lte.as_deref());
        push_opt(&mut q, "obligated_gte", self.obligated_gte.as_deref());
        push_opt(&mut q, "obligated_lte", self.obligated_lte.as_deref());
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
        push_opt(&mut q, "expiring_gte", self.expiring_gte.as_deref());
        push_opt(&mut q, "expiring_lte", self.expiring_lte.as_deref());
        push_opt(&mut q, "awarding_agency", self.awarding_agency.as_deref());
        push_opt(&mut q, "funding_agency", self.funding_agency.as_deref());
        push_opt(&mut q, "piid", self.piid.as_deref());
        push_opt(
            &mut q,
            "solicitation_identifier",
            self.solicitation_identifier.as_deref(),
        );

        // SDK aliases win over canonical names when both are set.
        push_opt(
            &mut q,
            "naics",
            first_non_empty(&[self.naics_code.as_deref(), self.naics.as_deref()]),
        );
        push_opt(
            &mut q,
            "psc",
            first_non_empty(&[self.psc_code.as_deref(), self.psc.as_deref()]),
        );
        push_opt(
            &mut q,
            "recipient",
            first_non_empty(&[self.recipient_name.as_deref(), self.recipient.as_deref()]),
        );
        push_opt(
            &mut q,
            "uei",
            first_non_empty(&[self.recipient_uei.as_deref(), self.uei.as_deref()]),
        );
        push_opt(
            &mut q,
            "set_aside",
            first_non_empty(&[self.set_aside_type.as_deref(), self.set_aside.as_deref()]),
        );
        push_opt(
            &mut q,
            "search",
            first_non_empty(&[self.keyword.as_deref(), self.search.as_deref()]),
        );

        if let Some(sort) = self.sort.as_deref().filter(|s| !s.is_empty()) {
            let order = self.order.as_deref().unwrap_or("");
            let prefix = if order == "desc" { "-" } else { "" };
            q.push(("ordering".into(), format!("{prefix}{sort}")));
        } else {
            push_opt(&mut q, "ordering", self.ordering.as_deref());
        }

        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

impl Client {
    /// `GET /api/contracts/` — one page of federal contract records.
    pub async fn list_contracts(&self, opts: ListContractsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/contracts/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/contracts/{key}/` — a single federal contract record.
    pub async fn get_contract(&self, key: &str, opts: Option<ListOptions>) -> Result<Record> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "get_contract: key is required".into(),
                response: None,
            });
        }
        let mut q = Vec::new();
        opts.unwrap_or_default().apply(&mut q);
        let path = format!("/api/contracts/{}/", urlencoding(key));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/contracts/{key}/subawards/` — subawards reported against a
    /// single prime contract.
    pub async fn list_contract_subawards(
        &self,
        key: &str,
        opts: Option<EntitySubresourceOptions>,
    ) -> Result<Page<Record>> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "list_contract_subawards: key is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/contracts/{}/subawards/", urlencoding(key));
        let bytes = self.get_bytes(&path, &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/contracts/{key}/transactions/` — raw transaction history
    /// backing a single contract.
    pub async fn list_contract_transactions(
        &self,
        key: &str,
        opts: Option<EntitySubresourceOptions>,
    ) -> Result<Page<Record>> {
        if key.is_empty() {
            return Err(Error::Validation {
                message: "list_contract_transactions: key is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/contracts/{}/transactions/", urlencoding(key));
        let bytes = self.get_bytes(&path, &q).await?;
        Page::decode(&bytes)
    }

    /// Stream every federal contract record matching `opts`. The stream follows
    /// `?cursor=` (or `?page=` fallback) on the server's `next` URL.
    ///
    /// ```no_run
    /// # use tango::{Client, ListContractsOptions};
    /// # use futures::TryStreamExt;
    /// # async fn run(client: Client) -> tango::Result<()> {
    /// let mut s = client.iterate_contracts(
    ///     ListContractsOptions::builder().awarding_agency("9700").build(),
    /// );
    /// while let Some(record) = s.try_next().await? {
    ///     let _ = record;
    /// }
    /// # Ok(()) }
    /// ```
    pub fn iterate_contracts(&self, opts: ListContractsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_contracts(next).await })
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
    fn alias_prefers_sdk_name_over_canonical() {
        let opts = ListContractsOptions::builder()
            .naics_code("541512")
            .naics("999999")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "naics").as_deref(), Some("541512"));
    }

    #[test]
    fn alias_falls_through_to_canonical() {
        let opts = ListContractsOptions::builder().naics("541512").build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "naics").as_deref(), Some("541512"));
    }

    #[test]
    fn sort_plus_order_builds_ordering() {
        let opts = ListContractsOptions::builder()
            .sort("obligated")
            .order("desc")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-obligated"));
    }

    #[test]
    fn sort_without_order_defaults_ascending() {
        let opts = ListContractsOptions::builder().sort("award_date").build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("award_date"));
    }

    #[test]
    fn cursor_wins_over_page() {
        let opts = ListContractsOptions::builder()
            .page(3u32)
            .cursor("abc".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("abc"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[test]
    fn shape_emits() {
        let opts = ListContractsOptions::builder()
            .shape(crate::SHAPE_CONTRACTS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_CONTRACTS_MINIMAL)
        );
    }

    #[tokio::test]
    async fn get_contract_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_contract("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("key")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_contract_subawards_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_contract_subawards("", None)
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("key")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_contract_transactions_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_contract_transactions("", None)
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("key")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
