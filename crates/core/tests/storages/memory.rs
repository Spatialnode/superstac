use std::collections::HashMap;

use superstac_core::{
    storages::{factory::StorageBackend, memory::MemoryStorage},
    errors::SuperSTACError,
    models::{
        catalog::{Catalog, CatalogFilters, CatalogUpdate, HealthCheckFrequencyStrategy},
        provider::{ CatalogProvider, CatalogProviderFilters, CatalogProviderUpdate},
        settings::{LogLevel, Settings, SettingsUpdate},
    },
    utils::get_date_time,
};

////////////////////////######################################## UTILS ########################################////////////////////////////////////////////////////

fn create_catalog() -> Catalog {
    Catalog::new(
        "test-id",
        Some("Test STAC"),
        "https://test.com",
        Some("A test description"),
        None,
    )
    .expect("Catalog should be created")
}

fn create_store() -> MemoryStorage {
    MemoryStorage::init()
}

fn create_provider(id: &str) -> CatalogProvider {
    CatalogProvider::new(
        id.to_string(),
        Some("Microsoft Provider".to_string()),
        Some("This is the Microsoft Provider".to_string()),
        Some("https://website.com".to_string()),
        Some("https://www.google.com/logo.png".to_string()),
        None,
        None,
    )
    .expect("Provider should be created")
}

fn create_provider_with_result(id: &str) -> Result<CatalogProvider, SuperSTACError> {
    CatalogProvider::new(
        id.to_string(),
        Some("Microsoft Provider".to_string()),
        Some("This is the Microsoft Provider".to_string()),
        Some("https://website.com".to_string()),
        Some("https://www.google.com/logo.png".to_string()),
        None,
        None,
    )
}

////////////////////////######################################## GENERAL ########################################////////////////////////////////////////////////////

#[test]
fn memory_store_will_initiatialize_with_defaults() {
    let store = create_store();

    assert!(store.list_catalogs(None).unwrap().len() == 0);
    assert!(store.list_providers(None).unwrap().len() == 0);
    let settings = store.get_settings();
    assert_eq!(settings.auto_fix_duplicate_catalog_id, true);
    assert_eq!(settings.auto_fix_duplicate_provider_id, true);
    assert_eq!(settings.logging_enabled, true);
    assert_eq!(settings.log_level, LogLevel::Info);
    assert_eq!(
        settings.health_check_strategy,
        HealthCheckFrequencyStrategy::Hourly
    );
    assert_eq!(settings.healthy_status_code_range, (200, 299));
}

////////////////////////######################################## CATALOG ########################################////////////////////////////////////////////////////

#[test]
fn memory_store_rejects_catalog_with_invalid_id() {
    let catalog_id = "$$$";
    let catalog = Catalog::new(
        catalog_id,
        Some("Test STAC"),
        "https://test.com",
        Some("A test description"),
        None,
    );

    assert!(catalog.is_err());
    let err = catalog.unwrap_err();
    assert_eq!(err.to_string(), "validation: invalid identifier: only ASCII letters, digits, hyphen, and underscore are allowed");
}

#[test]
fn memory_store_creates_catalog_with_defaults_when_provider_and_settings_are_not_provided() {
    let mut store = create_store();
    let catalog_id = "test-id";
    // create a catalog in the store
    let catalog = store
        .create_catalog(
            Catalog::new(
                catalog_id,
                Some("Test STAC"),
                "https://test.com",
                Some("A test description"),
                None,
            )
            .unwrap(),
            None,
        )
        .expect("Could not create catalog even with valid parameters");

    assert_eq!(catalog.id, catalog_id.to_string());
    assert_eq!(
        catalog.created_at.unwrap().date_naive(),
        get_date_time().date_naive()
    );
    assert_eq!(catalog.provider, None);

    assert_eq!(catalog.health_status.available, false);

    assert_eq!(catalog.health_status.endpoint, catalog.url);

    // get the catalogs to confirm the length and content.

    assert!(store.list_catalogs(None).unwrap().len() == 1);

    assert!(store.get_catalog(catalog_id).unwrap().id == catalog_id);
}

#[test]
fn memory_store_panics_when_creating_catalog_with_invalid_provider() {
    let mut store = create_store();

    // create a catalog in the store
    let catalog = store.create_catalog(create_catalog(), Some(&"unknown-provider".to_string()));
    assert!(catalog.is_err());

    let err = catalog.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: provider 'unknown-provider' does not exist"
    );
}

