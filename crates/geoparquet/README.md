# SuperSTAC GeoParquet

Save STAC metadata for your study area and search it again without fetching the
same records from each provider. You can change dates and filters within the
saved area and time range. Imagery stays with the provider.

[Try the notebook](https://spatialnode.com/superstac/docs/python/geoparquet-notebook) ·
[Full guide](https://spatialnode.com/superstac/docs/guides/geoparquet)

## Save an inventory

Build and install the CLI from the repository root:

```sh
cargo install --path crates/cli --features geoparquet
cp docs/public/examples/superstac.yml ./superstac.yml
```

Start with a small area and date range. This saves Sentinel-2 metadata near Madrid:

```sh
superstac ingest --catalog earth-search --name madrid-feb \
  --collection sentinel-2-l2a --datetime 2025-02-01/2025-03-01 \
  --bbox=-4.5,39.5,-3,41 --output ./data --max-dataset-mib 1024
```

Use catalog IDs and collection names from your configuration. `madrid-feb` names
this saved area and date range, called a **scope**. Use another name to add another
area or month. Repeat `--catalog` to save metadata from more catalogs.

Progress appears as pages arrive. If the download stops, repeat the command with
`--resume`, keeping its name, filters, and page size unchanged. If the provider’s
continuation token has expired, restart without `--resume`.

## Search it

```sh
superstac --mode snapshot --dataset ./data --json search \
  --collection sentinel-2-l2a --datetime 2025-02-10/2025-02-15 \
  --bbox=-4,40,-3.5,40.5 --limit 20
```

Snapshot mode searches saved metadata without contacting providers. The inventory
must cover your query; check response metadata for coverage failures. Reopen a
snapshot client after updating the inventory so it sees the changes.

To use live catalogs when saved data is missing or too old:

```sh
superstac --mode auto --dataset ./data --max-snapshot-age-seconds 86400 \
  search --collection sentinel-2-l2a --datetime 2025-02-10/2025-02-15 \
  --bbox=-4,40,-3.5,40.5 --limit 20
```

Auto mode defaults to a maximum age of one day, measured from the start of the
download. It checks coverage and freshness for each search. Live results are not
saved automatically; run `ingest` to save metadata you want to reuse.

## Update and clean up

Run ingestion again with the same name to replace that scope. Other scopes stay
unchanged. To fetch only a recent acquisition window, use `--incremental-since`;
see the [update instructions](https://spatialnode.com/superstac/docs/guides/geoparquet#update-an-inventory)
for the required bounds. These updates cannot detect deletions or changes outside
the window, so use a full refresh when those changes matter.

The default storage limit is **1,024 MiB** of Parquet files, including old files
and unfinished downloads. Raise it with `--max-dataset-mib`, or use `0` for no
limit. Allow space for both old and new files during updates.

```sh
superstac compact --output ./data --items-per-file 10000 --max-dataset-mib 2048
superstac cleanup --output ./data              # preview files to remove
superstac cleanup --output ./data --apply      # remove unused Parquet files
```

Compaction combines updates and removes duplicate records while preserving
coverage and freshness. Cleanup removes files no longer needed by the saved
inventory or unfinished downloads. Release snapshot clients before applying
cleanup; `shutdown()` stops monitoring but keeps the client’s file lock.

Version-1 inventories remain readable. Their next successful ingestion writes a
version-2 manifest, which older SuperSTAC builds cannot read.

## Python and Rust

Python wheels include GeoParquet. Use `Client.ingest()` to save metadata and
`Client.from_yaml("superstac.yml", mode="snapshot", dataset="./data")` to search it.
`AsyncClient` supports the same options, with awaitable ingestion, search, and
maintenance methods.

In Rust, enable the engine’s `geoparquet` feature. Use `ingest_catalog` with
`IngestOptions`, `SuperSTACEngine::from_dataset` for local search, or
`SuperSTACEngine::automatic(storage, path, max_age)` for live fallback.
`compact_dataset` and `cleanup_dataset` are also available through the engine.
Datasets currently use local storage.

See the [guide](https://spatialnode.com/superstac/docs/guides/geoparquet) for Python
examples, progress callbacks, and more search options, or the
[benchmark](https://spatialnode.com/superstac/docs/guides/benchmarks) for local search
measurements and how to reproduce them.
