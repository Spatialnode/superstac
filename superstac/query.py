"""SuperSTAC Query Manager"""

from typing import List, Optional, Dict, Any
from pystac_client import Client
from pystac import Item
from superstac._logging import logger
from superstac.utils import BAND_MAP


# todo - support auth and headers for client.open() in query_catalog_with_pystac param.
# test resolving band alias


def query_catalog_with_pystac(
    name: str,
    url: str,
    required_assets: Optional[List[str]] = None,
    max_items: int = 100,
    **search_kwargs: Any,
) -> List[Dict[str, Any]]:
    """
    Search a single catalog using pystac-client and normalize the assets.

    Args:
        name: Unique catalog name.
        url: STAC API URL.
        required_assets: Band aliases to filter and rename assets.
        max_items: Maximum number of results.
        **search_kwargs: All valid parameters supported by pystac-client's search().

    Returns:
        List of STAC items (as dicts), normalized with filtered assets and catalog_name.
    """
    try:
        client = Client.open(url)
        logger.debug(f"Querying STAC with: {search_kwargs.items()}")
        search = client.search(max_items=max_items, **search_kwargs)
        items: List[Item] = list(search.items())

        collection_id = None
        collections = search_kwargs.get("collections")
        if isinstance(collections, list) and collections:
            collection_id = collections[0]
        elif isinstance(collections, str):
            collection_id = collections

        band_map = BAND_MAP.get(collection_id, {})
        normalized = []

        for item in items:
            item_assets = item.assets
            normalized_assets = {}

            for alias, actual in band_map.items():
                if not required_assets or alias in required_assets:
                    if actual in item_assets:
                        normalized_assets[alias] = item_assets[actual].to_dict()

            if normalized_assets:
                item_dict = item.to_dict()
                item_dict["assets"] = normalized_assets
                item_dict["catalog_name"] = name
                normalized.append(item_dict)

        logger.info(
            f"[{name}] Returned {len(normalized)} items for collection '{collection_id}'"
        )
        return normalized

    except Exception as e:
        logger.warning(f"[{name}] Catalog query failed: {e}")
        return []


if __name__ == "__main__":
    results = query_catalog_with_pystac(
        name="aws",
        url="https://earth-search.aws.element84.com/v1",
        collections=["sentinel-2-l2a"],
        bbox=[6.0, 49.0, 7.0, 50.0],
        datetime="2024-01-01/2024-01-31",
        query={"eo:cloud_cover": {"lt": 20}},
        required_assets=["red", "nir"],
        sortby=[{"field": "properties.datetime", "direction": "desc"}],
    )
    print(results)
