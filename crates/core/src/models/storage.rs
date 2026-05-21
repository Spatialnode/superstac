use crate::storages::{factory::StorageBackend, memory::MemoryStorage};
use serde::{Deserialize, Serialize};

/// Backend selector. SQLite and Postgres variants currently fall back to
/// the in-memory backend — they're placeholders for future work.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Storage {
    Memory,
    Sqlite(String),
    Postgres(String),
}

impl Storage {
    /// Construct an empty backend. Config loaders (e.g. YAML) call this and
    /// then populate the returned backend.
    pub fn init(self) -> Box<dyn StorageBackend> {
        match self {
            Storage::Memory => Box::new(MemoryStorage::init()),
            Storage::Sqlite(_path) => Box::new(MemoryStorage::init()),
            Storage::Postgres(_url) => Box::new(MemoryStorage::init()),
        }
    }
}
