"""superSTAC Search Module"""

import asyncio
from typing import Any, List, Optional

from pystac import Item
from superstac.catalog import CatalogManager
from superstac.catalog_registry import get_catalog_registry
from superstac.query import (
    query_catalog_with_pystac,
    query_catalog_with_pystac_async_wrapper,
)
from superstac._logging import logger


def federated_search(
    collections: List[str],
    registry: Optional[CatalogManager] = None,
    catalog_names: Optional[List[str]] = None,
    **kwargs: Any,
) -> List[Item]:
    """
    Search one or more catalogs for matching STAC items.

    Args:
        bbox (List[float]): Bounding box [minx, miny, maxx, maxy].
        datetime (str): Datetime string or range.
        collections (List[str]): List of collection IDs to search.
        required_bands (Optional[List[str]]): List of required asset keys.
        registry (Optional[CatalogManager]): CatalogManager instance.
        catalog_names (Optional[List[str]]): Names of catalogs to search. If None, search all available.
        max_items (int): Max items per query.

    Returns:
        List[Dict[str, Any]]: List of matching STAC items.
    """
    cr = registry or get_catalog_registry()

    results = []

    if catalog_names is None:
        search_catalogs = {
            name: entry
            for name, entry in cr.catalogs.items()
            if entry.is_available and entry.catalog
        }
    else:

        search_catalogs = {}
        for name in catalog_names:
            entry = cr.catalogs.get(name)
            if entry and entry.is_available and entry.catalog:
                search_catalogs[name] = entry
            else:
                logger.warning(f"Catalog '{name}' not found or unavailable.")

    for catalog_name, catalog_entry in search_catalogs.items():

        try:
            catalog_collections = [
                c.id for c in catalog_entry.catalog.get_all_collections()  # type: ignore - search catalogs are already confirmed to be available and have the catalog object above.
            ]
        except Exception as e:
            logger.warning(f"Failed to get collections for catalog {catalog_name}: {e}")
            continue

        for coll in collections:
            if coll in catalog_collections:
                logger.info(
                    f"Searching collection '{coll}' in catalog '{catalog_name}'"
                )
                try:
                    items = query_catalog_with_pystac(
                        catalog=catalog_entry,
                        collection=coll,
                        **kwargs,
                    )
                    results.extend(items)
                except Exception as e:
                    logger.error(
                        f"Error querying catalog '{catalog_name}', collection '{coll}': {e}"
                    )
            else:
                logger.debug(
                    f"Collection '{coll}' not found in catalog '{catalog_name}'"
                )

    logger.info(f"Federated search completed with {len(results)} total items found.")
    return results


async def federated_search_async(
    collections: List[str],
    registry: Optional[CatalogManager] = None,
    catalog_names: Optional[List[str]] = None,
    **kwargs: Any,
) -> List[Item]:
    """
    Async search one or more catalogs for matching STAC items.

    Args:
        collections (List[str]): List of collection IDs to search.
        registry (Optional[CatalogManager]): CatalogManager instance.
        catalog_names (Optional[List[str]]): Names of catalogs to search. If None, search all available.
        **kwargs: Additional keyword arguments forwarded to query_catalog_with_pystac.

    Returns:
        List[Dict[str, Any]]: List of matching STAC items.
    """
    cr = registry or get_catalog_registry()

    if catalog_names is None:
        search_catalogs = {
            name: entry
            for name, entry in cr.catalogs.items()
            if entry.is_available and entry.catalog
        }
    else:
        search_catalogs = {}
        for name in catalog_names:
            entry = cr.catalogs.get(name)
            if entry and entry.is_available and entry.catalog:
                search_catalogs[name] = entry
            else:
                logger.warning(f"Catalog '{name}' not found or unavailable.")

    async def query_catalog(catalog_entry) -> List[Item]:
        try:
            catalog_collections = [
                c.id for c in catalog_entry.catalog.get_all_collections()  # type: ignore
            ]
        except Exception as e:
            logger.warning(
                f"Failed to get collections for catalog {catalog_entry.name}: {e}"
            )
            return []

        matched_collections = [
            coll for coll in collections if coll in catalog_collections
        ]

        if not matched_collections:
            logger.debug(
                f"No matching collections found in catalog '{catalog_entry.name}' "
                f"for requested collections."
            )
            return []

        logger.info(
            f"Searching collections {matched_collections} in catalog '{catalog_entry.name}'"
        )

        try:
            items = await query_catalog_with_pystac_async_wrapper(
                catalog=catalog_entry,
                collection=(
                    matched_collections
                    if len(matched_collections) > 1
                    else matched_collections[0]
                ),
                **kwargs,
            )
            return items
        except Exception as e:
            logger.error(f"Error querying catalog '{catalog_entry.name}': {e}")
            return []

    tasks = [query_catalog(entry) for _, entry in search_catalogs.items()]
    results_lists = await asyncio.gather(*tasks)

    results = [item for sublist in results_lists for item in sublist]

    logger.info(f"Federated search completed with {len(results)} total items found.")
    return results
