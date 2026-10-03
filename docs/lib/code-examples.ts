export const examples = {
  Python: `from superstac import Client

catalogs = [
    {
        "id": "earth-search",
        "url": "https://earth-search.aws.element84.com/v1",
    },
    {
        "id": "microsoft",
        "url": "https://planetarycomputer.microsoft.com/api/stac/v1",
    },
]
client = Client(catalogs=catalogs)

try:
    result = client.search(
        collections=["sentinel-2-l2a"],
        limit=5,
    )
    for item in result.items():
        print(item["id"])
finally:
    client.shutdown()`,
  JavaScript: `import init, { SuperSTAC } from '@spatialnode/superstac-wasm';

await init();
const client = new SuperSTAC({
  catalogs: [
    {
      id: 'earth-search',
      url: 'https://earth-search.aws.element84.com/v1',
    },
    {
      id: 'microsoft',
      url: 'https://planetarycomputer.microsoft.com/api/stac/v1',
    },
  ],
});

try {
  const result = await client.search({
    collections: ['sentinel-2-l2a'],
    limit: 5,
  });
  for (const { item, seen_in } of result.items) {
    console.log(item.id, seen_in);
  }
  console.log(result.metadata.failures);
} finally {
  client.free();
}`,
  Rust: `use superstac_config::init_from_yaml;
use superstac_core::models::storage::Storage;
use superstac_engine::SuperSTACEngine;
use superstac_search::query::SearchQuery;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = init_from_yaml(
        Storage::Memory,
        "superstac.yml",
    )?;
    let engine = SuperSTACEngine::new(config);
    let query = SearchQuery {
        collections: vec!["sentinel-2-l2a".into()],
        limit: Some(5),
        ids: None,
        bbox: None,
        intersects: None,
        datetime: None,
        sortby: None,
    };

    let result = engine.search(query).await;
    engine.shutdown().await;
    println!("{} items", result?.metadata.total_items);
    Ok(())
}`,
  CLI: String.raw`# Save the quickstart config as superstac.yml.
superstac collections

# Search an area.
superstac search \
  -c sentinel-2-l2a \
  -b 6.0,49.0,7.0,50.0 \
  -l 5

# Return the results as JSON.
superstac --json search \
  -c sentinel-2-l2a \
  -l 5`,
};
