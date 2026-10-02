use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand, ValueEnum};
use stac::Bbox;
use superstac_config::init_from_yaml;
use superstac_core::models::{settings::LogLevel, settings::Settings, storage::Storage};
use superstac_engine::SuperSTACEngine;
use superstac_search::query::SearchQuery;
use tracing_subscriber::EnvFilter;

/// Many catalogs. One search. Search across STAC catalogs from the command line.
#[derive(Parser)]
#[command(name = "superstac", version, about, long_about = None)]
struct Cli {
    /// Path to the SuperSTAC config file.
    #[arg(long, default_value = "superstac.yml", global = true)]
    config: PathBuf,

    /// Search local snapshots only. Repeat for each catalog: --geoparquet ID=PATH.
    /// Paths are relative to the working directory; requires the geoparquet feature.
    #[arg(long, global = true, value_name = "CATALOG=PATH", value_parser = parse_snapshot)]
    geoparquet: Vec<(String, PathBuf)>,

    /// Search a managed dataset's completed snapshots (requires geoparquet).
    #[arg(long, global = true, conflicts_with = "geoparquet")]
    dataset: Option<PathBuf>,

    /// Choose live APIs, local snapshots, or fresh-snapshot-first routing.
    #[arg(long, global = true, value_enum)]
    mode: Option<SearchMode>,

    /// Maximum snapshot age in auto mode, measured from ingestion start (default 86400).
    #[arg(long, global = true)]
    max_snapshot_age_seconds: Option<u64>,

    /// Increase log verbosity (debug-level).
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    verbose: bool,

    /// Quiet logs (warn-level only).
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Output as JSON instead of human-readable text.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum SearchMode {
    Live,
    Snapshot,
    Auto,
}

#[derive(Subcommand)]
enum Command {
    /// Federated search across catalogs.
    Search(SearchArgs),

    /// Fetch a scoped API snapshot into a managed GeoParquet dataset.
    Ingest(IngestArgs),

    /// Inspect available collections.
    Collections(CollectionsArgs),
    /// Reclaim retired Parquet files (dry run unless --apply).
    Cleanup(MaintenanceArgs),
    /// Rewrite snapshots into larger files and deduplicate records.
    Compact(CompactArgs),
}

#[derive(Args)]
struct SearchArgs {
    /// Collection IDs to query. Repeatable; canonical names — alias mapping
    /// happens internally.
    #[arg(short, long = "collection")]
    collections: Vec<String>,

    /// Bounding box `w,s,e,n` (four comma-separated floats).
    #[arg(short, long, value_parser = parse_bbox)]
    bbox: Option<Bbox>,

    /// Datetime range, e.g. `2024-01-01/2024-01-31` or a single instant.
    #[arg(short, long)]
    datetime: Option<String>,

    /// Max items per catalog.
    #[arg(short, long, default_value_t = 10)]
    limit: usize,

    /// Specific item IDs to fetch (repeatable).
    #[arg(long = "id")]
    ids: Vec<String>,
    /// Repeatable STAC sort field, e.g. --sortby=-datetime.
    #[arg(long)]
    sortby: Vec<String>,
}

#[derive(Args)]
#[command(group(clap::ArgGroup::new("scope").required(true).multiple(true)
    .args(["collections", "datetime", "bbox", "all"])))]
struct IngestArgs {
    /// Configured catalog IDs to ingest sequentially. Repeatable.
    #[arg(long = "catalog", required = true)]
    catalogs: Vec<String>,
    /// Canonical collection IDs. Repeatable.
    #[arg(long = "collection")]
    collections: Vec<String>,
    #[arg(long)]
    datetime: Option<String>,
    #[arg(long, value_parser = parse_bbox)]
    bbox: Option<Bbox>,
    /// Explicitly ingest the entire catalog instead of a filtered scope.
    #[arg(long, conflicts_with_all = ["collections", "datetime", "bbox"])]
    all: bool,
    #[arg(long, default_value = "./data")]
    output: PathBuf,
    /// Scope name; refreshes only this name. Omit to derive it from coverage.
    #[arg(long)]
    name: Option<String>,
    /// Overlay records acquired since this timestamp; requires an existing --name.
    #[arg(long, requires = "name")]
    incremental_since: Option<String>,
    /// Retained Parquet budget including old/partial generations; 0 disables the cap.
    #[arg(long, default_value_t = 1024)]
    max_dataset_mib: u64,
    /// Disable progress output.
    #[arg(long)]
    no_progress: bool,
    /// Resume with the same catalogs, scope, name, and page size.
    #[arg(long)]
    resume: bool,
    /// Server page size; all pages are fetched regardless of search limits.
    #[arg(long, default_value_t = 500)]
    page_size: usize,
    /// Approximate rows per file, flushed at page boundaries and split by collection.
    #[arg(long, default_value_t = 10_000)]
    items_per_file: usize,
    #[arg(long, default_value_t = 60)]
    timeout_seconds: u64,
}

