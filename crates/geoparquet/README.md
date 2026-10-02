# SuperSTAC GeoParquet

Ingest reusable STAC metadata scopes, then search local snapshots or route to
live APIs when local coverage is missing or stale. Imagery is not downloaded.

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

The newest single scope covering the entire query is selected for each catalog.
Queries are allowed to differ from ingestion within the recorded coverage.
Scope unions are not inferred: a query spanning two separately ingested months
needs a single covering scope or live fallback. Coverage failures appear in
per-catalog response metadata. Snapshot mode has no freshness cutoff, performs
no provider monitoring, and never falls back to HTTP. An empty complete snapshot
correctly returns no items. Collection discovery and sorting are not supported.

Snapshot engines retain their loaded manifest; reopen one to see updates.
File metadata prunes irrelevant collections, dates and bounding boxes. Remaining
files are scanned in batches of 1,024 rows. There is no row-group predicate
pushdown or spatial index yet.

### Automatic selection

```sh
superstac --mode auto --dataset ./data --max-snapshot-age-seconds 86400 \
  search --collection sentinel-2-l2a --datetime 2025-02-10/2025-02-15 \
  --bbox=-4,40,-3.5,40.5 --limit 20
```

For each configured catalog, auto uses one complete, covering scope whose source
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
Collection discovery and sorting are not supported in auto mode.

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
refreshes require headroom for both old and new files. Cleanup, compaction and
incremental synchronization are future work. A dataset-wide OS lock prevents
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
not panic. They can be adapted to application UIs or future Python callbacks.
The Python build feature exists, but Python client methods for these operations
are not exposed yet. Remote object storage is not supported.
