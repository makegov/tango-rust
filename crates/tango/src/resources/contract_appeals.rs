//! `GET /api/contract_appeals/` — Contract Disputes Act appeal decisions from the Civilian Board of Contract Appeals (CBCA) and the Armed Services Board of Contract Appeals (ASBCA).
//!
//! These are disputes under an existing contract — claims, terminations, delays, defective specifications — decided by a board.
//! They are NOT bid protests, which challenge an award before performance and live on [`Client::list_protests`](crate::Client::list_protests).
//! A company can appear in both corpora, and nothing joins the two.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt, push_opt_bool};
use crate::models::ContractAppealRecord;
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_contract_appeals`] and [`Client::iterate_contract_appeals`].
///
/// The `_after` / `_before` date suffixes mirror the Python SDK's naming.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListContractAppealsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use
    /// [`SHAPE_CONTRACT_APPEALS_MINIMAL`](crate::SHAPE_CONTRACT_APPEALS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Full-text search over the decision.
    /// Required for `ordering = "rank"`, which is otherwise meaningless.
    #[builder(into)]
    pub search: Option<String>,
    /// Deciding board: `"cbca"` (civilian) or `"asbca"` (defense).
    #[builder(into)]
    pub board: Option<String>,
    /// Docket number.
    /// A consolidated decision carries several, and this matches any one of them.
    #[builder(into)]
    pub docket: Option<String>,
    /// The contractor bringing the appeal.
    #[builder(into)]
    pub appellant: Option<String>,
    /// The judge who wrote the decision.
    #[builder(into)]
    pub judge: Option<String>,
    /// Normalized decision kind.
    #[builder(into)]
    pub decision_type: Option<String>,
    /// Whether the decision is still on a current board listing.
    /// `Option<bool>` so that `false` is a real filter value rather than an absent one.
    #[builder(into)]
    pub listed: Option<bool>,
    /// The board's identifier for the decision document.
    #[builder(into)]
    pub document_id: Option<String>,

    /// Lower bound on `decision_date` (ISO `YYYY-MM-DD`, inclusive).
    #[builder(into)]
    pub decision_date_after: Option<String>,
    /// Upper bound on `decision_date` (inclusive).
    #[builder(into)]
    pub decision_date_before: Option<String>,

    /// One of `decision_date`, `appellant`, `first_listed_at`, `rank`, each optionally `-` prefixed.
    /// The server defaults to `-decision_date`. `rank` requires a non-empty [`search`](Self::search).
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListContractAppealsOptions {
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
        push_opt(&mut q, "board", self.board.as_deref());
        push_opt(&mut q, "docket", self.docket.as_deref());
        push_opt(&mut q, "appellant", self.appellant.as_deref());
        push_opt(&mut q, "judge", self.judge.as_deref());
        push_opt(&mut q, "decision_type", self.decision_type.as_deref());
        push_opt_bool(&mut q, "listed", self.listed);
        push_opt(&mut q, "document_id", self.document_id.as_deref());
        push_opt(
            &mut q,
            "decision_date_after",
            self.decision_date_after.as_deref(),
        );
        push_opt(
            &mut q,
            "decision_date_before",
            self.decision_date_before.as_deref(),
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

/// Options for [`Client::get_contract_appeal`].
/// The detail endpoint returns a typed [`ContractAppealRecord`]; `shape` lets callers override the server's default.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetContractAppealOptions {
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

impl GetContractAppealOptions {
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
    /// `GET /api/contract_appeals/` — one page of board-of-contract-appeals decisions.
    pub async fn list_contract_appeals(
        &self,
        opts: ListContractAppealsOptions,
    ) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/contract_appeals/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/contract_appeals/{uuid}/` — fetch a single decision by UUID.
    ///
    /// Returns a typed [`ContractAppealRecord`]; forward-compatible server fields land in
    /// [`ContractAppealRecord::extra`].
    ///
    /// `decision_text` — the full text of the decision — needs an Enterprise plan, and below that its key is ABSENT rather than null.
    /// So `record.decision_text == None` means "not served to this caller", never "this decision has no text"; `text_status` and `text_char_count` answer that at every plan.
    pub async fn get_contract_appeal(
        &self,
        uuid: &str,
        opts: Option<GetContractAppealOptions>,
    ) -> Result<ContractAppealRecord> {
        if uuid.is_empty() {
            return Err(Error::Validation {
                message: "get_contract_appeal: uuid is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/contract_appeals/{}/", urlencoding(uuid));
        self.get_json::<ContractAppealRecord>(&path, &q).await
    }

    /// Stream every contract-appeal decision matching `opts`.
    pub fn iterate_contract_appeals(&self, opts: ListContractAppealsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_contract_appeals(next).await })
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
    fn list_contract_appeals_all_filters_emit() {
        let opts = ListContractAppealsOptions::builder()
            .search("differing site conditions")
            .board("cbca")
            .docket("CBCA 1234")
            .appellant("Acme Construction")
            .judge("Smith")
            .decision_type("decision")
            .listed(true)
            .document_id("doc-9912")
            .decision_date_after("2024-01-01")
            .decision_date_before("2024-12-31")
            .ordering("-decision_date")
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "search").as_deref(),
            Some("differing site conditions")
        );
        assert_eq!(get_q(&q, "board").as_deref(), Some("cbca"));
        assert_eq!(get_q(&q, "docket").as_deref(), Some("CBCA 1234"));
        assert_eq!(get_q(&q, "appellant").as_deref(), Some("Acme Construction"));
        assert_eq!(get_q(&q, "judge").as_deref(), Some("Smith"));
        assert_eq!(get_q(&q, "decision_type").as_deref(), Some("decision"));
        assert_eq!(get_q(&q, "listed").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "document_id").as_deref(), Some("doc-9912"));
        assert_eq!(
            get_q(&q, "decision_date_after").as_deref(),
            Some("2024-01-01")
        );
        assert_eq!(
            get_q(&q, "decision_date_before").as_deref(),
            Some("2024-12-31")
        );
        assert_eq!(get_q(&q, "ordering").as_deref(), Some("-decision_date"));
    }

    #[test]
    fn list_contract_appeals_listed_false_is_a_filter_not_an_absence() {
        let opts = ListContractAppealsOptions::builder().listed(false).build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "listed").as_deref(), Some("false"));
    }

    #[test]
    fn list_contract_appeals_zero_value_omitted() {
        let opts = ListContractAppealsOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_contract_appeals_cursor_wins_over_page() {
        let opts = ListContractAppealsOptions::builder()
            .page(2u32)
            .cursor("xyz".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("xyz"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[test]
    fn list_contract_appeals_shape_emits() {
        let opts = ListContractAppealsOptions::builder()
            .shape(crate::SHAPE_CONTRACT_APPEALS_MINIMAL)
            .flat(true)
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_CONTRACT_APPEALS_MINIMAL)
        );
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
    }

    #[test]
    fn list_contract_appeals_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let opts = ListContractAppealsOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[test]
    fn minimal_shape_does_not_name_the_paid_decision_text() {
        assert!(!crate::SHAPE_CONTRACT_APPEALS_MINIMAL.contains("decision_text"));
    }

    #[test]
    fn get_contract_appeal_options_emit() {
        let opts = GetContractAppealOptions::builder()
            .shape("uuid,decision_text")
            .flat(true)
            .flat_lists(true)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("uuid,decision_text"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists").as_deref(), Some("true"));
    }

    #[test]
    fn contract_appeal_record_decodes_from_sample_json() {
        let value = serde_json::json!({
            "uuid": "3f2a6c1e-8b7d-4a21-9f00-0d1c2e3b4a55",
            "board": "cbca",
            "docket_numbers": ["CBCA 1234", "CBCA 1235"],
            "docket_source": "listing",
            "docket_raw": "CBCA 1234, 1235",
            "decision_date": "2024-04-20",
            "decision_date_raw": "April 20, 2024",
            "decision_date_repaired": false,
            "appellant": "Acme Construction",
            "judge": "Smith",
            "decision_type": "decision",
            "decision_type_raw": "DECISION",
            "url": "https://example.gov/decisions/1234.pdf",
            "document_id": "doc-9912",
            "listing_url": "https://example.gov/decisions/2024",
            "listing_year": 2024,
            "first_listed_at": "2024-04-21T06:15:00Z",
            "listed": true,
            "text_status": "extracted",
            "text_char_count": 48213,
            "decision_text": "The appeal is sustained.",
            "future_field": "still here"
        });
        let rec: ContractAppealRecord = serde_json::from_value(value).expect("decode");
        assert_eq!(
            rec.uuid.as_deref(),
            Some("3f2a6c1e-8b7d-4a21-9f00-0d1c2e3b4a55")
        );
        assert_eq!(rec.board.as_deref(), Some("cbca"));
        assert_eq!(
            rec.docket_numbers.as_deref(),
            Some(["CBCA 1234".to_string(), "CBCA 1235".to_string()].as_slice())
        );
        assert_eq!(rec.docket_source.as_deref(), Some("listing"));
        assert_eq!(rec.docket_raw.as_deref(), Some("CBCA 1234, 1235"));
        assert_eq!(rec.decision_date.as_deref(), Some("2024-04-20"));
        assert_eq!(rec.decision_date_raw.as_deref(), Some("April 20, 2024"));
        assert_eq!(rec.decision_date_repaired, Some(false));
        assert_eq!(rec.appellant.as_deref(), Some("Acme Construction"));
        assert_eq!(rec.judge.as_deref(), Some("Smith"));
        assert_eq!(rec.decision_type.as_deref(), Some("decision"));
        assert_eq!(rec.decision_type_raw.as_deref(), Some("DECISION"));
        assert_eq!(
            rec.url.as_deref(),
            Some("https://example.gov/decisions/1234.pdf")
        );
        assert_eq!(rec.document_id.as_deref(), Some("doc-9912"));
        assert_eq!(
            rec.listing_url.as_deref(),
            Some("https://example.gov/decisions/2024")
        );
        assert_eq!(rec.listing_year, Some(2024));
        assert_eq!(rec.first_listed_at.as_deref(), Some("2024-04-21T06:15:00Z"));
        assert_eq!(rec.listed, Some(true));
        assert_eq!(rec.text_status.as_deref(), Some("extracted"));
        assert_eq!(rec.text_char_count, Some(48213));
        assert_eq!(
            rec.decision_text.as_deref(),
            Some("The appeal is sustained.")
        );
        // Unknown / not-first-classed fields land in `extra` via #[serde(flatten)].
        assert_eq!(
            rec.extra.get("future_field").and_then(|v| v.as_str()),
            Some("still here")
        );
    }

    #[test]
    fn contract_appeal_record_decodes_without_decision_text() {
        let value = serde_json::json!({
            "uuid": "3f2a6c1e-8b7d-4a21-9f00-0d1c2e3b4a55",
            "board": "asbca",
            "decision_date": "2024-04-20",
            "appellant": "Acme Construction",
            "text_status": "extracted",
            "text_char_count": 48213
        });
        let rec: ContractAppealRecord = serde_json::from_value(value).expect("decode");
        // Below Enterprise the key is absent, not null — and the record still reports that text exists.
        assert_eq!(rec.decision_text, None);
        assert!(!rec.extra.contains_key("decision_text"));
        assert_eq!(rec.text_status.as_deref(), Some("extracted"));
        assert_eq!(rec.text_char_count, Some(48213));
        assert_eq!(rec.docket_numbers, None);
    }

    #[test]
    fn contract_appeal_record_round_trips_without_reintroducing_absent_keys() {
        let rec = ContractAppealRecord {
            uuid: Some("abc".into()),
            board: Some("cbca".into()),
            ..Default::default()
        };
        let value = serde_json::to_value(&rec).expect("serialize");
        let obj = value.as_object().expect("object");
        assert_eq!(obj.len(), 2);
        assert!(!obj.contains_key("decision_text"));
    }

    #[tokio::test]
    async fn get_contract_appeal_validates_empty_uuid() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_contract_appeal("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("uuid is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
