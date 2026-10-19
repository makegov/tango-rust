//! `GET /api/federal_register/` — Federal Register documents: rules, proposed rules, notices and presidential documents published since 1994 (Tango API 5.3.0).
//!
//! The detail route takes a document's `uuid`, not its `document_number`: the Federal Register reused some document numbers before 2016, so a number is not unique on its own.
//! Look a document up by number with [`ListFederalRegisterDocumentsOptions::document_number`].
//!
//! `full_text` can run to several MB. It is served on [`Client::get_federal_register_document`] only when named in `shape`, and neither default shape includes it.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_federal_register_documents`] and [`Client::iterate_federal_register_documents`].
///
/// All seventeen of the API's filters are named fields, plus `ordering`.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListFederalRegisterDocumentsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_FEDERAL_REGISTER_MINIMAL`](crate::SHAPE_FEDERAL_REGISTER_MINIMAL) or roll your own. `full_text` is not available on the list.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Ranked full-text search over the title, abstract and action. Wrap in double quotes for a phrase.
    #[builder(into)]
    pub search: Option<String>,
    /// Exact FR document number, e.g. `2016-31922`. OR several with `|`. A number the Federal Register reused before 2016 matches every document that carries it.
    #[builder(into)]
    pub document_number: Option<String>,
    /// Document type: `Notice`, `Rule`, `Proposed Rule`, `Presidential Document`, `Correction`, `Sunshine Act Document` or `Uncategorized Document` (case-insensitive). OR several with `|`. Documents published before 2008 are mostly `Uncategorized Document`.
    #[builder(into)]
    pub r#type: Option<String>,
    /// A Tango agency name, abbreviation, code or organization key, e.g. `EPA`. Matches a document when any agency it lists falls within that organization, so a department includes its sub-agencies. OR several with `|`.
    #[builder(into)]
    pub agency: Option<String>,
    /// The Federal Register's own agency slug, e.g. `environmental-protection-agency`. OR several with `|`.
    #[builder(into)]
    pub fr_agency: Option<String>,
    /// Published on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub publication_date_after: Option<String>,
    /// Published on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub publication_date_before: Option<String>,
    /// Effective on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub effective_on_after: Option<String>,
    /// Effective on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub effective_on_before: Option<String>,
    /// Comment period closes on or after (`YYYY-MM-DD`).
    #[builder(into)]
    pub comments_close_on_after: Option<String>,
    /// Comment period closes on or before (`YYYY-MM-DD`).
    #[builder(into)]
    pub comments_close_on_before: Option<String>,
    /// `Some(true)` for documents whose comment period closes today or later, `Some(false)` for those already closed. Documents with no comment deadline match neither.
    pub comments_open: Option<bool>,
    /// A CFR title number, e.g. `40`.
    #[builder(into)]
    pub cfr_title: Option<String>,
    /// A CFR part number, e.g. `52`. Requires [`cfr_title`](Self::cfr_title), and both must match the same CFR reference.
    #[builder(into)]
    pub cfr_part: Option<String>,
    /// Significant under Executive Order 12866.
    pub significant: Option<bool>,
    /// A Regulation Identifier Number, e.g. `2060-AV16`. OR several with `|`.
    #[builder(into)]
    pub rin: Option<String>,
    /// Exact executive order number.
    #[builder(into)]
    pub executive_order_number: Option<String>,

    /// Sort key: `publication_date` (the server default, as `-publication_date`), `effective_on`, `comments_close_on`, `document_number` or `rank`, prefixed with `-` for descending. `rank` requires a non-empty [`search`](Self::search).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListFederalRegisterDocumentsOptions {
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
        push_opt(&mut q, "document_number", self.document_number.as_deref());
        push_opt(&mut q, "type", self.r#type.as_deref());
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "fr_agency", self.fr_agency.as_deref());
        for (key, value) in [
            ("publication_date_after", &self.publication_date_after),
            ("publication_date_before", &self.publication_date_before),
            ("effective_on_after", &self.effective_on_after),
            ("effective_on_before", &self.effective_on_before),
            ("comments_close_on_after", &self.comments_close_on_after),
            ("comments_close_on_before", &self.comments_close_on_before),
        ] {
            push_opt(&mut q, key, value.as_deref());
        }
        push_opt_bool(&mut q, "comments_open", self.comments_open);
        push_opt(&mut q, "cfr_title", self.cfr_title.as_deref());
        push_opt(&mut q, "cfr_part", self.cfr_part.as_deref());
        push_opt_bool(&mut q, "significant", self.significant);
        push_opt(&mut q, "rin", self.rin.as_deref());
        push_opt(
            &mut q,
            "executive_order_number",
            self.executive_order_number.as_deref(),
        );
        push_opt(&mut q, "ordering", self.ordering.as_deref());
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_federal_register_document`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetFederalRegisterDocumentOptions {
    /// Shape selector. When empty, the server returns its detail default, which [`SHAPE_FEDERAL_REGISTER_COMPREHENSIVE`](crate::SHAPE_FEDERAL_REGISTER_COMPREHENSIVE) mirrors. Name `full_text` to get the document's plain text, which can run to several MB.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetFederalRegisterDocumentOptions {
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
    /// `GET /api/federal_register/` — one page of Federal Register documents, newest publication first by default.
    pub async fn list_federal_register_documents(
        &self,
        opts: ListFederalRegisterDocumentsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        self.get_page("/api/federal_register/", &q).await
    }

    /// `GET /api/federal_register/{uuid}/` — one Federal Register document by its `uuid`.
    ///
    /// The route takes the `uuid`, not the `document_number`, which is not unique on its own; find a document by number with [`list_federal_register_documents`](Self::list_federal_register_documents) and [`ListFederalRegisterDocumentsOptions::document_number`].
    /// `full_text` is returned only when named in `shape`.
    pub async fn get_federal_register_document(
        &self,
        document_uuid: &str,
        opts: Option<GetFederalRegisterDocumentOptions>,
    ) -> Result<Record> {
        if document_uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_federal_register_document: document_uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/federal_register/{}/", urlencoding(document_uuid));
        self.get_json::<Record>(&path, &q).await
    }

    /// Stream every Federal Register document matching `opts`.
    pub fn iterate_federal_register_documents(
        &self,
        opts: ListFederalRegisterDocumentsOptions,
    ) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_federal_register_documents(next).await })
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
    fn list_federal_register_documents_all_filters_emit() {
        let opts = ListFederalRegisterDocumentsOptions::builder()
            .search("\"air quality\"")
            .document_number("2016-31922")
            .r#type("Rule|Proposed Rule")
            .agency("EPA")
            .fr_agency("environmental-protection-agency")
            .publication_date_after("2024-01-01")
            .publication_date_before("2024-12-31")
            .effective_on_after("2024-02-01")
            .effective_on_before("2024-11-30")
            .comments_close_on_after("2024-03-01")
            .comments_close_on_before("2024-10-31")
            .comments_open(false)
            .cfr_title("40")
            .cfr_part("52")
            .significant(true)
            .rin("2060-AV16")
            .executive_order_number("14028")
            .ordering("rank")
            .build();
        let q = opts.to_query();
        for (k, v) in [
            ("search", "\"air quality\""),
            ("document_number", "2016-31922"),
            ("type", "Rule|Proposed Rule"),
            ("agency", "EPA"),
            ("fr_agency", "environmental-protection-agency"),
            ("publication_date_after", "2024-01-01"),
            ("publication_date_before", "2024-12-31"),
            ("effective_on_after", "2024-02-01"),
            ("effective_on_before", "2024-11-30"),
            ("comments_close_on_after", "2024-03-01"),
            ("comments_close_on_before", "2024-10-31"),
            ("comments_open", "false"),
            ("cfr_title", "40"),
            ("cfr_part", "52"),
            ("significant", "true"),
            ("rin", "2060-AV16"),
            ("executive_order_number", "14028"),
            ("ordering", "rank"),
        ] {
            assert_eq!(get_q(&q, k).as_deref(), Some(v), "{k}");
        }
        assert_eq!(q.len(), 18);
    }

    #[test]
    fn list_federal_register_documents_zero_value_omitted() {
        assert!(ListFederalRegisterDocumentsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn default_shapes_never_name_full_text() {
        for shape in [
            crate::SHAPE_FEDERAL_REGISTER_MINIMAL,
            crate::SHAPE_FEDERAL_REGISTER_COMPREHENSIVE,
        ] {
            assert!(!shape.split(',').any(|f| f == "full_text"), "{shape}");
        }
    }

    #[tokio::test]
    async fn empty_uuid_is_rejected_before_any_request() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client
            .get_federal_register_document("", None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }
}
