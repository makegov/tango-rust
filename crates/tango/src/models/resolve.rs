//! Types for `POST /api/resolve/`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Resolver target — which catalog the resolver searches against.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResolveTargetType {
    /// SAM.gov vendor catalog.
    Entity,
    /// Federal organization catalog (departments / agencies / offices).
    Organization,
}

/// Request body for [`Client::resolve`](crate::Client::resolve).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolveInput {
    /// Free-text name to match.
    pub name: String,
    /// Target catalog.
    pub target_type: ResolveTargetType,
    /// Optional state filter (e.g. `"VA"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Optional city filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Optional free-text additional context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// A single candidate from a resolve call.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ResolveCandidate {
    /// Canonical UEI (entity target) or organization key (organization target).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// Human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Confidence label (`"low"` / `"medium"` / `"high"`). Pro+ tier only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_tier: Option<String>,
    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Response from [`Client::resolve`](crate::Client::resolve).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ResolveResult {
    /// Number of candidates returned (Free tier caps at 3, Pro+ at 5).
    #[serde(default)]
    pub count: u32,
    /// The candidates, ranked by confidence.
    #[serde(default)]
    pub candidates: Vec<ResolveCandidate>,
    /// Forward-compatible bucket for any unrecognized fields the server adds.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolve_result_captures_unknown_fields_via_extra() {
        let body = json!({
            "count": 1,
            "candidates": [{
                "identifier": "ABC",
                "display_name": "ACME",
                "match_tier": "high",
                "future_score": 0.92
            }],
            "future_meta": {"version": 2}
        });
        let r: ResolveResult = serde_json::from_value(body).expect("decode");
        assert_eq!(r.count, 1);
        assert!(r.extra.contains_key("future_meta"));
        let candidate = &r.candidates[0];
        assert_eq!(candidate.identifier.as_deref(), Some("ABC"));
        assert!(candidate.extra.contains_key("future_score"));
    }

    #[test]
    fn resolve_target_type_round_trips() {
        let v = serde_json::to_value(ResolveTargetType::Entity).unwrap();
        assert_eq!(v, json!("entity"));
        let back: ResolveTargetType = serde_json::from_value(json!("organization")).unwrap();
        assert_eq!(back, ResolveTargetType::Organization);
    }
}