#[derive(Args)]
struct MaintenanceArgs {
    #[arg(long, default_value = "./data")]
    output: PathBuf,
    #[arg(long)]
    apply: bool,
}
#[derive(Args)]
struct CompactArgs {
    #[arg(long, default_value = "./data")]
    output: PathBuf,
    #[arg(long, default_value_t = 10000)]
    items_per_file: usize,
    #[arg(long, default_value_t = 1024)]
    max_dataset_mib: u64,
}

#[derive(Args)]
struct CollectionsArgs {
    /// Catalog ID. If omitted, lists all collections across catalogs.
    catalog: Option<String>,

    /// Collection ID (requires `catalog`). Prints full metadata.
    collection: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let db = match init_from_yaml(Storage::Memory, &cli.config.to_string_lossy()) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("error: failed to load config: {}", e);
            return ExitCode::FAILURE;
        }
    };

    init_tracing(&db.get_settings(), cli.verbose, cli.quiet);

    if !cli.json {
        let n = db.list_catalogs(None).map(|c| c.len()).unwrap_or(0);
        println!("loaded {} catalogs", n);
    }

    if let Command::Ingest(args) = &cli.command {
        if cli.dataset.is_some()
            || !cli.geoparquet.is_empty()
            || cli.mode.is_some()
            || cli.max_snapshot_age_seconds.is_some()
        {
            eprintln!("error: ingest reads APIs; use --output to choose its dataset destination");
            return ExitCode::FAILURE;
        }
        return run_ingest(db.as_ref(), args, cli.json, cli.quiet).await;
    }

    if matches!(&cli.command, Command::Cleanup(_) | Command::Compact(_)) {
        return run_maintenance(&cli.command, cli.json);
    }

    let mode = cli.mode.unwrap_or_else(|| {
        if cli.dataset.is_some() || !cli.geoparquet.is_empty() {
            SearchMode::Snapshot
        } else {
            SearchMode::Live
        }
    });
    if (mode == SearchMode::Auto && (cli.dataset.is_none() || !cli.geoparquet.is_empty()))
        || (mode == SearchMode::Snapshot && cli.dataset.is_none() && cli.geoparquet.is_empty())
        || (mode == SearchMode::Live && (cli.dataset.is_some() || !cli.geoparquet.is_empty()))
        || (mode != SearchMode::Auto && cli.max_snapshot_age_seconds.is_some())
    {
        eprintln!("error: auto requires --dataset; snapshot requires --dataset or --geoparquet; live takes neither; max-snapshot-age-seconds applies only to auto");
        return ExitCode::FAILURE;
    }

    let engine = if let Some(root) = &cli.dataset {
        #[cfg(not(feature = "geoparquet"))]
        {
            let _ = root;
            eprintln!("error: rebuild superstac-cli with --features geoparquet to search datasets");
            return ExitCode::FAILURE;
        }
        #[cfg(feature = "geoparquet")]
        match if mode == SearchMode::Auto {
            Ok(SuperSTACEngine::automatic(
                db,
                root.clone(),
                std::time::Duration::from_secs(cli.max_snapshot_age_seconds.unwrap_or(86400)),
            ))
        } else {
            SuperSTACEngine::from_dataset(db, root)
        } {
            Ok(engine) => engine,
            Err(e) => {
                eprintln!("error: failed to open dataset: {e}");
                return ExitCode::FAILURE;
            }
        }
    } else if cli.geoparquet.is_empty() {
        SuperSTACEngine::new(db)
    } else {
        #[cfg(not(feature = "geoparquet"))]
        {
            eprintln!(
                "error: rebuild superstac-cli with --features geoparquet to search snapshots"
            );
            return ExitCode::FAILURE;
        }
        #[cfg(feature = "geoparquet")]
        {
            let mut paths = std::collections::HashMap::new();
            for (id, path) in cli.geoparquet {
                if paths.insert(id.clone(), path).is_some() {
                    eprintln!("error: multiple snapshot paths for catalog '{id}'");
                    return ExitCode::FAILURE;
                }
            }
            match SuperSTACEngine::from_geoparquet(db, paths) {
                Ok(engine) => engine,
                Err(e) => {
                    eprintln!("error: failed to configure snapshots: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    };

    if let Err(e) = engine.start().await {
        eprintln!("error: engine failed to start: {}", e);
        return ExitCode::FAILURE;
    }

    let exit = match cli.command {
        Command::Search(args) => run_search(&engine, args, cli.json).await,
        Command::Collections(args) => run_collections(&engine, args, cli.json).await,
        Command::Cleanup(_) | Command::Compact(_) | Command::Ingest(_) => {
            unreachable!("ingestion is handled before engine startup")
        }
    };

    engine.shutdown().await;
    exit
}

#[cfg(feature = "geoparquet")]
async fn run_ingest(
    db: &dyn superstac_core::storages::factory::StorageBackend,
    args: &IngestArgs,
    json: bool,
    quiet: bool,
) -> ExitCode {
    use superstac_engine::{ingest_catalog, IngestOptions, IngestScope};
    let mut catalogs = Vec::new();
    for id in &args.catalogs {
        if catalogs
            .iter()
            .any(|c: &superstac_core::models::catalog::Catalog| &c.id == id)
        {
            eprintln!("error: duplicate catalog '{id}'");
            return ExitCode::FAILURE;
        }
        match db.get_catalog(id) {
            Ok(catalog) => catalogs.push(catalog.clone()),
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    let mut completed = Vec::new();
    for catalog in catalogs {
        let mut options = IngestOptions::new(
            &args.output,
            IngestScope {
                collections: args.collections.clone(),
                bbox: args.bbox,
                datetime: args.datetime.clone(),
            },
        );
        options.page_size = args.page_size;
        options.items_per_file = args.items_per_file;
        options.request_timeout = std::time::Duration::from_secs(args.timeout_seconds);
        options.resume = args.resume;
        options.name = args.name.clone();
        options.incremental_since = args.incremental_since.clone();
        options.max_dataset_bytes = if args.max_dataset_mib == 0 {
            None
        } else {
            match args.max_dataset_mib.checked_mul(1024 * 1024) {
                Some(bytes) => Some(bytes),
                None => {
                    eprintln!("error: storage budget is too large");
                    return ExitCode::FAILURE;
                }
            }
        };
        if !args.no_progress && !quiet {
            options.progress = Some(ingest_progress());
        }
        if !quiet {
            eprintln!("ingesting '{}' into {}", catalog.id, args.output.display());
        }
        match ingest_catalog(&catalog, options).await {
            Ok(snapshot) => {
                if !quiet {
                    eprintln!(
                        "completed '{}' scope '{}': {} items, {} pages, {} files",
                        catalog.id,
                        snapshot.name,
                        snapshot.items,
                        snapshot.pages,
                        snapshot.files.len()
                    );
                }
                completed.push(snapshot);
            }
            Err(e) => {
                eprintln!("error: {e}");
                if json {
                    print_json(&completed);
                }
                return ExitCode::FAILURE;
            }
        }
    }
    if json {
        print_json(&completed);
    }
    ExitCode::SUCCESS
}

fn run_maintenance(command: &Command, json: bool) -> ExitCode {
    #[cfg(feature = "geoparquet")]
    {
        let result = match command {
            Command::Cleanup(args) => superstac_engine::cleanup_dataset(&args.output, args.apply)
                .and_then(|report| {
                    serde_json::to_value(report).map_err(|e| {
                        superstac_core::errors::SuperSTACError::SearchFailed(e.to_string())
                    })
                }),
            Command::Compact(args) => {
                let budget = if args.max_dataset_mib == 0 {
                    None
                } else {
                    match args.max_dataset_mib.checked_mul(1048576) {
                        Some(n) => Some(n),
                        None => {
                            eprintln!("error: budget too large");
                            return ExitCode::FAILURE;
                        }
                    }
                };
                superstac_engine::compact_dataset(&args.output, args.items_per_file, budget)
                    .and_then(|manifest| {
                        serde_json::to_value(manifest).map_err(|e| {
                            superstac_core::errors::SuperSTACError::SearchFailed(e.to_string())
                        })
                    })
            }
            _ => unreachable!(),
        };
        match result {
            Ok(value) => {
                if json {
                    print_json(&value);
                } else {
                    println!("{}", serde_json::to_string_pretty(&value).unwrap());
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "geoparquet"))]
    {
        let _ = (command, json);
        eprintln!("error: rebuild with --features geoparquet");
        ExitCode::FAILURE
    }
}

#[cfg(not(feature = "geoparquet"))]
async fn run_ingest(
    _db: &dyn superstac_core::storages::factory::StorageBackend,
    _args: &IngestArgs,
    _json: bool,
    _quiet: bool,
) -> ExitCode {
    eprintln!("error: rebuild superstac-cli with --features geoparquet to ingest snapshots");
    ExitCode::FAILURE
}

#[cfg(feature = "geoparquet")]
fn ingest_progress() -> superstac_engine::ProgressCallback {
    use std::{
        io::{IsTerminal, Write},
        sync::{Arc, Mutex},
        time::{Duration, Instant},
    };
    use superstac_engine::IngestPhase;
    let terminal = std::io::stderr().is_terminal();
    let last = Mutex::new(None::<Instant>);
    Arc::new(move |event| {
        let mut last = last.lock().unwrap_or_else(|e| e.into_inner());
        let final_event = matches!(event.phase, IngestPhase::Completed | IngestPhase::Failed);
        let interval = if terminal {
            Duration::from_millis(250)
        } else {
            Duration::from_secs(2)
        };
        if !final_event
            && event.phase != IngestPhase::Retrying
            && last.is_some_and(|t| t.elapsed() < interval)
        {
            return;
        }
        *last = Some(Instant::now());
        let estimate = event
            .total_items
            .filter(|total| *total > 0 && *total >= event.items_received)
            .map(|total| {
                let percent = event.items_received as f64 * 100.0 / total as f64;
                let eta = if event.items_per_second > 0.0 {
                    format!(
                        ", ETA ~{:.0}s",
                        (total - event.items_received) as f64 / event.items_per_second
                    )
                } else {
                    String::new()
                };
                format!(" / ~{total} ({percent:.1}%{eta})")
            })
            .unwrap_or_default();
        let line = format!("{} {:?}: {} received{}, {} saved | {} pages, {} files, {:.1} MiB saved | {:.0}s, {:.0} items/s{}",
            event.catalog_id, event.phase, event.items_received, estimate, event.items_saved,
            event.pages, event.files, event.bytes_saved as f64 / 1048576.0,
            event.elapsed_seconds, event.items_per_second,
            event.message.as_ref().map(|m| format!(" | {m}")).unwrap_or_default());
        let mut stderr = std::io::stderr().lock();
        if terminal {
            let _ = write!(stderr, "\r\x1b[K{line}");
            if final_event {
                let _ = writeln!(stderr);
            }
        } else {
            let _ = writeln!(stderr, "{line}");
        }
        let _ = stderr.flush();
    })
}

fn parse_snapshot(value: &str) -> Result<(String, PathBuf), String> {
    match value.split_once('=') {
        Some((id, path)) if !id.is_empty() && !path.is_empty() => {
            Ok((id.to_owned(), PathBuf::from(path)))
        }
        _ => Err("expected CATALOG=PATH, e.g. earth-search=./data/items.parquet".into()),
    }
}

fn parse_bbox(s: &str) -> Result<Bbox, String> {
    let parts: Vec<f64> = s
        .split(',')
        .map(|p| p.trim().parse::<f64>().map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;

    match parts.as_slice() {
        [w, s, e, n] => Ok(Bbox::new(*w, *s, *e, *n)),
        _ => Err(format!(
            "expected 4 comma-separated floats, got {}",
            parts.len()
        )),
    }
}

fn init_tracing(settings: &Settings, verbose: bool, quiet: bool) {
    if !settings.logging_enabled {
        return;
    }

    // CLI flags override settings; RUST_LOG overrides both.
    let default_level = if verbose {
        "debug"
    } else if quiet {
        "warn"
    } else {
        match settings.log_level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warn",
        }
    };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

async fn run_collections(engine: &SuperSTACEngine, args: CollectionsArgs, json: bool) -> ExitCode {
    match (args.catalog, args.collection) {
        (Some(catalog_id), Some(collection_id)) => {
            describe_collection(engine, &catalog_id, &collection_id, json).await
        }
        (Some(catalog_id), None) => list_one_catalog(engine, &catalog_id, json).await,
        (None, _) => list_all_collections(engine, json).await,
    }
}

async fn list_all_collections(engine: &SuperSTACEngine, json: bool) -> ExitCode {
    match engine.list_collections().await {
        Ok(items) => {
            if json {
                let view: Vec<_> = items
                    .iter()
                    .map(|c| serde_json::json!({ "id": c.id, "catalogs": c.catalogs }))
                    .collect();
                print_json(&view);
            } else {
                for ca in items {
                    println!("{}\t{}", ca.id, ca.catalogs.join(", "));
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: failed to list collections: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn list_one_catalog(engine: &SuperSTACEngine, catalog_id: &str, json: bool) -> ExitCode {
    match engine.collections_by_catalog().await {
        Ok(map) => match map.get(catalog_id) {
            Some(collections) => {
                if json {
                    print_json(collections);
                } else {
                    for c in collections {
                        println!("{}", c);
                    }
                }
                ExitCode::SUCCESS
            }
            None => {
                eprintln!(
                    "error: no introspection data for '{}' (unknown or not yet introspected)",
                    catalog_id
                );
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("error: failed to fetch collections: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn describe_collection(
    engine: &SuperSTACEngine,
    catalog_id: &str,
    collection_id: &str,
    json: bool,
) -> ExitCode {
    match engine.describe_collection(catalog_id, collection_id).await {
        Ok(Some(c)) => {
            if json {
                print_json(&c);
            } else {
                println!("{} — {}", c.id, c.title.as_deref().unwrap_or("(no title)"));
                println!("catalog: {}", catalog_id);
                println!();
                match serde_json::to_string_pretty(&c) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        eprintln!("error: failed to serialize collection: {}", e);
                        return ExitCode::FAILURE;
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Ok(None) => {
            eprintln!(
                "error: collection '{}' not found in catalog '{}'",
                collection_id, catalog_id
            );
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("error: failed to describe collection: {}", e);
            ExitCode::FAILURE
        }
    }
}

async fn run_search(engine: &SuperSTACEngine, args: SearchArgs, json: bool) -> ExitCode {
    let query = SearchQuery {
        collections: args.collections,
        ids: if args.ids.is_empty() {
            None
        } else {
            Some(args.ids)
        },
        intersects: None,
        bbox: args.bbox,
        datetime: args.datetime,
        limit: Some(args.limit),
        sortby: (!args.sortby.is_empty()).then_some(args.sortby),
    };

    let response = match engine.search(query).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: search failed: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let exit =
        if response.metadata.catalogs_queried > 0 && response.metadata.catalogs_succeeded == 0 {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    if json {
        print_json(&response);
        return exit;
    }

    let m = &response.metadata;
    println!(
        "found {} items ({}/{} catalogs)",
        m.total_items, m.catalogs_succeeded, m.catalogs_queried
    );

    if m.duplicates_removed > 0 {
        println!("({} duplicates removed)", m.duplicates_removed);
    }

    for f in &m.failures {
        println!("  ! {}: {}", f.catalog_id, f.reason);
    }

    if !m.unsupported_collections.is_empty() {
        println!(
            "  ? no catalog serves: {}",
            m.unsupported_collections.join(", ")
        );
    }

    if !response.items.is_empty() {
        println!();
        for item in response.items {
            println!("{}\t{}", item.item.id, item.seen_in.join(", "));
        }
    }

    exit
}

fn print_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("error: failed to serialize as JSON: {}", e),
    }
}
