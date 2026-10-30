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
- Every `Page<T>` carries `count_type` (`"exact"` or `"approximate"`, from that response's `X-Results-CountType` header; `None` when the endpoint does not send it) and `meta` (the response's `meta` object as sent). Past 1,000 matches the contract, IDV, opportunity and notice lists may report a query-planner estimate as `count`. On an agency-filtered list, each resolved entry of `meta["resolved_filters"][<filter name>]` says how its token matched in `matched_by` (`key`, `code`, `name`, `alias` or `fuzzy`; Tango API 5.9.0+), and an unresolved entry has none.

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

- **`uei` / `recipient_uei` accept several UEIs joined by `|`** and match awards to any of them, up to 500 (Tango API 5.11.0). The same holds for `uei` on the IDV, OTA and OT IDV lists.
- **`commercial_item_acquisition_procedures`** (Tango API 5.10.0) filters by the FPDS code, case-insensitive: `A` (commercial products/services), `B` (products or services pursuant to FAR 12.102(f)), `C` (services pursuant to FAR 12.102(g)), `D` (commercial procedures not used) or `E` (DoD Section 803 Commercial Solutions Opening). OR several with `|`, e.g. `A|E`; an unknown or multi-character code matches nothing. It is on `ListContractsOptions` and `ListIDVsOptions` only: the API does not offer it on OTAs or OT IDVs.
- **`obligations_by_fiscal_year(fiscal_year,obligated)`** (Tango API 5.11.0) is a shape expand on contracts, IDVs, OTAs and OT IDVs, list and detail. It sums the award's transactions per federal fiscal year (October 1 starts the next one), oldest first, each rounded to cents; a de-obligation nets against its own year, so a year can be zero or negative. It is returned only when named, and it is always null on IDVs and OT IDVs, whose transactions report a running lifetime total. Decode it with `models::FiscalYearObligation::from_record(&row)`, which returns `Ok(None)` when the field is null or not in the shape.

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

Options: `ListVehiclesOptions`, `GetVehicleOptions`, `ListVehicleAwardeesOptions` (adds `search`), `ListVehicleOrdersOptions` (adds `ordering` over `award_date`, the default, `obligated` and `total_contract_value`).

