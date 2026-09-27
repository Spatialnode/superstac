---
title: Collection and asset aliases
description: Use consistent names when catalogs expose different collection IDs or band keys.
---

Aliases let your application speak one vocabulary while each catalog keeps its local names. They are configured per catalog.

## Collection aliases

The direction is **canonical name → catalog-local name**. For example, a catalog using `S2MSI2A` locally could be configured with:

```yaml
# Fragment of one catalog entry, not a complete config file.
collection_aliases:
  sentinel-2-l2a: S2MSI2A
```

Search for `sentinel-2-l2a` in your application. SuperSTAC translates the outgoing query to `S2MSI2A` for that catalog. When response unification is enabled, the returned item's `collection` is changed back to the canonical name.

Use aliases only after confirming the actual collection IDs advertised by your endpoint. This mapping is illustrative, not a promise about any provider's current schema.

## Asset aliases

Asset mappings are nested under the **canonical collection ID**, and follow the same direction:

```yaml
# Fragment of one catalog entry.
asset_aliases:
  sentinel-2-l2a:
    blue: B02
    green: B03
    red: B04
    nir: B08
```

With unification enabled, an asset stored under `B04` becomes `red` in the returned item. Asset URLs are not downloaded, authenticated, or reprojected by this operation. Aliases rename keys; they do not make datasets scientifically equivalent.

## Disable response normalization

```python
from superstac import Client

client = Client(settings={"unify_response": False})
```

This preserves provider-local collection and asset names in results. Outgoing collection-name translation still occurs. Missing alias entries fall back to their original names.

See the [full configuration reference](/docs/reference/configuration/) for where these fragments belong.
