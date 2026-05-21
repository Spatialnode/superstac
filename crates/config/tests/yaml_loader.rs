use std::path::PathBuf;
use std::time::Duration;

use superstac_config::init_from_yaml;
use superstac_core::{
    errors::SuperSTACError,
    models::{catalog::HealthCheckFrequencyStrategy, settings::LogLevel, storage::Storage},
    storages::factory::StorageBackend,
};

fn get_store(config_file: &str) -> Result<Box<dyn StorageBackend>, SuperSTACError> {
    let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(config_file);

    init_from_yaml(Storage::Memory, config_path.to_str().unwrap())
}

#[test]
fn storage_will_initialize_configurations_from_valid_config_file() {
    let store = get_store("superstac.yml");
    assert!(store.is_ok());

    let store = store.expect("Could not initialize storage from YAML config.");

    let catalogs = store.list_catalogs(None).expect("Could not list catalogs");
    assert_eq!(catalogs.len(), 2, "Should have loaded 2 catalogs");

    let earth_search = catalogs
        .iter()
        .find(|c| c.id == "earth-search")
        .expect("Missing 'earth-search' catalog");
    assert_eq!(
        earth_search.url,
        "https://earth-search.aws.element84.com/v1"
    );
    assert_eq!(
        earth_search.settings.health_check_strategy,
        HealthCheckFrequencyStrategy::Hourly
    );

    let settings = store.get_settings();
    assert_eq!(settings.log_level, LogLevel::Info);
    assert!(settings.logging_enabled);

    assert_eq!(
        settings.health_check_strategy,
        HealthCheckFrequencyStrategy::Custom(Duration::from_secs(900))
    );

    let providers = store
        .list_providers(None)
        .expect("Could not list providers");
    assert_eq!(providers.len(), 2, "Should have loaded 2 providers");

    let google_provider = providers
        .iter()
        .find(|p| p.id == "google")
        .expect("Missing 'google' provider");
    assert_eq!(google_provider.name.as_deref(), Some("Google Earth Engine"));
    assert_eq!(
        google_provider.website_url.as_deref(),
        Some("https://google.com")
    );
}

#[test]
fn storage_will_not_initialize_configuration_from_invalid_config_file_name() {
    let store = get_store(".invalid_name.yml");

    assert!(store.is_err());

    match store {
        Err(SuperSTACError::Storage(msg)) => {
            assert!(msg
                .to_string()
                .contains("invalid config file. expected `superstac.yml` or `superstac.yaml`"));
        }
        _ => panic!("Expected a specific Io error"),
    }
}

#[test]
fn storage_will_not_initialize_configuration_from_invalid_config_file() {
    let store = get_store("superstac.yaml");

    assert!(store.is_err());

    match store {
        Err(SuperSTACError::Storage(msg)) => {
            assert!(msg.to_string().contains(
                "serialization error: failed to parse YAML: unexpected end of input at line 2, column 1"
            ));
        }
        _ => panic!("Expected a specific Io error"),
    }
}

#[test]
fn yaml_config_loads_collection_aliases() {
    let store = get_store("superstac.yml")
        .expect("Could not initialize storage from YAML config.");

    let catalogs = store.list_catalogs(None).expect("Could not list catalogs");

    let earth_search = catalogs
        .iter()
        .find(|c| c.id == "earth-search")
        .expect("Missing 'earth-search' catalog");

    assert_eq!(earth_search.collection_aliases.len(), 2);
    assert_eq!(
        earth_search.collection_aliases.get("sentinel-2-l2a"),
        Some(&"S2MSI2A".to_string())
    );
    assert_eq!(
        earth_search.collection_aliases.get("sentinel-1-grd"),
        Some(&"S1GRD".to_string())
    );

    // A catalog without aliases declared should have an empty map (pass-through).
    let microsoft = catalogs
        .iter()
        .find(|c| c.id == "microsoft")
        .expect("Missing 'microsoft' catalog");
    assert!(microsoft.collection_aliases.is_empty());
}

#[test]
fn yaml_config_loads_asset_aliases() {
    let store = get_store("superstac.yml")
        .expect("Could not initialize storage from YAML config.");

    let catalogs = store.list_catalogs(None).expect("Could not list catalogs");

    let earth_search = catalogs
        .iter()
        .find(|c| c.id == "earth-search")
        .expect("Missing 'earth-search' catalog");

    let s2 = earth_search
        .asset_aliases
        .get("sentinel-2-l2a")
        .expect("Missing sentinel-2-l2a asset aliases");
    assert_eq!(s2.get("blue"), Some(&"B02".to_string()));
    assert_eq!(s2.get("nir"), Some(&"B08".to_string()));
}

#[test]
fn storage_will_not_initialize_configuration_from_unknown_file() {
    let store = get_store("does-not-exist/superstac.yaml");

    assert!(store.is_err());

    match store {
        Err(SuperSTACError::Storage(msg)) => {
            assert!(msg.to_string().contains(
                "failed to read/write storage: failed to open YAML file: No such file or directory (os error 2)"
            ));
        }
        _ => panic!("Expected a specific Io error"),
    }
}
