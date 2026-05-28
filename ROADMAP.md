# ROADMAP

This roadmap tracks the Rust SDK only. The goal is to stay closely aligned with the Tango Go / Python / Node SDKs while keeping a first-class Rust experience (typed builders, `thiserror`-based error tree, idiomatic async via `tokio` + `reqwest`).

`tango-go` is the canonical mirror for feature parity — when a new endpoint or response shape lands there, it should land here next.

## Now (0.1)

- [X] Port the full Tango client surface from the Go/Python/Node SDKs to Rust.
- [X] Dynamic response shaping (`shape`, `flat`, `flat_lists`) with full parity.
- [X] Typed error tree built on `thiserror` with `source()`/`Unwrap`-chain ergonomics.
- [X] Cursor + page-numbered pagination (`Stream`-based iterators).
- [X] Webhook signing + verification (`tango-webhooks` crate) wire-compatible with sibling SDKs.
- [X] Webhook CRUD via the API client (endpoints, alerts, event types, test delivery, sample payloads).
- [X] Sub-resource walks for IDVs, entities, agencies, vehicles.
- [X] OTAs / OTIDVs, GSA eLibrary, IT Dashboard, protests, LCATs.

## 0.2 (API sync to v4.6.9)

- [X] Budget surface: `list_budget_accounts` / `iterate_budget_accounts`, `get_budget_account`, `get_budget_account_quarters`, `get_budget_account_recipients` (`/api/budget/accounts/`).
- [X] Singleton detail GETs: `get_contract`, `get_opportunity`, `get_notice`, `get_forecast`, `get_grant`, `get_subaward`.
- [X] Contract sub-routes: `list_contract_subawards`, `list_contract_transactions`.
- [X] `get_entity_budget_flows` (`/api/entities/{uei}/budget-flows/`).
- [X] `grant_id` filter on grants; `cage` filter on entities.
- [X] Removed fabricated `get_idv_summary` / `list_idv_summary_awards` (paths never existed upstream).

## Next

- [ ] Edition 2024 migration once MSRV catches up (re-evaluate `rust-version` floor).
- [ ] Optional `blocking` feature (sync facade over the async client, gated behind a Cargo feature).
- [ ] Newtype identifiers for `Uei`, `EntityId`, `AgencyId`, etc. (compile-time safety, no string typo bugs).
- [ ] Comprehensive integration tests against the live Tango API.
- [ ] Typed structs for remaining lookup/metric endpoints (NAICS, PSC, entity metrics, attachment search).

## Later (0.3)

- [ ] WASM target exploration (`wasm32-unknown-unknown` / browser-friendly build).
- [ ] First-class `axum` and `actix-web` middleware crates for `tango-webhooks` (verify-and-extract handlers).
- [ ] More targeted examples and recipes in `docs/` and `examples/`.
- [ ] Structured logging hooks (`tracing` integration with span attributes per request).

## Maybe Someday

- [ ] Generated method coverage from the Tango OpenAPI spec to keep parity drift to zero.
- [ ] Alternative HTTP transports (e.g. swap `reqwest` for `hyper` directly, or pluggable `Client` trait).
- [ ] First-class OpenTelemetry spans for all HTTP calls.
- [ ] CLI utilities built on top of the Rust SDK.
