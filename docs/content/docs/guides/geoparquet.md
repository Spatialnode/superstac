---
title: GeoParquet inventories
description: Save the metadata for your area of interest and explore it repeatedly without querying every provider again.
---

**Build your satellite-data inventory. Search it again and again.**

Save metadata for your area of interest, then explore different dates and scenes
without repeating every provider request. When saved coverage is missing or stale,
auto mode queries the live catalogs. Imagery stays with the provider.

For example, retain Sentinel-2 metadata for a study area and revisit it throughout
your analysis. Refresh that inventory when you need newer records.

```sh
cargo build -p superstac-cli --features geoparquet
```

Catalog IDs must exist in `superstac.yml`. Collection arguments use configured
canonical names; stored items retain the provider's original names.

## Ingestion and progress

```sh
superstac ingest --catalog earth-search --name madrid-feb \
  --collection sentinel-2-l2a --datetime 2025-02-01/2025-03-01 \
  --bbox=-4.5,39.5,-3,41 --output ./data --max-dataset-mib 1024
```

Repeat `--catalog` to ingest multiple catalogs sequentially. Supply a collection,
spatial or temporal filter, or explicitly use `--all` to ingest an entire catalog.
Ingestion runs only when requested, never automatically at startup or on search.

Progress goes to stderr; `--json` stdout remains machine-readable. Terminals get
an updating line, redirected output gets throttled log lines. `--no-progress`
disables progress; `--quiet` also suppresses summaries. Events distinguish items
received from items saved in a durable checkpoint, and report pages, file count,
compressed bytes saved, elapsed time and throughput. A provider's optional
`numberMatched` (or `context.matched`) supplies an approximate percentage and ETA.
Without a total, only counters are displayed. Estimates can change during paging.

`--page-size 500` controls the API page size, not the total result count. All next
links are followed, including on empty pages. Initial requests use POST `/search`;
GET/POST continuation links, relative URLs, and STAC body/header merging are
supported. Connection errors and HTTP 429/5xx receive up to three attempts.
`--timeout-seconds 60` bounds each request; long Retry-After pauses stop with a
resumable checkpoint.

`--items-per-file 10000` is a row target, flushed at page boundaries and split by
collection. A 64 MiB response-byte threshold also flushes buffers; individual
API responses are capped at 32 MiB. Parsed objects and encoding require extra
memory. Files infer schemas across a bounded group; unsupported attributes or
schema combinations fail instead of being silently discarded. Asset/link hrefs
are made absolute before writing.

## Retained scopes and refresh

`--name madrid-feb` identifies a scope within a catalog. A successful ingest using
the same name replaces only that scope; other names and catalogs are retained.
Changing the bounds under the same name intentionally replaces its old coverage.

Without a name, SuperSTAC reuses a matching existing scope, or derives a stable
name from normalized coverage. Different regions/months therefore coexist.

Repeat an interrupted command with `--resume`, keeping name, scope, catalog and
page size the same. Resume starts at the last checkpoint; uncommitted buffered
pages may be fetched again. Expired provider continuation tokens require a new
run without `--resume`. If a matching snapshot is already complete and has no
checkpoint, resume returns it without downloading again (it does not refresh it).

The version-2 manifest groups snapshots as `catalogs[catalog_id][scope_name]`.
Version-1 manifests remain readable as a scope named `legacy`. The next successful
ingestion publishes version 2; interrupted unnamed version-1 jobs can be resumed.
Older SuperSTAC versions cannot read a manifest after that upgrade.

Snapshots record source URL, scope, start/completion times, counts and file
bounds. Completion means pagination was exhausted for the requested scope. It
does not make a changing provider transactionally consistent or guarantee that
an older snapshot is current.

## Search modes

### Snapshot only

```sh
superstac --mode snapshot --dataset ./data --json search \
  --collection sentinel-2-l2a --datetime 2025-02-10/2025-02-15 \
  --bbox=-4,40,-3.5,40.5 --limit 20
```

The newest single scope covering the query is preferred. Otherwise, complete
scopes can jointly cover the requested collections, bounding box and time range.
Coverage is checked as a union, not just an enclosing bounding box: gaps still
require live fallback. Intersects queries use their enclosing bbox conservatively.
Overlapping records are deduplicated by source-local collection and item ID;
newest scopes and incremental delta records take precedence. Queries can differ
from ingestion within this recorded coverage. Coverage failures appear in
per-catalog response metadata. Snapshot mode has no freshness cutoff, performs
no provider monitoring, and never falls back to HTTP. An empty complete snapshot
correctly returns no items. Collection discovery describes the **saved inventory**, including locally derived
STAC Collection extents; it does not claim to reproduce the provider's full
collection metadata. Sorting is supported per catalog with repeated
`--sortby=-datetime` / `--sortby=eo:cloud_cover` (Python: `sortby=["-datetime"]`).
Null values sort last. Sorted queries scan all candidate rows and retain only the
requested top results; the final federated response is not globally sorted.

