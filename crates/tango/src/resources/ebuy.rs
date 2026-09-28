//! `GET /api/ebuy/requests/` — GSA eBuy requests (RFQs, RFPs and RFIs) posted under your own GSA schedule contracts.
//!
//! Requires the Pro plan or above, and is scoped to your account: you see a request only while your account is linked to a schedule contract it was posted under.
//! A caller with no linked contract gets an empty list, not an error — [`Client::get_ebuy_access`] tells "no access" apart from "no matches".
//! A request outside your scope 404s on lookup, the same answer as an id that never existed.
//!
//! `status` is frozen at the state it was last seen in. Only currently-active requests are published, so a request that closes stops appearing rather than getting a final row: `Open` means "open the last time it was seen", and `last_seen` is the staleness signal.
//! The contract number a request was posted under is never returned in any payload, and `buyer_agency_code` and some other buyer and contact fields are sparse on older requests.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::internal::{apply_pagination, push_opt};
use crate::models::EbuyAccess;
use crate::pagination::{FetchFn, Page, PageStream};
use crate::resources::agencies::urlencoding;
use crate::Record;
use bon::Builder;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Options for [`Client::list_ebuy_requests`] and [`Client::iterate_ebuy_requests`].
///
/// Every string filter except the date bounds accepts `|` for OR.
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct ListEbuyRequestsOptions {
    /// 1-based page number.
    #[builder(into)]
    pub page: Option<u32>,
    /// Page size (server caps at 100).
    #[builder(into)]
    pub limit: Option<u32>,
    /// Keyset cursor.
    #[builder(into)]
    pub cursor: Option<String>,
    /// Comma-separated field selector. Use [`SHAPE_EBUY_REQUESTS_MINIMAL`](crate::SHAPE_EBUY_REQUESTS_MINIMAL) or roll your own; `organization(*)` and `attachments(*)` are the expands.
    #[builder(into)]
    pub shape: Option<String>,
    /// Collapse nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When [`flat`](Self::flat) is also true, flatten list-valued fields.
    #[builder(default)]
    pub flat_lists: bool,

    /// Full-text search over title, description, reference number, request id and attachment text. Results rank by relevance unless [`ordering`](Self::ordering) is set, and a title, description or identifier hit always ranks above an attachment-text-only hit.
    #[builder(into)]
    pub search: Option<String>,
    /// eBuy request id, exact (e.g. `RFQ1835158`).
    #[builder(into)]
    pub rfq_id: Option<String>,
    /// The buyer's own solicitation or reference number. Dash-insensitive.
    #[builder(into)]
    pub reference_number: Option<String>,
    /// Request type: `RFQ`, `RFP` or `RFI`.
    #[builder(into)]
    pub request_type: Option<String>,
    /// Status as last seen: `Open` or `Cancelled`. Frozen, not a currency signal — an `Open` request may have closed since; read `last_seen` for staleness.
    #[builder(into)]
    pub status: Option<String>,
    /// Special Item Number the request was posted under.
    #[builder(into)]
    pub sin: Option<String>,
    /// GSA schedule the request was posted under.
    #[builder(into)]
    pub schedule: Option<String>,
    /// Buying department name, exactly as published (free text).
    #[builder(into)]
    pub buyer_agency: Option<String>,
    /// Agency name, abbreviation or code, matched against the buyer's resolved organization including every sub-agency and office beneath it.
    #[builder(into)]
    pub agency: Option<String>,
    /// Narrow to one of your own linked contracts. A contract you are not linked to returns nothing rather than an error.
    #[builder(into)]
    pub contract_number: Option<String>,

    /// Lower bound on `issue_date` (ISO `YYYY-MM-DD`, inclusive).
    #[builder(into)]
    pub issue_date_after: Option<String>,
    /// Upper bound on `issue_date` (inclusive of that whole day).
    #[builder(into)]
    pub issue_date_before: Option<String>,
    /// Lower bound on `close_date` (inclusive).
    #[builder(into)]
    pub close_date_after: Option<String>,
    /// Upper bound on `close_date` (inclusive of that whole day).
    #[builder(into)]
    pub close_date_before: Option<String>,

    /// Sort key: `issue_date`, `close_date`, `last_seen` or `modified`, prefixed with `-` for descending. The server default is `-issue_date`.
    #[builder(into)]
    pub ordering: Option<String>,

    /// Escape hatch for filter keys not yet first-classed on this struct.
    #[builder(default)]
    pub extra: BTreeMap<String, String>,
}

