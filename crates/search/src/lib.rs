//! Federated STAC search.
//!
//! Given a list of catalogs and a [`query::SearchQuery`],
//! [`executor::SearchExecutor`] queries them concurrently (with retry,
//! per-catalog timeouts), delegates retrieval to [`backend::SearchBackend`],
//! and aggregates results in [`aggregator`]. The default [`stac_api::StacApiBackend`]
//! handles API pagination and canonicalization via [`unifier`].
//!
//! This crate is pure logic — it knows nothing about engine state or storage.
//! Use `superstac_engine` for the runtime that wires storage + search +
//! health together.

pub mod query;
pub mod response;
pub mod translator;
pub mod options;
pub mod executor;
pub mod aggregator;
pub mod unifier;

pub mod backend;
pub mod stac_api;

pub mod runtime;
#[cfg(target_arch = "wasm32")]
pub mod browser;
