//! `AgencyRecord` — typed response from `GET /api/agencies/{code}/`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A federal agency record.
///
/// Returned by [`Client::get_agency`](crate::Client::get_agency). The Tango
/// API uses CGAC codes as the identifier (e.g. `"9700"` for the Department of
/// Defense). All fields are optional; unknown server-side fields are captured
/// in [`extra`](Self::extra) so a schema addition never silently drops data.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgencyRecord {
    /// The agency's internal identifier (Tango-specific).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agency_id: Option<String>,

    /// Human-readable agency name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Common abbreviation (e.g. `"DOD"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abbreviation: Option<String>,

    /// Code (typically the CGAC code).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Parent department metadata (nested object the server returns
    /// verbatim).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department: Option<Value>,

    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decode_minimal() {
        let body = json!({
            "agency_id": "97",
            "name": "Department of Defense",
            "abbreviation": "DOD",
            "code": "9700"
        });
        let a: AgencyRecord = serde_json::from_value(body).expect("decode");
        assert_eq!(a.name.as_deref(), Some("Department of Defense"));
        assert_eq!(a.code.as_deref(), Some("9700"));
        assert!(a.extra.is_empty());
    }

    #[test]
    fn extra_captures_forward_compatible_fields() {
        let body = json!({
            "name": "Department of Defense",
            "code": "9700",
            "future_field": {"version": 2}
        });
        let a: AgencyRecord = serde_json::from_value(body).expect("decode");
        assert!(a.extra.contains_key("future_field"));
        assert_eq!(
            a.extra.get("future_field").and_then(|v| v.get("version")),
            Some(&json!(2))
        );
    }

    #[test]
    fn round_trip_emits_extras() {
        let body = json!({
            "name": "GSA",
            "code": "4700",
            "future": "x"
        });
        let a: AgencyRecord = serde_json::from_value(body.clone()).expect("decode");
        let re = serde_json::to_value(&a).expect("re-encode");
        assert_eq!(re.get("future"), Some(&json!("x")));
        assert_eq!(re.get("name"), Some(&json!("GSA")));
    }
}
