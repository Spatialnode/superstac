"""SuperSTAC Catalog Registry"""

from typing import Union
from superstac.catalog import CatalogManager


_catalog_registry = CatalogManager()


def get_catalog_registry() -> CatalogManager:
    """
    Returns the singleton CatalogManager instance.
    """
    return _catalog_registry


def register_catalog(*args, **kwargs):
    """
    Shortcut to register a catalog globally.
    """
    return _catalog_registry.register_catalog(*args, **kwargs)


def load_catalogs_from_config(file: Union[str, None] = None):
    """
    Loads catalogs from YAML into the global registry.
    """
    return _catalog_registry.load_catalogs_from_config(file)


def clear_registry():
    """
    Optional: Reset the registry, mainly for testing.
    """
    _catalog_registry.catalogs.clear()
