# API reference

Method-by-method reference for every public method on `tango::Client`. For the deep architectural walk-through see [`ARCHITECTURE.md`](ARCHITECTURE.md); for builder/transport/error specifics see [`CLIENT.md`](CLIENT.md).

The canonical, always-up-to-date reference is the auto-generated rustdoc at **<https://docs.rs/tango>**. This document is a curated overview keyed to the resource groups in the plan.

## Conventions

- All `Client` methods are `async`. Every call returns `Result<T, tango::Error>`.
- Methods named `list_*` return `Page<T>` for a single page; methods named `iterate_*` return a `PageStream<T>` for streaming every page.
- `get_*(id, opts)` methods validate `id` non-empty client-side; an empty id returns `Error::Validation` without making an HTTP call.
- Options builders are `#[derive(Builder)]` from `bon`. Construct with `.builder()...build()`.
- "Shape-driven" endpoints (most lists, all entity/contract/IDV/vehicle/opportunity/protest details) accept `shape`, `flat`, and `flat_lists` parameters. See [`SHAPES.md`](SHAPES.md).
- Method paths in the table below assume the resolved base URL (`https://tango.makegov.com` by default).

## Resources

### Agencies (`agencies.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_agencies(opts)` | `GET /api/agencies/` | `Page<Record>` |
| `get_agency(code, opts)` | `GET /api/agencies/{code}/` | **`AgencyRecord`** (typed) |
| `list_agency_awarding_contracts(code, opts)` / `iterate_*` | `GET /api/agencies/{code}/contracts/awarding/` | `Page<Record>` / `PageStream<Record>` |
| `list_agency_funding_contracts(code, opts)` / `iterate_*` | `GET /api/agencies/{code}/contracts/funding/` | `Page<Record>` / `PageStream<Record>` |

Options: `ListAgenciesOptions`, `GetAgencyOptions`, `AgencyContractsOptions` (alias: `ListAgencyAwardingContractsOptions`, `ListAgencyFundingContractsOptions`).

### Contracts (`contracts.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_contracts(opts)` / `iterate_contracts(opts)` | `GET /api/contracts/` | `Page<Record>` / `PageStream<Record>` |

Options: `ListContractsOptions`. SDK-friendly filter aliases (`naics_code`, `psc_code`, `recipient_name`, `recipient_uei`, `set_aside_type`, `keyword`) map onto canonical API names. When both are set, the SDK alias wins (mirrors Node/Python). `sort`+`order` combine into `ordering` with `-` prefix for descending.

### IDVs (`idvs.rs`, `idv_subresources.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_idvs(opts)` / `iterate_idvs(opts)` | `GET /api/idvs/` | `Page<Record>` / `PageStream<Record>` |
| `get_idv(key, opts)` | `GET /api/idvs/{key}/` | `Record` |
| `list_idv_awards(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/awards/` | `Page<Record>` / `PageStream<Record>` |
| `list_idv_child_idvs(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/child-idvs/` | `Page<Record>` / `PageStream<Record>` |
| `list_idv_transactions(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/transactions/` | `Page<Record>` / `PageStream<Record>` |
| `list_idv_lcats(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/lcats/` | `Page<Record>` / `PageStream<Record>` |
| `get_idv_summary(key)` *(deprecated)* | `GET /api/idvs/{key}/summary/` | `Record` |
| `list_idv_summary_awards(key, opts)` *(deprecated)* | `GET /api/idvs/{key}/summary/awards/` | `Page<Record>` |

Options: `ListIDVsOptions`, `GetIDVOptions`, `IdvSubresourceOptions` (shared across the sub-resource list endpoints).

