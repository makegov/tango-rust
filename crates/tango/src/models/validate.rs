//! Types for `POST /api/validate/`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Identifier type the validator should check.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ValidateInputType {
    /// Contract PIID.
    Piid,
    /// Solicitation number.
    Solicitation,
    /// UEI (Unique Entity Identifier).
    Uei,
}

/// Request body for [`Client::validate`](crate::Client::validate).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidateInput {
    /// Identifier type to check.
    #[serde(rename = "type")]
    pub kind: ValidateInputType,
    /// Identifier value to validate.
    pub value: String,
}

/// Response from [`Client::validate`](crate::Client::validate).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ValidateResult {
    /// Outcome label: `"valid"`, `"invalid"`, or `"low_confidence"`.
    pub result: String,
    /// Echoed identifier type.
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Echoed identifier value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Structured failure reasons when `result` is non-valid.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<Value>,
    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validate_result_captures_unknown_fields_via_extra() {
        let body = json!({
            "result": "valid",
            "type": "uei",
            "value": "ABC123DEF456",
            "future_field": "x"
        });
        let r: ValidateResult = serde_json::from_value(body).expect("decode");
        assert_eq!(r.result, "valid");
        assert_eq!(r.kind.as_deref(), Some("uei"));
        assert!(r.extra.contains_key("future_field"));
    }

    #[test]
    fn validate_input_type_round_trips() {
        let v = serde_json::to_value(ValidateInputType::Uei).unwrap();
        assert_eq!(v, json!("uei"));
        let back: ValidateInputType = serde_json::from_value(json!("piid")).unwrap();
        assert_eq!(back, ValidateInputType::Piid);
    }
}
