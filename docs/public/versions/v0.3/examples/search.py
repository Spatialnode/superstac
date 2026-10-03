import json
from pathlib import Path
from superstac import Client

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
        bbox=[6, 45, 7, 46],
        datetime="2024-06-01T00:00:00Z/2024-06-30T23:59:59Z",
        limit=5,
    )
    for item in result.items():
        print(item["id"], item["properties"].get("datetime"))
    for failure in result.metadata["failures"]:
        print("Catalog failed:", failure["catalog_id"], failure["reason"])
    Path("items.geojson").write_text(
        json.dumps(result.to_geojson(), indent=2), encoding="utf-8"
    )
    print(f"Saved {len(result)} scene records to items.geojson")
finally:
    client.shutdown()
