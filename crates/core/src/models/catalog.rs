use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::errors::{SuperSTACError, ValidationError};

use crate::utils::{get_datetime_utc, parse_url, validate_identifier};

/// How often to poll a catalog's health endpoint.
///
/// YAML accepts a named variant (`minutely`, `hourly`, ...) or a
/// `Custom` duration string like `"15s"`, `"30m"`, `"2h"`.
#[derive(Clone, Debug, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthCheckFrequencyStrategy {
    Minutely,
    #[default]
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Custom(Duration),
}

/// Per-catalog overrides. Anything unset falls back to [`Default`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogSettings {
    pub health_check_strategy: HealthCheckFrequencyStrategy,
    /// Status codes considered "healthy" (inclusive). Default `(200, 299)`.
    pub healthy_status_code_range: (u16, u16),
    pub enable_background_health_monitor: Option<bool>,
}

impl Default for CatalogSettings {
    fn default() -> Self {
        CatalogSettings {
            health_check_strategy: HealthCheckFrequencyStrategy::Hourly,
            healthy_status_code_range: (200, 299),
            enable_background_health_monitor: None,
        }
    }
}
impl HealthCheckFrequencyStrategy {
    pub fn as_duration(&self) -> Duration {
        match self {
            HealthCheckFrequencyStrategy::Minutely => Duration::from_secs(60),
            HealthCheckFrequencyStrategy::Hourly => Duration::from_secs(60 * 60),
            HealthCheckFrequencyStrategy::Daily => Duration::from_secs(60 * 60 * 24),
            HealthCheckFrequencyStrategy::Weekly => Duration::from_secs(60 * 60 * 24 * 7),
            HealthCheckFrequencyStrategy::Monthly => Duration::from_secs(60 * 60 * 24 * 30),
            HealthCheckFrequencyStrategy::Custom(dur) => *dur,
        }
    }
}

impl<'de> Deserialize<'de> for HealthCheckFrequencyStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.trim().to_lowercase();

        match s.as_str() {
            "minutely" => Ok(HealthCheckFrequencyStrategy::Minutely),
            "hourly" => Ok(HealthCheckFrequencyStrategy::Hourly),
            "daily" => Ok(HealthCheckFrequencyStrategy::Daily),
            "weekly" => Ok(HealthCheckFrequencyStrategy::Weekly),
            "monthly" => Ok(HealthCheckFrequencyStrategy::Monthly),
            _ => {
                // Parse custom duration like "15m", "30s", "1h"
                let dur = if s.ends_with("s") {
                    let n = &s[..s.len() - 1]
                        .parse::<u64>()
                        .map_err(serde::de::Error::custom)?;
                    Duration::from_secs(*n)
                } else if s.ends_with("m") {
                    let n = &s[..s.len() - 1]
                        .parse::<u64>()
                        .map_err(serde::de::Error::custom)?;
                    Duration::from_secs(*n * 60)
                } else if s.ends_with("h") {
                    let n = &s[..s.len() - 1]
                        .parse::<u64>()
                        .map_err(serde::de::Error::custom)?;
                    Duration::from_secs(*n * 3600)
                } else {
                    return Err(serde::de::Error::custom(format!("Invalid duration: {}", s)));
                };
                Ok(HealthCheckFrequencyStrategy::Custom(dur))
            }
        }
    }
}

impl FromStr for HealthCheckFrequencyStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.ends_with("s") {
            let secs = s[..s.len() - 1]
                .parse::<u64>()
                .map_err(|_| "Invalid seconds")?;
            Ok(HealthCheckFrequencyStrategy::Custom(Duration::from_secs(
                secs,
            )))
        } else if s.ends_with("m") {
            let mins = s[..s.len() - 1]
                .parse::<u64>()
                .map_err(|_| "Invalid minutes")?;
            Ok(HealthCheckFrequencyStrategy::Custom(Duration::from_secs(
                mins * 60,
            )))
        } else if s.ends_with("h") {
            let hours = s[..s.len() - 1]
                .parse::<u64>()
                .map_err(|_| "Invalid hours")?;
            Ok(HealthCheckFrequencyStrategy::Custom(Duration::from_secs(
                hours * 3600,
            )))
        } else {
            Err(format!("Invalid custom duration: {}", s))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct HealthStatus {
    pub endpoint: String,
    pub available: bool,
    pub last_checked: Option<DateTime<Utc>>,
    /// The status code response from the STAC Catalog Server.
    pub status_code: u16,
}


pub fn get_default_health_status(url: String) -> HealthStatus {
    HealthStatus {
        // defaults to false. Always assumes the health status is down. It will be updated after the first health check.
        available: false,
        // Defaults to the catalog url.
        endpoint: url,
        last_checked: Some(get_datetime_utc()),
        status_code: 200,
    }
}
/// The capabilities of the STAC Catalog.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogCapabilities {
    filtering: String,
}

