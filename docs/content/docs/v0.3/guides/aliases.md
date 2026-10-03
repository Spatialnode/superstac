---
title: Collection and asset aliases
description: Use consistent names when catalogs expose different collection IDs or band keys.
---

Catalogs can use different names for the same collection or band. Set aliases on each catalog so your queries and results use the names you choose.

## Try it with Earth Search

This example calls the collection `optical` and the `visual` asset `rgb`. It searches using your names and prints the renamed asset URL when available. Aliases rename metadata keys; they do not create or transform imagery.

```python
from superstac import Client

client = Client(catalogs=[{
    "id": "earth-search",
    "url": "https://earth-search.aws.element84.com/v1",
    "collection_aliases": {"optical": "sentinel-2-l2a"},
    "asset_aliases": {"optical": {"rgb": "visual"}},
}])
try:
    result = client.search(
        collections=["optical"],
        bbox=[6, 45, 7, 46],
        datetime="2024-06-01T00:00:00Z/2024-06-30T23:59:59Z",
        limit=3,
    )
    for item in result.items():
        print(item["collection"], item.get("assets", {}).get("rgb", {}).get("href"))
    print(result.metadata["failures"])
finally:
    client.shutdown()
```

In a browser app, the same mappings go on the catalog object:

```js
import init, { SuperSTAC } from '@spatialnode/superstac-wasm';

await init();
const client = new SuperSTAC({
  catalogs: [{
    id: 'earth-search',
    url: 'https://earth-search.aws.element84.com/v1',
    collection_aliases: { optical: 'sentinel-2-l2a' },
    asset_aliases: { optical: { rgb: 'visual' } },
  }],
});
try {
  const result = await client.search({
    collections: ['optical'],
    bbox: [6, 45, 7, 46],
    datetime: '2024-06-01T00:00:00Z/2024-06-30T23:59:59Z',
    limit: 3,
  });
  for (const { item } of result.items) {
    console.log(item.collection, item.assets?.rgb?.href);
  }
  console.warn(result.metadata.failures);
} finally {
  client.free();
}
```

For the CLI or Rust, add these mappings to the Earth Search entry in the [sample YAML](/superstac/versions/v0.3/examples/superstac.yml), then search for `optical`:

```yaml
collection_aliases:
  optical: sentinel-2-l2a
asset_aliases:
  optical:
    rgb: visual
```

```bash
superstac search --collection optical --bbox=6,45,7,46 --limit 3
```

## Collection aliases

Put the name you want to use on the left and the catalog’s actual name on the right: **your name → catalog name**. For a catalog whose collection is called `S2MSI2A`:

```yaml
# Fragment of one catalog entry, not a complete config file.
collection_aliases:
  sentinel-2-l2a: S2MSI2A
```

Search for `sentinel-2-l2a` in your application. SuperSTAC translates the outgoing query to `S2MSI2A` for that catalog. When response unification is enabled, the returned item's `collection` is changed back to the canonical name.

Check the catalog’s collection IDs before adding an alias. `S2MSI2A` here is an example; use the ID your catalog advertises.

## Asset aliases

Group asset aliases under your collection name, called the **canonical collection ID**. Put your band name on the left and the catalog’s asset key on the right:

```yaml
# Fragment of one catalog entry.
asset_aliases:
  sentinel-2-l2a:
    blue: B02
    green: B03
    red: B04
    nir: B08
```

With unification enabled, `B04` appears as `red` in the returned item. Its URL and data stay the same. Check that the bands you give the same name are suitable for your analysis; an alias does not change resolution, calibration, or access requirements.

## Keep the catalog’s names in results

```python
from superstac import Client

client = Client(settings={"unify_response": False})
```

Results keep the catalog’s collection and asset names. You can still search with your collection aliases: SuperSTAC translates them when sending the query. Names without an alias stay unchanged.

See the [full configuration reference](/docs/v0.3/reference/configuration/) for where these fragments belong.
