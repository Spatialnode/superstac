---
title: Health, retries, and failures
description: Configure search concurrency and understand how partial failures affect results.
---

## Bound the work

```python
from superstac import Client

client = Client(settings={
    "max_concurrent_catalogs": 4,
    "per_catalog_timeout_seconds": 20,
    "max_retry_attempts": 2,
    "retry_initial_backoff_ms": 100,
    "retry_max_backoff_ms": 2000,
    "max_items_per_catalog": 100,
})
```

Use positive values for concurrency, timeouts, attempts, and item caps. `max_retry_attempts` counts the initial attempt: `1` disables retry. Backoff grows exponentially up to the configured ceiling.

The timeout applies to each catalog search attempt, including its item collection. It is not a timeout for the entire client lifecycle: initial health checks and collection discovery use separate requests.

The current retry classification is broad: search errors are retried, including some permanent failures. Do not assume only HTTP 5xx responses are retried.

## Inspect partial failures

A catalog failure can be recorded while other catalogs return useful items. Always inspect the response metadata when completeness matters.

```python
from superstac import Client

client = Client.open("https://earth-search.aws.element84.com/v1")
try:
    search = client.search(collections=["sentinel-2-l2a"], limit=5)
    for failure in search.metadata["failures"]:
        print(failure["catalog_id"], failure["reason"])
finally:
    client.shutdown()
```

A response with zero items is not necessarily a successful empty search. Check both `catalogs_queried` and `catalogs_failed`. The executor can return a response even if all selected catalogs fail.

## Health monitoring in the current alpha

Search normally selects healthy catalogs only. Startup checks each endpoint and starts monitors for catalogs that initially pass. A catalog that is unhealthy at startup does not currently receive a background monitor, so automatic recovery is not guaranteed; restart the engine to recheck it.

Set polling frequency and healthy status range on each catalog. The current monitor reads **catalog settings**, rather than inheriting the similarly named global settings.

```yaml
# Fragment within a catalog entry.
settings:
  health_check_strategy: "15m"
  healthy_status_code_range: [200, 299]
  enable_background_health_monitor: false
```

Disabling the monitor does not skip the initial health check. The global `enable_background_health_monitor` update is not currently applied by memory storage; use the per-catalog setting above.

Always call `shutdown()` (or `await client.shutdown()`) to stop background tasks when finished.
