"""SuperSTAC Catalog Manager"""

from pathlib import Path
import attr
from typing import Any, Dict, Optional, Union

from superstac.enums import CatalogOutputFormat
from superstac.exceptions import (
    CatalogConfigFileNotFound,
    InvalidCatalogSchemaError,
    InvalidCatalogYAMLError,
)
from superstac.models import CatalogEntry, AuthInfo
import yaml


@attr.s(auto_attribs=True)
class CatalogManager:
    catalogs: Dict[str, CatalogEntry] = attr.Factory(dict)

    def register_catalog(
        self,
        name: str,
        url: str,
        is_private: Optional[bool] = False,
        summary: Optional[str] = None,
        auth: Optional[AuthInfo] = None,
    ) -> CatalogEntry:
        """Register a single STAC catalog in state.

        Args:
            name (str): The name of the catalog.
            url (str): A valid URL to the catalog.
            is_private (Optional[bool], optional): Indicates if the catalog requires authentication or not. Defaults to False.
            summary (Optional[str], optional): A short description of the catalog. Defaults to None.
            auth (Optional[AuthInfo], optional): Authentication parameters for the catalog. Defaults to None.

        Raises:
            InvalidCatalogSchemaError: If an invalid parameter is encountered.

        Returns:
            CatalogEntry: The registered STAC catalog.
        """
        if is_private and auth is None:
            raise InvalidCatalogSchemaError(
                f"Authentication parameters is required for private catalogs. If this is a mistake, you can set 'is_private' to False or provide the {AuthInfo.__annotations__} parameters."
            )
        self.catalogs[name] = CatalogEntry(
            name=name,
            url=url,
            summary=summary,
            is_private=is_private,
            auth=AuthInfo(**auth.__dict__) if auth and not is_private else None,
        )
        return self.catalogs[name]

    def get_available_catalogs(
        self, format: Union[str, CatalogOutputFormat] = CatalogOutputFormat.DICT
    ) -> list[Union[dict[str, Any], str]]:
        """Get the available STAC catalogs.

        Raises:
            ValueError: When an invalid format is provided.

        Returns:
            list[CatalogEntry]: The list of all available STAC catalogs.
        """
        if isinstance(format, str):
            try:
                format = CatalogOutputFormat(format.lower())
            except ValueError:
                raise ValueError(f"Invalid format: {format}")

        return [
            c.as_dict() if format == CatalogOutputFormat.DICT else c.as_json()
            for c in self.catalogs.values()
            if c.is_available
        ]

    def load_catalogs_from_config(
        self, file: Union[str, Path, None] = None
    ) -> Dict[str, CatalogEntry]:
        if file is None:
            base_dir = Path(__file__).parent
            file = base_dir / ".superstac.yml"

        path = Path(file).expanduser().resolve()

        if not path.exists():
            raise CatalogConfigFileNotFound(f"Config file not found at {path}")

        try:
            with open(path, "r") as f:
                data = yaml.safe_load(f) or {}
        except yaml.YAMLError as e:
            raise InvalidCatalogYAMLError(f"YAML parsing failed: {e}") from e
        except Exception as e:
            raise InvalidCatalogYAMLError(
                f"Unexpected error reading config: {e}"
            ) from e

        catalogs = data.get("catalogs")
        if not isinstance(catalogs, dict):
            raise InvalidCatalogSchemaError(
                f"Missing or invalid 'catalogs' section in config file: {path}"
            )

        # Register each catalog
        for name, spec in catalogs.items():
            self.register_catalog(
                name=name,
                url=spec.get("url"),
                is_private=spec.get("is_private", False),
                summary=spec.get("summary"),
                auth=AuthInfo(**spec["auth"]) if "auth" in spec else None,
            )

        return self.catalogs


## TEST


if __name__ == "__main__":
    # cm = CatalogManager()
    # cm.register_catalog(name="My Catalog", url="https://example.com/stac")
    # print(cm.load_catalogs_from_config())
    # print(cm.get_available_catalogs())
    ...
