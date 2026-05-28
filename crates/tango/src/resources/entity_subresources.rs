//! Entity sub-resources: contracts, IDVs, OTAs, OTIDVs, subawards, LCATs,
//! metrics.
//!
//! Endpoints under `/api/entities/{uei}/…/` that share a common parameter
//! shape. The Go SDK splits these into `EntitySubresourceOptions`,
//! `EntitySubawardsOptions`, and `EntityLcatsOptions`; the Rust port
//! consolidates these behind a single [`EntitySubresourceOptions`] since the
//! surface SDK params for these endpoints are identical (pagination + shape +
//! ordering + search + joiner). Subaward callers should pass `ordering`
//! through verbatim — the server enforces its own allowlist.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options shared by every entity sub-resource list endpoint
/// (`/api/entities/{uei}/contracts/`, `/idvs/`, `/otas/`, `/otidvs/`,
/// `/subawards/`, `/lcats/`).
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct EntitySubresourceOptions {
    /// 1-based page number. Mutually exclusive with [`cursor`](Self::cursor).
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size.
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor for cursor-paginated endpoints.
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
    /// Joiner for flattened keys (only sent when `flat=true`).
    #[builder(into)]
    pub joiner: Option<String>,
    /// Server-side sort spec. Endpoint-specific allowlist; prefix `-` for
    /// descending. The subawards endpoint enforces a stricter allowlist than
    /// most — pass values verbatim and let the server validate.
    #[builder(into)]
    pub ordering: Option<String>,
    /// Free-text search filter where supported by the endpoint.
    #[builder(into)]
    pub search: Option<String>,
    /// Escape hatch for filter keys not yet first-classed.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl EntitySubresourceOptions {
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
        if self.flat {
            if let Some(j) = self.joiner.as_deref().filter(|s| !s.is_empty()) {
                q.push(("joiner".into(), j.into()));
            }
        }
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

impl Client {
    /// `GET /api/entities/{uei}/contracts/` — contracts awarded to this entity.
    pub async fn list_entity_contracts(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> Result<Page<Record>> {
        list_entity_subresource(self, uei, "contracts", opts).await
    }

    /// Stream every contract awarded to `uei`.
    pub fn iterate_entity_contracts(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> PageStream<Record> {
        iterate_entity_subresource(self, uei.to_string(), "contracts", opts)
    }

    /// `GET /api/entities/{uei}/idvs/` — IDVs held by this entity.
    pub async fn list_entity_idvs(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> Result<Page<Record>> {
        list_entity_subresource(self, uei, "idvs", opts).await
    }

    /// Stream every IDV held by `uei`.
    pub fn iterate_entity_idvs(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> PageStream<Record> {
        iterate_entity_subresource(self, uei.to_string(), "idvs", opts)
    }

    /// `GET /api/entities/{uei}/otas/` — Other Transaction Awards held by this
    /// entity.
    pub async fn list_entity_otas(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> Result<Page<Record>> {
        list_entity_subresource(self, uei, "otas", opts).await
    }

    /// Stream every OTA held by `uei`.
    pub fn iterate_entity_otas(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> PageStream<Record> {
        iterate_entity_subresource(self, uei.to_string(), "otas", opts)
    }

    /// `GET /api/entities/{uei}/otidvs/` — Other Transaction IDVs held by this
    /// entity.
    pub async fn list_entity_otidvs(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> Result<Page<Record>> {
        list_entity_subresource(self, uei, "otidvs", opts).await
    }

    /// Stream every OTIDV held by `uei`.
    pub fn iterate_entity_otidvs(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> PageStream<Record> {
        iterate_entity_subresource(self, uei.to_string(), "otidvs", opts)
    }

    /// `GET /api/entities/{uei}/subawards/` — subawards reported against
    /// contracts held by this entity.
    pub async fn list_entity_subawards(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> Result<Page<Record>> {
        list_entity_subresource(self, uei, "subawards", opts).await
    }

    /// Stream every subaward reported under `uei`.
    pub fn iterate_entity_subawards(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> PageStream<Record> {
        iterate_entity_subresource(self, uei.to_string(), "subawards", opts)
    }

    /// `GET /api/entities/{uei}/lcats/` — Labor Categories advertised by this
    /// entity.
    pub async fn list_entity_lcats(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> Result<Page<Record>> {
        list_entity_subresource(self, uei, "lcats", opts).await
    }

    /// Stream every LCAT advertised by `uei`.
    pub fn iterate_entity_lcats(
        &self,
        uei: &str,
        opts: EntitySubresourceOptions,
    ) -> PageStream<Record> {
        iterate_entity_subresource(self, uei.to_string(), "lcats", opts)
    }

    /// `GET /api/entities/{uei}/budget-flows/` — funding-account budget flows
    /// attributed to this entity. Returns a paginated list of funding-account
    /// rows.
    pub async fn get_entity_budget_flows(
        &self,
        uei: &str,
        opts: Option<EntitySubresourceOptions>,
    ) -> Result<Page<Record>> {
        list_entity_subresource(self, uei, "budget-flows", opts.unwrap_or_default()).await
    }

    /// `GET /api/entities/{uei}/metrics/{months}/{period_grouping}/` — rolling
    /// windowed metrics for this entity. Mirrors the signature of the sibling
    /// SDKs (Node / Python / Go).
    ///
    /// `months` is the rolling-window length (must be > 0). `period_grouping`
    /// is typically `"month"` or `"quarter"`.
    pub async fn get_entity_metrics(
        &self,
        uei: &str,
        months: u32,
        period_grouping: &str,
    ) -> Result<Record> {
        if uei.is_empty() {
            return Err(Error::Validation {
                message: "get_entity_metrics: uei is required".into(),
                response: None,
            });
        }
        if months == 0 {
            return Err(Error::Validation {
                message: "get_entity_metrics: months must be > 0".into(),
                response: None,
            });
        }
        if period_grouping.is_empty() {
            return Err(Error::Validation {
                message: "get_entity_metrics: period_grouping is required".into(),
                response: None,
            });
        }
        let path = format!(
            "/api/entities/{}/metrics/{}/{}/",
            urlencoding(uei),
            months,
            urlencoding(period_grouping),
        );
        self.get_json::<Record>(&path, &[]).await
    }
}

async fn list_entity_subresource(
    client: &Client,
    uei: &str,
    segment: &str,
    opts: EntitySubresourceOptions,
) -> Result<Page<Record>> {
    if uei.is_empty() {
        return Err(Error::Validation {
            message: "entity sub-resource: uei is required".into(),
            response: None,
        });
    }
    let q = opts.to_query();
    let path = format!("/api/entities/{}/{segment}/", urlencoding(uei));
    let bytes = client.get_bytes(&path, &q).await?;
    Page::decode(&bytes)
}

fn iterate_entity_subresource(
    client: &Client,
    uei: String,
    segment: &'static str,
    opts: EntitySubresourceOptions,
) -> PageStream<Record> {
    let opts = Arc::new(opts);
    let uei = Arc::new(uei);
    let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
        let mut next = (*opts).clone();
        next.page = page;
        next.cursor = cursor;
        let uei = uei.clone();
        Box::pin(async move { list_entity_subresource(&client, &uei, segment, next).await })
    });
    PageStream::new(client.clone(), fetch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_q(q: &[(String, String)], k: &str) -> Option<String> {
        q.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone())
    }

    #[test]
    fn options_emit_pagination_shape_and_search() {
        let opts = EntitySubresourceOptions::builder()
            .limit(10u32)
            .shape("contracts(minimal)")
            .ordering("-award_date")
            .search("software")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "limit").as_deref(), Some("10"));
        assert_eq!(get_q(&q, "shape").as_deref(), Some("contracts(minimal)"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-award_date"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("software"));
        assert!(!q.iter().any(|(k, _)| k == "joiner"));
    }

    #[test]
    fn joiner_only_when_flat() {
        let opts = EntitySubresourceOptions::builder()
            .joiner("__".to_string())
            .build();
        let q = opts.to_query();
        assert!(!q.iter().any(|(k, _)| k == "joiner"));

        let opts = EntitySubresourceOptions::builder()
            .flat(true)
            .joiner("__".to_string())
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("joiner".into(), "__".into())));
    }

    #[test]
    fn extra_forwards_arbitrary_params() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "val".to_string());
        let opts = EntitySubresourceOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert!(q.contains(&("custom_x".into(), "val".into())));
    }

    #[tokio::test]
    async fn list_entity_contracts_empty_uei_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_entity_contracts("", EntitySubresourceOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("uei")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_entity_subawards_empty_uei_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_entity_subawards("", EntitySubresourceOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("uei")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_entity_budget_flows_empty_uei_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_entity_budget_flows("", None)
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("uei")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_entity_metrics_empty_uei_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_entity_metrics("", 12, "month")
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("uei")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_entity_metrics_zero_months_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_entity_metrics("ABC123DEF456", 0, "month")
            .await
            .expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn get_entity_metrics_empty_period_grouping_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .get_entity_metrics("ABC123DEF456", 12, "")
            .await
            .expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }
}