impl ListEbuyRequestsOptions {
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
        push_opt(&mut q, "rfq_id", self.rfq_id.as_deref());
        push_opt(&mut q, "reference_number", self.reference_number.as_deref());
        push_opt(&mut q, "request_type", self.request_type.as_deref());
        push_opt(&mut q, "status", self.status.as_deref());
        push_opt(&mut q, "sin", self.sin.as_deref());
        push_opt(&mut q, "schedule", self.schedule.as_deref());
        push_opt(&mut q, "buyer_agency", self.buyer_agency.as_deref());
        push_opt(&mut q, "agency", self.agency.as_deref());
        push_opt(&mut q, "contract_number", self.contract_number.as_deref());
        push_opt(&mut q, "issue_date_after", self.issue_date_after.as_deref());
        push_opt(
            &mut q,
            "issue_date_before",
            self.issue_date_before.as_deref(),
        );
        push_opt(&mut q, "close_date_after", self.close_date_after.as_deref());
        push_opt(
            &mut q,
            "close_date_before",
            self.close_date_before.as_deref(),
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

/// Options for [`Client::get_ebuy_request`].
#[derive(Debug, Clone, Default, Builder, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetEbuyRequestOptions {
    /// Shape selector. When empty, the server returns every field plus `organization(*)` and `attachments(*)`.
    #[builder(into)]
    pub shape: Option<String>,
    /// Flatten nested objects into dot-separated keys.
    #[builder(default)]
    pub flat: bool,
    /// When `flat=true`, also flatten list-valued nested fields.
    #[builder(default)]
    pub flat_lists: bool,
}

impl GetEbuyRequestOptions {
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
    /// `GET /api/ebuy/requests/` — one page of the eBuy requests visible to your account.
    ///
    /// Requires the Pro plan (below it the API returns 403). With no linked schedule contract the page is empty rather than an error; call [`get_ebuy_access`](Self::get_ebuy_access) to tell the two apart.
    pub async fn list_ebuy_requests(&self, opts: ListEbuyRequestsOptions) -> Result<Page<Record>> {
        let q = opts.to_query();
        let bytes = self.get_bytes("/api/ebuy/requests/", &q).await?;
        Page::decode(&bytes)
    }

    /// `GET /api/ebuy/requests/{rfq_id}/` — one eBuy request.
    ///
    /// The default shape carries every field plus `organization(*)` and `attachments(*)`. An attachment with `is_link = true` is an outbound URL in `doc_path` with no stored document behind it.
    /// A request outside your scope returns [`Error::NotFound`], the same as an unknown id.
    pub async fn get_ebuy_request(
        &self,
        rfq_id: &str,
        opts: Option<GetEbuyRequestOptions>,
    ) -> Result<Record> {
        if rfq_id.is_empty() {
            return Err(Error::Validation {
                message: "get_ebuy_request: rfq_id is required".into(),
                response: None,
            });
        }
        let q = opts.unwrap_or_default().to_query();
        let path = format!("/api/ebuy/requests/{}/", urlencoding(rfq_id));
        self.get_json::<Record>(&path, &q).await
    }

