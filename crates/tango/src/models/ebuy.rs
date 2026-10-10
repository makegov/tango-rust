//! `EbuyAccess` — typed response from `GET /api/ebuy/access/`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Whether the caller can read GSA eBuy requests.
///
/// Returned by [`Client::get_ebuy_access`](crate::Client::get_ebuy_access).
/// Without access, [`Client::list_ebuy_requests`](crate::Client::list_ebuy_requests) returns an empty page rather than an error, so this is how to tell "no access" from "no matches".
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EbuyAccess {
    /// `true` when the caller can read eBuy requests.
    #[serde(default)]
    pub enabled: bool,

    /// Why access is off: `"tier_required"` (below the Pro plan) or `"no_contract_grant"` (no GSA schedule contract linked to the account). `None` when [`enabled`](Self::enabled) is true. `"tier_required"` wins when both apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// The caller's own linked GSA schedule contract numbers, sorted.
    #[serde(default)]
    pub contracts: Vec<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
