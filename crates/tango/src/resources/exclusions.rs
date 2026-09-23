//! `GET /api/exclusions/` — SAM.gov exclusions (debarments, suspensions and other ineligibility records).
//!
//! Whether an exclusion is in force is derived at query time from its dates, so filter with `active` rather than trusting a stored flag.
//! Most exclusions are individuals and carry no UEI.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_exclusions`] and [`Client::iterate_exclusions`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListExclusionsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_EXCLUSIONS_MINIMAL`](crate::SHAPE_EXCLUSIONS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Whether the exclusion is in force as of today: not delisted, already activated, and not yet terminated. Derived at query time. `Some(false)` reaches the server as a filter value.
    pub active: Option<bool>,
    /// Whether the exclusion has dropped out of the SAM catalog because SAM lifted or withdrew it. Distinct from a natural expiration, which leaves the record listed.
    pub delisted: Option<bool>,
    /// Classification type (e.g. `Firm|Individual`).
    #[builder(into)]
    pub classification_type: Option<String>,
    /// Exclusion type. Supports OR via `|`.
    #[builder(into)]
    pub exclusion_type: Option<String>,
    /// Exclusion program (e.g. `Reciprocal`).
    #[builder(into)]
    pub exclusion_program: Option<String>,
    /// Code of the agency that issued the exclusion.
    #[builder(into)]
    pub excluding_agency_code: Option<String>,
    /// Name of the agency that issued the exclusion.
    #[builder(into)]
    pub excluding_agency_name: Option<String>,
    /// SAM UEI on the exclusion record.
    #[builder(into)]
    pub uei: Option<String>,
    /// CAGE code. Supports OR via `|`.
    #[builder(into)]
    pub cage_code: Option<String>,
    /// National Provider Identifier. Supports OR via `|`.
    #[builder(into)]
    pub npi: Option<String>,
    /// UEI of the linked Tango entity. Set only when the exclusion's UEI matches a registered entity.
    #[builder(into)]
    pub entity_uei: Option<String>,
    /// Activation date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub activate_date_after: Option<String>,
    /// Activation date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub activate_date_before: Option<String>,
    /// Termination date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub termination_date_after: Option<String>,
    /// Termination date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub termination_date_before: Option<String>,
    /// SAM update date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub update_date_after: Option<String>,
    /// SAM update date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub update_date_before: Option<String>,
    /// Full-text search across names, excluding agency, and comments.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `activate_date`, `termination_date`, `create_date`, `update_date`, `modified`, each optionally `-` prefixed.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListExclusionsOptions {
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
        push_opt_bool(&mut q, "active", self.active);
        push_opt_bool(&mut q, "delisted", self.delisted);
        push_opt(
            &mut q,
            "classification_type",
            self.classification_type.as_deref(),
        );
        push_opt(&mut q, "exclusion_type", self.exclusion_type.as_deref());
        push_opt(
            &mut q,
            "exclusion_program",
            self.exclusion_program.as_deref(),
        );
        push_opt(
            &mut q,
            "excluding_agency_code",
            self.excluding_agency_code.as_deref(),
        );
        push_opt(
            &mut q,
            "excluding_agency_name",
            self.excluding_agency_name.as_deref(),
        );
        push_opt(&mut q, "uei", self.uei.as_deref());
        push_opt(&mut q, "cage_code", self.cage_code.as_deref());
        push_opt(&mut q, "npi", self.npi.as_deref());
        push_opt(&mut q, "entity_uei", self.entity_uei.as_deref());
        push_opt(
            &mut q,
            "activate_date_after",
            self.activate_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "activate_date_before",
            self.activate_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "termination_date_after",
            self.termination_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "termination_date_before",
            self.termination_date_before.as_deref(),
        );
        push_opt(
            &mut q,
            "update_date_after",
            self.update_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "update_date_before",
            self.update_date_before.as_deref(),
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

/// Options for [`Client::get_exclusion`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetExclusionOptions {
    /// Shape selector. When empty, the server returns its default detail shape.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetExclusionOptions {
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
    /// `GET /api/exclusions/` — one page of exclusion records.
    pub async fn list_exclusions(&self, opts: ListExclusionsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/exclusions/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/exclusions/{exclusion_key}/` — a single exclusion.
    pub async fn get_exclusion(
        &self,
        exclusion_key: &str,
        opts: Option<GetExclusionOptions>,
    ) -> Result<Record> {
        if exclusion_key.is_empty() {
            return Err(Error::Validation {
                message: "get_exclusion: exclusion_key is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/exclusions/{}/", urlencoding(exclusion_key));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every exclusion matching `opts`.
    pub fn iterate_exclusions(&self, opts: ListExclusionsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_exclusions(next).await })
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
    fn list_exclusions_all_filters_emit() {
        let opts = ListExclusionsOptions::builder()
            .active(true)
            .delisted(true)
            .classification_type("v-classification_type")
            .exclusion_type("v-exclusion_type")
            .exclusion_program("v-exclusion_program")
            .excluding_agency_code("v-excluding_agency_code")
            .excluding_agency_name("v-excluding_agency_name")
            .uei("v-uei")
            .cage_code("v-cage_code")
            .npi("v-npi")
            .entity_uei("v-entity_uei")
            .activate_date_after("v-activate_date_after")
            .activate_date_before("v-activate_date_before")
            .termination_date_after("v-termination_date_after")
            .termination_date_before("v-termination_date_before")
            .update_date_after("v-update_date_after")
            .update_date_before("v-update_date_before")
            .search("v-search")
            .ordering("v-ordering")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "active").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "delisted").as_deref(), Some("true"));
        assert_eq!(
            get_q(&q, "classification_type").as_deref(),
            Some("v-classification_type")
        );
        assert_eq!(
            get_q(&q, "exclusion_type").as_deref(),
            Some("v-exclusion_type")
        );
        assert_eq!(
            get_q(&q, "exclusion_program").as_deref(),
            Some("v-exclusion_program")
        );
        assert_eq!(
            get_q(&q, "excluding_agency_code").as_deref(),
            Some("v-excluding_agency_code")
        );
        assert_eq!(
            get_q(&q, "excluding_agency_name").as_deref(),
            Some("v-excluding_agency_name")
        );
        assert_eq!(get_q(&q, "uei").as_deref(), Some("v-uei"));
        assert_eq!(get_q(&q, "cage_code").as_deref(), Some("v-cage_code"));
        assert_eq!(get_q(&q, "npi").as_deref(), Some("v-npi"));
        assert_eq!(get_q(&q, "entity_uei").as_deref(), Some("v-entity_uei"));
        assert_eq!(
            get_q(&q, "activate_date_after").as_deref(),
            Some("v-activate_date_after")
        );
        assert_eq!(
            get_q(&q, "activate_date_before").as_deref(),
            Some("v-activate_date_before")
        );
        assert_eq!(
            get_q(&q, "termination_date_after").as_deref(),
            Some("v-termination_date_after")
        );
        assert_eq!(
            get_q(&q, "termination_date_before").as_deref(),
            Some("v-termination_date_before")
        );
        assert_eq!(
            get_q(&q, "update_date_after").as_deref(),
            Some("v-update_date_after")
        );
        assert_eq!(
            get_q(&q, "update_date_before").as_deref(),
            Some("v-update_date_before")
        );
        assert_eq!(get_q(&q, "search").as_deref(), Some("v-search"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("v-ordering"));
        assert_eq!(q.len(), 19);
    }

    #[test]
    fn list_exclusions_active_false_is_a_filter_not_an_absence() {
        let opts = ListExclusionsOptions::builder().active(false).build();
        assert_eq!(get_q(&opts.to_query(), "active").as_deref(), Some("false"));
    }

    #[test]
    fn list_exclusions_zero_value_omitted() {
        assert!(ListExclusionsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn list_exclusions_cursor_wins_over_page_and_shape_emits() {
        let opts = ListExclusionsOptions::builder()
            .page(3u32)
            .cursor("c0".to_string())
            .shape(crate::SHAPE_EXCLUSIONS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_EXCLUSIONS_MINIMAL)
        );
    }

    #[test]
    fn list_exclusions_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let q = ListExclusionsOptions::builder()
            .extra(extra)
            .build()
            .to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_exclusion_validates_empty_exclusion_key() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_exclusion("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("exclusion_key is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn get_options_emits() {
        let q = GetExclusionOptions::builder()
            .shape("a,b")
            .flat(true)
            .flat_lists(true)
            .build()
            .to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("a,b"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists").as_deref(), Some("true"));
    }
}