Snapshot engines retain their loaded manifest; reopen one to see updates.
File metadata prunes irrelevant collections, dates and bounding boxes. Remaining
files are scanned in batches of 1,024 rows, pruning row groups using collection,
ID, instant datetime, and available GeoParquet bbox statistics. Missing or nullable
statistics are handled conservatively. New files use 1,024-row groups and order
records by acquisition time within each file. Overlays and multi-scope unions
scan without pruning so an older version of a moved item cannot reappear.
Compaction restores pruning for incremental scopes by resolving their overlays.
Decoding is bounded, but deduplication keeps item keys in memory (proportional to
scanned unique items). Local collection summaries also scan the saved files.

### Automatic selection

```sh
superstac --mode auto --dataset ./data --max-snapshot-age-seconds 86400 \
  search --collection sentinel-2-l2a --datetime 2025-02-10/2025-02-15 \
  --bbox=-4,40,-3.5,40.5 --limit 20
```

For each configured catalog, auto uses complete covering scopes whose source
URL still matches and whose age is within the configured maximum (default one
day). Age is measured from ingestion **start**, conservatively accounting for
long downloads. A value of zero effectively forces live requests. Freshness is
rechecked per query and the latest manifest is read; old provider health and
collection inventories do not suppress searches.

Missing/stale coverage or missing/corrupt local data routes the whole catalog
query to its API. Local partial results are not merged with live results. A
fresh snapshot with zero matches does not trigger a live request. Catalog
timeouts still apply to the overall attempt. Tracing logs report `source=live`
or `source=snapshot` on stderr when logging is enabled.

**Auto is read-only:** it does not persist API results, launch ingestion, create
a dataset, or mark limited search results as complete coverage. Use `ingest` to
save a new scope explicitly. A missing dataset is a cache miss, not an error.
Collection discovery in auto mode queries the live APIs on demand; it does not
start background monitoring. Sorting is forwarded to live APIs on fallback.

### Live and standalone files

```sh
superstac --mode live search --collection sentinel-2-l2a --limit 10
superstac --geoparquet earth-search=./items.parquet search --limit 20
```

Without `--mode`, existing behavior is preserved: a dataset/file argument selects
snapshot mode, otherwise live mode. Standalone files have no managed coverage or
freshness guarantees. Repeat `--geoparquet CATALOG=PATH` for multiple catalogs.

## Storage policy

The ingestion default is a **1,024 MiB budget** for files retained under `runs/`,
including active, old, orphaned and checkpointed generations. Increase it with
`--max-dataset-mib`; `0` explicitly disables the cap. Metadata/checkpoints and
filesystem overhead are not included. The writer stops before exceeding its
remaining byte budget, retains the last published manifest, and leaves a
checkpoint. Raise the budget and resume to continue.

No automatic eviction or deletion occurs. Staleness only affects auto routing;
it does not delete archives. Retired generations remain for active readers, so
refreshes require headroom for both old and new files. Explicit cleanup, compaction and incremental refresh are available below. A dataset-wide OS lock prevents
concurrent writers and releases on process exit.

```text
data/
  manifest.json
  checkpoints/              # may contain provider continuation tokens/headers
  runs/snapshot-.../
    collection=.../part-....parquet
```

## Rust

With the engine's `geoparquet` feature enabled, use `ingest_catalog` and
`IngestOptions`, `SuperSTACEngine::from_dataset`, or
`SuperSTACEngine::automatic(storage, path, max_age)`.

`IngestOptions::progress` accepts an `Arc<dyn Fn(&IngestProgress) + Send + Sync>`.
Callbacks receive `IngestPhase` and counters, must return promptly, and should
not panic. `cleanup_dataset` and `compact_dataset` are also re-exported by
`superstac-engine`. Remote object storage is not supported.

## Incremental refresh

```sh
superstac ingest --catalog earth-search --name madrid-feb \
  --collection sentinel-2-l2a --datetime 2025-02-01/2025-03-05 \
  --bbox=-4.5,39.5,-3,41 --output ./data \
  --incremental-since 2025-02-25T00:00:00Z
```

