//! Webhook management API — CRUD for endpoints and filter-based alerts.
//!
//! This module covers the *management* surface (`/api/webhooks/...`). Signing
//! and verification of inbound deliveries live in the separate `tango-webhooks`
//! crate so a receiver service doesn't have to pull the full SDK.

use crate::client::Client;
use crate::error::{Error, Result};
use crate::models::{
    WebhookAlert, WebhookAlertCreateInput, WebhookAlertUpdateInput, WebhookEndpoint,
    WebhookEndpointCreateInput, WebhookEndpointUpdateInput, WebhookEventTypesResponse,
    WebhookSamplePayloadResponse, WebhookTestDeliveryResult,
};
use crate::pagination::Page;
use crate::resources::agencies::urlencoding;
use crate::ListOptions;
use serde::Serialize;

/// Body for `POST /api/webhooks/endpoints/test-delivery/`. The canonical key
/// is `endpoint` (per tango#2252).
#[derive(Serialize)]
struct TestDeliveryBody<'a> {
    endpoint: &'a str,
}

// ---------------------------------------------------------------------------
// Webhook endpoints
// ---------------------------------------------------------------------------

impl Client {
    /// `GET /api/webhooks/event-types/` — list the event types the server can emit.
    pub async fn list_webhook_event_types(&self) -> Result<WebhookEventTypesResponse> {
        self.get_json::<WebhookEventTypesResponse>("/api/webhooks/event-types/", &[])
            .await
    }

    /// `GET /api/webhooks/endpoints/` — list the caller's configured webhook
    /// endpoints. Pagination only — no resource-specific filters.
    pub async fn list_webhook_endpoints(&self, opts: ListOptions) -> Result<Page<WebhookEndpoint>> {
        let mut q = Vec::new();
        opts.apply(&mut q);
        let bytes = self.get_bytes("/api/webhooks/endpoints/", &q).await?;
        Page::<WebhookEndpoint>::decode(&bytes)
    }

