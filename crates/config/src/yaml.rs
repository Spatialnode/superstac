use std::{fs::File, io::Read, path::Path};

use superstac_core::{
    errors::{StorageError, SuperSTACError},
    models::storage::Storage,
    storages::factory::StorageBackend,
};

use crate::{config::SuperStacConfig, populate::populate_backend_from_config};

/// Read a `superstac.yml` (or `.yaml`) file and return a populated backend.
///
/// Filename must be exactly `superstac.yml` or `superstac.yaml` — anything
/// else is rejected upfront to catch typos.
pub fn init_from_yaml(
    storage: Storage,
    file_path: &str,
) -> Result<Box<dyn StorageBackend>, SuperSTACError> {
    let path = Path::new(file_path);

    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let is_name_valid = stem == "superstac";
        let is_ext_valid = ext == "yml" || ext == "yaml";

        if !is_name_valid || !is_ext_valid {
            return Err(StorageError::Io(format!(
                "invalid config file. expected `superstac.yml` or `superstac.yaml`, found `{}`",
                path.display()
            ))
            .into());
        }
    } else {
        return Err(StorageError::Io("invalid file path".to_string()).into());
    }

    tracing::debug!(path = %path.display(), "loading config");

    let mut file = File::open(path)
        .map_err(|e| StorageError::Io(format!("failed to open YAML file: {}", e)))?;

    let mut yaml_str = String::new();
    file.read_to_string(&mut yaml_str)
        .map_err(|e| StorageError::Io(format!("failed to read YAML file: {}", e)))?;

    let config: SuperStacConfig = serde_saphyr::from_str(&yaml_str)
        .map_err(|e| StorageError::Serde(format!("failed to parse YAML: {}", e)))?;

    let mut backend = storage.init();
    populate_backend_from_config(backend.as_mut(), config)?;
    Ok(backend)
}