#[test]
fn memory_store_does_not_panic_when_creating_catalog_with_duplicate_ids_and_settings_to_auto_fix_is_enabled(
) {
    let mut store = create_store();

    // create first catalog in the store
    let _ = store.create_catalog(create_catalog(), None);
    // create another catalog with the same id
    let catalog2 = store.create_catalog(create_catalog(), None);

    // by default the setting is enabled to autofix duplicate id, so that should happen.
    assert!(catalog2.is_ok());
}

#[test]
fn memory_store_panics_when_creating_catalog_with_duplicate_id_and_settings_to_auto_fix_is_disabled(
) {
    let mut store = create_store();

    // create first catalog in the store
    let _ = store.create_catalog(create_catalog(), None);

    // update settings
    store.update_settings(SettingsUpdate {
        auto_fix_duplicate_catalog_id: Some(false),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });
    // create another catalog with the same id
    let catalog2 = store.create_catalog(create_catalog(), None);

    assert!(catalog2.is_err());
    let err = catalog2.unwrap_err();
    assert_eq!(err.to_string(), "storage: catalog id 'test-id' already exists (set auto_fix_duplicate_catalog_id=true or use a unique id)");
}

#[test]
fn memory_store_accepts_updating_catalog_with_valid_parameters() {
    let mut store = create_store();

    // create a catalog in the store
    let catalog = store.create_catalog(create_catalog(), None).unwrap();

    // update the description
    let description = Some("This is a new description".to_string());
    let updated_catalog = store.update_catalog(
        &catalog.id,
        CatalogUpdate {
            provider: None,
            title: None,
            url: None,
            description: description.clone(),
            settings: None,
        },
    );
    assert!(updated_catalog.is_ok());
    assert_eq!(updated_catalog.as_ref().unwrap().description, description);
    // confirm that only description is updated i.e others remains intact
    assert_eq!(updated_catalog.unwrap().url, catalog.url);
}

#[test]
fn memory_store_rejects_updating_catalog_with_invalid_provider() {
    let mut store = create_store();

    // create a catalog in the store
    let catalog = store.create_catalog(create_catalog(), None).unwrap();

    // update the provider

    let updated_catalog = store.update_catalog(
        &catalog.id,
        CatalogUpdate {
            provider: Some("unknown-provider".to_string()),
            title: None,
            url: None,
            description: None,
            settings: None,
        },
    );
    assert!(updated_catalog.is_err());
    let err = updated_catalog.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: provider 'unknown-provider' does not exist"
    );
}

#[test]
fn memory_store_deletes_catalog_successfully_if_it_exists() {
    let mut store = create_store();

    // create a catalog in the store
    let catalog = store.create_catalog(create_catalog(), None).unwrap();

    // delete the catalog
    let deleted = store.delete_catalog(&catalog.id);
    assert!(deleted.is_ok());

    assert_eq!(store.list_catalogs(None).unwrap().len(), 0);
}

#[test]
fn memory_store_fails_to_delete_catalog_if_it_does_not_exist() {
    let mut store = create_store();

    // delete the catalog
    let deleted = store.delete_catalog(&"unknown-catalog");
    assert!(deleted.is_err());
    let err = deleted.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: catalog 'unknown-catalog' does not exist"
    );
}

#[test]
fn memory_store_deletes_multiple_catalogs_successfully_if_they_exist() {
    let mut store = create_store();

    // create a catalog in the store
    let catalog1 = store.create_catalog(create_catalog(), None).unwrap();

    let catalog2 = store.create_catalog(create_catalog(), None).unwrap();

    // delete the catalogs
    let deleted = store.delete_catalogs(vec![&catalog1.id, &catalog2.id]);
    assert!(deleted.is_ok());

    assert_eq!(store.list_catalogs(None).unwrap().len(), 0);
}

