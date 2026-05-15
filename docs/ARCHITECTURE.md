# Architecture

A walk-through of how the `tango` and `tango-webhooks` crates are laid out, why they're shaped the way they are, and what happens when you call a method on `Client`. Read this first if you want to extend the SDK; the other docs are reference material.

## Workspace + crate layout

The repo is a Cargo workspace with two published members:

```text
tango-rust/                           (workspace root)
├── Cargo.toml                        # workspace manifest, pinned shared deps
├── crates/
│   ├── tango/                        # the SDK
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                # crate doc + re-exports
│   │       ├── client.rs             # Client + bon-derived ClientBuilder
│   │       ├── transport.rs          # request loop, retries, header parsing,
│   │       │                         # validation-message extraction
│   │       ├── error.rs              # Error enum (thiserror) + helpers
│   │       ├── pagination.rs         # Page<T> + PageStream<T>
│   │       ├── shapes.rs             # SHAPE_* constants + DEFAULT_BASE_URL
│   │       ├── internal.rs           # ListOptions + query-string helpers
│   │       ├── version.rs            # VERSION const
│   │       ├── models/               # Typed input/output structs
│   │       │   ├── agency.rs
│   │       │   ├── protest.rs
│   │       │   ├── resolve.rs
│   │       │   ├── validate.rs
│   │       │   └── webhook.rs
│   │       └── resources/            # One module per resource family
│   │           ├── agencies.rs
│   │           ├── contracts.rs
│   │           ├── idvs.rs
│   │           ├── idv_subresources.rs
│   │           ├── entities.rs
│   │           ├── entity_subresources.rs
│   │           ├── vehicles.rs
│   │           ├── vehicle_subresources.rs
│   │           ├── opportunities.rs   # opps / notices / forecasts / grants / attachment search
│   │           ├── otas.rs            # OTAs + OTIDVs
│   │           ├── subawards.rs
│   │           ├── gsa.rs
│   │           ├── protests.rs
│   │           ├── itdashboard.rs
│   │           ├── lcats.rs           # dispatcher (entity vs IDV)
│   │           ├── lookups.rs        # organizations, NAICS, PSC, business types,
│   │           │                     # offices, departments, MAS SINs, assistance listings
│   │           ├── metrics.rs        # naics/psc/entity getters + dispatcher
│   │           ├── resolve_validate.rs
│   │           ├── webhooks_api.rs   # webhook endpoint + alert CRUD (server-side)
│   │           └── meta.rs           # get_version + list_api_keys
│   └── tango-webhooks/               # signing + verification (no transport deps)
│       ├── Cargo.toml
│       └── src/lib.rs                # generate / verify / parse + constants
├── docs/                             # in-repo guides (this directory)
├── .github/workflows/                # CI: test / lint / coverage / release / docs-dispatch
├── justfile                          # task recipes
└── README.md
```

### Why two crates?

`tango-webhooks` exists separately because signature verification has a fundamentally different consumer profile: a webhook receiver service shouldn't have to pull `reqwest`, the retry loop, the rate-limit state, or the resource methods just to compute an HMAC. Keeping the signing crate tiny — `hmac`, `sha2`, `subtle`, `hex`, and nothing else — keeps verifier builds fast and dependency surface auditable.

The SDK crate doesn't depend on the webhooks crate (the signing primitives aren't needed inside the API client). The two are wire-compatible: a signature produced by `tango_webhooks::generate` verifies fine on the server, and vice versa.

### Why a single resource directory?

Idiomatic Rust HTTP SDKs (e.g. `octocrab`, `aws-sdk-rust`) keep the public surface flat on a single `Client` type so callers can write `client.list_contracts(...)` rather than `client.contracts().list(...)`. The internal layout — one file per resource family — is purely organizational; nothing in the public surface reflects it.

### Typed structs vs. `Record`

Most list/get methods return `Record` (an alias for `serde_json::Map<String, Value>`) or `Page<Record>`. The dynamic-shape feature means the response schema varies per call — generating a typed struct per shape is unprofitable.

