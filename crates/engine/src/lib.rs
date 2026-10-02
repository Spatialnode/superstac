//! Runtime that ties storage, search, and health monitoring together.
//!
//! [`SuperSTACEngine`] is the entry point: construct with a storage backend,
//! call `start()` to run health checks and `/collections` introspection, then
//! `search()`, `list_collections()`, etc.

pub mod engine;
pub mod health;
pub mod types;
pub mod capabilities;
pub mod discovery;

pub use engine::SuperSTACEngine;
pub use discovery::CollectionAvailability;
pub use types::SharedStorage;

#[cfg(feature = "geoparquet")]
pub use superstac_geoparquet::{
    ingest::{ingest_catalog, IngestOptions},
    maintenance::{cleanup as cleanup_dataset, compact as compact_dataset, CleanupReport},
    manifest::{DatasetManifest, IngestScope, Snapshot},
    progress::{IngestPhase, IngestProgress, ProgressCallback},
};

/// Version of the SuperSTAC engine in this build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