#[test]
fn memory_store_fails_to_delete_multiple_catalogs_if_one_does_not_exist() {
    let mut store = create_store();

    // create a catalog in the store
    let catalog1 = store.create_catalog(create_catalog(), None).unwrap();

    // delete the catalogs
    let deleted = store.delete_catalogs(vec![&catalog1.id, "unknown_catalog"]);

    assert!(deleted.is_err());

    // returns the one that wasn't found
    let err = deleted.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: catalogs not found: [\"unknown_catalog\"]"
    );

    // deletes the one that was found
    assert_eq!(store.list_catalogs(None).unwrap().len(), 0);
}

#[test]
fn memory_store_retrieves_catalog_if_it_exists() {
    let mut store = create_store();

    // create a catalog in the store
    let catalog = store.create_catalog(create_catalog(), None).unwrap();

    let retrieved_catalog = store.get_catalog(&catalog.id);

    assert_eq!(retrieved_catalog.unwrap().id, catalog.id);
}

#[test]
fn memory_store_rejects_unknown_catalog_id() {
    let mut store = create_store();

    // create a catalog in the store
    let _ = store.create_catalog(create_catalog(), None).unwrap();

    let retrieved_catalog = store.get_catalog("unknown_id");

    assert!(retrieved_catalog.is_err());

    let err = retrieved_catalog.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: catalog 'unknown_id' does not exist"
    );
}

#[test]
fn memory_store_returns_all_catalogs_without_filtering() {
    let mut store = create_store();
    // create a catalog in the store
    let _ = store.create_catalog(create_catalog(), None).unwrap();
    // create another catalog in the store
    let _ = store.create_catalog(create_catalog(), None).unwrap();

    let catalogs = store.list_catalogs(None).unwrap();
    // retrieve the catalogs - should be 2
    assert!(catalogs.len() == 2);
}

#[test]
fn memory_store_returns_matched_catalogs_when_filtered() {
    let mut store = create_store();
    // create a catalog in the store
    let catalog1 = store.create_catalog(create_catalog(), None).unwrap();
    // create another catalog in the store
    let _ = store.create_catalog(create_catalog(), None).unwrap();

    let catalogs = store.list_catalogs(None).unwrap();

    // retrieve the catalogs before filtering - should be 2
    assert!(catalogs.len() == 2);

    // filter by id and it should return only one

    let filtered_catalogs = store
        .list_catalogs(Some(CatalogFilters {
            id: Some("test-id".to_string()),
            ..CatalogFilters::default()
        }))
        .unwrap();

    assert!(filtered_catalogs.len() == 1);

    // The content should be the one of catalog1
    assert_eq!(filtered_catalogs.first().unwrap().id, catalog1.id);
    assert_eq!(filtered_catalogs.first().unwrap().url, catalog1.url);

    // Filter by string search in the title. For this add another catalog with a different name/description
    let creation_date = get_date_time();

    let catalog3 = store
        .create_catalog(
            Catalog::new(
                "new_id",
                Some("A different STAC"),
                "https://test.com",
                Some("A test description"),
                None,
            )
            .unwrap(),
            None,
        )
        .expect("Could not create catalog even with valid parameters");

    // search for 'different'
    let filtered_catalogs = store
        .list_catalogs(Some(CatalogFilters {
            title: Some("different".to_string()),
            ..CatalogFilters::default()
        }))
        .unwrap();

    assert!(filtered_catalogs.len() == 1);

    // The content should be the one of catalog1
    assert_eq!(filtered_catalogs.first().unwrap().id, catalog3.id);
    assert_eq!(filtered_catalogs.first().unwrap().title, catalog3.title);

    // filter for available - all should not be available

    let filtered_catalogs = store
        .list_catalogs(Some(CatalogFilters {
            available: Some(false),
            ..CatalogFilters::default()
        }))
        .unwrap();

    assert!(filtered_catalogs.len() == 3);

    let filtered_catalogs = store
        .list_catalogs(Some(CatalogFilters {
            available: Some(true),
            ..CatalogFilters::default()
        }))
        .unwrap();

    assert!(filtered_catalogs.len() == 0);

    // catalog3 was created after the 'creation_date' above. So if we filter by `created_after`, it should be one response i.e catalog3.

    let filtered_catalogs = store
        .list_catalogs(Some(CatalogFilters {
            created_after: Some(creation_date),
            ..CatalogFilters::default()
        }))
        .unwrap();

    assert!(filtered_catalogs.len() == 1);
    assert_eq!(filtered_catalogs.first().unwrap().id, catalog3.id);
    // if we filter by `created_before`, then it should be 2 responses, i.e catalog 1 and 2

    let filtered_catalogs = store
        .list_catalogs(Some(CatalogFilters {
            created_before: Some(creation_date),
            ..CatalogFilters::default()
        }))
        .unwrap();

    assert!(filtered_catalogs.len() == 2);
    assert!(filtered_catalogs.first().unwrap().id.contains(&catalog1.id));
}

