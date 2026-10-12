import json
from pathlib import Path
from superstac import Client

area = {
    "type": "Polygon",
    "coordinates": [[[6, 45], [7, 45], [7, 46], [6, 46], [6, 45]]],
}
client = Client.open("https://earth-search.aws.element84.com/v1")
try:
    result = client.search(
        collections=["sentinel-2-l2a"],
        intersects=area,
        datetime="2024-06-01T00:00:00Z/2024-06-30T23:59:59Z",
        sortby=["-datetime"],
        limit=20,
    )
    # Filter only the records returned by this search.
    clearer = [
        item for item in result.items()
        if isinstance(item["properties"].get("eo:cloud_cover"), (int, float))
        and item["properties"]["eo:cloud_cover"] <= 20
    ]
    Path("clearer-scenes.geojson").write_text(json.dumps({
        "type": "FeatureCollection",
        "features": clearer,
    }, indent=2), encoding="utf-8")
    print(f"Kept {len(clearer)} of {len(result)} returned records")
    for failure in result.metadata["failures"]:
        print("Catalog failed:", failure)

    # Look up a real ID from the results instead of guessing one.
    if clearer:
        item = clearer[0]
        lookup = client.search(
            collections=[item["collection"]], ids=[item["id"]], limit=1
        )
        for found in lookup.items():
            for name, asset in found.get("assets", {}).items():
                print(name, asset.get("type"), asset.get("href"))
        print("Lookup failures:", lookup.metadata["failures"])
finally:
    client.shutdown()
