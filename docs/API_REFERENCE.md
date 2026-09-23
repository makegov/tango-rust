# API reference

Method-by-method reference for every public method on `tango::Client`. For the deep architectural walk-through see [`ARCHITECTURE.md`](ARCHITECTURE.md); for builder/transport/error specifics see [`CLIENT.md`](CLIENT.md).

The canonical, always-up-to-date reference is the auto-generated rustdoc at **<https://docs.rs/makegov-tango>**. This document is a curated overview keyed to the resource groups in the plan.

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
| `get_contract(key, opts)` | `GET /api/contracts/{key}/` | `Record` |
| `list_contract_subawards(key, opts)` | `GET /api/contracts/{key}/subawards/` | `Page<Record>` |
| `list_contract_transactions(key, opts)` | `GET /api/contracts/{key}/transactions/` | `Page<Record>` |

Options: `ListContractsOptions` (list), `ListOptions` (`get_contract`), `EntitySubresourceOptions` (sub-routes; `/transactions/` honours only pagination and `ordering`). SDK-friendly filter aliases (`naics_code`, `psc_code`, `recipient_name`, `recipient_uei`, `set_aside_type`, `keyword`) map onto canonical API names. When both are set, the SDK alias wins (mirrors Node/Python). `sort`+`order` combine into `ordering` with `-` prefix for descending.

### IDVs (`idvs.rs`, `idv_subresources.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_idvs(opts)` / `iterate_idvs(opts)` | `GET /api/idvs/` | `Page<Record>` / `PageStream<Record>` |
| `get_idv(key, opts)` | `GET /api/idvs/{key}/` | `Record` |
| `list_idv_awards(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/awards/` | `Page<Record>` / `PageStream<Record>` |
| `list_idv_child_idvs(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/idvs/` | `Page<Record>` / `PageStream<Record>` |
| `list_idv_transactions(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/transactions/` | `Page<Record>` / `PageStream<Record>` |
| `list_idv_lcats(key, opts)` / `iterate_*` | `GET /api/idvs/{key}/lcats/` | `Page<Record>` / `PageStream<Record>` |

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
| `get_entity_budget_flows(uei, opts)` | `GET /api/entities/{uei}/budget-flows/` | `Page<Record>` |
| `get_entity_metrics(uei, months, period_grouping)` | `GET /api/entities/{uei}/metrics/{months}/{period_grouping}/` | `Record` |

Options: `ListEntitiesOptions`, `GetEntityOptions`, `EntitySubresourceOptions` (shared across sub-resource list endpoints), `EntityBudgetFlowsOptions` (`get_entity_budget_flows`: pagination plus `fiscal_year`). `ListEntitiesOptions` exposes `cage` as an alias for `cage_code`; the server rejects a request that sets both. Budget flows are contract flows only: grant and assistance flows are not in that index.

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
| `get_opportunity(opportunity_id, opts)` | `GET /api/opportunities/{opportunity_id}/` | `Record` |
| `list_notices(opts)` / `iterate_*` | `GET /api/notices/` | `Page<Record>` / `PageStream<Record>` |
| `get_notice(notice_id, opts)` | `GET /api/notices/{notice_id}/` | `Record` |
| `list_forecasts(opts)` / `iterate_*` | `GET /api/forecasts/` | `Page<Record>` / `PageStream<Record>` |
| `get_forecast(id, opts)` | `GET /api/forecasts/{id}/` | `Record` |
| `list_grants(opts)` / `iterate_*` | `GET /api/grants/` | `Page<Record>` / `PageStream<Record>` |
| `get_grant(grant_id, opts)` | `GET /api/grants/{grant_id}/` | `Record` |
| `search_opportunity_attachments(opts)` *(deprecated)* | `GET /api/opportunities/attachment-search/` | `Record` |

Options: `ListOpportunitiesOptions`, `ListNoticesOptions`, `ListForecastsOptions`, `ListGrantsOptions`, `SearchOpportunityAttachmentsOptions`. The singleton `get_*` methods take `Option<ListOptions>`. `ListGrantsOptions` exposes a typed `grant_id` filter; `ListNoticesOptions` adds `notice_id`, `department` and `office`; `ListOpportunitiesOptions` adds `opportunity_id`; `ListForecastsOptions` adds `id`.

