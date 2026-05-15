# Client

Complete reference for constructing and configuring a `tango::Client`. For a deeper architectural walk-through see [`ARCHITECTURE.md`](ARCHITECTURE.md); for the per-resource methods see [`API_REFERENCE.md`](API_REFERENCE.md).

## Constructing a client

`Client::builder()` returns a `bon`-derived builder. The required field is `api_key`; everything else is optional with documented defaults.

```rust
use tango::Client;
use std::time::Duration;

# fn run() -> tango::Result<()> {
let client = Client::builder()
    .api_key("your-tango-api-key")
    .timeout(Duration::from_secs(60))
    .retries(5)
    .retry_backoff(Duration::from_millis(500))
    .user_agent("my-service/1.0".to_string())
    .build()?;
# Ok(()) }
```

When `TANGO_API_KEY` is set in the environment, you can construct the client with no explicit API key:

```rust
use tango::Client;
# fn run() -> tango::Result<()> {
// Reads TANGO_API_KEY (required) and TANGO_BASE_URL (optional) from env.
let client = Client::from_env()?;
// Equivalent to: Client::builder().build()
# Ok(()) }
```

### Builder fields

| Field | Type | Default | Notes |
| ----- | ---- | ------- | ----- |
| `api_key` | `Option<String>` (req'd) | `TANGO_API_KEY` env var | Empty string falls back to env var. |
| `base_url` | `Option<String>` | `TANGO_BASE_URL` env, else `https://tango.makegov.com` | Trailing slash optional. |
| `timeout` | `Option<Duration>` | 30s | `Duration::ZERO` disables the deadline. |
| `retries` | `Option<u32>` | 3 | First attempt doesn't count; total attempts = `retries + 1`. |
| `retry_backoff` | `Option<Duration>` | 250ms | Doubles each attempt, capped at 10s. `Retry-After` overrides. |
| `user_agent` | `Option<String>` | `tango-rust/<version>` | Set to a service identifier in production. |
| `http_client` | `Option<reqwest::Client>` | new default `reqwest::Client` | Inject for proxy/tracing/etc. Its built-in timeout is ignored — per-request deadlines are applied here. |

## Concurrency

`Client` is `Clone + Send + Sync`. All state lives behind an internal `Arc`, so cloning is cheap. Share a single instance across tokio tasks:

```rust
# use tango::Client;
# async fn run(client: Client) -> tango::Result<()> {
let c = client.clone();
let handle = tokio::spawn(async move {
    let _ = c.get_version().await;
});
handle.await.unwrap();
# Ok(()) }
```

There is no notion of per-request configuration that lives on the client. To deviate from the defaults on a specific call (e.g. a longer timeout for a slow analytics query), construct a separate client.

## Authentication

The SDK sends `X-API-KEY: <key>` on every request — **not** `Authorization: Bearer ...`. This matches the canonical Tango header and the sibling SDKs (`tango-node`, `tango-python`, `tango-go`).

Empty / missing API keys are not rejected at construction time — the API returns a 401 on the first request, which surfaces as `Error::Auth { response }`. To validate the key up front, call any cheap method (e.g. `client.get_version().await`).

## Environment variables

| Variable | Purpose |
| -------- | ------- |
| `TANGO_API_KEY` | Fallback API key when none is passed to the builder. |
| `TANGO_BASE_URL` | Fallback base URL when none is passed and `api_key` is also coming from env. Useful for pointing at staging. |

The builder's explicit setters always win over env vars.

## Timeouts

`timeout` is per-request (applied to each retry attempt independently). When a deadline elapses, the SDK returns `Error::Timeout { timeout }` and the retry loop sees it as retryable.

`Duration::ZERO` disables the deadline. Useful for very long-running export endpoints; otherwise leave the default at 30s.

The underlying `reqwest::Client::timeout` is ignored — the SDK applies its own deadline via `RequestBuilder::timeout`.

## Retries and backoff

By default the SDK retries 4 times (3 retries + initial attempt) on:

- HTTP `5xx` server-side faults.
- HTTP `408` (server-reported request timeout).
- HTTP `429` (rate limited). When the response carries `Retry-After`, that value is used (capped at 10s); otherwise exponential backoff applies.
- Transport-level errors: DNS, connection refused, TCP reset, TLS, hyper-internal errors.
- Per-request timeouts.

The wait between attempts is `min(retry_backoff × 2^attempt, 10s)`. The default initial backoff of 250ms expands to 0.25s → 0.5s → 1s → 2s before the cap kicks in.

Set `.retries(0)` to disable retries entirely:

```rust
# use tango::Client;
let client = Client::builder().api_key("x").retries(0).build().unwrap();
```

The SDK retries are **independent of the caller's retry policy**. If you wrap the SDK in your own retry layer, you usually want `.retries(0)` to avoid compounding waits.

`Error::is_retryable()` exposes the SDK's retry decision for callers building custom retry middleware.

## Observability

### Rate-limit snapshot

```rust
# use tango::Client;
# async fn run(client: Client) -> tango::Result<()> {
let _ = client.get_version().await?;
if let Some(info) = client.rate_limit_info() {
    println!(
        "{:?}/{:?} remaining; resets in {:?}s; bucket={:?}",
        info.remaining, info.limit, info.reset_in, info.limit_type,
    );
}
# Ok(()) }
```

Populated from the server's `X-RateLimit-Remaining`, `X-RateLimit-Limit`, `X-RateLimit-Reset`, `X-RateLimit-Type`, and `Retry-After` headers. Any field absent on the response is `None`. Headers that fail integer parsing also surface as `None`.

### Last response headers

```rust
# use tango::Client;
# async fn run(client: Client) -> tango::Result<()> {
let _ = client.get_version().await?;
if let Some(headers) = client.last_response_headers() {
    if let Some(req_id) = headers.get("x-request-id") {
        println!("request id: {req_id:?}");
    }
}
# Ok(()) }
```

Useful for surfacing `X-Request-Id` / `X-Tango-Trace-Id` in your application logs.

Both accessors return `Option<...>` and return `None` when no request has completed on this client yet.

## Error handling

Every fallible call returns `Result<T, tango::Error>`. `Error` is a single `thiserror`-derived enum; match on the variant for programmatic dispatch:

```rust
use tango::{Client, Error};

# async fn run(client: Client) -> tango::Result<()> {
match client.get_agency("9700", None).await {
    Ok(agency) => println!("{}", agency.name.unwrap_or_default()),
    Err(Error::Auth { .. }) => eprintln!("bad API key"),
    Err(Error::NotFound { .. }) => println!("no such agency"),
    Err(Error::RateLimit { retry_after, limit_type, .. }) => {
        println!("rate limited; retry in {retry_after}s (bucket: {limit_type:?})");
    }
    Err(Error::Validation { message, .. }) => eprintln!("bad input: {message}"),
    Err(Error::Timeout { timeout }) => eprintln!("timed out after {timeout:?}"),
    Err(Error::Api { status, message, response }) => {
        eprintln!("status={status}: {message}; body={response:?}");
    }
    Err(Error::Transport(e)) => eprintln!("network error: {e}"),
    Err(Error::Decode(e)) => eprintln!("malformed response: {e}"),
    Err(Error::Build(s)) => eprintln!("SDK internal: {s}"),
}
# Ok(()) }
```

Helpers:

- `err.status() -> Option<u16>` — HTTP status when the error has one.
- `err.is_retryable() -> bool` — whether the SDK considers it retryable (it already retries internally; this is for callers building their own policies).
- `err.response() -> Option<&ErrorBody>` — the parsed server error body for variants that carry one.

The `response` payload on each variant exposes:

- `response.message` — the SDK's best guess at a human-readable line extracted from the body (envelope keys `detail` / `message` / `error` first, then sorted-key iteration over the remaining keys).
- `response.raw` — the full decoded JSON `Value`, in case you want to introspect a custom shape.

## Custom HTTP client

For proxy, tracing, or middleware injection, pass a pre-built `reqwest::Client`:

```rust
# use tango::Client;
# use std::time::Duration;
let http = reqwest::Client::builder()
    .pool_max_idle_per_host(50)
    .build()
    .unwrap();

let client = Client::builder()
    .api_key("x")
    .http_client(http)
    .build()
    .unwrap();
```

The custom client's built-in `timeout` is ignored — `Client::builder().timeout(...)` is the per-request deadline.

## Iteration

For paginated walks, the `iterate_*` methods return a `PageStream<T>` (which is `futures::Stream<Item = Result<T>>`):

```rust
use tango::{Client, ListContractsOptions};
use futures::TryStreamExt;

# async fn run(client: Client) -> tango::Result<()> {
let mut s = client.iterate_contracts(
    ListContractsOptions::builder().awarding_agency("9700").build(),
);
while let Some(c) = s.try_next().await? {
    // process one contract at a time
    let _ = c;
}
# Ok(()) }
```

For "give me everything":

```rust
# use tango::{Client, ListContractsOptions};
# async fn run(client: Client) -> tango::Result<()> {
let all = client
    .iterate_contracts(ListContractsOptions::builder().awarding_agency("9700").build())
    .collect_all()
    .await?;
# Ok(()) }
```

Use `try_next` rather than `collect_all` for large result sets to bound memory.
