use parking_lot::Mutex;
use std::sync::Arc;
use superstac_core::storages::factory::StorageBackend;

pub type SharedStorage = Arc<Mutex<Box<dyn StorageBackend + Send + Sync>>>;
