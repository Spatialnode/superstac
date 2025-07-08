"""SuperSTAC Models."""

from __future__ import annotations

from typing import Optional
from urllib.parse import urlparse

import attr

from superstac.enums import AuthType


@attr.s(auto_attribs=True, kw_only=True)
class AuthInfo:
    type: AuthType
    token: Optional[str] | None = None
    username: Optional[str] = None
    password: Optional[str] | None = None
    header_key: Optional[str] | None = None


@attr.s(auto_attribs=True, kw_only=True)
class CatalogEntry:
    """_summary_.

    Raises:
        ValueError: _description_

    """

    name: str
    url: str = attr.ib(validator=attr.validators.instance_of(str))
    auth: Optional[AuthInfo] = None
    is_available: bool = False
    latency_ms: Optional[float] = None
    conforms_to: Optional[list[str]] = None
    collections: Optional[list[str]] = None
    extensions: Optional[list[str]] = None

    @url.validator
    def _validate_url(self, _, value) -> None:
        result = urlparse(value)
        if not result.scheme.startswith("http"):
            msg = "Invalid URL: must start with http or https"
            raise ValueError(msg)