/// A STAC catalog endpoint registered with SuperSTAC.
///
/// Construct via [`Catalog::new`] for direct use, or by loading a YAML
/// `superstac.yml` (which deserializes [`CatalogConfig`] and converts via
/// `TryFrom`). `health_status` and `supported_collections` are managed by
/// the engine — don't set them manually.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Catalog {
    pub id: String,
    /// Linked provider id, or `None` if standalone.
    pub provider: Option<String>,
    pub title: Option<String>,
    pub url: String,
    pub description: Option<String>,
    /// Per-catalog overrides for health check frequency, healthy status range, etc.
    pub settings: CatalogSettings,
    /// Updated by the engine's health monitor — don't set manually.
    pub health_status: HealthStatus,
    pub capabilities: Option<CatalogCapabilities>,
    /// Maps canonical collection IDs (e.g. `sentinel-2-l2a`) to this catalog's
    /// local collection IDs (e.g. `S2MSI2A`). Empty map means pass-through.
    #[serde(default)]
    pub collection_aliases: HashMap<String, String>,
    /// Per-collection asset rename rules, keyed by canonical collection ID.
    /// Inner map: canonical asset key -> this catalog's local asset key.
    /// Empty means pass-through (catalog already uses canonical names).
    #[serde(default)]
    pub asset_aliases: HashMap<String, HashMap<String, String>>,
    /// The set of canonical collection IDs this catalog is known to support.
    /// `None` means not yet introspected — searches pass through to this catalog.
    /// `Some(set)` is authoritative — searches for collections outside the set
    /// will skip this catalog.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_collections: Option<HashSet<String>>,
    /// Auto-populated on creation.
    pub created_at: Option<DateTime<Utc>>,
    /// Auto-populated on every update.
    pub updated_at: Option<DateTime<Utc>>,
}

impl Catalog {
    /// Validate and construct a catalog. Errors if the id contains non-ASCII
    /// characters or the URL doesn't parse.
    pub fn new(
        id: &str,
        title: Option<impl Into<String>>, 
        url: &str,
        description: Option<impl Into<String>>,
        settings: Option<CatalogSettings>,
    ) -> Result<Self, SuperSTACError> {
        
        validate_identifier(&id)?;
        
        let valid_url = parse_url(url)
            .map_err(|err| SuperSTACError::from(ValidationError::InvalidUrl(err.to_string())))?;

        let url_string = valid_url.to_string();

        Ok(Self {
            id: id.to_string(),
            provider: None,
            title: title.map(|t| t.into()),
            url: url_string.clone(),
            description: description.map(|d| d.into()),
            settings: settings.unwrap_or_default(),
            health_status: get_default_health_status(url_string),
            capabilities: None,
            collection_aliases: HashMap::new(),
            asset_aliases: HashMap::new(),
            supported_collections: None,
            created_at: Some(get_datetime_utc()),
            updated_at: None,
        })
    }

    /// Builder-style: attach collection aliases.
    pub fn with_collection_aliases(mut self, aliases: HashMap<String, String>) -> Self {
        self.collection_aliases = aliases;
        self
    }

    /// Returns this catalog's local collection name for `canonical`, or
    /// `canonical` itself when no alias is configured.
    pub fn resolve_collection<'a>(&'a self, canonical: &'a str) -> &'a str {
        self.collection_aliases
            .get(canonical)
            .map(String::as_str)
            .unwrap_or(canonical)
    }

    /// Inverse of `resolve_collection`: returns the canonical collection name
    /// for the given catalog-local name. Falls back to `local` when no alias
    /// maps to it.
    pub fn canonical_collection<'a>(&'a self, local: &'a str) -> &'a str {
        for (canonical, l) in &self.collection_aliases {
            if l == local {
                return canonical;
            }
        }
        local
    }

    /// Returns true if this catalog should be queried for the given canonical
    /// collection IDs. Catalogs with `supported_collections = None` (not yet
    /// introspected) pass through. Empty `requested` (no collection filter)
    /// matches every catalog.
    pub fn supports_any_of(&self, requested: &[String]) -> bool {
        match &self.supported_collections {
            None => true,
            Some(set) => requested.is_empty() || requested.iter().any(|c| set.contains(c)),
        }
    }

    /// Stamp `updated_at` with the current time.
    pub fn set_update_date(&mut self) {
        self.updated_at = Some(get_datetime_utc());
    }

    /// Apply a partial update. `None` fields are left alone.
    pub fn update(
        &mut self,
        description: Option<String>,
        url: Option<String>,
        title: Option<String>,
        settings: Option<CatalogSettings>,
    ) -> Result<(), ValidationError> {
        if let Some(updated_url) = url {
            self.url = parse_url(updated_url.as_str())
                .map_err(|err| ValidationError::InvalidUrl(err.to_string()))?
                .to_string();
        }

        self.title = title;
        self.description = description;

        if let Some(updated_settings) = settings {
            self.settings = updated_settings;
        }

        self.set_update_date();
        Ok(())
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id
    }

    pub fn set_provider(&mut self, provider: &str) {
        self.provider = Some(provider.to_owned())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogFilters {
    /// Performs an exact match on the `id` field.
    pub id: Option<String>,
    /// Performs a string search on the catalog provider's name and description.
    pub provider: Option<String>,
    /// Performs a string search on the catalog title.
    pub title: Option<String>,
    /// Performs a string search on the catalog description.
    pub description: Option<String>,
    /// Performs a boolean search on the catalog health status.
    pub available: Option<bool>,

    /// Performs a date search on the `created_at` field. Filters for for date `after` the provided date.
    pub created_after: Option<DateTime<Utc>>,
    /// Performs a date search on the `created_at` field. Filters for for date `before` the provided date.
    pub created_before: Option<DateTime<Utc>>,
    /// Performs a date search on the `updated_at` field. Filters for for date `after` the provided date.
    pub updated_after: Option<DateTime<Utc>>,
    /// Performs a date search on the `updated_at` field. Filters for for date `before` the provided date.
    pub updated_before: Option<DateTime<Utc>>,
}

impl Default for CatalogFilters {
    fn default() -> Self {
        CatalogFilters {
            id: None,
            provider: None,
            title: None,
            description: None,
            available: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogUpdate {
    /// Attach a provider to a catalog using the provider id.
    pub provider: Option<String>,
    /// Updates the catalog title.
    pub title: Option<String>,
    /// Updates the catalog description.
    pub description: Option<String>,
    /// Updates the catalog url.
    pub url: Option<String>,
    /// Updates the catalog settings.
    pub settings: Option<CatalogSettings>,
}
