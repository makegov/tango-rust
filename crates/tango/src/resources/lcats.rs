//! LCATs (Labor Categories) — owner-scoped sub-resource dispatcher.
//!
//! Labor Categories never live at the top level: they're always scoped to
//! an entity (by UEI) or an IDV (by key). The Tango API exposes them at:
//!   - `GET /api/entities/{uei}/lcats/`
//!   - `GET /api/idvs/{key}/lcats/`
//!
//! [`Client::list_lcats`] is a thin dispatcher over those two paths — use
//! it as a single entry point when you want to vary the owner type at
//! runtime; otherwise call the sub-resource methods directly.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::entity_subresources::EntitySubresourceOptions;
use crate::resources::idv_subresources::IdvSubresourceOptions;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_lcats`] and [`Client::iterate_lcats`].
///
/// Exactly one of [`uei`](Self::uei) or [`idv_key`](Self::idv_key) must be
/// set, or the call returns [`Error::Validation`]. If both are set,
/// [`uei`](Self::uei) wins (mirrors the Go SDK).
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListLcatsOptions {
    /// Entity UEI — dispatches to `/api/entities/{uei}/lcats/`.
    #[builder(into)]
    pub uei: Option<String>,
    /// IDV key — dispatches to `/api/idvs/{key}/lcats/`.
    #[builder(into)]
    pub idv_key: Option<String>,

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

    // ----- Sub-resource filters -----
    /// Free-text search filter (labor category title, description).
    #[builder(into)]
    pub search: Option<String>,
    /// Server-side sort spec (prefix `-` for descending).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not first-classed here.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

/// Dispatch decision: which sub-resource path this `ListLcatsOptions`
/// resolves to.
///
/// Extracted into a helper so unit tests can verify the dispatch logic
/// without standing up a mock HTTP server.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LcatsTarget<'a> {
    Entity(&'a str),
    Idv(&'a str),
}

impl ListLcatsOptions {
    /// Resolve which sub-resource the caller's `uei` / `idv_key` selects.
    /// UEI wins when both are set; returns `None` when neither is set.
    fn target(&self) -> Option<LcatsTarget<'_>> {
        if let Some(uei) = self.uei.as_deref().filter(|s| !s.is_empty()) {
            return Some(LcatsTarget::Entity(uei));
        }
        if let Some(key) = self.idv_key.as_deref().filter(|s| !s.is_empty()) {
            return Some(LcatsTarget::Idv(key));
        }
        None
    }

    /// Project the dispatcher options onto the entity sub-resource shape.
    fn to_entity_opts(&self) -> EntitySubresourceOptions {
        EntitySubresourceOptions {
            page: self.page,
            limit: self.limit,
            cursor: self.cursor.clone(),
            shape: self.shape.clone(),
            flat: self.flat,
            flat_lists: self.flat_lists,
            joiner: None,
            ordering: self.ordering.clone(),
            search: self.search.clone(),
            extra: self.extra.clone(),
        }
    }

    /// Project the dispatcher options onto the IDV sub-resource shape.
    fn to_idv_opts(&self) -> IdvSubresourceOptions {
        IdvSubresourceOptions {
            page: self.page,
            limit: self.limit,
            cursor: self.cursor.clone(),
            shape: self.shape.clone(),
            flat: self.flat,
            flat_lists: self.flat_lists,
            joiner: None,
            ordering: self.ordering.clone(),
            search: self.search.clone(),
            extra: self.extra.clone(),
        }
    }

    /// Materialise the outgoing query-pair list for the dispatched call.
    /// Mirrors the entity sub-resource shape (Wave A); used by tests and
    /// by the iterate stream's fetch closure.
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

fn validation_missing_owner() -> Error {
    Error::Validation {
        message: "list_lcats: one of uei or idv_key is required".into(),
        response: None,
    }
}

impl Client {
    /// Dispatcher: list LCATs for the owner specified by `opts.uei` or
    /// `opts.idv_key`.
    ///
    /// - When `uei` is set: delegates to [`Client::list_entity_lcats`]
    ///   (`GET /api/entities/{uei}/lcats/`).
    /// - Else when `idv_key` is set: delegates to [`Client::list_idv_lcats`]
    ///   (`GET /api/idvs/{key}/lcats/`).
    /// - Else: returns [`Error::Validation`].
    ///
    /// When both are set, `uei` wins (mirrors the Go SDK).
    pub async fn list_lcats(&self, opts: ListLcatsOptions) -> Result<Page<Record>> {
        match opts.target().ok_or_else(validation_missing_owner)? {
            LcatsTarget::Entity(uei) => {
                let sub_opts = opts.to_entity_opts();
                self.list_entity_lcats(uei, sub_opts).await
            }
            LcatsTarget::Idv(key) => {
                let sub_opts = opts.to_idv_opts();
                self.list_idv_lcats(key, sub_opts).await
            }
        }
    }

    /// Stream every LCAT for the owner specified by `opts`. Same dispatch
    /// rules as [`Client::list_lcats`]; the validation error surfaces on
    /// the first poll.
    pub fn iterate_lcats(&self, opts: ListLcatsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_lcats(next).await })
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

    // ----- Dispatch decision -----

    #[test]
    fn target_uei_dispatches_to_entity() {
        let opts = ListLcatsOptions::builder().uei("UEI123").build();
        assert_eq!(opts.target(), Some(LcatsTarget::Entity("UEI123")));
    }

    #[test]
    fn target_idv_key_dispatches_to_idv() {
        let opts = ListLcatsOptions::builder().idv_key("IDV-001").build();
        assert_eq!(opts.target(), Some(LcatsTarget::Idv("IDV-001")));
    }

    #[test]
    fn target_both_set_uei_wins() {
        let opts = ListLcatsOptions::builder().uei("U1").idv_key("I1").build();
        assert_eq!(opts.target(), Some(LcatsTarget::Entity("U1")));
    }

    #[test]
    fn target_neither_set_returns_none() {
        let opts = ListLcatsOptions::default();
        assert!(opts.target().is_none());
    }

    #[test]
    fn target_treats_empty_strings_as_unset() {
        let opts = ListLcatsOptions::builder()
            .uei(String::new())
            .idv_key(String::new())
            .build();
        assert!(opts.target().is_none());
    }

    // ----- Validation surface -----

    #[tokio::test]
    async fn list_lcats_requires_owner_default() {
        let c = Client::builder().api_key("k").build().expect("client");
        let err = c
            .list_lcats(ListLcatsOptions::default())
            .await
            .expect_err("must error");
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uei") && message.contains("idv_key"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn list_lcats_requires_owner_explicit_empty() {
        let c = Client::builder().api_key("k").build().expect("client");
        let opts = ListLcatsOptions::builder()
            .uei(String::new())
            .idv_key(String::new())
            .build();
        let err = c.list_lcats(opts).await.expect_err("must error");
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ----- Forwarded filters -----

    #[test]
    fn forwards_search_and_ordering() {
        let opts = ListLcatsOptions::builder()
            .uei("UEI123")
            .search("software engineer")
            .ordering("labor_category")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("software engineer"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("labor_category"));
    }

    #[test]
    fn forwards_pagination_and_shape() {
        let opts = ListLcatsOptions::builder()
            .uei("UEI123")
            .page(2u32)
            .limit(50u32)
            .shape("labor_category,rate")
            .flat(true)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "page").as_deref(), Some("2"));
        assert_eq!(get_q(&q, "limit").as_deref(), Some("50"));
        assert_eq!(get_q(&q, "shape").as_deref(), Some("labor_category,rate"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
    }

    #[test]
    fn extra_keys_pass_through() {
        let mut extra = BTreeMap::new();
        extra.insert("custom".into(), "value".into());
        let opts = ListLcatsOptions::builder().uei("UEI1").extra(extra).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "custom").as_deref(), Some("value"));
    }
}
