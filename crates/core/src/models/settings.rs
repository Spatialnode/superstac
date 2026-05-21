use serde::{Deserialize, Serialize};

use crate::{errors::ValidationError, models::catalog::HealthCheckFrequencyStrategy};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warning,
    Debug,
}

/// Workspace-wide settings. Loaded from YAML (`settings:` block) or built
/// from [`Settings::default`]. Field-level docs cover each knob.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub health_check_strategy: HealthCheckFrequencyStrategy,
    /// Status codes considered "healthy" (inclusive). Default `(200, 299)`.
    pub healthy_status_code_range: (u16, u16),
    /// When true, duplicate catalog ids get auto-suffixed instead of erroring.
    pub auto_fix_duplicate_catalog_id: bool,
    /// Same as above, for provider ids.
    pub auto_fix_duplicate_provider_id: bool,
    pub log_level: LogLevel,
    /// Disable tracing-subscriber init entirely when false.
    pub logging_enabled: bool,
    /// Drop unhealthy catalogs from federated search. Default true.
    pub search_healthy_catalogs_only: Option<bool>,
    /// Collapse items sharing the same `Item.id` across catalogs into one,
    /// with provenance tracked via `SearchItem.seen_in`. Defaults to true.
    pub deduplicate_items: Option<bool>,
    /// Rewrite each item's `collection` and `assets` to canonical names
    /// using each catalog's alias maps. Defaults to true.
    pub unify_response: Option<bool>,
    /// Maximum number of catalogs to query concurrently. Defaults to 8.
    pub max_concurrent_catalogs: Option<usize>,
    /// Per-catalog request timeout in seconds. Defaults to 30.
    pub per_catalog_timeout_seconds: Option<u64>,
    /// Maximum attempts per catalog (1 = no retry). Defaults to 2.
    pub max_retry_attempts: Option<u8>,
    /// Initial retry backoff in milliseconds. Defaults to 100.
    pub retry_initial_backoff_ms: Option<u64>,
    /// Maximum retry backoff in milliseconds (caps exponential growth). Defaults to 2000.
    pub retry_max_backoff_ms: Option<u64>,
    /// Hard cap on items returned per catalog (prevents runaway pagination).
    /// Defaults to 1000.
    pub max_items_per_catalog: Option<usize>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            health_check_strategy: HealthCheckFrequencyStrategy::Hourly,
            healthy_status_code_range: (200, 299),
            auto_fix_duplicate_catalog_id: true,
            auto_fix_duplicate_provider_id: true,
            log_level: LogLevel::Info,
            logging_enabled: true,
            search_healthy_catalogs_only: Some(true),
            deduplicate_items: Some(true),
            unify_response: Some(true),
            max_concurrent_catalogs: Some(8),
            per_catalog_timeout_seconds: Some(30),
            max_retry_attempts: Some(2),
            retry_initial_backoff_ms: Some(100),
            retry_max_backoff_ms: Some(2000),
            max_items_per_catalog: Some(1000),
        }
    }
}

/// Partial [`Settings`] update — `None` fields are left alone.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsUpdate {
    pub health_check_strategy: Option<HealthCheckFrequencyStrategy>,
    pub healthy_status_code_range: Option<(u16, u16)>,
    pub auto_fix_duplicate_catalog_id: Option<bool>,
    pub auto_fix_duplicate_provider_id: Option<bool>,
    pub log_level: Option<LogLevel>,
    pub logging_enabled: Option<bool>,
    pub search_healthy_catalogs_only: Option<bool>,
    pub deduplicate_items: Option<bool>,
    pub unify_response: Option<bool>,
    pub max_concurrent_catalogs: Option<usize>,
    pub per_catalog_timeout_seconds: Option<u64>,
    pub max_retry_attempts: Option<u8>,
    pub retry_initial_backoff_ms: Option<u64>,
    pub retry_max_backoff_ms: Option<u64>,
    pub max_items_per_catalog: Option<usize>,
}

impl TryFrom<Settings> for SettingsUpdate {
    type Error = ValidationError;
    fn try_from(value: Settings) -> Result<Self, Self::Error> {
        Ok(SettingsUpdate {
            healthy_status_code_range: Some(value.healthy_status_code_range),
            health_check_strategy: Some(value.health_check_strategy),
            auto_fix_duplicate_catalog_id: Some(value.auto_fix_duplicate_catalog_id),
            auto_fix_duplicate_provider_id: Some(value.auto_fix_duplicate_provider_id),
            log_level: Some(value.log_level),
            logging_enabled: Some(value.logging_enabled),
            search_healthy_catalogs_only: value.search_healthy_catalogs_only,
            deduplicate_items: value.deduplicate_items,
            unify_response: value.unify_response,
            max_concurrent_catalogs: value.max_concurrent_catalogs,
            per_catalog_timeout_seconds: value.per_catalog_timeout_seconds,
            max_retry_attempts: value.max_retry_attempts,
            retry_initial_backoff_ms: value.retry_initial_backoff_ms,
            retry_max_backoff_ms: value.retry_max_backoff_ms,
            max_items_per_catalog: value.max_items_per_catalog,
        })
    }
}
