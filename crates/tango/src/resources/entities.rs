//! `GET /api/entities/` — list and stream federal vendor / recipient records.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_entities`] and [`Client::iterate_entities`].
///
/// Mirrors the Go SDK's `ListEntitiesOptions`.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListEntitiesOptions {
    // ----- Pagination + shape -----
    /// 1-based page number. Mutually exclusive with [`cursor`](Self::cursor).
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

    // ----- Resource filters -----
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,
    /// CAGE code filter.
    #[builder(into)]
    pub cage_code: Option<String>,
    /// NAICS code filter.
    #[builder(into)]
    pub naics: Option<String>,
    /// Entity name filter.
    #[builder(into)]
    pub name: Option<String>,
    /// PSC code filter.
    #[builder(into)]
    pub psc: Option<String>,
    /// SAM purpose-of-registration code.
    #[builder(into)]
    pub purpose_of_registration_code: Option<String>,
    /// Socioeconomic indicator filter.
    #[builder(into)]
    pub socioeconomic: Option<String>,
    /// US state filter (2-letter code).
    #[builder(into)]
    pub state: Option<String>,
    /// Lower bound for total obligated awards (dollars).
    #[builder(into)]
    pub total_awards_obligated_gte: Option<String>,
    /// Upper bound for total obligated awards (dollars).
    #[builder(into)]
    pub total_awards_obligated_lte: Option<String>,
    /// UEI filter.
    #[builder(into)]
    pub uei: Option<String>,
    /// ZIP code filter.
    #[builder(into)]
    pub zip_code: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListEntitiesOptions {
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
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "cage_code", self.cage_code.as_deref());
        push_opt(&mut q, "naics", self.naics.as_deref());
        push_opt(&mut q, "name", self.name.as_deref());
        push_opt(&mut q, "psc", self.psc.as_deref());
        push_opt(
            &mut q,
            "purpose_of_registration_code",
            self.purpose_of_registration_code.as_deref(),
        );
        push_opt(&mut q, "socioeconomic", self.socioeconomic.as_deref());
        push_opt(&mut q, "state", self.state.as_deref());
        push_opt(
            &mut q,
            "total_awards_obligated_gte",
            self.total_awards_obligated_gte.as_deref(),
        );
        push_opt(
            &mut q,
            "total_awards_obligated_lte",
            self.total_awards_obligated_lte.as_deref(),
        );
        push_opt(&mut q, "uei", self.uei.as_deref());
        push_opt(&mut q, "zip_code", self.zip_code.as_deref());

        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_entity`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetEntityOptions {
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

impl GetEntityOptions {
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
    /// `GET /api/entities/` — one page of entity records.
    pub async fn list_entities(&self, opts: ListEntitiesOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/entities/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/entities/{uei}/` — fetch a single entity by UEI or CAGE code.
    pub async fn get_entity(&self, uei: &str, opts: Option<GetEntityOptions>) -> Result<Record> {
        if uei.is_empty() {
            return Err(Error::Validation {
                message: "get_entity: uei is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/entities/{}/", urlencoding(uei));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every entity record matching `opts`.
    pub fn iterate_entities(&self, opts: ListEntitiesOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_entities(next).await })
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
    fn list_entities_all_filters_emit() {
        let opts = ListEntitiesOptions::builder()
            .search("Acme")
            .cage_code("1ABC5")
            .naics("541512")
            .name("Acme Corp")
            .psc("D302")
            .purpose_of_registration_code("Z1")
            .socioeconomic("A5")
            .state("VA")
            .total_awards_obligated_gte("100000")
            .total_awards_obligated_lte("999999")
            .uei("UEI123456789")
            .zip_code("22201")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "search").as_deref(), Some("Acme"));
        assert_eq!(get_q(&q, "cage_code").as_deref(), Some("1ABC5"));
        assert_eq!(get_q(&q, "naics").as_deref(), Some("541512"));
        assert_eq!(get_q(&q, "name").as_deref(), Some("Acme Corp"));
        assert_eq!(get_q(&q, "psc").as_deref(), Some("D302"));
        assert_eq!(
            get_q(&q, "purpose_of_registration_code").as_deref(),
            Some("Z1")
        );
        assert_eq!(get_q(&q, "socioeconomic").as_deref(), Some("A5"));
        assert_eq!(get_q(&q, "state").as_deref(), Some("VA"));
        assert_eq!(
            get_q(&q, "total_awards_obligated_gte").as_deref(),
            Some("100000")
        );
        assert_eq!(
            get_q(&q, "total_awards_obligated_lte").as_deref(),
            Some("999999")
        );
        assert_eq!(get_q(&q, "uei").as_deref(), Some("UEI123456789"));
        assert_eq!(get_q(&q, "zip_code").as_deref(), Some("22201"));
    }

    #[test]
    fn list_entities_zero_value_omitted() {
        let opts = ListEntitiesOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_entities_pagination_applied() {
        let opts = ListEntitiesOptions::builder()
            .page(2u32)
            .limit(25u32)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "page").as_deref(), Some("2"));
        assert_eq!(get_q(&q, "limit").as_deref(), Some("25"));
    }

    #[test]
    fn get_entity_opts_emits_shape_and_flat() {
        let opts = GetEntityOptions::builder()
            .shape(crate::SHAPE_ENTITIES_MINIMAL)
            .flat(true)
            .flat_lists(true)
            .build();
        let q = opts.to_query();
        assert!(q.contains(&("shape".into(), crate::SHAPE_ENTITIES_MINIMAL.into())));
        assert!(q.contains(&("flat".into(), "true".into())));
        assert!(q.contains(&("flat_lists".into(), "true".into())));
    }

    #[tokio::test]
    async fn get_entity_empty_uei_returns_validation() {
        let client = Client::builder().api_key("x").build().expect("build");
        let err = client.get_entity("", None).await.expect_err("must error");
        match err {
            Error::Validation { message, .. } => assert!(message.contains("uei")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
