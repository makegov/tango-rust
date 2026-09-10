<!-- markdownlint-disable MD024 MD013 -->
# Changelog

All notable changes to the `makegov-tango` and `makegov-tango-webhooks` crates are documented here.

This project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### `makegov-tango`

#### Added

- **`attachments(extracted_text)` — SLED document bodies on the Small plan and above** (Tango API 4.25.1; parity with tango-python, tango-node and tango-go). This SDK returns `Record`, so the leaf needs no type change — what it needed was saying so. Documented on `get_sled_opportunity`, on `SHAPE_SLED_OPPORTUNITIES_COMPREHENSIVE` and in `docs/API_REFERENCE.md`: the leaf must be **named** (no `SHAPE_SLED_*` constant includes it, and `attachments(*)` does not carry it, because the API resolves the body only for a caller who asked); the **key is absent rather than null** when the text is not being served; and a **contested document never returns text at any plan**. `suggested_shapes_do_not_name_the_paid_document_body` pins the constants. Searching document text stays ungated on every plan and returns no fragment of it.

- **State, local and education (SLED) procurement** (Tango API 4.25.0; parity with tango-python, tango-node and tango-go). `resources/sled.rs` adds `list_sled_opportunities` / `get_sled_opportunity`, `list_sled_opportunity_revisions`, `get_sled_coverage`, `list_sled_forecasts` / `get_sled_forecast`, plus `iterate_sled_opportunities` and `iterate_sled_forecasts`. Four `bon`-derived options builders (`ListSledOpportunitiesOptions`, `ListSledOpportunityRevisionsOptions`, `ListSledForecastsOptions`, `GetSledOptions`) name every one of the API's 27 solicitation filters and 13 forecast filters, and five `SHAPE_SLED_*` constants land in `shapes.rs`.

  Four behaviors are documented on the options builders because each misleads a caller who assumes federal semantics. **Leaving both `status` and `active` unset returns open solicitations only** — that default is the API's, and `list_sled_opportunities` deliberately does not synthesize `status=open`, since doing so would make `active = Some(false)` unreachable (pinned by a test). **`status` is Tango-derived and refreshed every fifteen minutes**; the portal's own word is served as `source_status`, is frozen at last capture, and is not a liveness filter. **Category scheme tagging is mid-migration**, so `naics` matches only the small tagged share and `category_code` is the escape hatch. And **`meta.attachment_count` can be lower than the length of `attachments`**, because an auto-generated portal cover sheet is listed and flagged `is_generated_summary` but excluded from the count.

  `active`, `has_documents` and `source_declared` are `Option<bool>` rather than `bool`, so `false` reaches the server as a filter value instead of collapsing into the default — the same reason `push_opt_bool` exists.

  `SHAPE_SLED_REVISIONS_MINIMAL` omits `changes` on purpose: the per-field before/after needs a Small plan, so naming it in a suggested shape would 403 a Free caller. `changed_fields` is in the shape and available at every plan.

#### Documentation

- New **State & Local — SLED** section in `docs/API_REFERENCE.md` covering all six methods and both defaults that surprise people.
- `docs/WEBHOOKS.md` alert semantics gained the date-lapse rule and its one exception. An exclusion or DIBBS solicitation reaching its date fires nothing, because open/closed is derived at query time — but `alerts.sled_opportunity.match` **does** fire on a closing, since SLED liveness is a stored column a fifteen-minute sweep writes.

## [0.1.0] — 2026-05-15

First public release of the Tango Rust SDK.

This is an initial **v0.1.0** rather than v1.0.0 — the sibling `tango-node` and `tango-python` SDKs are at v1.0.0; `tango-go` is at v0.1.0 with the same surface. Same logic: the transport, error model, retry / rate-limit handling, pagination, and webhook signing are at sibling-SDK quality and not expected to change. The resource-method surface stabilizes once integration with the sibling SDKs is verified against the live API. Final 1.0.0 is the target.

### `makegov-tango` v0.1.0

#### Added

- `Client` with a `bon`-derived builder (`Client::builder().api_key(...).build()?`) plus `Client::from_env()` for environments where `TANGO_API_KEY` (and optionally `TANGO_BASE_URL`) are set.
- Async transport built on `reqwest` + `tokio`: connection pooling, per-request `timeout`, configurable `retries` and `retry_backoff`.
- `X-API-KEY` authentication (matches `tango-node` / `tango-python` / `tango-go`).
- Single-enum `Error` with rich payload variants (`Auth`, `NotFound`, `Validation`, `RateLimit`, `Timeout`, `Api`, `Transport`, `Decode`, `Build`). Public helpers: `err.is_retryable()`, `err.status()`, `err.response()`.
- Automatic retry on 5xx / 408 / 429 / transport errors with exponential backoff (250ms base, doubling, capped at 10s). Server `Retry-After` overrides the backoff.
- `Client::rate_limit_info()` and `Client::last_response_headers()` for observability (parity with the Python `rate_limit_info` / `last_response_headers` properties).
- `Page<T>` envelope + `PageStream<T>` (async `futures::Stream`) for paginated walks. The stream follows `?cursor=` (keyset) when the server returns one, falling back to `?page=` for offset-paginated endpoints. `PageStream::collect_all()` drains the stream into a single `Vec`.
- All 21 shape presets exported as `SHAPE_*` constants (`SHAPE_CONTRACTS_MINIMAL`, `SHAPE_ENTITIES_COMPREHENSIVE`, ...). Same values as the sibling SDKs.
- Typed response models in `tango::models` mirror the typed surface in the Node SDK:
  - `AgencyRecord` (returned by `get_agency`)
  - `ProtestRecord` (returned by `get_protest`)
  - `ResolveInput`, `ResolveResult`, `ResolveCandidate`, `ResolveTargetType`
  - `ValidateInput`, `ValidateInputType`, `ValidateResult`
  - `WebhookEndpoint`, `WebhookEndpointCreateInput`, `WebhookEndpointUpdateInput`
  - `WebhookEventType`, `WebhookEventTypesResponse`
  - `WebhookSampleDelivery`, `WebhookSamplePayloadResponse`, `WebhookTestDeliveryResult`
  - `WebhookAlert`, `WebhookAlertCreateInput`, `WebhookAlertUpdateInput`

  Each carries `#[serde(flatten)] pub extra: HashMap<String, Value>` so a server-side schema addition surfaces in `extra["new_field"]` rather than being silently dropped.

