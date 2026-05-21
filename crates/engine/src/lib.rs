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