    /// `GET /api/ebuy/requests/{rfq_id}/attachments/{doc_seq_num}/download/` — a short-lived download URL for one stored attachment.
    ///
    /// The API answers with a redirect to a presigned URL valid for about five minutes. This method does not follow it: it returns the redirect target, so fetch it promptly and without your API key.
    ///
    /// Errors: [`Error::ExternalLink`] (carrying the link's `url`) when the attachment is an outbound link rather than a stored document, and [`Error::NotFound`] when the request is outside your scope or the document has not been captured yet.
    pub async fn get_ebuy_attachment_url(&self, rfq_id: &str, doc_seq_num: u32) -> Result<String> {
        if rfq_id.is_empty() {
            return Err(Error::Validation {
                message: "get_ebuy_attachment_url: rfq_id is required".into(),
                response: None,
            });
        }
        let path = format!(
            "/api/ebuy/requests/{}/attachments/{doc_seq_num}/download/",
            urlencoding(rfq_id)
        );
        match self.get_redirect_location(&path, &[]).await {
            Err(Error::Validation {
                message,
                response: Some(body),
            }) => {
                let link = body
                    .raw
                    .as_ref()
                    .and_then(|v| v.get("url"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string);
                match link {
                    Some(url) => Err(Error::ExternalLink {
                        url,
                        response: Some(body),
                    }),
                    None => Err(Error::Validation {
                        message,
                        response: Some(body),
                    }),
                }
            }
            other => other,
        }
    }

    /// `GET /api/ebuy/access/` — whether your account can read eBuy requests, and why not when it can't.
    pub async fn get_ebuy_access(&self) -> Result<EbuyAccess> {
        self.get_json::<EbuyAccess>("/api/ebuy/access/", &[]).await
    }

    /// Stream every eBuy request matching `opts`.
    pub fn iterate_ebuy_requests(&self, opts: ListEbuyRequestsOptions) -> PageStream<Record> {
        let opts = Arc::new(opts);
        let fetch: FetchFn<Record> = Box::new(move |client, page, cursor| {
            let mut next = (*opts).clone();
            next.page = page;
            next.cursor = cursor;
            Box::pin(async move { client.list_ebuy_requests(next).await })
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
    fn list_ebuy_requests_all_filters_emit() {
        let opts = ListEbuyRequestsOptions::builder()
            .search("cybersecurity")
            .rfq_id("RFQ1835158")
            .reference_number("W912DY-26-Q-0012")
            .request_type("RFQ|RFI")
            .status("Open")
            .sin("54151S")
            .schedule("MAS")
            .buyer_agency("Department of the Army")
            .agency("DOD")
            .contract_number("C-1")
            .issue_date_after("2026-01-01")
            .issue_date_before("2026-06-30")
            .close_date_after("2026-02-01")
            .close_date_before("2026-07-31")
            .ordering("-close_date")
            .build();
        let q = opts.to_query();
        for (k, v) in [
            ("search", "cybersecurity"),
            ("rfq_id", "RFQ1835158"),
            ("reference_number", "W912DY-26-Q-0012"),
            ("request_type", "RFQ|RFI"),
            ("status", "Open"),
            ("sin", "54151S"),
            ("schedule", "MAS"),
            ("buyer_agency", "Department of the Army"),
            ("agency", "DOD"),
            ("contract_number", "C-1"),
            ("issue_date_after", "2026-01-01"),
            ("issue_date_before", "2026-06-30"),
            ("close_date_after", "2026-02-01"),
            ("close_date_before", "2026-07-31"),
            ("ordering", "-close_date"),
        ] {
            assert_eq!(get_q(&q, k).as_deref(), Some(v), "{k}");
        }
        assert_eq!(q.len(), 15);
    }

    #[test]
    fn list_ebuy_requests_zero_value_omitted() {
        assert!(ListEbuyRequestsOptions::builder()
            .build()
            .to_query()
            .is_empty());
    }

    #[test]
    fn get_ebuy_request_options_emit() {
        let q = GetEbuyRequestOptions::builder()
            .shape("rfq_id,attachments(*)")
            .flat(true)
            .build()
            .to_query();
        assert_eq!(get_q(&q, "shape").as_deref(), Some("rfq_id,attachments(*)"));
        assert_eq!(get_q(&q, "flat").as_deref(), Some("true"));
        assert_eq!(get_q(&q, "flat_lists"), None);
    }

    #[test]
    fn ebuy_access_decodes() {
        let access: EbuyAccess = serde_json::from_value(serde_json::json!({
            "enabled": false,
            "reason": "no_contract_grant",
            "contracts": []
        }))
        .expect("decode");
        assert!(!access.enabled);
        assert_eq!(access.reason.as_deref(), Some("no_contract_grant"));
        assert!(access.contracts.is_empty());
    }

    #[tokio::test]
    async fn empty_rfq_id_is_rejected_before_any_request() {
        let client = Client::builder().api_key("x").build().expect("client");
        let err = client.get_ebuy_request("", None).await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
        let err = client.get_ebuy_attachment_url("", 1).await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }
}