////////////////////////######################################## RELATIONSHIPS ########################################////////////////////////////////////////////////////

#[test]
fn catalog_provider_relationship_updates_in_provider_when_catalog_is_created_with_a_provider() {
    let mut store = create_store();

    // create providers
    let provider1 = store
        .create_provider(create_provider("microsoft"))
        .expect("Provider could not be created even with valid arguments.");

    // create catalogs linked to the same provider

    let catalog1 = store
        .create_catalog(create_catalog(), Some(&provider1.id))
        .expect("Could not create catalog with provider");

    let catalog2 = store
        .create_catalog(create_catalog(), Some(&provider1.id))
        .expect("Could not create catalog with provider");

    // refetch provider

    let provider1 = store
        .get_provider(&provider1.id)
        .expect("Should return Provider1 data");

    // Confirm it's linked correctly in the providers' catalog_id
    let provider_catalogs = provider1
        .catalog_ids
        .as_ref()
        .expect("Should include catalog_ids");

    assert!(provider_catalogs.len() == 2);
    assert!(provider_catalogs.contains(&catalog1.id));
    assert!(provider_catalogs.contains(&catalog2.id));

    // Also refresh the catalog and confirm the catalog has the provider id
    let catalog1 = store
        .get_catalog(&catalog1.id)
        .expect("Catalog should exist.");

    assert_eq!(
        &provider1.id,
        catalog1.provider.as_ref().expect("Provider should exist.")
    );
}

#[test]
fn catalog_provider_relationship_updates_in_provider_when_catalog_is_deleted() {
    let mut store = create_store();

    // create provider
    let provider1 = store
        .create_provider(create_provider("microsoft"))
        .expect("Provider could not be created even with valid arguments.");

    // create catalog linked to the same provider

    let catalog1 = store
        .create_catalog(create_catalog(), Some(&provider1.id))
        .expect("Could not create catalog with provider");

    // delete catalog
    store
        .delete_catalog(&catalog1.id)
        .expect("Could not delete the catalog despite been created earlier.");

    // refresh provider
    let provider1 = store
        .get_provider(&provider1.id)
        .expect("Should return Provider1 data");

    // confirm provider is updated
    let provider_catalogs = provider1
        .catalog_ids
        .as_ref()
        .expect("Should include catalog_ids");

    assert!(provider_catalogs.len() == 0);
}

#[test]
fn catalog_provider_relationship_updates_correctly_when_catalog_is_updated_with_another_provider() {
    let mut store = create_store();

    // create provider
    let provider1 = store
        .create_provider(create_provider("microsoft"))
        .expect("Provider could not be created even with valid arguments.");

    // create provider
    let provider2 = store
        .create_provider(create_provider("google"))
        .expect("Provider could not be created even with valid arguments.");

    // create catalog linked to the first provider

    let catalog1 = store
        .create_catalog(create_catalog(), Some(&provider1.id))
        .expect("Could not create catalog with provider");

    // Update the catalog to use the second provider

    store
        .update_catalog(
            &catalog1.id,
            CatalogUpdate {
                provider: Some(provider2.id.clone()),
                title: None,
                url: None,
                description: None,
                settings: None,
            },
        )
        .expect("Could not update catalog even with the right parameters");

    // refresh first and second provider
    let provider1 = store
        .get_provider(&provider1.id)
        .expect("Should return Provider1 data");

    let provider2 = store
        .get_provider(&provider2.id)
        .expect("Should return Provider1 data");

    // confirm the first provider doesn't have the catalog again
    assert!(provider1.catalog_ids.is_none());

    // confirm the second provider now have the catalog
    let provider2_catalogs = provider2
        .catalog_ids
        .as_ref()
        .expect("Should include catalog_ids");

    assert!(provider2_catalogs.len() == 1);
    assert!(provider2_catalogs.contains(&catalog1.id));
}

