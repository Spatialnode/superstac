//! Config-file loading for superstac. Reads `superstac.yml` and seeds a
//! storage backend with providers, catalogs, and settings.

pub mod config;
pub mod populate;
pub mod yaml;

pub use config::SuperStacConfig;
pub use populate::populate_backend_from_config;
pub use yaml::init_from_yaml;
