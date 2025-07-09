"""SuperSTAC Models"""

from dataclasses import asdict, dataclass
import json
from typing import Optional
from urllib.parse import urlparse

from superstac.enums import AuthType


# Todo - make auth type have a default ?
@dataclass
class AuthInfo:
    """AuthInfo"""

    type: AuthType
    token: Optional[str] = None
    username: Optional[str] = None
    password: Optional[str] = None
    header_key: Optional[str] = None

    def as_dict(self):
        return asdict(self)

    def as_json(self):
        return json.dumps(self.as_dict(), indent=2)


@dataclass
class CatalogEntry:
    """CatalogEntry"""

    name: str
    url: str
    """URL to the STAC catalog."""
    is_private: Optional[bool] = False
    """Indicates whether the catalog is a private catalog or not."""
    summary: Optional[str] = None
    """Short description of the catalog."""
    auth: Optional[AuthInfo] = None
    """Authentication parameters."""
    is_available: Optional[bool] = True
    """Defaults to True on instantiation. It will be updated based on the status code of the catalog. If 200 True and False if otherwise."""

    def __post_init__(self):
        # Validate the URL
        parsed_url = urlparse(self.url)
        if not all([parsed_url.scheme, parsed_url.netloc]):
            raise ValueError(
                f"Invalid URL: '{self.url}' - Must have a scheme (e.g., http/https) and network location."
            )

        if parsed_url.scheme not in ("http", "https"):
            raise ValueError(
                f"Invalid URL scheme: '{parsed_url.scheme}' for '{self.url}' - Only http and https are allowed."
            )

    def as_dict(self):
        return asdict(self)

    def as_json(self):
        return json.dumps(self.as_dict(), indent=2)
