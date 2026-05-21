use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StorageError {
    #[error("failed to read/write storage: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serde(String),

    #[error("catalog id '{0}' already exists (set auto_fix_duplicate_catalog_id=true or use a unique id)")]
    CatalogIdAlreadyExist(String),

    #[error("provider id '{0}' already exists (set auto_fix_duplicate_provider_id=true or use a unique id)")]
    CatalogProviderIdAlreadyExist(String),

    #[error("provider '{0}' does not exist")]
    CatalogProviderDoesNotExist(String),

    #[error("catalog '{0}' does not exist")]
    CatalogDoesNotExist(String),

    #[error("catalogs not found: {0:?}")]
    CatalogsNotFound(Vec<String>),

    #[error("providers not found: {0:?}")]
    ProvidersNotFound(Vec<String>),

    #[error("{0}")]
    CatalogAlreadyHasProvider(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("invalid url: {0}")]
    InvalidUrl(String),
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("invalid identifier: {0}")]
    InvalidIdentifier(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum SuperSTACError {
    #[error("validation: {0}")]
    Validation(#[from] ValidationError),
    #[error("storage: {0}")]
    Storage(#[from] StorageError),
    #[error("search failed: {0}")]
    SearchFailed(String),
}
