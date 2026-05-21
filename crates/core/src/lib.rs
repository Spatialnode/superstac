//! Domain models, storage trait, and shared utilities for superstac.
//!
//! - [`models`]: `Catalog`, `Provider`, `Settings`, etc.
//! - [`storages`]: the [`storages::factory::StorageBackend`] trait and the
//!   in-memory implementation.
//! - [`errors`]: typed error variants used across the workspace.

pub mod storages;
pub mod models;
pub mod errors;
pub mod utils;