### Entities (`entities.rs`, `entity_subresources.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_entities(opts)` / `iterate_entities(opts)` | `GET /api/entities/` | `Page<Record>` / `PageStream<Record>` |
| `get_entity(uei, opts)` | `GET /api/entities/{uei}/` | `Record` |
| `list_entity_contracts(uei, opts)` / `iterate_*` | `GET /api/entities/{uei}/contracts/` | `Page<Record>` / `PageStream<Record>` |
| `list_entity_idvs(uei, opts)` / `iterate_*` | `GET /api/entities/{uei}/idvs/` | `Page<Record>` / `PageStream<Record>` |
| `list_entity_otas(uei, opts)` / `iterate_*` | `GET /api/entities/{uei}/otas/` | `Page<Record>` / `PageStream<Record>` |
| `list_entity_otidvs(uei, opts)` / `iterate_*` | `GET /api/entities/{uei}/otidvs/` | `Page<Record>` / `PageStream<Record>` |
| `list_entity_subawards(uei, opts)` / `iterate_*` | `GET /api/entities/{uei}/subawards/` | `Page<Record>` / `PageStream<Record>` |
| `list_entity_lcats(uei, opts)` / `iterate_*` | `GET /api/entities/{uei}/lcats/` | `Page<Record>` / `PageStream<Record>` |
| `get_entity_metrics(uei, months, period_grouping)` | `GET /api/entities/{uei}/metrics/{months}/{period_grouping}/` | `Record` |

Options: `ListEntitiesOptions`, `GetEntityOptions`, `EntitySubresourceOptions` (shared across sub-resource list endpoints).

### Vehicles (`vehicles.rs`, `vehicle_subresources.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_vehicles(opts)` / `iterate_vehicles(opts)` | `GET /api/vehicles/` | `Page<Record>` / `PageStream<Record>` |
| `get_vehicle(uuid, opts)` | `GET /api/vehicles/{uuid}/` | `Record` |
| `list_vehicle_awardees(uuid, opts)` / `iterate_*` | `GET /api/vehicles/{uuid}/awardees/` | `Page<Record>` / `PageStream<Record>` |
| `list_vehicle_orders(uuid, opts)` / `iterate_*` | `GET /api/vehicles/{uuid}/orders/` | `Page<Record>` / `PageStream<Record>` |

Options: `ListVehiclesOptions`, `GetVehicleOptions`, `ListVehicleAwardeesOptions`, `ListVehicleOrdersOptions`.

### Opportunities / Notices / Forecasts / Grants (`opportunities.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_opportunities(opts)` / `iterate_*` | `GET /api/opportunities/` | `Page<Record>` / `PageStream<Record>` |
| `list_notices(opts)` / `iterate_*` | `GET /api/notices/` | `Page<Record>` / `PageStream<Record>` |
| `list_forecasts(opts)` / `iterate_*` | `GET /api/forecasts/` | `Page<Record>` / `PageStream<Record>` |
| `list_grants(opts)` / `iterate_*` | `GET /api/grants/` | `Page<Record>` / `PageStream<Record>` |
| `search_opportunity_attachments(opts)` | `GET /api/opportunities/attachment-search/` | `Page<Record>` |

Options: `ListOpportunitiesOptions`, `ListNoticesOptions`, `ListForecastsOptions`, `ListGrantsOptions`, `SearchOpportunityAttachmentsOptions`. The attachment-search method validates `q` non-empty client-side.

### OTAs / OTIDVs (`otas.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_otas(opts)` / `iterate_otas(opts)` | `GET /api/otas/` | `Page<Record>` / `PageStream<Record>` |
| `get_ota(key, opts)` | `GET /api/otas/{key}/` | `Record` |
| `list_otidvs(opts)` / `iterate_otidvs(opts)` | `GET /api/otidvs/` | `Page<Record>` / `PageStream<Record>` |
| `get_otidv(key, opts)` | `GET /api/otidvs/{key}/` | `Record` |
| `list_otidv_awards(key, opts)` / `iterate_*` | `GET /api/otidvs/{key}/awards/` | `Page<Record>` / `PageStream<Record>` |

Options: `ListOTAsOptions`, `GetOTAOptions`, `ListOTIDVsOptions`, `GetOTIDVOptions`, `ListOTIDVAwardsOptions`.

