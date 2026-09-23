//! `GET /api/sbir/` — SBIR/STTR topics and DoD DSIP solicitation cycles.
//!
//! `activity` (open/closed) is derived at query time from the close or end date; there is no stored flag.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_sbir_topics`] and [`Client::iterate_sbir_topics`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSbirTopicsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_SBIR_TOPICS_MINIMAL`](crate::SHAPE_SBIR_TOPICS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Whether the topic is accepting proposals today: `open`, `closed`, or `unknown` (no close date and no parent solicitation to inherit one from).
    #[builder(into)]
    pub activity: Option<String>,
    /// Partial, case-insensitive match on the raw agency text (e.g. `DOD`, `NIH`). Not organization-resolved, so variant spellings do not normalize.
    #[builder(into)]
    pub agency: Option<String>,
    /// Topic number.
    #[builder(into)]
    pub topic_number: Option<String>,
    /// Parent solicitation number.
    #[builder(into)]
    pub solicitation_number: Option<String>,
    /// Solicitation year.
    #[builder(into)]
    pub year: Option<String>,
    /// Document source.
    #[builder(into)]
    pub doc_source: Option<String>,
    /// Close date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub close_date_after: Option<String>,
    /// Close date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub close_date_before: Option<String>,
    /// Open date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub open_date_after: Option<String>,
    /// Open date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub open_date_before: Option<String>,
    /// Release date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub release_date_after: Option<String>,
    /// Release date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub release_date_before: Option<String>,
    /// Case-insensitive substring match across title, description, and topic number.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `activity`, `close_date`, `open_date`, `release_date`, `year`, `modified`, each optionally `-` prefixed.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListSbirTopicsOptions {
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
        push_opt(&mut q, "activity", self.activity.as_deref());
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "topic_number", self.topic_number.as_deref());
        push_opt(
            &mut q,
            "solicitation_number",
            self.solicitation_number.as_deref(),
        );
        push_opt(&mut q, "year", self.year.as_deref());
        push_opt(&mut q, "doc_source", self.doc_source.as_deref());
        push_opt(&mut q, "close_date_after", self.close_date_after.as_deref());
        push_opt(
            &mut q,
            "close_date_before",
            self.close_date_before.as_deref(),
        );
        push_opt(&mut q, "open_date_after", self.open_date_after.as_deref());
        push_opt(&mut q, "open_date_before", self.open_date_before.as_deref());
        push_opt(
            &mut q,
            "release_date_after",
            self.release_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "release_date_before",
            self.release_date_before.as_deref(),
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

/// Options for [`Client::list_sbir_solicitations`] and [`Client::iterate_sbir_solicitations`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListSbirSolicitationsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_SBIR_SOLICITATIONS_MINIMAL`](crate::SHAPE_SBIR_SOLICITATIONS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Whether the cycle is open today: `open` or `closed`.
    #[builder(into)]
    pub activity: Option<String>,
    /// Program (`SBIR` or `STTR`).
    #[builder(into)]
    pub program: Option<String>,
    /// Solicitation number.
    #[builder(into)]
    pub solicitation_number: Option<String>,
    /// Cycle name.
    #[builder(into)]
    pub cycle_name: Option<String>,
    /// Raw feed status.
    #[builder(into)]
    pub solicitation_status: Option<String>,
    /// Out-of-cycle solicitation flag. `Some(false)` reaches the server as a filter value.
    pub out_of_cycle: Option<bool>,
    /// Solicitation year.
    #[builder(into)]
    pub year: Option<String>,
    /// Cycle start date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub start_date_after: Option<String>,
    /// Cycle start date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub start_date_before: Option<String>,
    /// Cycle end date on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub end_date_after: Option<String>,
    /// Cycle end date on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub end_date_before: Option<String>,
    /// Case-insensitive substring match across title and solicitation number.
    #[builder(into)]
    pub search: Option<String>,
    /// One of `activity`, `start_date`, `end_date`, `year`, `modified`, each optionally `-` prefixed.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListSbirSolicitationsOptions {
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
        push_opt(&mut q, "activity", self.activity.as_deref());
        push_opt(&mut q, "program", self.program.as_deref());
        push_opt(
            &mut q,
            "solicitation_number",
            self.solicitation_number.as_deref(),
        );
        push_opt(&mut q, "cycle_name", self.cycle_name.as_deref());
        push_opt(
            &mut q,
            "solicitation_status",
            self.solicitation_status.as_deref(),
        );
        push_opt_bool(&mut q, "out_of_cycle", self.out_of_cycle);
        push_opt(&mut q, "year", self.year.as_deref());
        push_opt(&mut q, "start_date_after", self.start_date_after.as_deref());
        push_opt(
            &mut q,
            "start_date_before",
            self.start_date_before.as_deref(),
        );
        push_opt(&mut q, "end_date_after", self.end_date_after.as_deref());
        push_opt(&mut q, "end_date_before", self.end_date_before.as_deref());
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

/// Options for [`Client::get_sbir_topic`], [`Client::get_sbir_solicitation`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetSbirOptions {
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

impl GetSbirOptions {
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
    /// `GET /api/sbir/topics/` — one page of SBIR/STTR topic records.
    pub async fn list_sbir_topics(&self, opts: ListSbirTopicsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/sbir/topics/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/sbir/topics/{topic_id}/` — a single SBIR/STTR topic.
    pub async fn get_sbir_topic(
        &self,
        topic_id: &str,
        opts: Option<GetSbirOptions>,
    ) -> Result<Record> {
        if topic_id.is_empty() {
            return Err(Error::Validation {
                message: "get_sbir_topic: topic_id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/sbir/topics/{}/", urlencoding(topic_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every SBIR/STTR topic matching `opts`.
    pub fn iterate_sbir_topics(&self, opts: ListSbirTopicsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_sbir_topics(next).await })
        });
        PageStream::new(self.clone(), fetch)
    }

    /// `GET /api/sbir/solicitations/` — one page of DoD DSIP solicitation cycle records.
    pub async fn list_sbir_solicitations(
        &self,
        opts: ListSbirSolicitationsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/sbir/solicitations/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/sbir/solicitations/{solicitation_id}/` — a single DoD DSIP solicitation cycle.
    pub async fn get_sbir_solicitation(
        &self,
        solicitation_id: &str,
        opts: Option<GetSbirOptions>,
    ) -> Result<Record> {
        if solicitation_id.is_empty() {
            return Err(Error::Validation {
                message: "get_sbir_solicitation: solicitation_id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/sbir/solicitations/{}/", urlencoding(solicitation_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every DoD DSIP solicitation cycle matching `opts`.
    pub fn iterate_sbir_solicitations(
        &self,
        opts: ListSbirSolicitationsOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_sbir_solicitations(next).await })
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
    fn list_sbir_topics_all_filters_emit() {
        let opts = ListSbirTopicsOptions::builder()
            .activity("v-activity")
            .agency("v-agency")
            .topic_number("v-topic_number")
            .solicitation_number("v-solicitation_number")
            .year("v-year")
            .doc_source("v-doc_source")
            .close_date_after("v-close_date_after")
            .close_date_before("v-close_date_before")
            .open_date_after("v-open_date_after")
            .open_date_before("v-open_date_before")
            .release_date_after("v-release_date_after")
            .release_date_before("v-release_date_before")
            .search("v-search")
            .ordering("v-ordering")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "activity").as_deref(), Some("v-activity"));
        assert_eq!(get_q(&q, "agency").as_deref(), Some("v-agency"));
        assert_eq!(get_q(&q, "topic_number").as_deref(), Some("v-topic_number"));
        assert_eq!(
            get_q(&q, "solicitation_number").as_deref(),
            Some("v-solicitation_number")
        );
        assert_eq!(get_q(&q, "year").as_deref(), Some("v-year"));
        assert_eq!(get_q(&q, "doc_source").as_deref(), Some("v-doc_source"));
        assert_eq!(
            get_q(&q, "close_date_after").as_deref(),
            Some("v-close_date_after")
        );
        assert_eq!(
            get_q(&q, "close_date_before").as_deref(),
            Some("v-close_date_before")
        );
        assert_eq!(
            get_q(&q, "open_date_after").as_deref(),
            Some("v-open_date_after")
        );
        assert_eq!(
            get_q(&q, "open_date_before").as_deref(),
            Some("v-open_date_before")
        );
        assert_eq!(
            get_q(&q, "release_date_after").as_deref(),
            Some("v-release_date_after")
        );
        assert_eq!(
            get_q(&q, "release_date_before").as_deref(),
            Some("v-release_date_before")
        );
        assert_eq!(get_q(&q, "search").as_deref(), Some("v-search"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("v-ordering"));
        assert_eq!(q.len(), 14);
    }

    #[test]
    fn list_sbir_topics_zero_value_omitted() {
        assert!(ListSbirTopicsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn list_sbir_topics_cursor_wins_over_page_and_shape_emits() {
        let opts = ListSbirTopicsOptions::builder()
            .page(3u32)
            .cursor("c0".to_string())
            .shape(crate::SHAPE_SBIR_TOPICS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_SBIR_TOPICS_MINIMAL)
        );
    }

    #[test]
    fn list_sbir_topics_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let q = ListSbirTopicsOptions::builder()
            .extra(extra)
            .build()
            .to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_sbir_topic_validates_empty_topic_id() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_sbir_topic("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("topic_id is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn list_sbir_solicitations_all_filters_emit() {
        let opts = ListSbirSolicitationsOptions::builder()
            .activity("v-activity")
            .program("v-program")
            .solicitation_number("v-solicitation_number")
            .cycle_name("v-cycle_name")
            .solicitation_status("v-solicitation_status")
            .out_of_cycle(true)
            .year("v-year")
            .start_date_after("v-start_date_after")
            .start_date_before("v-start_date_before")
            .end_date_after("v-end_date_after")
            .end_date_before("v-end_date_before")
            .search("v-search")
            .ordering("v-ordering")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "activity").as_deref(), Some("v-activity"));
        assert_eq!(get_q(&q, "program").as_deref(), Some("v-program"));
        assert_eq!(
            get_q(&q, "solicitation_number").as_deref(),
            Some("v-solicitation_number")
        );
        assert_eq!(get_q(&q, "cycle_name").as_deref(), Some("v-cycle_name"));
        assert_eq!(
            get_q(&q, "solicitation_status").as_deref(),
            Some("v-solicitation_status")
        );
        assert_eq!(get_q(&q, "out_of_cycle").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "year").as_deref(), Some("v-year"));
        assert_eq!(
            get_q(&q, "start_date_after").as_deref(),
            Some("v-start_date_after")
        );
        assert_eq!(
            get_q(&q, "start_date_before").as_deref(),
            Some("v-start_date_before")
        );
        assert_eq!(
            get_q(&q, "end_date_after").as_deref(),
            Some("v-end_date_after")
        );
        assert_eq!(
            get_q(&q, "end_date_before").as_deref(),
            Some("v-end_date_before")
        );
        assert_eq!(get_q(&q, "search").as_deref(), Some("v-search"));
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("v-ordering"));
        assert_eq!(q.len(), 13);
    }

    #[test]
    fn list_sbir_solicitations_out_of_cycle_false_is_a_filter_not_an_absence() {
        let opts = ListSbirSolicitationsOptions::builder()
            .out_of_cycle(false)
            .build();
        assert_eq!(
            get_q(&opts.to_query(), "out_of_cycle").as_deref(),
            Some("false")
        );
    }

    #[test]
    fn list_sbir_solicitations_zero_value_omitted() {
        assert!(ListSbirSolicitationsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn list_sbir_solicitations_cursor_wins_over_page_and_shape_emits() {
        let opts = ListSbirSolicitationsOptions::builder()
            .page(3u32)
            .cursor("c0".to_string())
            .shape(crate::SHAPE_SBIR_SOLICITATIONS_MINIMAL)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("c0"));
        assert_eq!(get_q(&q, "page"), None);
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_SBIR_SOLICITATIONS_MINIMAL)
        );
    }

    #[test]
    fn list_sbir_solicitations_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let q = ListSbirSolicitationsOptions::builder()
            .extra(extra)
            .build()
            .to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[tokio::test]
    async fn get_sbir_solicitation_validates_empty_solicitation_id() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_sbir_solicitation("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("solicitation_id is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn get_options_emits() {
        let q = GetSbirOptions::builder()
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
