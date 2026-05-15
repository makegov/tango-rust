# Response shaping

Dynamic response shaping is the Tango API's signature feature: instead of always receiving every field on a resource, you tell the server exactly which fields you want, and it returns only those. Payloads stay small, responses stay fast, and the SDK doesn't have to chase schema drift.

This guide covers the shape grammar, the 21 built-in `SHAPE_*` constants, the `flat` / `flat_lists` modifiers, and the trade-offs to think about when picking a shape.

## What is a shape?

A shape is a comma-separated list of fields to return. The simplest shape is a flat field list:

```text
key,piid,award_date
```

You can nest fields with `parent(child1, child2)`:

```text
key,piid,recipient(display_name,uei),total_contract_value
```

You can request every field at a given level with `*`:

```text
key,piid,recipient(*)
```

You can alias fields with `::`:

```text
recipient::vendor(display_name,uei)
```

The grammar (informally):

```text
shape       := field_list
field_list  := field ("," field)*
field       := field_name [alias] [nested]
field_name  := identifier | "*"
alias       := "::" identifier
nested      := "(" field_list ")"
identifier  := [a-zA-Z_][a-zA-Z0-9_]*
```

The SDK doesn't parse or validate shape strings client-side — it passes the value through as the `shape` query parameter. The server returns `400` with a parse error in the body if the shape is malformed; that surfaces as `Error::Validation { message, .. }`.

## Using a shape

Every list and get options builder has a `.shape(...)` setter:

```rust
use tango::{Client, ListContractsOptions};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let page = client.list_contracts(
    ListContractsOptions::builder()
        .shape("key,piid,award_date,recipient(display_name)")
        .limit(25u32)
        .awarding_agency("9700")
        .build(),
).await?;
# Ok(()) }
```

For convenience, the SDK ships 21 preset constants. Use them when you don't need a custom selector:

```rust
use tango::{Client, ListContractsOptions, SHAPE_CONTRACTS_MINIMAL};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let page = client.list_contracts(
    ListContractsOptions::builder().shape(SHAPE_CONTRACTS_MINIMAL).build(),
).await?;
# Ok(()) }
```

## Shape preset constants

All 21 constants live in `shapes.rs` and are re-exported at the crate root. They mirror the `ShapeConfig.*` enums in `tango-node` and `tango-python` exactly — same names, same field selectors, same intent.