- The default vehicle shapes request `holder_count` (distinct companies holding one of the vehicle's IDVs) and `order_winner_count` (distinct companies that have won a task order under it). `awardee_count` is a deprecated alias of `order_winner_count`.
- The embedded solicitation accepts the opportunity attachment counts: `opportunity(meta(files_count,links_count))`.

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

Options: `ListOpportunitiesOptions`, `ListNoticesOptions`, `ListForecastsOptions`, `ListGrantsOptions`. The singleton `get_*` methods take `Option<ListOptions>`. `ListGrantsOptions` exposes a typed `grant_id` filter; `ListNoticesOptions` adds `notice_id`, `department` and `office`; `ListOpportunitiesOptions` adds `opportunity_id`; `ListForecastsOptions` adds `id`.

`ListOpportunitiesOptions` also takes `awarded` (`Option<bool>`) and `awardee_uei` (case-insensitive, `|` for several), Tango API 5.5.0. Both search every opportunity rather than only active ones, so set `active` too to narrow to open opportunities.

**Award fields** (Tango API 5.5.0) are not in the default shape, so name them: `awarded`, `award_date`, `award_amount`, `awardee`, `awardee_uei`, `award_count`, `solicitation_opportunity_id` and `awards(opportunity_id,notice_id,award_number,award_date,award_amount,awardee,awardee_uei)`. `award_date`, `award_amount`, `awardee` and `awardee_uei` are filled only where SAM.gov posted an award notice, and `award_amount` is the text SAM.gov published, not a number. When an award posted as its own opportunity references its solicitation, its `solicitation_opportunity_id` points back, and the solicitation's `awards(...)` lists up to ten of the most recent linked awards (`award_count` is the full count). Notices accept `award_date`, `award_amount`, `awardee` and `awardee_uei`.

**Attachment counts** (Tango API 5.9.0): an attachment's `type` is `file` (a document attached to the notice; only files have extracted text) or `link` (a URL the notice lists). Opportunities carry `meta(attachments_count,files_count,links_count)` and notices `attachment_count`, `file_count` and `link_count`. The total still counts every attachment, so files plus links need not equal it, and the file and link counts are null until the record has been counted: **treat null as unknown, not zero**. No default shape names them.

**`attachments(extraction_status)`** (Tango API 5.11.0) reports `pending`, `extracting`, `completed`, `failed` or `unsupported` for each attachment on an opportunity or notice, which says why `extracted_text` is missing. It is available on every plan and returned only when named: `attachments(*)` does not carry it.

**There is no attachment-search method.** The API retired `/api/opportunities/attachment-search/` (it returns 404 for every query). `list_opportunities` with `search` matches attachment text and returns a `snippet` for the hit.

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

- **The list endpoint rejects an unknown filter name** with a 400 and a did-you-mean, rather than silently returning the unfiltered page. Typed fields cover the identity filters (`federal_account_symbol`, `fiscal_year` and its range, `data_through_period` with its range and `data_through_period_isnull`, `agency_code`, `bureau_name`, `account_title`, `bea_category`, `on_off_budget`, `subfunction_code`, `account_category` and `account_category_in`); the numeric `__gte` / `__lte` range filters and the other `__in` variants go through `extra`.
- **`data_through_period`** is in the default shape, right after `fiscal_year`: the File A fiscal period (1–12) the account-year's figures run through, so the latest fiscal year is partial until it reaches 12. It is null when the row has no File A data. Read it with `models::budget_data_through_period(&row)`, which returns `Option<u8>`.
- **`account_category`** is an open string, `budgetary` or `credit_financing` today. Credit financing accounts have null `enacted_ba` and are left out of organization budget totals. The default ordering is latest fiscal year first, then largest `enacted_ba`, with null `enacted_ba` last and `id` breaking ties.
- **`source_anomalies`** is in the default shape: a list of problems found in the source data behind the row, `[]` when the row is clean. Decode it with `models::BudgetSourceAnomaly::from_record(&row)`. Each element names a `code` (an open string; `contract_exceeds_obligations`, `assistance_exceeds_obligations` and `contract_without_obligations` today), the `action` taken (`capped` changes the served value, `flagged` does not), the `reported_value` and `served_value`, and the `source` rows behind it. There is no filter on it.
- **`{id}` is the row's numeric `id`**, not the federal account symbol.
- **`quarters` covers FY2021 onward**; an earlier account-year returns an empty page.
- **`recipients` is contract flows only.** Each row carries the resolved `funding_office` and `recipient` plus a capped `contracts` list; a row that hits the cap sets `contracts_truncated`.
- **`agency`** (Tango API 5.3.0) takes a Tango agency name, abbreviation, code or organization key (e.g. `EPA`) and matches the account's resolved organization, including every sub-agency beneath it; `|` for several. A token that does not resolve falls back to an exact `agency_code` match. The `organization(...)` expand (`agency_code`, `agency_name`, `department_code`, `department_name`, `office_code`, `office_name`, `organization_id`) is accepted in shapes, as is `organization_id`; the default shape is unchanged.
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

### GSA eBuy (`ebuy.rs`)

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_ebuy_requests(opts)` / `iterate_ebuy_requests(opts)` | `GET /api/ebuy/requests/` | `Page<Record>` / `PageStream<Record>` |
| `get_ebuy_request(rfq_id, opts)` | `GET /api/ebuy/requests/{rfq_id}/` | `Record` |
| `get_ebuy_attachment_url(rfq_id, doc_seq_num)` | `GET /api/ebuy/requests/{rfq_id}/attachments/{doc_seq_num}/download/` | `String` (the redirect target) |
| `get_ebuy_access()` | `GET /api/ebuy/access/` | **`EbuyAccess`** (typed) |

Options: `ListEbuyRequestsOptions` (`search`, `rfq_id`, `reference_number`, `request_type`, `status`, `sin`, `schedule`, `buyer_agency`, `agency`, `contract_number`, `issue_date_after` / `_before`, `close_date_after` / `_before`, `ordering` over `issue_date`, `close_date`, `last_seen` and `modified`), `GetEbuyRequestOptions`. Shape: `SHAPE_EBUY_REQUESTS_MINIMAL`, which mirrors the API's default list shape. Available from Tango API 5.1.0.

**Requires the Pro plan, and is scoped to your own GSA schedule contracts.** You see a request only while your account is linked to a contract it was posted under. With no linked contract the list is **empty rather than an error**, so call `get_ebuy_access` to tell "no access" from "no matches": it returns `enabled`, a `reason` (`tier_required` or `no_contract_grant`; `tier_required` wins when both apply) and your linked `contracts`. A request outside your scope 404s on lookup, the same as an unknown id. `contract_number` narrows to one of your own contracts; a contract you are not linked to returns nothing.

**`status` is frozen at the state it was last seen in.** Only currently-active requests are published, so a request that closes stops appearing rather than getting a final row: `Open` means "open the last time it was seen". Read `last_seen` for staleness. The contract number a request was posted under is never returned in any payload, and `buyer_agency_code` and some other buyer and contact fields are sparse on older requests.

**`get_ebuy_attachment_url` does not follow the redirect.** The API answers with a redirect to a presigned URL valid for about five minutes; the method returns that URL, so fetch it promptly. It runs on a client with redirects disabled (override it with the builder's `no_redirect_http_client` if your proxy or TLS setup must apply). An attachment with `is_link = true` has no stored document: the call returns `Error::ExternalLink { url, .. }` carrying the link. A document not yet captured returns `Error::NotFound`.

**The document text is `attachments(extracted_text)`** (API 5.9.1+). It must be **named** — neither the default shape nor `attachments(*)` carries it, and `SHAPE_EBUY_REQUESTS_MINIMAL` does not include it. Ask for it with a shape such as `rfq_id,attachments(doc_seq_num,doc_name,extracted_text)`. Its key is **absent rather than null** on an attachment with no text: a link, an empty scan, or a document not yet extracted.

### Federal Register (`federal_register.rs`)

Rules, proposed rules, notices and presidential documents published in the Federal Register since 1994 (Tango API 5.3.0).

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_federal_register_documents(opts)` / `iterate_federal_register_documents(opts)` | `GET /api/federal_register/` | `Page<Record>` / `PageStream<Record>` |
| `get_federal_register_document(uuid, opts)` | `GET /api/federal_register/{uuid}/` | `Record` |

Options: `ListFederalRegisterDocumentsOptions` (`search`, `document_number`, `r#type`, `agency`, `fr_agency`, `publication_date_after` / `_before`, `effective_on_after` / `_before`, `comments_close_on_after` / `_before`, `comments_open`, `cfr_title`, `cfr_part`, `significant`, `rin`, `executive_order_number`, and `ordering` over `publication_date`, `effective_on`, `comments_close_on`, `document_number` and `rank`), `GetFederalRegisterDocumentOptions`. Shapes: `SHAPE_FEDERAL_REGISTER_MINIMAL` and `SHAPE_FEDERAL_REGISTER_COMPREHENSIVE`, which mirror the API's list and detail defaults.

- **The detail route takes the `uuid`, not the `document_number`.** The Federal Register reused some document numbers before 2016, so a number is not unique on its own; look one up with `document_number` on the list, which matches every document carrying it.
- **`full_text` is detail-only and must be named.** It can run to several MB, so neither default shape includes it.
- `agency` is a Tango organization and matches a document when any agency it lists falls within it (a department includes its sub-agencies); `fr_agency` is the Federal Register's own agency slug, e.g. `environmental-protection-agency`.
- `comments_open = Some(true)` keeps documents whose comment period closes today or later and `Some(false)` those already closed; a document with no comment deadline matches neither. `cfr_part` requires `cfr_title`, and both must match the same CFR reference. `ordering = "rank"` requires a non-empty `search`.
- Documents published before 2008 are mostly typed `Uncategorized Document`.

### Award announcements (`award_announcements.rs`)

Federal award announcements from two sources, named on every row by `source` (Tango API 5.14.0), free at every tier.

| Method | Endpoint | Returns |
| ------ | -------- | ------- |
| `list_award_announcements(opts)` / `iterate_award_announcements(opts)` | `GET /api/news/award_announcements/` | `Page<Record>` / `PageStream<Record>` |
| `get_award_announcement(uuid, opts)` | `GET /api/news/award_announcements/{uuid}/` | `Record` |

Options: `ListAwardAnnouncementsOptions` (`search`, `source`, `agency`, `component`, `published_date_after` / `_before`, `award_date_after` / `_before`, `amount_min`, `amount_max`, `piid`, `awardee`, `uei`, `action_type`, `announcement_type`, `civilian`, `small_business`, `is_modification`, `is_multiple_award`, `include_withdrawn`, and `ordering` over `published_date`, `amount` and `rank`), `GetAwardAnnouncementOptions`. Shapes: `SHAPE_AWARD_ANNOUNCEMENTS_MINIMAL` and `SHAPE_AWARD_ANNOUNCEMENTS_COMPREHENSIVE`, which mirror the API's list and detail defaults.

- **`source` says who wrote the row.** `official_release` rows are the awards of $7.5 million or more that the Department of War publishes in its daily contracts release, with the text as published. `makegov_generated` rows cover new civilian IDVs of $25 million or more and every new OTA and OT IDV: they are written by MakeGov from FPDS data with a language model and are **not government text**. A generated announcement is dropped when an official release covers the same award.
- **Withdrawn announcements are hidden from lists** unless `include_withdrawn = Some(true)`, and stay retrievable by `uuid`.
- **`text` must be named on a list.** It can run past 16,000 characters, so only the detail default carries it.
- The expands are `awardees(name,uei,location,small_business)` and `organization(organization_id,office_code,office_name,agency_code,agency_name,department_code,department_name)`, and `*` selects every key of either.
- `piid` takes a contract number as printed (`N00019-25-C-0070`) or bare (`N0001925C0070`); dashes and case are ignored. `awardee` is a case-insensitive substring match of at least three characters, and shorter values return a 400. `uei` matches only where one was resolved, since official releases print none.
- `small_business = Some(true)` keeps announcements with at least one small-business awardee and `Some(false)` those whose awardees are stated not to be small; generated announcements state neither, so they match neither. `component` is the release's section header as printed, compared case-insensitively, and generated announcements have none.
- `ordering = "rank"` requires a non-empty `search`, and a `search` with no `ordering` sorts by `-rank`. Otherwise the default is `-published_date`. Descending `published_date` and `amount` put rows with no value last.
- This endpoint pages by `page`, so `ListAwardAnnouncementsOptions` has no `cursor`.

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
- **`agency`** (Tango API 5.3.0) scopes by the excluding agency: a Tango agency name, abbreviation, code or organization key, including every component of it; `|` for several. A token that does not resolve falls back to an exact `excluding_agency_code` match. The `organization(...)` expand (`agency_code`, `agency_name`, `department_code`, `department_name`, `office_code`, `office_name`, `organization_id`) is accepted in shapes, as is `organization_id`.

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
- The topic `agency` filter is organization-resolved and rolls up the hierarchy (`HHS` includes NIH topics); a token that does not resolve falls back to a partial match on the raw agency text.
- `ListSbirTopicsOptions::cycle_name` filters topics by their exact DoD DSIP cycle (e.g. `DOD_SBIR_2026_P1_CBZ`, Tango API 5.3.1); only topics fed from DSIP carry one. `cycle_name` is also a topic field.
- The `organization(...)` expand (`agency_code`, `agency_name`, `department_code`, `department_name`, `office_code`, `office_name`, `organization_id`) is accepted in shapes on topics and solicitations.

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

Options: `ListItdashboardOptions`, `GetItdashboardOptions`. `previous_uii` finds the investment(s) that superseded a retired UII. `agency` (Tango API 5.3.0) matches the investment's resolved organization, including every sub-agency and office beneath it, falling back to a partial `agency_name` match.

Some filters are tier-gated by the server: `search`, `agency_name`, `agency` and `previous_uii` are available at every plan; `agency_code`, `type_of_investment` and the `updated_time` bounds need Pro; `cio_rating`, `cio_rating_max` and `performance_risk` need Business or above. A gated filter below its plan returns a 403.

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

Options: `ListOrganizationsOptions`, `ListNaicsOptions`, `ListPscOptions` (`has_awards = Some(true)` keeps only codes with contract award history), `ListMasSinsOptions`, `ListAssistanceListingsOptions`, `ListBusinessTypesOptions`, `ListOfficesOptions`, `ListDepartmentsOptions`.

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
