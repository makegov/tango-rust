//! `ContractAppealRecord` — typed response from `GET /api/contract_appeals/{uuid}/`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A Contract Disputes Act appeal decision from a board of contract appeals.
///
/// Returned by [`Client::get_contract_appeal`](crate::Client::get_contract_appeal).
/// These are contract-dispute decisions — a claim between the government and a contractor already under contract — and not bid protests; those are [`ProtestRecord`](crate::models::ProtestRecord).
///
/// Every field is optional because the response schema follows the requested `shape`: an unshaped list response carries only a core subset, so a field the caller did not ask for is ABSENT rather than null.
/// Unknown server-side fields fall through to [`extra`](Self::extra).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ContractAppealRecord {
    /// Tango identifier for the decision, and the path segment
    /// [`Client::get_contract_appeal`](crate::Client::get_contract_appeal) takes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,

    /// Deciding board: `"cbca"` (civilian) or `"asbca"` (defense).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,

    /// Every docket number the decision resolves.
    /// A consolidated decision carries several, which is why this is a list and the `docket` filter matches any member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docket_numbers: Option<Vec<String>>,

    /// Where the docket numbers were read from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docket_source: Option<String>,

    /// The docket string exactly as the board published it, before parsing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docket_raw: Option<String>,

    /// ISO date the decision issued.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_date: Option<String>,

    /// The decision date exactly as the board published it, before parsing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_date_raw: Option<String>,

    /// Whether [`decision_date`](Self::decision_date) was reconstructed rather than parsed straight from
    /// [`decision_date_raw`](Self::decision_date_raw).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_date_repaired: Option<bool>,

    /// The contractor bringing the appeal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appellant: Option<String>,

    /// The judge who wrote the decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge: Option<String>,

    /// Normalized decision kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_type: Option<String>,

    /// The decision kind exactly as the board published it, before normalization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_type_raw: Option<String>,

    /// Link to the decision document on the board's own site.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The board's identifier for the decision document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_id: Option<String>,

    /// Link to the board listing page the decision was found on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listing_url: Option<String>,

    /// Year of the board listing page.
    /// A board files a decision under the year it publishes the listing, which is not always the year it decided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listing_year: Option<i64>,

    /// Timestamp Tango first saw the decision on a listing page.
    /// The polling primitive: everything new since your last call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_listed_at: Option<String>,

    /// Whether the decision is still on a current board listing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listed: Option<bool>,

    /// State of the decision's text extraction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_status: Option<String>,

    /// Character count of the extracted decision text.
    /// Readable at every plan, so a caller can size a decision without being entitled to read it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_char_count: Option<i64>,

    /// The full text of the decision, on an Enterprise plan.
    /// Below Enterprise the key is ABSENT rather than null, so `None` here means "not served" and never "the decision has no text" — [`text_status`](Self::text_status) and
    /// [`text_char_count`](Self::text_char_count) answer that question at every plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_text: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