A small hand-picked set of methods return typed structs where the sibling SDKs already define them and the schema is stable:

| Method | Return type |
| ------ | ----------- |
| `get_agency` | `AgencyRecord` |
| `get_protest` | `ProtestRecord` |
| `resolve` | `ResolveResult` (with `Vec<ResolveCandidate>`) |
| `validate` | `ValidateResult` |
| `list_webhook_endpoints` / `get_*` / `create_*` / `update_*` | `Page<WebhookEndpoint>` / `WebhookEndpoint` |
| `list_webhook_alerts` / `get_*` / `create_*` / `update_*` | `Page<WebhookAlert>` / `WebhookAlert` |
| `list_webhook_event_types` | `WebhookEventTypesResponse` |
| `get_webhook_sample_payload` | `WebhookSamplePayloadResponse` |
| `test_webhook_endpoint` | `WebhookTestDeliveryResult` |

Every typed struct carries `#[serde(flatten)] pub extra: HashMap<String, Value>` so forward-compatible fields the server adds are preserved without dropping back to `Record`. See `models/` for the pattern.

## Major components

### `Client` and `ClientBuilder`

`Client` is the entry point. Construct it with the `bon`-derived builder:

```rust
let client = tango::Client::builder()
    .api_key("…")
    .timeout(std::time::Duration::from_secs(60))
    .build()?;
```

The builder enforces required-field correctness at compile time. Required: `api_key` (or the `TANGO_API_KEY` env var). Everything else is optional with documented defaults.

`Client` is `Clone + Send + Sync` — clone it freely across tasks. All state is held behind an `Arc` so clones are cheap.

There is no per-request configuration. Timeouts, retries, etc. are resolved at construction. To deviate on a specific call, construct a separate client.

### Transport (`transport.rs`)

The request loop is `send_with_retries(inner, method, url, body)`. Each attempt:

1. Builds a `reqwest::RequestBuilder` with `X-API-KEY`, `Accept`, `User-Agent`, and (for POST/PATCH) `Content-Type: application/json`.
2. Applies the per-request timeout.
3. Awaits the response, reads bytes.
4. Snapshots the response headers (rate-limit + everything else) for `client.rate_limit_info()` and `client.last_response_headers()`.
5. Returns `Ok(bytes)` on 2xx, or maps the non-2xx status to the appropriate `Error` variant.

Retries fire on `5xx`, `408`, `429`, transport errors (DNS / TLS / TCP), and timeouts. Backoff is exponential (`retry_backoff × 2^attempt`) capped at 10s. The server's `Retry-After` header (delta-seconds form) overrides the computed backoff when present on 429.

The body of a POST/PATCH is serialized once and re-used across retry attempts — we don't re-encode on each try.

### Error model (`error.rs`)

All fallible calls return `Result<T, Error>` where `Error` is a single `thiserror`-derived enum:

| Variant | When | Retryable? |
| ------- | ---- | ---------- |
| `Auth { response }` | HTTP 401 | no |
| `NotFound { response }` | HTTP 404 | no |
| `Validation { message, response }` | HTTP 400 | no |
| `RateLimit { retry_after, limit_type, response }` | HTTP 429 | **yes** |
| `Timeout { timeout }` | per-request deadline elapsed | **yes** |
| `Api { status, message, response }` | other non-2xx | **yes** for 5xx / 408 |
| `Transport(reqwest::Error)` | DNS / TLS / TCP failure | **yes** |
| `Decode(serde_json::Error)` | response body wasn't valid JSON | no |
| `Build(String)` | SDK couldn't construct the request | no |

Programmatic helpers: `err.is_retryable()`, `err.status()`, `err.response()`. The `response` payload on each variant carries the parsed server error body (when one was returned), so callers can introspect a custom error shape without re-parsing.

Validation-message extraction (`extract_validation_message`) walks the parsed body in priority order: `detail` → `message` → `error` envelope keys, then sorted-key iteration over the remaining keys (preferring array values over strings). The sort makes the surfaced message deterministic — Rust HashMaps have randomized iteration order, so naive walks would surface different messages across runs.