#[test]
fn catalog_provider_relationship_updates_correctly_when_provider_is_deleted() {
    let mut store = create_store();

    // create provider
    let provider = store
        .create_provider(create_provider("microsoft"))
        .expect("Provider could not be created even with valid arguments.");

    // create catalog linked to the  provider

    let catalog = store
        .create_catalog(create_catalog(), Some(&provider.id))
        .expect("Could not create catalog with provider");

    // Delete the provider
    store
        .delete_provider(&provider.id)
        .expect("Provider should be deleted.");

    // refresh the catalog
    let catalog = store
        .get_catalog(&catalog.id)
        .expect("Should return catalog data");

    // confirm the catalog doesn't have provider again
    assert!(catalog.provider.is_none());
}

////////////////////////######################################## SETTINGS ########################################////////////////////////////////////////////////////

#[test]
fn settings_update_toggles_deduplicate_items() {
    let store = create_store();

    store.update_settings(SettingsUpdate {
        deduplicate_items: Some(false),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });

    assert_eq!(store.get_settings().deduplicate_items, Some(false));
}

#[test]
fn settings_default_enables_deduplicate_items() {
    let store = create_store();
    assert_eq!(store.get_settings().deduplicate_items, Some(true));
}

#[test]
fn settings_update_toggles_unify_response() {
    let store = create_store();

    store.update_settings(SettingsUpdate {
        unify_response: Some(false),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });

    assert_eq!(store.get_settings().unify_response, Some(false));
}

#[test]
fn settings_default_enables_unify_response() {
    let store = create_store();
    assert_eq!(store.get_settings().unify_response, Some(true));
}

#[test]
fn settings_default_includes_federation_hardening_defaults() {
    let store = create_store();
    let s = store.get_settings();
    assert_eq!(s.max_concurrent_catalogs, Some(8));
    assert_eq!(s.per_catalog_timeout_seconds, Some(30));
    assert_eq!(s.max_retry_attempts, Some(2));
    assert_eq!(s.retry_initial_backoff_ms, Some(100));
    assert_eq!(s.retry_max_backoff_ms, Some(2000));
}

#[test]
fn settings_update_changes_max_concurrent_catalogs() {
    let store = create_store();

    store.update_settings(SettingsUpdate {
        max_concurrent_catalogs: Some(16),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });

    assert_eq!(store.get_settings().max_concurrent_catalogs, Some(16));
}

#[test]
fn settings_update_changes_per_catalog_timeout() {
    let store = create_store();

    store.update_settings(SettingsUpdate {
        per_catalog_timeout_seconds: Some(60),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });

    assert_eq!(store.get_settings().per_catalog_timeout_seconds, Some(60));
}

#[test]
fn settings_update_changes_retry_policy() {
    let store = create_store();

    store.update_settings(SettingsUpdate {
        max_retry_attempts: Some(5),
        retry_initial_backoff_ms: Some(250),
        retry_max_backoff_ms: Some(5000),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });

    let s = store.get_settings();
    assert_eq!(s.max_retry_attempts, Some(5));
    assert_eq!(s.retry_initial_backoff_ms, Some(250));
    assert_eq!(s.retry_max_backoff_ms, Some(5000));
}

#[test]
fn settings_update_succeeds_with_valid_info() {
    let store = create_store();

    // update settings
    store.update_settings(SettingsUpdate {
        auto_fix_duplicate_catalog_id: Some(false),
        auto_fix_duplicate_provider_id: Some(false),
        health_check_strategy: Some(HealthCheckFrequencyStrategy::Daily),
        healthy_status_code_range: Some((300, 500)),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });

    let settings = store.get_settings();

    assert_eq!(settings.auto_fix_duplicate_catalog_id, false);
    assert_eq!(settings.auto_fix_duplicate_provider_id, false);
    assert_eq!(
        settings.health_check_strategy,
        HealthCheckFrequencyStrategy::Daily
    );
    assert_eq!(settings.healthy_status_code_range, (300, 500));
}

////////////////////////######################################## PROVIDER ########################################////////////////////////////////////////////////////

#[test]
fn memory_store_rejects_provider_with_invalid_id() {
    let provider_id = "$$$";
    let provider = create_provider_with_result(provider_id);

    assert!(provider.is_err());
    let err = provider.unwrap_err();
    assert_eq!(err.to_string(), "validation: invalid identifier: only ASCII letters, digits, hyphen, and underscore are allowed");
}

