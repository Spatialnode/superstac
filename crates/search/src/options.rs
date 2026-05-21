use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Default,
)]
pub struct SearchOptions {
    pub max_items: Option<usize>,

    pub headers: Vec<(String, String)>,

    pub timeout_seconds: Option<u64>,
}

/// Cross-catalog federation knobs derived from `Settings`.
#[derive(Debug, Clone, Copy)]
pub struct FederationOptions {
    /// Collapse items sharing the same `Item.id` across catalogs.
    pub deduplicate: bool,
    /// Rewrite item collection + asset keys to canonical names.
    pub unify_response: bool,
    /// Maximum number of catalogs queried concurrently.
    pub max_concurrent: usize,
    /// Per-catalog request timeout (applied to each attempt).
    pub per_catalog_timeout: Duration,
    /// Retry policy for transient per-catalog failures.
    pub retry: RetryPolicy,
    /// Hard cap on items returned per catalog (prevents runaway pagination).
    pub max_items_per_catalog: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    /// Total attempts per catalog (1 = no retry).
    pub max_attempts: u8,
    /// Backoff before the first retry.
    pub initial_backoff: Duration,
    /// Cap on exponential backoff growth.
    pub max_backoff: Duration,
}

impl Default for FederationOptions {
    fn default() -> Self {
        Self {
            deduplicate: true,
            unify_response: true,
            max_concurrent: 8,
            per_catalog_timeout: Duration::from_secs(30),
            retry: RetryPolicy::default(),
            max_items_per_catalog: 1000,
        }
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 2,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_millis(2000),
        }
    }
}