### Subawards (`subawards.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_subawards(opts)` / `iterate_subawards(opts)` | `GET /api/subawards/` | `Page<Record>` / `PageStream<Record>` |

Options: `ListSubawardsOptions`. **Note**: the server rejects `id` and `amount` in subaward shapes; use `SHAPE_SUBAWARDS_MINIMAL` or a custom shape that avoids them.

### GSA eLibrary (`gsa.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_gsa_elibrary_contracts(opts)` / `iterate_*` | `GET /api/gsa_elibrary_contracts/` | `Page<Record>` / `PageStream<Record>` |
| `get_gsa_elibrary_contract(uuid, opts)` | `GET /api/gsa_elibrary_contracts/{uuid}/` | `Record` |

Options: `ListGsaElibraryContractsOptions`, `GetGsaElibraryContractOptions`.

### Protests (`protests.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_protests(opts)` / `iterate_protests(opts)` | `GET /api/protests/` | `Page<Record>` / `PageStream<Record>` |
| `get_protest(case_id, opts)` | `GET /api/protests/{case_id}/` | **`ProtestRecord`** (typed) |

Options: `ListProtestsOptions`, `GetProtestOptions`. No `ordering` (server rejects it for this resource).

### IT Dashboard (`itdashboard.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_itdashboard(opts)` / `iterate_itdashboard(opts)` | `GET /api/itdashboard/` | `Page<Record>` / `PageStream<Record>` |
| `get_itdashboard(uii, opts)` | `GET /api/itdashboard/{uii}/` | `Record` |

Options: `ListItdashboardOptions`, `GetItdashboardOptions`. Some filters are tier-gated by the server (free vs. pro vs. business+); see the rustdoc on the options struct.

### LCATs (`lcats.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_lcats(opts)` / `iterate_lcats(opts)` | dispatched | `Page<Record>` / `PageStream<Record>` |

`ListLcatsOptions` requires exactly one of `uei` or `idv_key`. The method dispatches to `list_entity_lcats` / `list_idv_lcats` (or the iterate equivalents). Neither set → `Error::Validation`.

### Lookups (`lookups.rs`)

| Resource | List | Get |
| -------- | ---- | --- |
| Organizations | `list_organizations` / `iterate_organizations` | `get_organization` |
| NAICS | `list_naics` / `iterate_naics` | `get_naics` |
| PSC | `list_psc` / `iterate_psc` | `get_psc` |
| MAS SINs | `list_mas_sins` | `get_mas_sin` |
| Assistance listings | `list_assistance_listings` | `get_assistance_listing` |
| Business types | `list_business_types` | `get_business_type` |
| Offices | `list_offices` | `get_office` |
| Departments | `list_departments` | `get_department` |