#[test]
fn memory_store_creates_provider_with_defaults_when_catalog_ids_and_settings_are_not_provided() {
    let mut store = create_store();
    let provider_id = "test-id";
    // create a catalog in the store
    let provider = store
        .create_provider(create_provider(provider_id))
        .expect("Could not create catalog even with valid parameters");

    assert_eq!(provider.id, provider_id.to_string());
    assert_eq!(
        provider.created_at.unwrap().date_naive(),
        get_date_time().date_naive()
    );
    assert_eq!(provider.catalog_ids, None);

    // get the catalogs to confirm the length and content.

    assert!(store.list_providers(None).unwrap().len() == 1);

    assert!(store.get_provider(provider_id).unwrap().id == provider_id);
}

// panic with invalid catalog_ids
#[test]
fn memory_store_panics_when_creating_provider_with_unknown_catalog_ids() {
    let mut store = create_store();

    // create a provider in the store
    let provider = CatalogProvider::new(
        "test-provider".to_string(),
        Some("Microsoft Provider".to_string()),
        Some("This is the Microsoft Provider".to_string()),
        Some("https://website.com".to_string()),
        Some("https://www.google.com/logo.png".to_string()),
        None,
        Some(vec!["unknown_catalogs".to_string()]),
    )
    .expect("Provider should be created");

    let created_provider = store.create_provider(provider);

    assert!(created_provider.is_err());

    let err = created_provider.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: catalog 'unknown_catalogs' does not exist"
    );
}

#[test]
fn memory_store_does_not_panic_when_creating_provider_with_duplicate_ids_and_settings_to_auto_fix_is_enabled(
) {
    let mut store = create_store();

    // create first provider in the store
    let _ = store.create_provider(create_provider("test-id"));

    // create another provider with the same id
    let provider = store.create_provider(create_provider("test-id"));

    // by default the setting is enabled to autofix duplicate id, so that should happen.
    assert!(provider.is_ok());
}

#[test]
fn memory_store_panics_when_creating_provider_with_duplicate_id_and_settings_to_auto_fix_is_disabled(
) {
    let mut store = create_store();

    // create first provider in the store
    let _ = store.create_provider(create_provider("test-id"));

    // update settings
    store.update_settings(SettingsUpdate {
        auto_fix_duplicate_provider_id: Some(false),
        ..SettingsUpdate::try_from(Settings::default()).unwrap()
    });
    // create another provider with the same id
    let provider = store.create_provider(create_provider("test-id"));

    assert!(provider.is_err());
    let err = provider.unwrap_err();
    assert_eq!(err.to_string(), "storage: provider id 'test-id' already exists (set auto_fix_duplicate_provider_id=true or use a unique id)");
}

#[test]
fn memory_store_accepts_updating_provider_with_valid_parameters() {
    let mut store = create_store();

    // create a provider in the store
    let provider = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    // update the description
    let description = Some("This is a new description".to_string());
    let updated_provider = store.update_provider(
        &provider.id,
        CatalogProviderUpdate {
            stac_version: None,
            name: Some("New title".to_string()),
            description: description.clone(),
            logo_url: None,
            website_url: None,
            catalog_ids: None,
        },
    );
    assert!(updated_provider.is_ok());
    assert_eq!(updated_provider.as_ref().unwrap().description, description);
    // confirm that only description is updated i.e others remains intact
    assert_eq!(updated_provider.unwrap().website_url, provider.website_url);
}

#[test]
fn memory_store_rejects_updating_provider_with_invalid_catalog_ids() {
    let mut store = create_store();

    // create a provider in the store
    let provider = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    // update the provider
    let updated_provider = store.update_provider(
        &provider.id,
        CatalogProviderUpdate {
            stac_version: None,
            name: Some("New title".to_string()),
            description: None,
            logo_url: None,
            website_url: None,
            catalog_ids: Some(vec!["unknown_catalog".to_string()]),
        },
    );
    assert!(updated_provider.is_err());
    let err = updated_provider.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: catalog 'unknown_catalog' does not exist"
    );
}