**`search_opportunity_attachments` is deprecated.** The API retired `/api/opportunities/attachment-search/`: it returns 404 for every query and keeps the route only so a missing `q` still gets its 400. `list_opportunities` with `search` matches attachment text and returns a `snippet` for the hit.

### OTAs / OTIDVs (`otas.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_otas(opts)` / `iterate_otas(opts)` | `GET /api/otas/` | `Page<Record>` / `PageStream<Record>` |
| `get_ota(key, opts)` | `GET /api/otas/{key}/` | `Record` |
| `list_otidvs(opts)` / `iterate_otidvs(opts)` | `GET /api/otidvs/` | `Page<Record>` / `PageStream<Record>` |
| `get_otidv(key, opts)` | `GET /api/otidvs/{key}/` | `Record` |
| `list_otidv_awards(key, opts)` / `iterate_*` | `GET /api/otidvs/{key}/awards/` | `Page<Record>` / `PageStream<Record>` |

Options: `ListOTAsOptions`, `GetOTAOptions`, `ListOTIDVsOptions`, `GetOTIDVOptions`, `ListOTIDVAwardsOptions`. Like contracts and IDVs, each list options struct carries a `key` filter that accepts `|`-separated award keys.

### Subawards (`subawards.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_subawards(opts)` / `iterate_subawards(opts)` | `GET /api/subawards/` | `Page<Record>` / `PageStream<Record>` |
| `get_subaward(key, opts)` | `GET /api/subawards/{key}/` | `Record` |

Options: `ListSubawardsOptions` (list), `ListOptions` (`get_subaward`). **Note**: the server rejects `id` and `amount` in subaward shapes; use `SHAPE_SUBAWARDS_MINIMAL` or a custom shape that avoids them.

### Budget (`budget.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_budget_accounts(opts)` / `iterate_budget_accounts(opts)` | `GET /api/budget/accounts/` | `Page<Record>` / `PageStream<Record>` |
| `get_budget_account(id, opts)` | `GET /api/budget/accounts/{id}/` | `Record` |
| `get_budget_account_quarters(id, opts)` | `GET /api/budget/accounts/{id}/quarters/` | `Page<Record>` |
| `get_budget_account_recipients(id, opts)` | `GET /api/budget/accounts/{id}/recipients/` | `Page<Record>` |

One row per federal account and fiscal year, covering the budget lifecycle (requested, enacted, apportioned, obligated, outlayed) with pre-computed ratios, trends and the contract / assistance breakdown.

Options: `ListBudgetAccountsOptions` (list), `ListOptions` (`get_budget_account`), `BudgetAccountQuartersOptions` (pagination plus `tas`), `BudgetAccountRecipientsOptions` (pagination plus `funding_organization_id`). `SHAPE_BUDGET_ACCOUNTS_MINIMAL` mirrors the API's default shape.

- **The list endpoint rejects an unknown filter name** with a 400 and a did-you-mean, rather than silently returning the unfiltered page. Typed fields cover the identity filters (`federal_account_symbol`, `fiscal_year` and its range, `agency_code`, `bureau_name`, `account_title`, `bea_category`, `on_off_budget`, `subfunction_code`); the numeric `__gte` / `__lte` range filters and the `__in` variants go through `extra`.
- **`{id}` is the row's numeric `id`**, not the federal account symbol.
- **`quarters` covers FY2021 onward**; an earlier account-year returns an empty page.
- **`recipients` is contract flows only.** Each row carries the resolved `funding_office` and `recipient` plus a capped `contracts` list; a row that hits the cap sets `contracts_truncated`.
- The `quarters` and `recipients` envelopes also carry `federal_account_symbol` and `fiscal_year`, which `Page` does not surface.

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

Records come from GAO, the U.S. Court of Federal Claims and the SBA Office of Hearings and Appeals. `source_system` is returned lowercase (`gao`, `cofc`, `sba_oha`) and `outcome` title-case (`Denied`, `Dismissed`, `Withdrawn`, `Sustained`; SBA OHA adds `Granted`, `Remanded`, `Reversed`, `Vacated`). Both filters are case-insensitive, but compare returned values in the API's casing. `naics_code` matches the NAICS code at issue in an SBA OHA size or NAICS appeal; GAO and COFC records carry none.

