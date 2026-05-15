//! Typed response and input models for fixed-schema Tango endpoints.
//!
//! For shape-driven list endpoints (contracts, IDVs, entities, …) the SDK
//! returns [`crate::Record`] (a [`serde_json::Map`]) instead — shape selection
//! changes the schema, so a fixed Rust struct can't capture every combination.
//! See [`crate::shapes`] for the available presets.
//!
//! # Forward-compatible `extra` field
//!
//! Every typed model carries `#[serde(flatten)] pub extra: HashMap<String, Value>`
//! so server-side schema additions surface as `extra["new_field"]` rather than
//! being silently dropped during deserialization. Round-tripping a typed model
//! through `serde_json::to_value` will re-emit those extras.

pub mod agency;
pub mod protest;
pub mod resolve;
pub mod validate;
pub mod webhook;

pub use agency::AgencyRecord;
pub use protest::ProtestRecord;
pub use resolve::{ResolveCandidate, ResolveInput, ResolveResult, ResolveTargetType};
pub use validate::{ValidateInput, ValidateInputType, ValidateResult};
pub use webhook::{
    WebhookAlert, WebhookAlertCreateInput, WebhookAlertUpdateInput, WebhookEndpoint,
    WebhookEndpointCreateInput, WebhookEndpointUpdateInput, WebhookEventType,
    WebhookEventTypesResponse, WebhookSampleDelivery, WebhookSamplePayloadResponse,
    WebhookTestDeliveryResult,
};