#[test]
fn memory_store_deletes_provider_successfully_if_it_exists() {
    let mut store = create_store();

    let provider = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    // delete the provider
    let deleted = store.delete_provider(&provider.id);

    assert!(deleted.is_ok());

    assert_eq!(
        store
            .list_providers(None)
            .expect("Providers should be returned")
            .len(),
        0
    );
}

#[test]
fn memory_store_fails_to_delete_provider_if_it_does_not_exist() {
    let mut store = create_store();

    // delete the provider
    let deleted = store.delete_provider(&"unknown-provider");
    assert!(deleted.is_err());
    let err = deleted.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: provider 'unknown-provider' does not exist"
    );
}

#[test]
fn memory_store_deletes_multiple_providers_successfully_if_they_exist() {
    let mut store = create_store();

    let provider1 = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    let provider2 = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    // delete the providers
    let deleted = store.delete_providers(vec![&provider1.id, &provider2.id]);
    assert!(deleted.is_ok());

    assert_eq!(
        store
            .list_providers(None)
            .expect("Providers should be returned")
            .len(),
        0
    );
}

#[test]
fn memory_store_fails_to_delete_multiple_providers_if_one_does_not_exist() {
    let mut store = create_store();

    let provider1 = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    // delete the providers
    let deleted = store.delete_providers(vec![&provider1.id, "unknown_catalog"]);

    assert!(deleted.is_err());

    // returns the one that wasn't found
    let err = deleted.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: providers not found: [\"unknown_catalog\"]"
    );

    // deletes the one that was found
    assert_eq!(
        store
            .list_providers(None)
            .expect("Providers should be returned")
            .len(),
        0
    );
}

#[test]
fn memory_store_retrieves_provider_if_it_exists() {
    let mut store = create_store();

    let provider = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    let retrieved_provider = store
        .get_provider(&provider.id)
        .expect("Provider should be retrieved");

    assert_eq!(retrieved_provider.id, provider.id);
}

#[test]
fn memory_store_rejects_unknown_provider_id() {
    let mut store = create_store();

    let _ = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    let retrieved_provider = store.get_provider("unknown_id");

    assert!(retrieved_provider.is_err());

    let err = retrieved_provider.unwrap_err();
    assert_eq!(
        err.to_string(),
        "storage: provider 'unknown_id' does not exist"
    );
}

#[test]
fn memory_store_returns_all_providers_without_filtering() {
    let mut store = create_store();

    let _ = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");
    let _ = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");

    let providers = store.list_providers(None).unwrap();
    // retrieve the catalogs - should be 2
    assert!(providers.len() == 2);
}


#[test]
fn memory_store_returns_matched_providers_when_filtered() {
    let mut store = create_store();
    let provider1 = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");
    let _ = store
        .create_provider(create_provider("test-provider"))
        .expect("Provider should be created");
    let providers = store.list_providers(None).unwrap();

    // retrieve the catalogs before filtering - should be 2
    assert!(providers.len() == 2);

    // filter by id and it should return only one

    let filtered_providers = store
        .list_providers(Some(CatalogProviderFilters {
            id: Some("test-provider".to_string()),
            name: None,
            description: None,
            stac_version: None,
            created_after: None,
            created_before: None,
            updated_before: None,
            updated_after: None,
            catalog_id: None,
        }))
        .unwrap();

    assert!(filtered_providers.len() == 1);

    // The content should be the one of provider1
    assert_eq!(filtered_providers.first().unwrap().id, provider1.id);
    assert_eq!(
        filtered_providers.first().unwrap().stac_version,
        provider1.stac_version
    );

    // Filter by string search in the title. For this add another catalog with a different name/description
    let creation_date = get_date_time();

    let provider3 = store
        .create_provider(
            CatalogProvider::new(
                "new_id".to_string(),
                Some("STAC Provider".to_string()),
                Some("A different STAC Provider".to_string()),
                Some("https://test.com".to_string()),
                Some("https://test.com".to_string()),
                Some("A test version".to_string()),
                None,
            )
            .expect("Provider should be created"),
        )
        .expect("Provider should be created");

    // search for 'different'
    let filtered_providers = store
        .list_providers(Some(CatalogProviderFilters {
            description: Some("different".to_string()),
            ..CatalogProviderFilters::default()
        }))
        .unwrap();

    assert!(filtered_providers.len() == 1);

    // The content should be the one of provider1
    assert_eq!(filtered_providers.first().unwrap().id, provider3.id);
    assert_eq!(
        filtered_providers.first().unwrap().description,
        provider3.description
    );

    // provider3 was created after the 'creation_date' above. So if we filter by `created_after`, it should be one response i.eprovider3.

    let filtered_providers = store
        .list_providers(Some(CatalogProviderFilters {
            created_after: Some(creation_date),
            ..CatalogProviderFilters::default()
        }))
        .unwrap();
    assert!(filtered_providers.len() == 1);
    assert_eq!(filtered_providers.first().unwrap().id, provider3.id);

    // if we filter by `created_before`, then it should be 2 responses, i.e catalog 1 and 2

    let filtered_providers = store
        .list_providers(Some(CatalogProviderFilters {
            created_before: Some(creation_date),
            ..CatalogProviderFilters::default()
        }))
        .unwrap();

    assert!(filtered_providers.len() == 2);
    assert!(filtered_providers
        .first()
        .unwrap()
        .id
        .contains(&provider1.id));
}

