"""High performance federated STAC search."""

from ._superstac import (
    AsyncClient,
    Client,
    Search,
    __version__,
)

__all__ = [
    "Client",
    "AsyncClient",
    "Search",
    "__version__",
]