"""SuperSTAC Query Manager"""

from typing import Any, List
from pystac import Item
from superstac._logging import logger
import asyncio

from superstac.models import CatalogEntry


def query_catalog_with_pystac(
    catalog: CatalogEntry,
    collection: str,
    **search_kwargs: Any,
) -> List[Item]:
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
        client = catalog.client
        logger.debug(f"Querying STAC with: {search_kwargs.items()}")
        search = client.search(collections=[collection], **search_kwargs)  # type: ignore
        items: List[Item] = list(search.items())
        collection_id = collection
        logger.info(
            f"[{catalog.name}] Returned {len(items)} items for collection '{collection_id}'"
        )
        return items

    except Exception as e:
        logger.warning(f"[{catalog.name}] Catalog query failed: {e}")
        return []


async def query_catalog_with_pystac_async_wrapper(*args, **kwargs) -> List[Item]:
    """
    Async wrapper for the synchronous `query_catalog_with_pystac` function,
    offloading it to a thread to avoid blocking the event loop.
    """
    results = await asyncio.to_thread(query_catalog_with_pystac, *args, **kwargs)
    return results
