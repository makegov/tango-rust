# Tango Rust SDK

[![Crates.io](https://img.shields.io/crates/v/tango.svg)](https://crates.io/crates/tango)
[![Documentation](https://docs.rs/tango/badge.svg)](https://docs.rs/tango)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Official, async-first Rust SDK for the [Tango API](https://tango.makegov.com) — federal contracts, IDVs, entities, opportunities, grants, vehicles, and more, with dynamic response shaping so you fetch only the fields you need.

> **In development — v0.1.0.** Not yet at sibling-SDK parity. The public API may shift before v1.0.0. Pin to a specific tag if you depend on this in production.

Sibling SDKs (`tango-node`, `tango-python`) are at v1.0.0; `tango-go` is at v0.1.0 with the same surface. This Rust port ships the full transport, error model, retry / rate-limit handling, webhook signing (in the separate `tango-webhooks` crate), and the ~75-method API surface. Same `0.x → 1.0` graduation logic as Go: the transport and types are stable; the surface stabilizes once it matches sibling parity.

## Features

- **Async-first** — built on `tokio` + `reqwest`. One runtime, clean `futures::Stream`-based pagination.
- **Dynamic response shaping** — request exactly the fields you need via 21 built-in [`SHAPE_*`](https://docs.rs/tango/latest/tango/#constants) presets or a custom comma-separated field list.
- **Typed errors** — single [`Error`](https://docs.rs/tango/latest/tango/enum.Error.html) enum with rich payload variants (`Auth`, `NotFound`, `Validation`, `RateLimit`, `Timeout`, `Api`, `Transport`, `Decode`, `Build`). Programmatic dispatch via `err.status()` and `err.is_retryable()`.
- **Smart retries** — automatic backoff on 5xx / 408 / 429 / transport errors, honoring the server's `Retry-After` header.
- **Compile-time-checked client builder** — via [`bon`](https://docs.rs/bon). Missing `api_key`? Won't compile.
- **Async pagination** — `PageStream<T>` implements `futures::Stream`. Yields one item at a time, fetches successive pages automatically.
- **Forward-compatible models** — every typed model carries `#[serde(flatten)] extra: HashMap<String, Value>` so a server-side schema addition surfaces in `record.extra["new_field"]` rather than being silently dropped.
- **Webhook signing** — in the separate [`tango-webhooks`](https://docs.rs/tango-webhooks) crate: zero transport deps, HMAC-SHA256 verification, constant-time via `subtle`.

## Installation

```toml
[dependencies]
tango = "0.1"

# Optional: webhook signing for receivers
tango-webhooks = "0.1"
```

Requires Rust **1.80** or later.

## Quick start

```rust,no_run
use tango::Client;
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> tango::Result<()> {
    let client = Client::builder().api_key("your-api-key").build()?;

    // Single page
    let page = client.list_contracts(
        tango::ListContractsOptions::builder()
            .awarding_agency("9700")
            .shape(tango::SHAPE_CONTRACTS_MINIMAL)
            .limit(25u32)
            .build(),
    ).await?;
    for record in page.results {
        println!("{:?}", record.get("piid"));
    }

    // Walk every page as a stream
    let mut stream = client.iterate_contracts(
        tango::ListContractsOptions::builder()
            .awarding_agency("9700")
            .fiscal_year("2025")
            .build(),
    );
    while let Some(record) = stream.try_next().await? {
        println!("{:?}", record.get("piid"));
    }

    Ok(())
}
```

### Get a typed agency

`get_agency` returns the typed [`AgencyRecord`](https://docs.rs/tango/latest/tango/models/struct.AgencyRecord.html); forward-compatible fields land in `agency.extra`.

```rust,no_run
# use tango::Client;
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let agency = client.get_agency("9700", None).await?;
println!("{}", agency.name.unwrap_or_default());
# Ok(()) }
```

### Get a fully-shaped entity

```rust,no_run
# use tango::{Client, GetEntityOptions, SHAPE_ENTITIES_COMPREHENSIVE};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let entity = client.get_entity(
    "ABC123DEF456",
    Some(GetEntityOptions::builder().shape(SHAPE_ENTITIES_COMPREHENSIVE).build()),
).await?;
# Ok(()) }
```

## Authentication

Pass an API key via `Client::builder().api_key(...)`, or set `TANGO_API_KEY` in the environment and call `Client::from_env()`. You can also override the base URL via `TANGO_BASE_URL` for staging environments, or via the builder:

```rust,no_run
# use tango::Client;
# async fn run() -> tango::Result<()> {
let client = Client::builder()
    .api_key("your-key")
    .base_url("https://staging.tango.makegov.com".to_string())
    .build()?;
# Ok(()) }
```

## Core concepts

### Dynamic response shaping

Every list and get endpoint accepts a `shape` parameter selecting which fields the API returns. The SDK ships 21 presets matching the Node, Python, and Go SDKs:

```rust,no_run
# use tango::{Client, ListContractsOptions, SHAPE_CONTRACTS_MINIMAL};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
// Preset:
let page = client.list_contracts(
    ListContractsOptions::builder().shape(SHAPE_CONTRACTS_MINIMAL).build(),
).await?;

// Custom:
let page = client.list_contracts(
    ListContractsOptions::builder()
        .shape("key,piid,recipient(uei,display_name)")
        .build(),
).await?;
# Ok(()) }
```

See [`docs/SHAPES.md`](docs/SHAPES.md) for the full grammar and the available presets.

### Pagination

List endpoints support both page-based and cursor-based pagination. `Page<T>` exposes the next URL plus the extracted `cursor` for keyset endpoints:

```rust,no_run
# use tango::{Client, ListIDVsOptions};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let page = client.list_idvs(ListIDVsOptions::builder().limit(50u32).build()).await?;
// Fetch the next page:
let next_page = client.list_idvs(
    ListIDVsOptions::builder().limit(50u32).cursor(page.cursor.unwrap_or_default()).build()
).await?;
# Ok(()) }
```

Or just iterate via `PageStream<T>`:

```rust,no_run
# use tango::{Client, ListIDVsOptions};
# use futures::TryStreamExt;
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let mut stream = client.iterate_idvs(ListIDVsOptions::builder().build());
while let Some(idv) = stream.try_next().await? {
    // process one IDV at a time
    let _ = idv;
}
# Ok(()) }
```

Or drain the whole thing:

```rust,no_run
# use tango::{Client, ListIDVsOptions};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let all = client.iterate_idvs(ListIDVsOptions::builder().build()).collect_all().await?;
# Ok(()) }
```

### Rate limits

After each request, `client.rate_limit_info()` returns a snapshot of the rate-limit headers:

```rust,no_run
# use tango::Client;
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
if let Some(info) = client.rate_limit_info() {
    println!(
        "remaining={:?} limit={:?} reset_in={:?}",
        info.remaining, info.limit, info.reset_in
    );
}
# Ok(()) }
```

The client also automatically retries 429s, honoring `Retry-After`. See [`docs/CLIENT.md`](docs/CLIENT.md) for the full retry semantics.

### Webhook verification

Webhook signing lives in the separate [`tango-webhooks`](https://crates.io/crates/tango-webhooks) crate so a receiver service doesn't have to pull in the full SDK:

```rust
use tango_webhooks::{verify, SIGNATURE_HEADER};

fn check(raw_body: &[u8], header_value: &str, secret: &str) -> bool {
    verify(raw_body, header_value, secret)
}
```

The CRUD methods for managing webhook endpoints and alerts live on `tango::Client`:
`list_webhook_endpoints`, `create_webhook_endpoint`, `update_webhook_endpoint`, `delete_webhook_endpoint`, `test_webhook_endpoint`, `list_webhook_alerts`, `create_webhook_alert`, etc.

See [`docs/WEBHOOKS.md`](docs/WEBHOOKS.md) for the full surface, signature format, and middleware patterns.

## Error handling

Every fallible call returns `Result<T, tango::Error>`. The `Error` enum carries rich payloads — match the variant to dispatch on the failure mode:

```rust,no_run
use tango::{Client, Error};

# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
match client.get_agency("9700", None).await {
    Ok(agency) => println!("{}", agency.name.unwrap_or_default()),
    Err(Error::NotFound { .. }) => println!("not found"),
    Err(Error::RateLimit { retry_after, limit_type, .. }) => {
        println!("rate limited; retry in {retry_after}s (bucket: {limit_type:?})");
    }
    Err(Error::Validation { message, .. }) => eprintln!("bad input: {message}"),
    Err(Error::Api { status, message, .. }) => eprintln!("status={status}: {message}"),
    Err(e) => return Err(e),
}
# Ok(()) }
```

`err.is_retryable()` reports the SDK's retry decision (it already retries internally; this is for callers building their own retry policies on top). `err.status()` returns the HTTP status when one is associated with the error.

## API methods

The SDK exposes ~75 methods on `Client` covering every endpoint in the sibling SDKs. The most-used 15 are listed here; the **full method-by-method reference lives in [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md)**.

| Resource | List | Get | Iterate |
| ---- | ---- | ---- | ---- |
| Agencies | `list_agencies` | `get_agency` *(typed: `AgencyRecord`)* | — |
| Contracts | `list_contracts` | — | `iterate_contracts` |
| Entities | `list_entities` | `get_entity` | `iterate_entities` |
| IDVs | `list_idvs` | `get_idv` | `iterate_idvs` |
| Vehicles | `list_vehicles` | `get_vehicle` | `iterate_vehicles` |
| OTAs | `list_otas` | `get_ota` | `iterate_otas` |
| OTIDVs | `list_otidvs` | `get_otidv` | `iterate_otidvs` |
| Opportunities | `list_opportunities` | — | `iterate_opportunities` |
| Notices | `list_notices` | — | `iterate_notices` |
| Forecasts | `list_forecasts` | — | `iterate_forecasts` |
| Grants | `list_grants` | — | `iterate_grants` |
| Protests | `list_protests` | `get_protest` *(typed: `ProtestRecord`)* | `iterate_protests` |
| IT Dashboard | `list_itdashboard` | `get_itdashboard` | `iterate_itdashboard` |
| NAICS / PSC | `list_naics` / `list_psc` | `get_naics` / `get_psc` | — |
| Webhooks (CRUD) | `list_webhook_endpoints` / `list_webhook_alerts` | `get_` / `create_` / `update_` / `delete_` / `test_` | — |

Sub-resources and lookups: `list_entity_contracts` / `_idvs` / `_otas` / `_otidvs` / `_subawards` / `_lcats` / `get_entity_metrics`, `list_idv_awards` / `_child_idvs` / `_transactions` / `_lcats`, `list_agency_awarding_contracts` / `_funding_contracts`, `list_vehicle_awardees` / `_orders`, `list_otidv_awards`, `list_gsa_elibrary_contracts`, `list_business_types`, `list_offices`, `list_departments`, `list_mas_sins`, `list_assistance_listings`, `list_lcats` (dispatcher). Meta: `resolve`, `validate`, `get_version`, `list_api_keys`, `search_opportunity_attachments`. Metrics dispatcher: `list_metrics`.

See [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md) for full signatures, filter fields, and quirks.

## Workspace layout

This is a Cargo workspace with two published crates:

```
.
├── crates/
│   ├── tango/             # main SDK
│   └── tango-webhooks/    # HMAC-SHA256 signing/verification (no transport deps)
├── docs/                  # in-repo guides
├── justfile               # task runner (test / fmt / lint / cover / release-check)
├── Cargo.toml             # workspace manifest
└── README.md
```

## Documentation

In-repo guides:

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — crate layout, request lifecycle, design rationale
- [`docs/CLIENT.md`](docs/CLIENT.md) — builder options, env vars, retry semantics, error model, rate-limit observability
- [`docs/API_REFERENCE.md`](docs/API_REFERENCE.md) — method-by-method reference for every public method
- [`docs/SHAPES.md`](docs/SHAPES.md) — shape grammar, the 21 presets, `flat` / `flat_lists`, trade-offs
- [`docs/WEBHOOKS.md`](docs/WEBHOOKS.md) — receiving deliveries, CRUD methods, troubleshooting

External:

- API docs (Rust): <https://docs.rs/tango> · <https://docs.rs/tango-webhooks>
- API reference: <https://docs.makegov.com>
- Sibling SDKs: [`@makegov/tango-node`](https://github.com/makegov/tango-node) · [`tango-python`](https://github.com/makegov/tango-python) · [`tango-go`](https://github.com/makegov/tango-go)

## Development

```bash
just test         # cargo test --workspace
just fmt          # cargo fmt --all
just lint         # cargo clippy --workspace --all-features -- -D warnings
just cover        # cargo llvm-cov --workspace --lcov --output-path lcov.info
just ci           # fmt-check + lint + test (run before opening a PR)
just release-check  # cargo publish --dry-run for both crates
```

## Requirements

- Rust **1.80** or later.

## License

MIT — see [LICENSE](LICENSE).

## Support

Open an issue at <https://github.com/makegov/tango-rust/issues> or email support@makegov.com.

## Contributing

PRs welcome. See [`CONTRIBUTING.md`](CONTRIBUTING.md). Run `just ci` before opening — fmt, clippy, and the full test suite must pass.
