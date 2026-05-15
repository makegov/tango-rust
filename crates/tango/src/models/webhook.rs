//! Typed models for the Tango webhook management API.
//!
//! Wire types are mirrored from `tango-node/src/models/Webhooks.ts`. Every
//! struct carries `#[serde(flatten)] extra: HashMap<String, Value>` so a
//! server-side schema addition surfaces as `extra["new_field"]` rather than
//! being dropped during deserialization.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Webhook endpoints
// ---------------------------------------------------------------------------

/// A configured webhook endpoint.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookEndpoint {
    /// Endpoint UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Human-readable name (unique per user).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Destination URL the server will POST deliveries to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Endpoint signing secret. Surfaced only on creation; subsequent reads
    /// typically omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
    /// Whether the endpoint is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// ISO timestamp the endpoint was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO timestamp the endpoint was last updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Request body for
/// [`Client::create_webhook_endpoint`](crate::Client::create_webhook_endpoint).
///
/// `name` and `callback_url` are required. The Tango API enforces
/// `unique(user, name)` on endpoints; the SDK validates this client-side
/// for a cleaner error than the server's 400.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookEndpointCreateInput {
    /// Human-readable name (must be unique per user).
    pub name: String,
    /// Destination URL.
    pub callback_url: String,
    /// Whether to enable on creation. Server defaults to `true` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Event types to subscribe to. Empty means "all event types".
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_types: Vec<String>,
}

/// PATCH body for
/// [`Client::update_webhook_endpoint`](crate::Client::update_webhook_endpoint).
/// Only `Some` fields are sent.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookEndpointUpdateInput {
    /// New name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New callback URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
    /// Enable/disable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// Replace the subscribed event-type list.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_types: Vec<String>,
}

// ---------------------------------------------------------------------------
// Webhook event types + sample payloads + test delivery
// ---------------------------------------------------------------------------

/// One event type the server can emit.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookEventType {
    /// Wire name of the event type (e.g. `"alerts.contract.match"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Schema version of the event body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<i64>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Response from
/// [`Client::list_webhook_event_types`](crate::Client::list_webhook_event_types).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookEventTypesResponse {
    /// The event types the server can emit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_types: Vec<WebhookEventType>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// One timestamped batch of synthetic events in a sample-payload response.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WebhookSampleDelivery {
    /// ISO timestamp for the sample.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// The events in this batch (each event is a free-form JSON object).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Value>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Wrapper around the per-type sample-delivery struct used in the all-types
/// variant of [`WebhookSamplePayloadResponse`].
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WebhookSamplePayloadSample {
    /// The sample delivery for one event type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_delivery: Option<WebhookSampleDelivery>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Response from
/// [`Client::get_webhook_sample_payload`](crate::Client::get_webhook_sample_payload).
///
/// Covers both variants the endpoint returns:
/// - The single-event-type variant (when `event_type` is passed): `event_type`,
///   `sample_delivery`, `signature_header`, `note` are populated.
/// - The all-types variant (when `event_type` is omitted): `samples` and
///   `usage` are populated.
///
/// Callers can branch on which fields are populated.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WebhookSamplePayloadResponse {
    /// Event type (single-variant only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
    /// Sample delivery body (single-variant only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_delivery: Option<WebhookSampleDelivery>,
    /// Example signature header line (single-variant only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_header: Option<String>,
    /// Free-text guidance from the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Map of event type → sample (all-types variant).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub samples: HashMap<String, WebhookSamplePayloadSample>,
    /// Free-text guidance (all-types variant).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Response from
/// [`Client::test_webhook_endpoint`](crate::Client::test_webhook_endpoint).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WebhookTestDeliveryResult {
    /// Whether the test POST succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    /// HTTP status code returned by the destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i64>,
    /// Response time of the destination, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<i64>,
    /// Echoed destination URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint_url: Option<String>,
    /// Human-readable message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Error message, when `success` is false.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Response body returned by the destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
    /// The synthetic body the server delivered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_payload: Option<Value>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

// ---------------------------------------------------------------------------
// Webhook alerts (filter-based subscription convenience API)
// ---------------------------------------------------------------------------

/// A filter-based webhook subscription.
///
/// The alerts API uses `name` + `filters` (whereas the canonical subscriptions
/// API uses `subscription_name` + `filter_definition`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct WebhookAlert {
    /// Alert UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert_id: Option<String>,
    /// Human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Resource type the alert filters (`"contract"`, `"opportunity"`, …; singular).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_type: Option<String>,
    /// Filter map applied against new resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<Value>,
    /// `"realtime"`, `"hourly"`, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    /// Cron expression when `frequency = "custom"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron_expression: Option<String>,
    /// Lifecycle status (`"active"`, `"paused"`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// ISO timestamp the alert was created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO timestamp the alert was last checked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_checked_at: Option<String>,
    /// Number of resources the alert has matched.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_count: Option<i64>,
    /// Forward-compatible bucket.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Request body for
/// [`Client::create_webhook_alert`](crate::Client::create_webhook_alert).
///
/// `name`, `query_type`, and a non-empty `filters` map are required.
/// `query_type` is SINGULAR (`"contract"`, not `"contracts"`). For accounts
/// with multiple webhook endpoints, set `endpoint` to the destination UUID;
/// single-endpoint accounts may omit it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebhookAlertCreateInput {
    /// Human-readable name.
    pub name: String,
    /// Singular resource type (e.g. `"contract"`).
    pub query_type: String,
    /// Filter map applied against new resources (non-empty).
    pub filters: Value,
    /// `"realtime"`, `"hourly"`, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    /// Cron expression when `frequency = "custom"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron_expression: Option<String>,
    /// Destination endpoint UUID (required when the account has more than one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

/// PATCH body for
/// [`Client::update_webhook_alert`](crate::Client::update_webhook_alert).
///
/// Only `name`, `frequency`, `cron_expression`, and `is_active` are writable
/// server-side. `query_type` and `filters` are read-only after creation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebhookAlertUpdateInput {
    /// New name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New frequency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    /// New cron expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron_expression: Option<String>,
    /// Enable / pause.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}
