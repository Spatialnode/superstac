"""Many catalogs. One search. Search across STAC catalogs with Python."""

from ._superstac import (
    AsyncClient,
    Client,
    Search,
    __version__,
    geoparquet_available,
)

__all__ = [
    "Client",
    "AsyncClient",
    "Search",
    "__version__",
    "geoparquet_available",
]