The named scope must already exist. Keep its source, collections, spatial bounds
and start time unchanged; its end may advance. `--incremental-since` must overlap
the old end (no gaps). The API fetches only the specified acquisition-time window;
returned records override earlier copies with the same collection/item ID.
Resume uses the same options and verifies that the base scope has not changed.

This is an acquisition-window overlay, not a provider change feed. It cannot
detect deletions or revisions/late arrivals outside that window. Use an overlap
appropriate to the provider, and periodic full refreshes (omit
`--incremental-since`) when those changes matter. Full refresh replaces that
scope's inventory. Incremental refresh deliberately preserves its original
freshness timestamp: untouched history has not been verified again.

## Compaction and cleanup

```sh
superstac compact --output ./data --items-per-file 10000 --max-dataset-mib 2048
superstac cleanup --output ./data              # preview paths and reclaimable bytes
superstac cleanup --output ./data --apply      # delete unreferenced Parquet files
```

Compaction streams each scope, deduplicates its records, and publishes replacement
files atomically. It retains coverage and freshness timestamps. It requires space
for both old and new generations; a failure leaves the old manifest readable.
Cleanup protects manifest files and durable checkpoint files. Applied cleanup
refuses active snapshot readers or writers; drop/recreate long-lived snapshot
clients before reclaiming files. Preview is safe while readers are active.
Empty directories and checkpoint metadata are retained.

## Python

Published wheels include GeoParquet. Source builds use the `geoparquet` Cargo
feature, enabled in `pyproject.toml`; minimal Rust builds can omit it. Check
`superstac.geoparquet_available` at runtime.

```python
from superstac import Client

client = Client.from_yaml("superstac.yml")
client.ingest(
    "earth-search", "./data", name="madrid-feb",
    collections=["sentinel-2-l2a"], datetime="2025-02-01/2025-03-01",
    bbox=[-4.5, 39.5, -3, 41],
    progress=lambda event: print(event["phase"], event["items_saved"]),
)
local = Client.from_yaml("superstac.yml", mode="snapshot", dataset="./data")
results = local.search(
    collections=["sentinel-2-l2a"], datetime="2025-02-10/2025-02-15",
    bbox=[-4, 40, -3.5, 40.5], sortby=["-datetime"], limit=20,
)
```

`Client`, `AsyncClient`, `open()` and `from_yaml()` accept `mode`, `dataset`, and
`max_snapshot_age_seconds`. Async clients expose awaitable `ingest`,
`compact_dataset`, and `cleanup_dataset`. Constructors and `from_yaml` remain
synchronous. Callbacks receive dictionaries, run on the worker thread, and should
return promptly; callback exceptions are reported as unraisable Python exceptions
without aborting ingestion. `shutdown()` stops monitoring; release the client to
release its snapshot reader lock. Existing snapshot clients retain their loaded
manifest; reopen after ingestion or compaction to see updates.

Ingestion keyword options include `name`, `collections`, `datetime`, `bbox`,
`resume`, `incremental_since`, `page_size`, `items_per_file`, `max_dataset_mib`,
`timeout_seconds` and explicit `all=True`. Unknown options raise an error.

## Performance checks

Run the reproducible synthetic benchmark without downloading provider data:

```sh
cargo test -p superstac-geoparquet --release benchmark_scoped_search -- --ignored --nocapture
SUPERSTAC_BENCH_ITEMS=1000000 cargo test -p superstac-geoparquet --release benchmark_scoped_search -- --ignored --nocapture
```

It compares the same bounded scanner with and without row-group pruning, verifies
identical results, and reports median latency over five runs. The fixture contains
time-ordered point items; it is a best-case selective time query, not a promise
about provider latency, arbitrary geometries, or gigabyte production archives.


## Writer provenance and support reports

Provider requests use `User-Agent: superstac/<version>`. Report CLI versions with
`superstac --version`, Python versions with `superstac.__version__`, or Rust engine
versions with `superstac_engine::VERSION`. Search diagnostics include
`metadata.superstac_version`.

Newly ingested and compacted Parquet files preserve standard `geo` and
`stac-geoparquet` metadata and add `created_by=superstac/<version>` plus a JSON
`superstac` key containing `version`, `crate`, and `written_at`. Older files are
unchanged until rewritten. To inspect one with PyArrow:

```python
import json
import pyarrow.parquet as pq
footer = pq.read_metadata("path/to/part.parquet")
print(footer.created_by)
print(json.loads(footer.metadata[b"superstac"]))
```