    /// `GET /api/webhooks/endpoints/{id}/` — fetch a single endpoint.
    pub async fn get_webhook_endpoint(&self, id: &str) -> Result<WebhookEndpoint> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "GetWebhookEndpoint: id is required".into(),
                response: None,
            });
        }
        let path = format!("/api/webhooks/endpoints/{}/", urlencoding(id));
        self.get_json::<WebhookEndpoint>(&path, &[]).await
    }

    /// `POST /api/webhooks/endpoints/` — create a new endpoint.
    ///
    /// `name` and `callback_url` are required. The Tango API enforces
    /// `unique(user, name)` on endpoints, so the SDK validates client-side
    /// for a cleaner error than the server's 400 on duplicates.
    pub async fn create_webhook_endpoint(
        &self,
        input: WebhookEndpointCreateInput,
    ) -> Result<WebhookEndpoint> {
        if input.name.is_empty() {
            return Err(Error::Validation {
                message: "CreateWebhookEndpoint: name is required (the Tango API enforces unique(user, name) on endpoints)".into(),
                response: None,
            });
        }
        if input.callback_url.is_empty() {
            return Err(Error::Validation {
                message: "CreateWebhookEndpoint: callback_url is required".into(),
                response: None,
            });
        }
        self.post_json::<_, WebhookEndpoint>("/api/webhooks/endpoints/", &input)
            .await
    }

    /// `PATCH /api/webhooks/endpoints/{id}/` — update an existing endpoint.
    /// Only `Some` fields on the input are sent.
    pub async fn update_webhook_endpoint(
        &self,
        id: &str,
        input: WebhookEndpointUpdateInput,
    ) -> Result<WebhookEndpoint> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "UpdateWebhookEndpoint: id is required".into(),
                response: None,
            });
        }
        let path = format!("/api/webhooks/endpoints/{}/", urlencoding(id));
        self.patch_json::<_, WebhookEndpoint>(&path, &input).await
    }

    /// `DELETE /api/webhooks/endpoints/{id}/` — remove an endpoint.
    pub async fn delete_webhook_endpoint(&self, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "DeleteWebhookEndpoint: id is required".into(),
                response: None,
            });
        }
        let path = format!("/api/webhooks/endpoints/{}/", urlencoding(id));
        self.delete_no_content(&path).await
    }

    /// `POST /api/webhooks/endpoints/test-delivery/` — trigger a test delivery
    /// for `endpoint_id`. The request body uses the canonical key `endpoint`
    /// (per tango#2252); the server still accepts the legacy `endpoint_id`
    /// alias but the SDK sends the canonical key.
    pub async fn test_webhook_endpoint(
        &self,
        endpoint_id: &str,
    ) -> Result<WebhookTestDeliveryResult> {
        if endpoint_id.is_empty() {
            return Err(Error::Validation {
                message: "TestWebhookEndpoint: endpoint_id is required".into(),
                response: None,
            });
        }
        let body = TestDeliveryBody {
            endpoint: endpoint_id,
        };
        self.post_json::<_, WebhookTestDeliveryResult>(
            "/api/webhooks/endpoints/test-delivery/",
            &body,
        )
        .await
    }

    /// `GET /api/webhooks/endpoints/sample-payload/` — fetch a sample delivery
    /// body. When `event_type` is `Some`, returns the single-event-type variant;
    /// when `None`, the server returns samples for every event type.
    pub async fn get_webhook_sample_payload(
        &self,
        event_type: Option<&str>,
    ) -> Result<WebhookSamplePayloadResponse> {
        let mut q = Vec::new();
        if let Some(ev) = event_type.filter(|s| !s.is_empty()) {
            q.push(("event_type".into(), ev.into()));
        }
        self.get_json::<WebhookSamplePayloadResponse>("/api/webhooks/endpoints/sample-payload/", &q)
            .await
    }

    // -----------------------------------------------------------------------
    // Webhook alerts (filter-subscription convenience API)
    // -----------------------------------------------------------------------

    /// `GET /api/webhooks/alerts/` — list the caller's filter-based subscriptions.
    pub async fn list_webhook_alerts(&self, opts: ListOptions) -> Result<Page<WebhookAlert>> {
        let mut q = Vec::new();
        opts.apply(&mut q);
        let bytes = self.get_bytes("/api/webhooks/alerts/", &q).await?;
        Page::<WebhookAlert>::decode(&bytes)
    }

    /// `GET /api/webhooks/alerts/{id}/` — fetch a single alert.
    pub async fn get_webhook_alert(&self, id: &str) -> Result<WebhookAlert> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "GetWebhookAlert: id is required".into(),
                response: None,
            });
        }
        let path = format!("/api/webhooks/alerts/{}/", urlencoding(id));
        self.get_json::<WebhookAlert>(&path, &[]).await
    }

    /// `POST /api/webhooks/alerts/` — create a filter-based subscription.
    ///
    /// `name`, `query_type` (singular), and a non-empty `filters` object are
    /// required. For accounts with multiple webhook endpoints, set
    /// `input.endpoint` to the destination endpoint UUID; single-endpoint
    /// accounts may omit it.
    pub async fn create_webhook_alert(
        &self,
        input: WebhookAlertCreateInput,
    ) -> Result<WebhookAlert> {
        if input.name.is_empty() {
            return Err(Error::Validation {
                message: "CreateWebhookAlert: name is required".into(),
                response: None,
            });
        }
        if input.query_type.is_empty() {
            return Err(Error::Validation {
                message:
                    r#"CreateWebhookAlert: query_type is required (singular, e.g. "contract")"#
                        .into(),
                response: None,
            });
        }
        let empty = match &input.filters {
            serde_json::Value::Null => true,
            serde_json::Value::Object(m) => m.is_empty(),
            serde_json::Value::Array(a) => a.is_empty(),
            _ => false,
        };
        if empty {
            return Err(Error::Validation {
                message: "CreateWebhookAlert: filters must be a non-empty object".into(),
                response: None,
            });
        }
        self.post_json::<_, WebhookAlert>("/api/webhooks/alerts/", &input)
            .await
    }

    /// `PATCH /api/webhooks/alerts/{id}/` — update an existing alert.
    /// Only `name`, `frequency`, `cron_expression`, and `is_active` are writable.
    pub async fn update_webhook_alert(
        &self,
        id: &str,
        input: WebhookAlertUpdateInput,
    ) -> Result<WebhookAlert> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "UpdateWebhookAlert: id is required".into(),
                response: None,
            });
        }
        let path = format!("/api/webhooks/alerts/{}/", urlencoding(id));
        self.patch_json::<_, WebhookAlert>(&path, &input).await
    }

    /// `DELETE /api/webhooks/alerts/{id}/` — remove an alert.
    pub async fn delete_webhook_alert(&self, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "DeleteWebhookAlert: id is required".into(),
                response: None,
            });
        }
        let path = format!("/api/webhooks/alerts/{}/", urlencoding(id));
        self.delete_no_content(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn client() -> Client {
        // Validation must trip BEFORE any HTTP call, so the unreachable base
        // URL is fine — the request must never be issued.
        Client::builder()
            .api_key("k")
            .base_url("http://localhost:1".to_string())
            .build()
            .expect("build client")
    }

    // ---- endpoints: empty-id rejection ----

    #[tokio::test]
    async fn get_endpoint_rejects_empty_id() {
        let err = client().get_webhook_endpoint("").await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn update_endpoint_rejects_empty_id() {
        let err = client()
            .update_webhook_endpoint("", WebhookEndpointUpdateInput::default())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn delete_endpoint_rejects_empty_id() {
        let err = client().delete_webhook_endpoint("").await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn test_endpoint_rejects_empty_id() {
        let err = client().test_webhook_endpoint("").await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ---- create endpoint: required fields ----

    #[tokio::test]
    async fn create_endpoint_rejects_empty_name() {
        let err = client()
            .create_webhook_endpoint(WebhookEndpointCreateInput {
                name: String::new(),
                callback_url: "https://example.com/hook".into(),
                is_active: None,
                event_types: vec![],
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn create_endpoint_rejects_empty_callback_url() {
        let err = client()
            .create_webhook_endpoint(WebhookEndpointCreateInput {
                name: "my-hook".into(),
                callback_url: String::new(),
                is_active: None,
                event_types: vec![],
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ---- alerts: empty-id rejection ----

    #[tokio::test]
    async fn get_alert_rejects_empty_id() {
        let err = client().get_webhook_alert("").await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn update_alert_rejects_empty_id() {
        let err = client()
            .update_webhook_alert("", WebhookAlertUpdateInput::default())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn delete_alert_rejects_empty_id() {
        let err = client().delete_webhook_alert("").await.unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    // ---- create alert: required fields + filters shape ----

    #[tokio::test]
    async fn create_alert_rejects_empty_name() {
        let err = client()
            .create_webhook_alert(WebhookAlertCreateInput {
                name: String::new(),
                query_type: "contract".into(),
                filters: json!({"piid": "X"}),
                frequency: None,
                cron_expression: None,
                endpoint: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn create_alert_rejects_empty_query_type() {
        let err = client()
            .create_webhook_alert(WebhookAlertCreateInput {
                name: "n".into(),
                query_type: String::new(),
                filters: json!({"piid": "X"}),
                frequency: None,
                cron_expression: None,
                endpoint: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn create_alert_rejects_null_filters() {
        let err = client()
            .create_webhook_alert(WebhookAlertCreateInput {
                name: "n".into(),
                query_type: "contract".into(),
                filters: serde_json::Value::Null,
                frequency: None,
                cron_expression: None,
                endpoint: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn create_alert_rejects_empty_object_filters() {
        let err = client()
            .create_webhook_alert(WebhookAlertCreateInput {
                name: "n".into(),
                query_type: "contract".into(),
                filters: json!({}),
                frequency: None,
                cron_expression: None,
                endpoint: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }

    #[tokio::test]
    async fn create_alert_rejects_empty_array_filters() {
        let err = client()
            .create_webhook_alert(WebhookAlertCreateInput {
                name: "n".into(),
                query_type: "contract".into(),
                filters: json!([]),
                frequency: None,
                cron_expression: None,
                endpoint: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Validation { .. }));
    }
}
