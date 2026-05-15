# Webhooks

This guide covers everything `tango-rust` provides for **building, testing, and operating webhook integrations against the Tango API**: signing, signature verification, and the client-side CRUD methods for managing endpoints and alerts.

There are two distinct crates for webhooks:

| Concern | Crate | Imports |
| ------- | ----- | ------- |
| Sign / verify deliveries | `tango-webhooks` | `hmac`, `sha2`, `subtle`, `hex` (no transport deps) |
| Create / list / update / delete endpoints + alerts (CRUD over the API) | `tango` (`Client` methods) | full SDK |

A webhook receiver service that only needs to verify deliveries can depend on `tango-webhooks` alone — it doesn't pull `reqwest`, the retry loop, or any resource methods. The two crates are wire-compatible: signatures produced by `tango_webhooks::generate` verify correctly against Tango deliveries, and vice versa.

For the API-level contract (signing scheme, event taxonomy, retry behavior), see the [Tango Webhooks Partner Guide](https://docs.makegov.com/webhooks-user-guide/).

## Concepts

Tango webhooks have three pieces of state:

| Concept | What it is | Type |
| ------- | ---------- | ---- |
| **Endpoint** | A URL Tango POSTs to, plus a generated signing secret | `WebhookEndpoint` |
| **Alert** | A saved query-filter that fires deliveries when matching records appear | `WebhookAlert` |
| **Delivery** | A single signed POST Tango makes when a matching event fires | (the request itself) |

A typical setup:

1. **Create an endpoint** with the public URL of your handler. The API returns a `secret` on creation — save it; it's used to sign every delivery.
2. **Create one or more alerts** describing the records your handler cares about (e.g. new IT-services contracts).
3. **Tango POSTs to your endpoint** when matching records appear. The body is JSON; the header `X-Tango-Signature: sha256=<hex>` is the HMAC-SHA256 of the raw body bytes keyed by your endpoint's secret.
4. **Your handler verifies the signature**, parses the body, and acts on it.

## Receiving deliveries — the `makegov-tango-webhooks` crate

```toml
[dependencies]
makegov-tango-webhooks = "0.1"
```

The crate publishes as `makegov-tango-webhooks` on crates.io and imports as `use tango_webhooks::...` thanks to a `[lib] name` shim.

Public surface:

| Symbol | Purpose |
| ------ | ------- |
| `SIGNATURE_HEADER` (const) | `"X-Tango-Signature"` |
| `SIGNATURE_PREFIX` (const) | `"sha256="` |
| `generate(body, secret) -> String` | Produce a wire-format `sha256=<hex>` signature (handy in tests). |
| `verify(body, header, secret) -> bool` | Constant-time check; never panics; returns false on absent / malformed / mismatched. |
| `parse(header) -> Option<ParsedSignature>` | Decompose a header value (algorithm + hex digest) for debugging. |
| `ParsedSignature { algorithm, signature }` | The decomposed form. |

### Verifying a delivery

```rust
use tango_webhooks::{verify, SIGNATURE_HEADER};

fn handle(raw_body: &[u8], header_value: &str, secret: &str) -> Result<(), &'static str> {
    if !verify(raw_body, header_value, secret) {
        return Err("invalid signature");
    }
    // raw_body is now trusted; parse it and act on it.
    Ok(())
}
```

**Always verify against the raw request body bytes you read off the wire.** Re-serializing parsed JSON produces a different signature (different whitespace, key ordering, float formatting). Capture the body before any framework deserialization.

### Generating a signature (for tests)

```rust
use tango_webhooks::generate;

let body = br#"{"event_type":"alerts.contract.match"}"#;
let sig = generate(body, "your-secret");
// sig == "sha256=<64-char hex>"
```

### Parsing a header

```rust
use tango_webhooks::parse;

let parsed = parse("sha256=abcd...")?;
assert_eq!(parsed.algorithm, "sha256");
assert_eq!(parsed.signature.len(), 64);
# Some(())
```

`parse` accepts both canonical `"sha256=<hex>"` and bare hex (legacy form). Returns `None` on empty, malformed, or non-hex input.

### Axum / Actix middleware

The `tango-webhooks` crate doesn't ship middleware adapters in v0.1 to keep the dependency surface minimal. Drop in a small middleware in your service:

#### Axum

```rust,no_run
use axum::{
    body::{Bytes, Body},
    extract::{Request, State},
    http::StatusCode,
    middleware::{from_fn_with_state, Next},
    response::IntoResponse,
    Router,
};
use tango_webhooks::{verify, SIGNATURE_HEADER};

#[derive(Clone)]
struct WebhookSecret(String);

async fn verify_signature(
    State(secret): State<WebhookSecret>,
    req: Request,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    let (parts, body) = req.into_parts();
    let header = parts
        .headers
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_string();

    let bytes: Bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    if !verify(&bytes, &header, &secret.0) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let req = Request::from_parts(parts, Body::from(bytes));
    Ok(next.run(req).await)
}
```

#### Actix

```rust,ignore
use actix_web::{dev::ServiceRequest, error::ErrorUnauthorized, web::BytesMut};
use tango_webhooks::{verify, SIGNATURE_HEADER};

async fn verify_signature_actix(
    req: ServiceRequest,
    secret: &str,
) -> Result<(ServiceRequest, BytesMut), actix_web::Error> {
    let header = req
        .headers()
        .get(SIGNATURE_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ErrorUnauthorized("missing signature"))?
        .to_string();

    let (req, mut payload) = req.into_parts();
    let mut buf = BytesMut::new();
    while let Some(chunk) = futures::TryStreamExt::try_next(&mut payload)
        .await
        .map_err(|_| ErrorUnauthorized("bad body"))? {
        buf.extend_from_slice(&chunk);
    }
    if !verify(&buf, &header, secret) {
        return Err(ErrorUnauthorized("invalid signature"));
    }
    Ok((req, buf))
}
```

We may ship dedicated `axum` / `actix-web` features on `tango-webhooks` in a future release; track [the roadmap](../ROADMAP.md) for status.

## Managing endpoints and alerts — the SDK side

These methods live on `tango::Client`:

### Endpoints

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_webhook_event_types()` | `GET /api/webhooks/event-types/` | `WebhookEventTypesResponse` |
| `list_webhook_endpoints(opts)` | `GET /api/webhooks/endpoints/` | `Page<WebhookEndpoint>` |
| `get_webhook_endpoint(id)` | `GET /api/webhooks/endpoints/{id}/` | `WebhookEndpoint` |
| `create_webhook_endpoint(input)` | `POST /api/webhooks/endpoints/` | `WebhookEndpoint` |
| `update_webhook_endpoint(id, input)` | `PATCH /api/webhooks/endpoints/{id}/` | `WebhookEndpoint` |
| `delete_webhook_endpoint(id)` | `DELETE /api/webhooks/endpoints/{id}/` | `()` |
| `test_webhook_endpoint(endpoint_id)` | `POST /api/webhooks/endpoints/test-delivery/` | `WebhookTestDeliveryResult` |
| `get_webhook_sample_payload(event_type)` | `GET /api/webhooks/endpoints/sample-payload/[?event_type=...]` | `WebhookSamplePayloadResponse` |

Client-side validation rejects empty `name` or `callback_url` on `create_webhook_endpoint` (the Tango API enforces `unique(user, name)` on endpoints and rejecting locally surfaces a cleaner `Validation` error than the server's 400 on duplicate).

### Alerts

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_webhook_alerts(opts)` | `GET /api/webhooks/alerts/` | `Page<WebhookAlert>` |
| `get_webhook_alert(id)` | `GET /api/webhooks/alerts/{id}/` | `WebhookAlert` |
| `create_webhook_alert(input)` | `POST /api/webhooks/alerts/` | `WebhookAlert` |
| `update_webhook_alert(id, input)` | `PATCH /api/webhooks/alerts/{id}/` | `WebhookAlert` |
| `delete_webhook_alert(id)` | `DELETE /api/webhooks/alerts/{id}/` | `()` |

**Important field semantics on alerts**:

- `query_type` is **singular** (`"contract"`, not `"contracts"`). Client-side validation rejects empty `query_type`.
- `filters` must be a non-empty JSON object. Client-side validation rejects `null` or `{}`.
- `endpoint` is required when the account has multiple endpoints; single-endpoint accounts can omit it (the server auto-resolves).
- `query_type` and `filters` are **read-only after creation**; `update_webhook_alert` only allows `name`, `frequency`, `cron_expression`, `is_active`.

### Example: create and test an endpoint

```rust,no_run
use tango::{Client, models::WebhookEndpointCreateInput};

# async fn run() -> tango::Result<()> {
let client = Client::from_env()?;

let endpoint = client.create_webhook_endpoint(WebhookEndpointCreateInput {
    name: "my-service-prod".into(),
    callback_url: "https://my-service.example.com/tango".into(),
    is_active: Some(true),
    event_types: vec!["alerts.contract.match".into()],
}).await?;

// The secret is only surfaced on creation — save it!
let secret = endpoint.secret.expect("server returns secret on create");
println!("save this somewhere safe: {secret}");

// Fire a synthetic delivery to confirm wiring.
let test = client.test_webhook_endpoint(
    endpoint.id.as_deref().unwrap(),
).await?;
println!("test status={:?} message={:?}", test.status_code, test.message);
# Ok(()) }
```

## Wire format

Every Tango delivery carries:

```text
POST /your-endpoint HTTP/1.1
Content-Type: application/json
X-Tango-Signature: sha256=<lowercase hex HMAC-SHA256 of raw body>
X-Tango-Event: alerts.contract.match
X-Tango-Delivery-Id: <uuid>
X-Tango-Delivery-Timestamp: <unix-seconds>

{ "event_type": "alerts.contract.match", "data": { ... } }
```

The signature is over the **raw body bytes** — capture them before any deserialization. The 32-byte HMAC-SHA256 output is rendered as 64 lowercase hex characters.

## Troubleshooting

**Signature always fails.** Almost always you're verifying against re-serialized JSON instead of the raw bytes. Tap your framework's "raw body" hook and pass those bytes to `verify`.

**Different signatures for the same body.** Whitespace or key ordering in the JSON changed between the server and your re-serialized copy. Use the raw bytes.

**Timing-out deliveries.** Tango retries failed deliveries with exponential backoff. To stop the retries, return `2xx` quickly (then process asynchronously); the API treats anything non-`2xx` as a failure.

**Want a quick test in dev?** Create an endpoint, then call `test_webhook_endpoint(id)`. The server fires a synthetic delivery; the response carries the destination's reply.
