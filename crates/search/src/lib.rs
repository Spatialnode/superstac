//! Federated STAC search.
//!
//! Given a list of catalogs and a [`query::SearchQuery`],
//! [`executor::SearchExecutor`] queries them concurrently (with retry,
//! pagination, and per-catalog timeouts), unifies item bodies via
//! [`unifier`], and aggregates results in [`aggregator`].
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
