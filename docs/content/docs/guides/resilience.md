---
title: Health, retries, and failures
description: Configure search concurrency and understand how partial failures affect results.
---

## Set timeouts and retry limits

Use these settings to control how much work a search can do and how long it waits for each catalog:

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

Use positive values for these settings. `max_retry_attempts` includes the first attempt, so `2` allows one retry and `1` turns retries off. The wait between retries doubles until it reaches `retry_max_backoff_ms`.

The timeout covers one attempt to search a catalog and collect its items. A full search can take longer because retries, startup health checks, and collection discovery take additional time.

Search errors are retried even when some failures are permanent. If repeated attempts fail, read the failure reason before increasing the retry limit.

## Inspect partial failures

You can still get results when one catalog fails. Check the failure list to see which catalogs are missing from your results:

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

A response with zero items is not necessarily a successful empty search. Check both `catalogs_queried` and `catalogs_failed`. You can receive a response even if every selected catalog fails.

## Check catalog health

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
