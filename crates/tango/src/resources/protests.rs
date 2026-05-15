//! `GET /api/protests/` — bid-protest records from GAO and the U.S. Court of
//! Federal Claims.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::models::ProtestRecord;
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_protests`] and [`Client::iterate_protests`].
///
/// Mirrors the Go SDK's `ListProtestsOptions`. The protests viewset does not
/// accept server-side ordering — passing it returns 400 — so there's no
/// `ordering` field. The `_after` / `_before` date suffixes mirror the
/// Python SDK's naming.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListProtestsOptions {
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
    /// [`SHAPE_PROTESTS_MINIMAL`](crate::SHAPE_PROTESTS_MINIMAL) or roll your own.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Source system filter (`"GAO"` or `"COFC"`).
    #[builder(into)]
    pub source_system: Option<String>,
    /// Outcome label (`"sustained"`, `"denied"`, …).
    #[builder(into)]
    pub outcome: Option<String>,
    /// Case type filter.
    #[builder(into)]
    pub case_type: Option<String>,
    /// Agency filter (CGAC code or name, depending on source system).
    #[builder(into)]
    pub agency: Option<String>,
    /// Source-system case number filter.
    #[builder(into)]
    pub case_number: Option<String>,
    /// Solicitation number filter.
    #[builder(into)]
    pub solicitation_number: Option<String>,
    /// Protester name filter.
    #[builder(into)]
    pub protester: Option<String>,
    /// Free-text search filter.
    #[builder(into)]
    pub search: Option<String>,

    /// Lower bound on `filed_date` (ISO `YYYY-MM-DD`, inclusive).
    #[builder(into)]
    pub filed_date_after: Option<String>,
    /// Upper bound on `filed_date` (inclusive).
    #[builder(into)]
    pub filed_date_before: Option<String>,
    /// Lower bound on `decision_date` (inclusive).
    #[builder(into)]
    pub decision_date_after: Option<String>,
    /// Upper bound on `decision_date` (inclusive).
    #[builder(into)]
    pub decision_date_before: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListProtestsOptions {
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
        push_opt(&mut q, "source_system", self.source_system.as_deref());
        push_opt(&mut q, "outcome", self.outcome.as_deref());
        push_opt(&mut q, "case_type", self.case_type.as_deref());
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "case_number", self.case_number.as_deref());
        push_opt(
            &mut q,
            "solicitation_number",
            self.solicitation_number.as_deref(),
        );
        push_opt(&mut q, "protester", self.protester.as_deref());
        push_opt(&mut q, "search", self.search.as_deref());
        push_opt(&mut q, "filed_date_after", self.filed_date_after.as_deref());
        push_opt(
            &mut q,
            "filed_date_before",
            self.filed_date_before.as_deref(),
        );
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
        for (k, v) in &self.extra {
            if !v.is_empty() {
                q.push((k.clone(), v.clone()));
            }
        }
        q
    }
}

/// Options for [`Client::get_protest`]. The detail endpoint returns a typed
/// [`ProtestRecord`]; `shape` lets callers override the server's default.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetProtestOptions {
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

impl GetProtestOptions {
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
    /// `GET /api/protests/` — one page of bid-protest records.
    pub async fn list_protests(&self, opts: ListProtestsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/protests/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/protests/{case_id}/` — fetch a single protest by case ID.
    ///
    /// Returns a typed [`ProtestRecord`]; forward-compatible server fields land
    /// in [`ProtestRecord::extra`].
    pub async fn get_protest(
        &self,
        case_id: &str,
        opts: Option<GetProtestOptions>,
    ) -> Result<ProtestRecord> {
        if case_id.is_empty() {
            return Err(Error::Validation {
                message: "get_protest: case_id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/protests/{}/", urlencoding(case_id));
        self.get_json::<ProtestRecord>(&path, &q).await
    }

    /// Stream every protest matching `opts`.
    pub fn iterate_protests(&self, opts: ListProtestsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_protests(next).await })
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
    fn list_protests_all_filters_emit() {
        let opts = ListProtestsOptions::builder()
            .source_system("GAO")
            .outcome("sustained")
            .case_type("Bid Protest")
            .agency("9700")
            .case_number("B-12345.1")
            .solicitation_number("SOL-001")
            .protester("Acme Corp")
            .search("infrastructure")
            .filed_date_after("2024-01-01")
            .filed_date_before("2024-12-31")
            .decision_date_after("2024-02-01")
            .decision_date_before("2024-12-31")
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "source_system").as_deref(), Some("GAO"));
        assert_eq!(get_q(&q, "outcome").as_deref(), Some("sustained"));
        assert_eq!(get_q(&q, "case_type").as_deref(), Some("Bid Protest"));
        assert_eq!(get_q(&q, "agency").as_deref(), Some("9700"));
        assert_eq!(get_q(&q, "case_number").as_deref(), Some("B-12345.1"));
        assert_eq!(get_q(&q, "solicitation_number").as_deref(), Some("SOL-001"));
        assert_eq!(get_q(&q, "protester").as_deref(), Some("Acme Corp"));
        assert_eq!(get_q(&q, "search").as_deref(), Some("infrastructure"));
        assert_eq!(get_q(&q, "filed_date_after").as_deref(), Some("2024-01-01"));
        assert_eq!(
            get_q(&q, "filed_date_before").as_deref(),
            Some("2024-12-31")
        );
        assert_eq!(
            get_q(&q, "decision_date_after").as_deref(),
            Some("2024-02-01")
        );
        assert_eq!(
            get_q(&q, "decision_date_before").as_deref(),
            Some("2024-12-31")
        );
    }

    #[test]
    fn list_protests_zero_value_omitted() {
        let opts = ListProtestsOptions::builder().build();
        let q = opts.to_query();
        assert!(q.is_empty(), "expected empty query, got {q:?}");
    }

    #[test]
    fn list_protests_cursor_wins_over_page() {
        let opts = ListProtestsOptions::builder()
            .page(2u32)
            .cursor("xyz".to_string())
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "cursor").as_deref(), Some("xyz"));
        assert_eq!(get_q(&q, "page"), None);
    }

    #[test]
    fn list_protests_shape_emits() {
        let opts = ListProtestsOptions::builder()
            .shape(crate::SHAPE_PROTESTS_MINIMAL)
            .flat(true)
            .build();
        let q = opts.to_query();
        assert_eq!(
            get_q(&q, "shape").as_deref(),
            Some(crate::SHAPE_PROTESTS_MINIMAL)
        );
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
    }

    #[test]
    fn list_protests_extra_emits() {
        let mut extra = BTreeMap::new();
        extra.insert("custom_x".to_string(), "xv".to_string());
        let opts = ListProtestsOptions::builder().extra(extra).build();
        let q = opts.to_query();
        assert!(q.contains(&("custom_x".into(), "xv".into())));
    }

    #[test]
    fn get_protest_options_emit() {
        let opts = GetProtestOptions::builder()
            .shape("docket(*)")
            .flat(true)
            .flat_lists(true)
            .build();
        let q = opts.to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("docket(*)"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists").as_deref(), Some("true"));
    }

    #[test]
    fn protest_record_decodes_from_sample_json() {
        let value = serde_json::json!({
            "case_id": "b-12345-1",
            "case_number": "B-12345.1",
            "title": "Acme Corp Protest",
            "source_system": "GAO",
            "outcome": "sustained",
            "filed_date": "2024-01-15",
            "decision_date": "2024-04-20",
            "agency": {"code": "9700", "name": "DoD"},
            "future_field": "still here"
        });
        let rec: ProtestRecord = serde_json::from_value(value).expect("decode");
        assert_eq!(rec.case_id.as_deref(), Some("b-12345-1"));
        assert_eq!(rec.case_number.as_deref(), Some("B-12345.1"));
        assert_eq!(rec.title.as_deref(), Some("Acme Corp Protest"));
        assert_eq!(rec.source_system.as_deref(), Some("GAO"));
        assert_eq!(rec.outcome.as_deref(), Some("sustained"));
        assert_eq!(rec.filed_date.as_deref(), Some("2024-01-15"));
        assert_eq!(rec.decision_date.as_deref(), Some("2024-04-20"));
        // Unknown / not-first-classed fields land in `extra` via #[serde(flatten)].
        assert!(rec.extra.contains_key("agency"));
        assert_eq!(
            rec.extra.get("future_field").and_then(|v| v.as_str()),
            Some("still here")
        );
    }

    #[tokio::test]
    async fn get_protest_validates_empty_case_id() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_protest("", None).await.unwrap_err();
        match err {
            Error::Validation { message, .. } => {
                assert!(message.contains("case_id is required"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }
}