Paths: `/api/organizations/`, `/api/naics/`, `/api/psc/`, `/api/mas_sins/`, `/api/assistance_listings/`, `/api/business_types/`, `/api/offices/`, `/api/departments/`. (The underscore/hyphen split is the server's; the SDK matches it.)

Options: `ListOrganizationsOptions`, `ListNaicsOptions`, `ListPscOptions`, `ListMasSinsOptions`, `ListAssistanceListingsOptions`, `ListBusinessTypesOptions`, `ListOfficesOptions`, `ListDepartmentsOptions`.

### Metrics (`metrics.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `get_naics_metrics(code, months, period_grouping)` | `GET /api/naics/{code}/metrics/{months}/{period_grouping}/` | `Record` |
| `get_psc_metrics(code, months, period_grouping)` | `GET /api/psc/{code}/metrics/{months}/{period_grouping}/` | `Record` |
| `list_metrics(opts)` | dispatched | `Record` |

Constants: `METRICS_OWNER_NAICS`, `METRICS_OWNER_PSC`, `METRICS_OWNER_ENTITY` for `ListMetricsOptions.owner_type`.

`list_metrics` dispatches to `get_naics_metrics`, `get_psc_metrics`, or `get_entity_metrics` based on `owner_type`. All three call windowed-metrics paths (`/api/{naics|psc|entities}/{id}/metrics/{months}/{period_grouping}/`).

### Resolve / Validate (`resolve_validate.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `resolve(input)` | `POST /api/resolve/` | `ResolveResult` |
| `validate(input)` | `POST /api/validate/` | `ValidateResult` |

Inputs and outputs are typed (`ResolveInput`, `ResolveResult`, `ResolveCandidate`, `ResolveTargetType`; `ValidateInput`, `ValidateInputType`, `ValidateResult`). `resolve` validates `name` non-empty; `validate` validates `value` non-empty.

### Webhooks API (`webhooks_api.rs`)

Webhook **endpoints**:

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_webhook_event_types()` | `GET /api/webhooks/event-types/` | `WebhookEventTypesResponse` |
| `list_webhook_endpoints(opts)` | `GET /api/webhooks/endpoints/` | `Page<WebhookEndpoint>` |
| `get_webhook_endpoint(id)` | `GET /api/webhooks/endpoints/{id}/` | `WebhookEndpoint` |
| `create_webhook_endpoint(input)` | `POST /api/webhooks/endpoints/` | `WebhookEndpoint` |
| `update_webhook_endpoint(id, input)` | `PATCH /api/webhooks/endpoints/{id}/` | `WebhookEndpoint` |
| `delete_webhook_endpoint(id)` | `DELETE /api/webhooks/endpoints/{id}/` | `()` |
| `test_webhook_endpoint(endpoint_id)` | `POST /api/webhooks/endpoints/test-delivery/` | `WebhookTestDeliveryResult` |
| `get_webhook_sample_payload(event_type)` | `GET /api/webhooks/endpoints/sample-payload/` | `WebhookSamplePayloadResponse` |

Webhook **alerts** (filter-based subscriptions):

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_webhook_alerts(opts)` | `GET /api/webhooks/alerts/` | `Page<WebhookAlert>` |
| `get_webhook_alert(id)` | `GET /api/webhooks/alerts/{id}/` | `WebhookAlert` |
| `create_webhook_alert(input)` | `POST /api/webhooks/alerts/` | `WebhookAlert` |
| `update_webhook_alert(id, input)` | `PATCH /api/webhooks/alerts/{id}/` | `WebhookAlert` |
| `delete_webhook_alert(id)` | `DELETE /api/webhooks/alerts/{id}/` | `()` |

Client-side validations (return `Error::Validation` before any HTTP call):
- `create_webhook_endpoint`: `name` non-empty, `callback_url` non-empty
- `create_webhook_alert`: `name` non-empty, `query_type` non-empty (singular!), `filters` non-empty
- All `get_*`, `update_*`, `delete_*`, `test_webhook_endpoint`: `id` non-empty

For HMAC-SHA256 signing/verification on incoming deliveries, use the separate [`tango-webhooks`](https://docs.rs/tango-webhooks) crate. See [`WEBHOOKS.md`](WEBHOOKS.md).

### Meta (`meta.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `get_version()` | `GET /api/version/` | `Record` |
| `list_api_keys(opts)` | `GET /api/api-keys/` | `Record` (single object, not paginated) |

Options: `ListApiKeysOptions`.

## Observability (on `Client`)

| Method | Returns | Notes |
| ------ | ------- | ----- |
| `client.base_url()` | `&str` | Resolved base URL. |
| `client.rate_limit_info()` | `Option<RateLimitInfo>` | Snapshot of `X-RateLimit-*` and `Retry-After` from the last response. |
| `client.last_response_headers()` | `Option<HeaderMap>` | Full response headers from the last completed request. |

## See also

- [`SHAPES.md`](SHAPES.md) — the full grammar and the 21 `SHAPE_*` preset constants.
- [`WEBHOOKS.md`](WEBHOOKS.md) — signing, verification, middleware, CRUD.
- [`CLIENT.md`](CLIENT.md) — builder options, env vars, retry semantics, error model.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — design walk-through.