Options: `ListProtestsOptions`, `GetProtestOptions`. No `ordering` (server rejects it for this resource).

### Contract appeals (`contract_appeals.rs`)

Contract Disputes Act appeal decisions from the Civilian Board of Contract Appeals (CBCA) and the Armed Services Board of Contract Appeals (ASBCA).

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_contract_appeals(opts)` / `iterate_contract_appeals(opts)` | `GET /api/contract_appeals/` | `Page<Record>` / `PageStream<Record>` |
| `get_contract_appeal(uuid, opts)` | `GET /api/contract_appeals/{uuid}/` | **`ContractAppealRecord`** (typed) |

Options: `ListContractAppealsOptions`, `GetContractAppealOptions`. Shape: `SHAPE_CONTRACT_APPEALS_MINIMAL`.

Filters: `search`, `board` (`cbca` / `asbca`), `docket`, `appellant`, `judge`, `decision_type`, `decision_date_after` / `_before`, `listed`, `document_id`. Ordering: `decision_date` (the server's default is `-decision_date`), `appellant`, `first_listed_at`, `rank` — and `rank` is only meaningful alongside a non-empty `search`.

**These are not bid protests.** An appeal is a dispute under a contract already awarded — a claim, a termination, a delay, a defective specification — decided by a board rather than by GAO or the Court of Federal Claims. A company can appear in both corpora and nothing joins the two, so an appeals search is not a substitute for a [protests](#protests-protestsrs) search or the reverse.

**One decision can resolve several dockets.** `docket_numbers` is a list for that reason, and the `docket` filter matches any member of it. `docket_raw`, `decision_date_raw` and `decision_type_raw` are the board's own strings, kept beside the parsed values; `decision_date_repaired` flags a date reconstructed rather than parsed straight through.

**`listed` is shelf presence, not validity.** It says the decision is still on a current board listing page. A board rotating its listings does not vacate the decisions that fall off them.

**The decision body is `decision_text`, on an Enterprise plan.** Below Enterprise the key is **absent rather than null**, so `None` means "not served to this caller" and never "this decision has no text" — `text_status` and `text_char_count` describe the text at every plan. `SHAPE_CONTRACT_APPEALS_MINIMAL` deliberately does not name it, so a list page never pays for text most callers are not served.

### DIBBS (`dibbs.rs`)

Defense Logistics Agency solicitations and awards from DIBBS.

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_dibbs_rfqs(opts)` / `iterate_dibbs_rfqs(opts)` | `GET /api/dibbs/rfqs/` | `Page<Record>` / `PageStream<Record>` |
| `get_dibbs_rfq(uuid, opts)` | `GET /api/dibbs/rfqs/{uuid}/` | `Record` |
| `list_dibbs_rfps(opts)` / `iterate_dibbs_rfps(opts)` | `GET /api/dibbs/rfps/` | `Page<Record>` / `PageStream<Record>` |
| `get_dibbs_rfp(uuid, opts)` | `GET /api/dibbs/rfps/{uuid}/` | `Record` |
| `list_dibbs_awards(opts)` / `iterate_dibbs_awards(opts)` | `GET /api/dibbs/awards/` | `Page<Record>` / `PageStream<Record>` |
| `get_dibbs_award(uuid, opts)` | `GET /api/dibbs/awards/{uuid}/` | `Record` |

Options: `ListDibbsRfqsOptions`, `ListDibbsRfpsOptions`, `ListDibbsAwardsOptions`, `GetDibbsOptions`. Shapes: `SHAPE_DIBBS_RFQS_MINIMAL`, `SHAPE_DIBBS_RFPS_MINIMAL`, `SHAPE_DIBBS_AWARDS_MINIMAL`.

- **Open/closed is derived at query time** from `return_by_date` (RFQs) or `closes_date` (RFPs); there is no stored flag. Filter with `open`, which is `Option<bool>` so `Some(false)` reaches the server.
- **An award row is one line item, and `total_contract_price` is the order total repeated on every line.** Never sum it across rows; deduplicate on `award_number` + `delivery_order_number` first.
- `entity` matches only awards whose CAGE code resolved to a registered entity; `awardee_cage` reaches every award.

