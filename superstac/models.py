from dataclasses import dataclass, field, asdict
from typing import Optional, List


@dataclass
class AuthInfo:
    type: str
    token: Optional[str] = None
    username: Optional[str] = None
    password: Optional[str] = None
    header_key: Optional[str] = None


@dataclass
class CatalogEntry:
    name: str
    url: str
    auth: Optional[AuthInfo] = None
    is_available: bool = False
    latency_ms: Optional[float] = None
    conforms_to: Optional[List[str]] = field(default_factory=list)
    collections: Optional[List[str]] = field(default_factory=list)
    extensions: Optional[List[str]] = field(default_factory=list)
