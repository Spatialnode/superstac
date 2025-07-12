"""SuperSTAC Catalog Registry"""

from superstac.catalog import CatalogManager


_catalog_registry = None


def get_catalog_registry() -> CatalogManager:
    """
    Returns the singleton CatalogManager instance.
    """
    global _catalog_registry
    if _catalog_registry is None:
        _catalog_registry = CatalogManager()
    return _catalog_registry


def clear_registry() -> None:
    """
    Reset the registry, mainly for testing.
    """
    if _catalog_registry:
        _catalog_registry.catalogs.clear()
    return None
