use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use stac::Bbox;
use superstac_config::init_from_yaml;
use superstac_core::models::{settings::LogLevel, settings::Settings, storage::Storage};
use superstac_engine::SuperSTACEngine;
use superstac_search::query::SearchQuery;
use tracing_subscriber::EnvFilter;

/// Federated STAC search across multiple catalogs.
#[derive(Parser)]
#[command(name = "superstac", version, about, long_about = None)]
struct Cli {
    /// Path to the superstac config file.
    #[arg(long, default_value = "superstac.yml", global = true)]
    config: PathBuf,

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

#[derive(Subcommand)]
enum Command {
    /// Federated search across catalogs.
    Search(SearchArgs),

    /// Inspect available collections.
    Collections(CollectionsArgs),
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

    let engine = SuperSTACEngine::new(db);

    if let Err(e) = engine.start().await {
        eprintln!("error: engine failed to start: {}", e);
        return ExitCode::FAILURE;
    }

    let exit = match cli.command {
        Command::Search(args) => run_search(&engine, args, cli.json).await,
        Command::Collections(args) => run_collections(&engine, args, cli.json).await,
    };

    engine.shutdown().await;
    exit
}

fn parse_bbox(s: &str) -> Result<Bbox, String> {
    let parts: Vec<f64> = s
        .split(',')
        .map(|p| p.trim().parse::<f64>().map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;

    match parts.as_slice() {
        [w, s, e, n] => Ok(Bbox::new(*w, *s, *e, *n)),
        _ => Err(format!("expected 4 comma-separated floats, got {}", parts.len())),
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

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

async fn run_collections(
    engine: &SuperSTACEngine,
    args: CollectionsArgs,
    json: bool,
) -> ExitCode {
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
                println!(
                    "{} — {}",
                    c.id,
                    c.title.as_deref().unwrap_or("(no title)")
                );
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
        ids: if args.ids.is_empty() { None } else { Some(args.ids) },
        intersects: None,
        bbox: args.bbox,
        datetime: args.datetime,
        limit: Some(args.limit),
        sortby: None,
    };

    let response = match engine.search(query).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: search failed: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if json {
        print_json(&response);
        return ExitCode::SUCCESS;
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

    ExitCode::SUCCESS
}

fn print_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("error: failed to serialize as JSON: {}", e),
    }
}