- Resource methods (~75 total) covering parity with sibling SDKs:
  - **Agencies** — `list_agencies`, `get_agency`, `list_agency_awarding_contracts` / `_funding_contracts` (+ iterators).
  - **Contracts** — `list_contracts`, `iterate_contracts`.
  - **IDVs** — `list_idvs`, `get_idv`, `iterate_idvs`, plus sub-resources: `list_idv_awards`, `list_idv_child_idvs`, `list_idv_transactions`, `list_idv_lcats`, `get_idv_summary` *(deprecated)*, `list_idv_summary_awards` *(deprecated)* (+ iterators).
  - **Entities** — `list_entities`, `get_entity`, `iterate_entities`, plus sub-resources: `list_entity_contracts`, `list_entity_idvs`, `list_entity_otas`, `list_entity_otidvs`, `list_entity_subawards`, `list_entity_lcats`, `get_entity_metrics` (+ iterators).
  - **Vehicles** — `list_vehicles`, `get_vehicle`, `iterate_vehicles`, `list_vehicle_awardees`, `list_vehicle_orders` (+ iterators).
  - **Opportunities / Notices / Forecasts / Grants** — `list_opportunities` / `iterate_opportunities`, `list_notices` / `iterate_notices`, `list_forecasts` / `iterate_forecasts`, `list_grants` / `iterate_grants`, `search_opportunity_attachments`.
  - **OTAs / OTIDVs** — `list_otas`, `get_ota`, `iterate_otas`, `list_otidvs`, `get_otidv`, `iterate_otidvs`, `list_otidv_awards`.
  - **Subawards** — `list_subawards`.
  - **GSA eLibrary** — `list_gsa_elibrary_contracts`, `get_gsa_elibrary_contract`.
  - **Protests** — `list_protests`, `get_protest` (typed: `ProtestRecord`), `iterate_protests`.
  - **IT Dashboard** — `list_itdashboard`, `get_itdashboard`, `iterate_itdashboard`.
  - **LCATs** — `list_lcats` (dispatches to entity or IDV sub-resource).
  - **Lookups** — `list_naics` / `get_naics`, `list_psc` / `get_psc`, `list_mas_sins` / `get_mas_sin`, `list_assistance_listings` / `get_assistance_listing`, `list_business_types` / `get_business_type`, `list_offices` / `get_office`, `list_departments` / `get_department`, `list_organizations` / `get_organization`.
  - **Metrics** — `get_naics_metrics`, `get_psc_metrics`, `get_entity_metrics`, `list_metrics` (dispatcher).
  - **Resolve / Validate** — `resolve`, `validate`.
  - **Webhooks API** — `list_webhook_event_types`, `list_webhook_endpoints`, `get_webhook_endpoint`, `create_webhook_endpoint`, `update_webhook_endpoint`, `delete_webhook_endpoint`, `test_webhook_endpoint`, `get_webhook_sample_payload`, `list_webhook_alerts`, `get_webhook_alert`, `create_webhook_alert`, `update_webhook_alert`, `delete_webhook_alert`. Client-side validation rejects empty `name` / `callback_url` on `create_webhook_endpoint` and empty `filters` on `create_webhook_alert`, matching the `tango-node` v1.0.0 behavior.
  - **Meta** — `get_version`, `list_api_keys`.

#### Design decisions

Locked at the planning stage:

- Async-only (tokio + reqwest); no `blocking` feature in 0.1.
- `reqwest` with `default-features=false` and `rustls-tls` (avoids OpenSSL).
- Single-enum `thiserror` `Error` with rich payload variants.
- `bon`-derived client builder for compile-time required-field checking.
- Hybrid response typing: typed structs for fixed-schema endpoints (Agency, Protest, webhook resources, resolve, validate); `Record = serde_json::Map<String, Value>` for shape-driven endpoints.
- `futures::Stream` for async pagination, plus `PageStream::collect_all()` convenience.
- Edition 2021, MSRV 1.80 (edition 2024 bump deferred to 0.2+).
- Bare `&str` / `String` for identifiers (no newtype `Uei` / `Piid` yet — mirrors sibling SDKs).

### `makegov-tango-webhooks` v0.1.0

#### Added

- `generate(body, secret) -> String` — HMAC-SHA256, returns `sha256=<lowercase hex>`.
- `verify(body, header, secret) -> bool` — constant-time via `subtle::ConstantTimeEq`; never panics; returns false for absent / malformed / mismatched.
- `parse(header) -> Option<ParsedSignature>` — accepts canonical `"sha256=<hex>"` and bare hex (legacy compatibility, mirrors Node/Python).
- `SIGNATURE_HEADER` and `SIGNATURE_PREFIX` constants.
- Zero transport dependencies — pulls in only `hmac`, `sha2`, `subtle`, `hex`. A verifier service can use this crate without linking the full SDK.
- 19 unit tests + 4 doctests; known-vector cross-check (HMAC-SHA256 of `"hello"` with secret `"shh"`) pinned identical to the Go SDK.