### Pagination (`pagination.rs`)

`Page<T>` is the decoded envelope returned by `list_*` methods:

```rust
pub struct Page<T> {
    pub count: u64,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub page_metadata: Option<Value>,
    pub cursor: Option<String>,  // extracted from `next` for keyset endpoints
    pub results: Vec<T>,
}
```

`PageStream<T>` is the async stream returned by `iterate_*` methods. It implements `futures::Stream<Item = Result<T, Error>>`. Internally:

1. On the first poll, it kicks off the first fetch.
2. When the fetch completes, it buffers `results` and inspects `next`:
   - If `next` is `None`, it sets `done = true` (but keeps the buffer).
   - If `next` carries a `?cursor=` query param, it stores the cursor for the next fetch.
   - Else, if `next` carries `?page=N`, it stores `N`.
3. Subsequent polls yield buffered items first, then kick off the next fetch when the buffer drains.

**Buffer-drain-before-honoring-done** is the same bug fix the Go SDK applied (Go's F1 fix): the last page legitimately holds results even after `next=null`, so yielding them before checking `done` is what makes single-page-N-items and last-page-N-items iteration correct.

`PageStream::collect_all()` is the convenience for "give me every result"; for huge result sets, prefer the streaming `try_next` to bound memory.

### Builder pattern (bon)

Every options struct is `#[derive(Builder)]` from `bon`. This gives callers a fluent, type-state-checked construction site:

```rust
let opts = ListContractsOptions::builder()
    .awarding_agency("9700")
    .fiscal_year("2025")
    .shape(SHAPE_CONTRACTS_MINIMAL)
    .limit(25u32)
    .build();
```

Top-level pagination fields (`page`, `limit`, `cursor`, `shape`, `flat`, `flat_lists`) are flattened onto every list-options builder for call-site brevity. The `internal::apply_pagination` helper handles the conversion to query params consistently.

## Request lifecycle

A typical call:

```rust
client.list_contracts(opts).await?
```

unfolds as:

1. `list_contracts` calls `opts.to_query()` to produce a `Vec<(String, String)>`.
2. It calls `Client::get_bytes("/api/contracts/", &q)`, which:
   - Resolves `base_url + path` and appends query pairs.
   - Hands the URL to `transport::send_with_retries`.
3. `send_with_retries` runs `attempt_once` up to `retries + 1` times.
4. `attempt_once` builds the `reqwest::Request`, sends it, captures headers/body, and either returns bytes (2xx) or constructs the appropriate `Error` variant (non-2xx).
5. On a retryable error, the retry loop sleeps `min(backoff(attempt), MAX_BACKOFF)` (or `Retry-After` if set) and tries again.
6. `list_contracts` calls `Page::<Record>::decode(&bytes)` and returns the typed page.

For typed return types (e.g. `get_agency` → `AgencyRecord`), the resource method calls `Client::get_json::<AgencyRecord>(path, &q)` which serializes the path/query the same way and decodes via `serde_json::from_slice`.

## Test layout

- Unit tests live in `#[cfg(test)] mod tests` blocks inline next to the code they exercise. Pure-logic tests (query-string builders, header parsing, error decoding) need no HTTP.
- HTTP-level tests use [`httpmock`](https://crates.io/crates/httpmock) — see the `client.rs` and `transport.rs` test modules, and the per-resource test modules.
- Doctests live on every public item that's worth a runnable example. They're `no_run` for anything that would hit the network.
- Integration tests gated by `#[ignore]` (so `just integration` runs them, `cargo test` skips them) live in `tests/` and require `TANGO_API_KEY` in the env.

## Versioning + release flow

- `tango` and `tango-webhooks` share the workspace `version` field and ship together.
- The release workflow publishes `tango-webhooks` first, then `tango`, on a `v*` tag.
- Pre-1.0 the public surface is stabilizing; 1.0 will ship once the API matches sibling SDKs and the live-API smoke pass is clean.
