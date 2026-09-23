//! IDV sub-resources: awards, child IDVs, transactions, LCATs.
//!
//! Endpoints under `/api/idvs/{key}/…/` that share a common parameter shape.
//! The Go SDK uses a mix of `ListIDVsOptions` (awards/child-idvs),
//! `ListOptions` (transactions), and `EntityLcatsOptions`
//! (lcats); the Rust port consolidates these behind a single
//! [`IdvSubresourceOptions`] (pagination + shape + ordering + search + joiner)
//! since the surface server-side params for these endpoints are identical at
//! the SDK level.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options shared by every IDV sub-resource list endpoint
/// (`/api/idvs/{key}/awards/`, `/child-idvs/`, `/transactions/`,
/// `/lcats/`).
///
/// Carries pagination + shape + ordering + search + joiner. Use the `extra`
/// field to forward filters not yet first-classed on this struct.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct IdvSubresourceOptions {
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
    /// Server-side sort spec (prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// Escape hatch for filter keys not yet first-classed.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl IdvSubresourceOptions {
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
    /// `GET /api/idvs/{key}/awards/` — task-order awards under a parent IDV.
    pub async fn list_idv_awards(
        &self,
        key: &str,
        opts: IdvSubresourceOptions,
    ) -> Result<Page<Record>> {
        list_idv_subresource(self, key, "awards", opts).await
    }

    /// Stream every task-order award under `key`.
    pub fn iterate_idv_awards(&self, key: &str, opts: IdvSubresourceOptions) -> PageStream<Record> {
        iterate_idv_subresource(self, key.to_string(), "awards", opts)
    }

    /// `GET /api/idvs/{key}/idvs/` — child IDVs nested under a parent IDV.
    pub async fn list_idv_child_idvs(
        &self,
        key: &str,
        opts: IdvSubresourceOptions,
    ) -> Result<Page<Record>> {
        list_idv_subresource(self, key, "idvs", opts).await
    }

    /// Stream every child IDV under `key`.
    pub fn iterate_idv_child_idvs(
        &self,
        key: &str,
        opts: IdvSubresourceOptions,
    ) -> PageStream<Record> {
        iterate_idv_subresource(self, key.to_string(), "idvs", opts)
    }

    /// `GET /api/idvs/{key}/transactions/` — raw transaction history backing
    /// an IDV.
    pub async fn list_idv_transactions(
        &self,
        key: &str,
        opts: IdvSubresourceOptions,
    ) -> Result<Page<Record>> {
        list_idv_subresource(self, key, "transactions", opts).await
    }

    /// Stream every transaction backing `key`.
    pub fn iterate_idv_transactions(
        &self,
        key: &str,
        opts: IdvSubresourceOptions,
    ) -> PageStream<Record> {
        iterate_idv_subresource(self, key.to_string(), "transactions", opts)
    }

    /// `GET /api/idvs/{key}/lcats/` — Labor Categories (LCATs) under an IDV.
    pub async fn list_idv_lcats(
        &self,
        key: &str,
        opts: IdvSubresourceOptions,
    ) -> Result<Page<Record>> {
        list_idv_subresource(self, key, "lcats", opts).await
    }

    /// Stream every LCAT under `key`.
    pub fn iterate_idv_lcats(&self, key: &str, opts: IdvSubresourceOptions) -> PageStream<Record> {
        iterate_idv_subresource(self, key.to_string(), "lcats", opts)
    }
}

async fn list_idv_subresource(
    client: &Client,
    key: &str,
    segment: &str,
    opts: IdvSubresourceOptions,
) -> Result<Page<Record>> {
    if key.is_empty() {
        return Err(Error::Validation {
            message: "IDV sub-resource: key is required".into(),
            response: None,
        });
    }
    let q = opts.to_query();
    let path = format!("/api/idvs/{}/{segment}/", urlencoding(key));
    let bytes = client.get_bytes(&path, &q).await?;
    Page::decode(&bytes)
}

fn iterate_idv_subresource(
    client: &Client,
    key: String,
    segment: &'static str,
    opts: IdvSubresourceOptions,
) -> PageStream<Record> {
    let opts = Arc::new(opts);
    let key = Arc::new(key);
    let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
        let mut next = (*opts).clone();
        next.page = page;
        next.cursor = cursor;
        let key = key.clone();
        Box::pin(async move { list_idv_subresource(&client, &key, segment, next).await })
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
    fn options_emit_pagination_and_search() {
        let opts = IdvSubresourceOptions::builder()
            .limit(10u32)
            .ordering("-award_date")
            .search("software")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "limit").as_deref(), Some("10"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-award_date"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("software"));
    }

    #[test]
    fn joiner_only_when_flat() {
        let opts = IdvSubresourceOptions::builder()
            .joiner("__".to_string())
            .build();
        let q = opts.to_query();
        assert!(!q.iter().any(|(k, _)| k == "joiner"));

        let opts = IdvSubresourceOptions::builder()
            .flat(true)
            .joiner("__".to_string())
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("joiner".into(), "__".into())));
    }

    #[test]
    fn extra_forwards_arbitrary_params() {
        let mut extra = BTreeMap::new();
        extra.insert("region".to_string(), "west".to_string());
        let opts = IdvSubresourceOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert!(q.contains(&("region".into(), "west".into())));
    }

    #[tokio::test]
    async fn list_idv_awards_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_idv_awards("", IdvSubresourceOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("key")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_idv_lcats_empty_key_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client
            .list_idv_lcats("", IdvSubresourceOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("key")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