### Exclusions (`exclusions.rs`)

SAM.gov exclusions: debarments, suspensions and other ineligibility records.

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_exclusions(opts)` / `iterate_exclusions(opts)` | `GET /api/exclusions/` | `Page<Record>` / `PageStream<Record>` |
| `get_exclusion(exclusion_key, opts)` | `GET /api/exclusions/{exclusion_key}/` | `Record` |

Options: `ListExclusionsOptions`, `GetExclusionOptions`. Shape: `SHAPE_EXCLUSIONS_MINIMAL`.

- **Whether an exclusion is in force is derived at query time**: not delisted, already activated, not yet terminated. Filter with `active` rather than trusting a stored flag.
- **`delisted` is not expiry.** It means SAM lifted or withdrew the record; a natural expiration leaves the record listed.
- Most exclusions are individuals and carry no UEI. `entity_uei` is set only when the record's UEI matches a registered entity.

### SBIR / STTR (`sbir.rs`)

SBIR/STTR topics and DoD DSIP solicitation cycles.

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_sbir_topics(opts)` / `iterate_sbir_topics(opts)` | `GET /api/sbir/topics/` | `Page<Record>` / `PageStream<Record>` |
| `get_sbir_topic(topic_id, opts)` | `GET /api/sbir/topics/{topic_id}/` | `Record` |
| `list_sbir_solicitations(opts)` / `iterate_sbir_solicitations(opts)` | `GET /api/sbir/solicitations/` | `Page<Record>` / `PageStream<Record>` |
| `get_sbir_solicitation(solicitation_id, opts)` | `GET /api/sbir/solicitations/{solicitation_id}/` | `Record` |

Options: `ListSbirTopicsOptions`, `ListSbirSolicitationsOptions`, `GetSbirOptions`. Shapes: `SHAPE_SBIR_TOPICS_MINIMAL`, `SHAPE_SBIR_SOLICITATIONS_MINIMAL`.

- **`activity` is open/closed, derived at query time** from the close or end date. A topic with no close date and no parent solicitation to inherit one from is `unknown`.
- The topic `agency` filter is a partial, case-insensitive match on the raw agency text, not organization-resolved, so variant spellings do not normalize.

### State & Local — SLED (`sled.rs`) — **Beta**

