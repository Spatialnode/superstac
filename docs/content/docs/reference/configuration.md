---
title: Configuration
description: Complete YAML structure, catalog fields, and global search settings.
---

## Configuration sources

Python accepts a dictionary with optional `catalogs`, `providers`, and `settings` keys. The YAML loader is stricter: the current `SuperStacConfig` requires all three top-level fields, and `Settings` has six required fields. Start from the [complete downloadable config](/superstac/examples/superstac.yml).

The filename must be **`superstac.yml` or `superstac.yaml`**, even when supplied by an explicit path.

## Catalog entries

| Field | Meaning |
| --- | --- |
| `id` | Required local identifier. |
| `url` | Always provide the STAC API root URL; conversion assumes it is present. |
| `provider` | Optional ID of an already registered provider. |
| `title`, `description` | Optional descriptive strings. |
| `settings` | Optional complete per-catalog settings object. |
| `collection_aliases` | Dictionary of canonical collection IDs to local IDs. |
| `asset_aliases` | Dictionary keyed by canonical collection ID, then canonical asset key to local asset key. |

If you include catalog `settings`, supply `health_check_strategy` and `healthy_status_code_range`. The optional `enable_background_health_monitor` controls that catalog's monitor.

```yaml
# One catalog entry, to place inside the catalogs list.
- id: earth-search
  url: https://earth-search.aws.element84.com/v1
  settings:
    health_check_strategy: "15m"
    healthy_status_code_range: [200, 299]
    enable_background_health_monitor: false
```

Use these per-catalog fields to control health checks. Their defaults are hourly and HTTP 200–299; current health monitoring reads the catalog values, not the similarly named global settings.

## Provider entries

`id` is required. Optional descriptive fields are `name`, `description`, `logo_url`, and `website_url`. Use `providers: []` when no grouping is needed. Assign membership with each catalog's `provider`; the config conversion does not use provider `catalog_ids`.

## Global settings

Defaults below describe a new in-memory backend. In YAML, the first six fields are still required explicitly; Rust `Default` is not automatically applied by the YAML deserializer.

| Field | Default | Notes |
| --- | --- | --- |
| `health_check_strategy` | `hourly` | Required in YAML; current monitors read per-catalog settings. |
| `healthy_status_code_range` | `[200, 299]` | Required in YAML; inclusive range; use catalog override for monitoring. |
| `auto_fix_duplicate_catalog_id` | `true` | Required in YAML; suffix duplicate IDs instead of failing. |
| `auto_fix_duplicate_provider_id` | `true` | Required in YAML; same behavior for providers. |
| `log_level` | `info` | Required in YAML; `info`, `warning`, or `debug`. |
| `logging_enabled` | `true` | Required in YAML; controls CLI tracing setup. |
| `search_healthy_catalogs_only` | `true` | Filter candidates by health. |
| `deduplicate_items` | `true` | Collapse records sharing `Item.id`. |
| `unify_response` | `true` | Normalize collection and asset keys using aliases. |
| `max_concurrent_catalogs` | `8` | Concurrent catalog searches. |
| `per_catalog_timeout_seconds` | `30` | Timeout per search attempt. |
| `max_retry_attempts` | `2` | Includes initial attempt; `1` means no retry. |
| `retry_initial_backoff_ms` | `100` | Delay before first retry. |
| `retry_max_backoff_ms` | `2000` | Backoff ceiling. |
| `max_items_per_catalog` | `1000` | Hard cap; effective cap is the smaller of this and the query limit. |
| `enable_background_health_monitor` | `true` | Current memory-storage update path does not apply this global field; use per-catalog override. |

Use positive values for execution limits. Optional fields omitted from YAML preserve the memory backend's defaults through the loader's partial update path.

## Health frequency values

Named YAML values are `minutely`, `hourly`, `daily`, `weekly`, and `monthly` (30 days). Custom strings accept integer seconds, minutes, or hours: `"30s"`, `"15m"`, `"2h"`. Use a positive duration.

## Updating Python settings

```python
from superstac import Client

client = Client()
client.update_settings({"max_concurrent_catalogs": 4, "max_retry_attempts": 1})
print(client.get_settings())
```

These settings are held in memory and affect later searches. Read [health and retries](/docs/guides/resilience/) for lifecycle limitations. These references document the working tree, so released packages may differ.
