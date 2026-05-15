//! # Tango — Rust SDK for the Tango federal-contracting API
//!
//! The Tango API gives programmatic access to federal-contracting data —
//! contracts, IDVs, entities, opportunities, grants, vehicles, protests, and
//! more — with dynamic response shaping so callers fetch only the fields they
//! need.
//!
//! This crate is the official, async-first Rust SDK. It mirrors the surface of
//! the [Node](https://github.com/makegov/tango-node),
//! [Python](https://github.com/makegov/tango-python), and
//! [Go](https://github.com/makegov/tango-go) SDKs while leaning into Rust
//! idioms (async streams, typed errors, compile-time-checked builders).
//!
//! ## Quick start
//!
//! ```no_run
//! use tango::Client;
//!
//! # async fn run() -> tango::Result<()> {
//! let client = Client::builder().api_key("your-api-key").build()?;
//!
//! let page = client
//!     .list_contracts(
//!         tango::ListContractsOptions::builder()
//!             .awarding_agency("9700")
//!             .shape(tango::SHAPE_CONTRACTS_MINIMAL)
//!             .limit(25u32)
//!             .build(),
//!     )
//!     .await?;
//!
//! for record in page.results {
//!     println!("{:?}", record.get("piid"));
//! }
//! # Ok(()) }
//! ```
//!
//! Or walk every page with the async stream API:
//!
//! ```no_run
//! # use tango::Client;
//! use futures::TryStreamExt;
//! # async fn run() -> tango::Result<()> {
//! # let client = Client::builder().api_key("x").build()?;
//! let mut stream = client.iterate_contracts(
//!     tango::ListContractsOptions::builder()
//!         .awarding_agency("9700")
//!         .build(),
//! );
//! while let Some(record) = stream.try_next().await? {
//!     // process one contract at a time
//!     let _ = record;
//! }
//! # Ok(()) }
//! ```
//!
//! ## Authentication
//!
//! Pass an API key via `Client::builder().api_key(...)`, or set
//! `TANGO_API_KEY` in the environment and call [`Client::from_env`].
//!
//! ## Error handling
//!
//! Every fallible call returns [`Result<T, Error>`]. Match on the variant to
//! dispatch on the failure mode:
//!
//! ```no_run
//! # use tango::{Client, Error};
//! # async fn run() -> tango::Result<()> {
//! # let client = Client::builder().api_key("x").build()?;
//! match client.get_agency("9700", None).await {
//!     Ok(agency) => println!("{}", agency.name.unwrap_or_default()),
//!     Err(Error::NotFound { .. }) => println!("not found"),
//!     Err(Error::RateLimit { retry_after, .. }) => println!("retry in {retry_after}s"),
//!     Err(e) => return Err(e),
//! }
//! # Ok(()) }
//! ```
//!
//! ## Response shaping
//!
//! List endpoints accept a `shape` parameter that picks which fields the API
//! returns. The SDK exposes ready-made presets — e.g.
//! [`SHAPE_CONTRACTS_MINIMAL`] — and you can also pass a custom field list.
//! See the [`shapes`] module.
//!
//! ## Webhook signing
//!
//! Webhook delivery verification is in the separate [`makegov-tango-webhooks`]
//! crate so a verifier service doesn't have to pull in the full SDK. See
//! `<https://docs.rs/makegov-tango-webhooks>` for HMAC-SHA256 signing and
//! verification.
//!
//! [`makegov-tango-webhooks`]: https://docs.rs/makegov-tango-webhooks

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Internal helpers used by Phase 2 resource modules. Suppressed for now so
// foundation work compiles cleanly before Wave-2 agents fill in the surface.
#![allow(dead_code)]

mod client;
mod error;
mod internal;
mod pagination;
mod resources;
mod transport;
mod version;

pub mod models;
pub mod shapes;

// ---------- Public surface ----------

pub use client::{Client, DEFAULT_RETRIES, DEFAULT_RETRY_BACKOFF, DEFAULT_TIMEOUT};
pub use error::{Error, ErrorBody, Result};
pub use internal::ListOptions;
pub use pagination::{Page, PageStream};
pub use shapes::{
    DEFAULT_BASE_URL, SHAPE_CONTRACTS_MINIMAL, SHAPE_ENTITIES_COMPREHENSIVE,
    SHAPE_ENTITIES_MINIMAL, SHAPE_FORECASTS_MINIMAL, SHAPE_GRANTS_MINIMAL,
    SHAPE_GSA_ELIBRARY_CONTRACTS_MINIMAL, SHAPE_IDVS_COMPREHENSIVE, SHAPE_IDVS_MINIMAL,
    SHAPE_ITDASHBOARD_INVESTMENTS_COMPREHENSIVE, SHAPE_ITDASHBOARD_INVESTMENTS_MINIMAL,
    SHAPE_NOTICES_MINIMAL, SHAPE_OPPORTUNITIES_MINIMAL, SHAPE_ORGANIZATIONS_MINIMAL,
    SHAPE_OTAS_MINIMAL, SHAPE_OTIDVS_MINIMAL, SHAPE_PROTESTS_MINIMAL, SHAPE_SUBAWARDS_MINIMAL,
    SHAPE_VEHICLES_COMPREHENSIVE, SHAPE_VEHICLES_MINIMAL, SHAPE_VEHICLE_AWARDEES_MINIMAL,
    SHAPE_VEHICLE_ORDERS_MINIMAL,
};
pub use transport::RateLimitInfo;
pub use version::VERSION;

// Re-export the resource option types so callers can `use tango::Foo;`
// without diving into module paths.
pub use resources::*;

/// A free-form list result record. Used as the `T` for shape-driven list
/// endpoints whose response schema depends on the requested `shape`.
///
/// `Record` is just a [`serde_json::Map`]. Use `record.get("field_name")` to
/// pull a field, or `serde_json::from_value(Value::Object(record))` to
/// deserialize into your own typed struct.
pub type Record = serde_json::Map<String, serde_json::Value>;