////////////////////////######################################## COLLECTION ALIASES ########################################////////////////////////////////////////////////////

#[test]
fn catalog_resolve_collection_returns_alias_when_present() {
    let mut catalog = create_catalog();
    catalog
        .collection_aliases
        .insert("sentinel-2-l2a".to_string(), "S2MSI2A".to_string());

    assert_eq!(catalog.resolve_collection("sentinel-2-l2a"), "S2MSI2A");
}

#[test]
fn catalog_resolve_collection_falls_back_to_canonical_when_no_alias() {
    let catalog = create_catalog();
    assert_eq!(catalog.resolve_collection("sentinel-2-l2a"), "sentinel-2-l2a");
}

#[test]
fn catalog_with_collection_aliases_attaches_map() {
    let mut aliases = HashMap::new();
    aliases.insert("sentinel-2-l2a".to_string(), "S2MSI2A".to_string());

    let catalog = create_catalog().with_collection_aliases(aliases);

    assert_eq!(catalog.resolve_collection("sentinel-2-l2a"), "S2MSI2A");
    assert_eq!(catalog.collection_aliases.len(), 1);
}

////////////////////////######################################## SOURCE SELECTION ########################################////////////////////////////////////////////////////

#[test]
fn memory_store_updates_supported_collections_for_existing_catalog() {
    let mut store = create_store();
    let catalog = store.create_catalog(create_catalog(), None).unwrap();

    let mut set = std::collections::HashSet::new();
    set.insert("sentinel-2-l2a".to_string());
    set.insert("landsat-c2-l2".to_string());

    let result = store.update_supported_collections(&catalog.id, Some(set.clone()));
    assert!(result.is_ok());

    let refreshed = store.get_catalog(&catalog.id).unwrap();
    assert_eq!(refreshed.supported_collections.as_ref(), Some(&set));
}

#[test]
fn memory_store_rejects_update_supported_collections_for_unknown_catalog() {
    let mut store = create_store();

    let result = store.update_supported_collections("unknown", None);
    assert!(result.is_err());
}

#[test]
fn catalog_supports_any_of_passes_through_when_uninitialized() {
    let catalog = create_catalog();
    assert!(catalog.supported_collections.is_none());
    assert!(catalog.supports_any_of(&["sentinel-2-l2a".to_string()]));
}

#[test]
fn catalog_supports_any_of_matches_when_introspected_set_overlaps() {
    let mut catalog = create_catalog();
    let mut set = std::collections::HashSet::new();
    set.insert("sentinel-2-l2a".to_string());
    catalog.supported_collections = Some(set);

    assert!(catalog.supports_any_of(&["sentinel-2-l2a".to_string()]));
    assert!(!catalog.supports_any_of(&["landsat-c2-l2".to_string()]));
    // Multi-collection query: at least one match -> include
    assert!(catalog.supports_any_of(&[
        "landsat-c2-l2".to_string(),
        "sentinel-2-l2a".to_string()
    ]));
}

#[test]
fn catalog_supports_any_of_matches_when_no_collection_filter() {
    let mut catalog = create_catalog();
    catalog.supported_collections = Some(std::collections::HashSet::new());
    // Empty request -> match all catalogs regardless of supported set
    assert!(catalog.supports_any_of(&[]));
}