| Constant | Intended use | Notes |
| -------- | ------------ | ----- |
| `SHAPE_CONTRACTS_MINIMAL` | `list_contracts` | key, piid, award_date, recipient (name), description, total_contract_value |
| `SHAPE_ENTITIES_MINIMAL` | `list_entities` | uei, legal_business_name, cage_code, business_types |
| `SHAPE_ENTITIES_COMPREHENSIVE` | `get_entity` | UEI + names + NAICS/PSC + addresses + federal_obligations + congressional district |
| `SHAPE_FORECASTS_MINIMAL` | `list_forecasts` | id, title, anticipated_award_date, fiscal_year, naics_code, status |
| `SHAPE_OPPORTUNITIES_MINIMAL` | `list_opportunities` | opportunity_id, title, solicitation_number, response_deadline, active |
| `SHAPE_NOTICES_MINIMAL` | `list_notices` | notice_id, title, solicitation_number, posted_date |
| `SHAPE_PROTESTS_MINIMAL` | `list_protests` | case_id, case_number, title, source_system, outcome, filed_date |
| `SHAPE_GRANTS_MINIMAL` | `list_grants` | grant_id, opportunity_number, title, status(*), agency_code |
| `SHAPE_IDVS_MINIMAL` | `list_idvs` | key, piid, award_date, recipient(name,uei), description, totals, idv_type |
| `SHAPE_IDVS_COMPREHENSIVE` | `get_idv` | all the above + period_of_performance, awarding/funding office, transactions, parent award |
| `SHAPE_VEHICLES_MINIMAL` | `list_vehicles` | uuid + solicitation + program + counts + obligations |
| `SHAPE_VEHICLES_COMPREHENSIVE` | `get_vehicle` | the above + agency_id + naics/psc + metrics |
| `SHAPE_VEHICLE_AWARDEES_MINIMAL` | `list_vehicle_awardees` | per-awardee summary across orders |
| `SHAPE_VEHICLE_ORDERS_MINIMAL` | `list_vehicle_orders` | per-order summary |
| `SHAPE_ORGANIZATIONS_MINIMAL` | `list_organizations` | key, fh_key, name, level, type, short_name |
| `SHAPE_OTAS_MINIMAL` | `list_otas` | OTA awards (mirror of contracts minimal) |
| `SHAPE_OTIDVS_MINIMAL` | `list_otidvs` | OTA IDVs |
| `SHAPE_SUBAWARDS_MINIMAL` | `list_subawards` | award_key + prime/sub recipient (id and amount NOT allowed) |
| `SHAPE_GSA_ELIBRARY_CONTRACTS_MINIMAL` | `list_gsa_elibrary_contracts` | uuid + contract_number + schedule + recipient + idv |
| `SHAPE_ITDASHBOARD_INVESTMENTS_MINIMAL` | `list_itdashboard` | matches `INVESTMENT_LIST_DEFAULT_SHAPE` server-side |
| `SHAPE_ITDASHBOARD_INVESTMENTS_COMPREHENSIVE` | `get_itdashboard` | matches `INVESTMENT_RETRIEVE_DEFAULT_SHAPE` server-side |

## `flat` and `flat_lists` modifiers

By default, nested objects come back nested. Set `flat=true` to collapse them into dot-separated keys at the top level:

```rust
# use tango::{Client, ListContractsOptions};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
let page = client.list_contracts(
    ListContractsOptions::builder()
        .shape("recipient(display_name,uei)")
        .flat(true)
        .build(),
).await?;
// Each record looks like:
//   { "recipient.display_name": "...", "recipient.uei": "..." }
// instead of:
//   { "recipient": { "display_name": "...", "uei": "..." } }
# Ok(()) }
```

With `flat=false` (default), arrays of nested objects come back as arrays. With both `flat=true` AND `flat_lists=true`, those arrays are spread into numbered keys:

```text
"period_of_performance.0.start_date": "...",
"period_of_performance.0.end_date": "...",
"period_of_performance.1.start_date": "...",
...
```

Most callers want `flat=false`. The `flat` family exists for spreadsheet exporters and downstream systems that can't handle JSON nesting.

## Custom shapes

The presets are starting points. Roll your own when the use case warrants:

```rust
use tango::{Client, ListContractsOptions};
# async fn run() -> tango::Result<()> {
# let client = Client::builder().api_key("x").build()?;
// Just the keys you'll use, no more.
let page = client.list_contracts(
    ListContractsOptions::builder()
        .shape("piid,obligated,recipient(uei),awarding_agency(name)")
        .build(),
).await?;
# Ok(()) }
```

The narrower the shape, the smaller the payload and the faster the response. For very wide-fanout endpoints (e.g. `list_entities`), shaving an unused field can shave a noticeable percentage of latency.

## Trade-offs

- **Wider shapes** are easier to start with — you don't have to think about which fields you need. But payloads grow quickly, particularly when you ask for nested aggregates (`federal_obligations(*)`, `metrics(*)`, etc.).
- **Narrower shapes** are leaner over the wire but bake in a dependency on the field list you chose. If the server adds a new field you'd want, you have to update the shape.
- **Stable typed structs** (e.g. `AgencyRecord`, `ProtestRecord`) deliberately accept any shape — unknown fields go to `extra`, expected fields go to named slots, so a shape change doesn't break the deserialization.

Pick the narrowest shape that gives you everything you need today, and revisit when the API surface grows.