State, local and education procurement: solicitations that never appear on SAM.gov because they were never federal. Coverage is partial and grows one jurisdiction at a time.

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_sled_opportunities(opts)` / `iterate_sled_opportunities(opts)` | `GET /api/sled/opportunities/` | `Page<Record>` / `PageStream<Record>` |
| `get_sled_opportunity(id, opts)` | `GET /api/sled/opportunities/{opportunity_id}/` | `Record` |
| `list_sled_opportunity_revisions(id, opts)` | `GET /api/sled/opportunities/{opportunity_id}/revisions/` | `Page<Record>` |
| `get_sled_coverage()` | `GET /api/sled/opportunities/coverage/` | `Record` |
| `list_sled_forecasts(opts)` / `iterate_sled_forecasts(opts)` | `GET /api/sled/forecasts/` | `Page<Record>` / `PageStream<Record>` |
| `get_sled_forecast(id, opts)` | `GET /api/sled/forecasts/{forecast_id}/` | `Record` |

Options: `ListSledOpportunitiesOptions`, `ListSledOpportunityRevisionsOptions`, `ListSledForecastsOptions`, `GetSledOptions`. Shapes: `SHAPE_SLED_OPPORTUNITIES_MINIMAL` / `_COMPREHENSIVE`, `SHAPE_SLED_REVISIONS_MINIMAL`, `SHAPE_SLED_FORECASTS_MINIMAL` / `_COMPREHENSIVE`. `ListSledOpportunitiesOptions::verbose` adds `description` and `contact` to each list row; `description` is otherwise detail-only because its longest values run past 120,000 characters.

**This data does not join to the federal data.** No UEI, no PIID, no agency-hierarchy key and no NAICS/PSC crosswalk; the `organization(*)` expand here is three strings, not the federal 7-key office payload.

**Leaving both `status` and `active` unset returns open solicitations only.** Only about a fifth of the corpus is open, and a portal drops a closed solicitation rather than restating it, so the API defaults the list to `status=open`. Set `status` explicitly to page the whole corpus; `status = "open|unknown"` also reaches the standing rosters and dateless RFIs that `unknown` covers. `get_sled_opportunity` returns a solicitation whatever its status.

`list_sled_opportunities` deliberately does **not** synthesize `status=open` client-side: doing so would make `active = Some(false)` unreachable, since `active=false` is the complement of open rather than an independent value. `active`, `has_documents` and `source_declared` are `Option<bool>` for the same reason — `false` has to be distinguishable from unset.

**`status` is Tango's answer, not the portal's.** Derived from the portal's word, the delisting, the deadline and the clock, and refreshed every fifteen minutes. A solicitation the portal stopped listing before its deadline carries `delisted_at`, reads `status = "closed"` and `status_reason = "delisted"` (a portal's own closed, awarded or cancelled still takes precedence); both suggested opportunity shapes include `delisted_at`. The portal's own word is served as `source_status`, is frozen at last capture, and is **not** filterable — most of what it calls open already has a passed deadline.

**Category scheme tagging is mid-migration**, so `naics` matches only the small tagged share. Use `category_code` to match a code under any scheme, including the untagged pre-migration strings.

**`meta.attachment_count` can be lower than the length of `attachments`.** Some portals auto-generate a cover sheet alongside the real documents; it is listed and flagged `is_generated_summary` but excluded from the count and from `has_documents`. The count answers "does this record hold its solicitation package"; the array answers "what files exist".

**The document body is `attachments(extracted_text)`, on a Small plan or above** (API 4.25.1+). It must be **named** — no `SHAPE_SLED_*` constant includes it and `attachments(*)` does not carry it, because the API resolves the body only for a caller who asked. Its key is **absent rather than null** whenever the text is not being served: below Small (withheld and named in `meta.upgrade_hints`), on a contested document, or where it could not be resolved. A **contested document never returns text at any plan**, because its stored bytes disagree with what the record advertised. Searching document text and reading it are separate — `search` matches inside attachment text on every plan and returns no fragment of it.

`get_sled_coverage` takes no parameters and is neither shaped nor paginated. **Call it before treating a per-state count as market size** — a thin result for a state is at least as likely to be a portal Tango does not read as a quiet market, and every state row carries all five status buckets whether or not they have rows.

Forecasts carry **no liveness at all**: no deadline to have passed, so no `status`, no `active`, and no open-only default. `estimated_advertisement_date` is the start of the published quarter rather than a posting date, and `estimated_value(min,max,raw)` is parsed from a free-text award band — a band naming one number is a floor, so `max` is null.

### IT Dashboard (`itdashboard.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_itdashboard(opts)` / `iterate_itdashboard(opts)` | `GET /api/itdashboard/` | `Page<Record>` / `PageStream<Record>` |
| `get_itdashboard(uii, opts)` | `GET /api/itdashboard/{uii}/` | `Record` |

Options: `ListItdashboardOptions`, `GetItdashboardOptions`. `previous_uii` finds the investment(s) that superseded a retired UII. Some filters are tier-gated by the server (free vs. pro vs. business+); see the rustdoc on the options struct.

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

A department's `code` is an integer (`97` for DoD), returned as a JSON number; `get_department` takes it as a string (`"97"`). An agency's own `code` is a string (`"9700"`), and its nested `department.code` is the department's integer.

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

For HMAC-SHA256 signing/verification on incoming deliveries, use the separate [`makegov-tango-webhooks`](https://docs.rs/makegov-tango-webhooks) crate. See [`WEBHOOKS.md`](WEBHOOKS.md).

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

- [`SHAPES.md`](SHAPES.md) — the full grammar and the 34 `SHAPE_*` preset constants.
- [`WEBHOOKS.md`](WEBHOOKS.md) — signing, verification, middleware, CRUD.
- [`CLIENT.md`](CLIENT.md) — builder options, env vars, retry semantics, error model.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — design walk-through.
