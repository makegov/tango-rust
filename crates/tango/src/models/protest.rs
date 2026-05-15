//! `ProtestRecord` — typed response from `GET /api/protests/{case_id}/`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A bid-protest case record.
///
/// Returned by [`Client::get_protest`](crate::Client::get_protest). Fields
/// match the server's shape preset for the detail endpoint; unknown fields
/// fall through to [`extra`](Self::extra).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ProtestRecord {
    /// Internal Tango identifier for the case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,

    /// Source-system case number (e.g. GAO file number).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,

    /// Human-readable title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Source system (`"GAO"`, `"COFC"`, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_system: Option<String>,

    /// Outcome label (`"sustained"`, `"denied"`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,

    /// ISO date the protest was filed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filed_date: Option<String>,

    /// ISO date the protest was decided, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision_date: Option<String>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
