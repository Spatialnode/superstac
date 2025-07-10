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

from superstac._logging import logger


@attr.s(auto_attribs=True)
class CatalogManager:

    catalogs: Dict[str, CatalogEntry] = attr.Factory(dict)

    def __attrs_post_init__(self):
        logger.info("Initialized superstac")

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
        logger.info(f"Registering catalog: {name}")
        logger.debug(
            f"Params - url: {url}, is_private: {is_private}, summary: {summary}, auth: {auth}"
        )
        if is_private and auth is None:
            logger.error(
                f"Private catalog '{name}' requires authentication but none was provided."
            )
            raise InvalidCatalogSchemaError(
                f"Authentication parameters is required for private catalogs. If this is a mistake, you can set 'is_private' to False or provide the {AuthInfo.__annotations__} parameters."
            )

        logger.info(f"Catalog '{name}' registered successfully.")
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
        logger.info("Retrieving available catalogs.")
        if isinstance(format, str):
            try:
                format = CatalogOutputFormat(format.lower())
            except ValueError:
                logger.error(f"Invalid output format: {format}")
                raise ValueError(f"Invalid format: {format}")

        available = [
            c.as_dict() if format == CatalogOutputFormat.DICT else c.as_json()
            for c in self.catalogs.values()
            if c.is_available
        ]

        logger.info(f"{len(available)} catalogs available in format '{format.value}'.")
        return available

    def load_catalogs_from_config(
        self, file: Union[str, Path, None] = None
    ) -> Dict[str, CatalogEntry]:
        """Load catalogs from configuration file.

        Args:
            file (Union[str, Path, None], optional): Path to the configuration file. Defaults to None.

        Raises:
            CatalogConfigFileNotFound: Raised when the catalog config file is not founds.
            InvalidCatalogYAMLError: Raised when the yaml file is invalid.
            InvalidCatalogSchemaError: Raised when there is a schema error in the provided config file.

        Returns:
            Dict[str, CatalogEntry]: The registered catalogs.
        """
        logger.info("Loading catalogs from configuration file.")
        if file is None:
            base_dir = Path(__file__).parent
            file = base_dir / ".superstac.yml"

        path = Path(file).expanduser().resolve()
        logger.debug(f"Resolved config path: {path}")

        if not path.exists():
            logger.error(f"Config file not found at path: {path}")
            raise CatalogConfigFileNotFound(f"Config file not found at {path}")

        try:
            with open(path, "r") as f:
                data = yaml.safe_load(f) or {}
            logger.info(f"Successfully loaded YAML config from: {path}")
        except yaml.YAMLError as e:
            logger.exception("YAML parsing failed.")
            raise InvalidCatalogYAMLError(f"YAML parsing failed: {e}") from e
        except Exception as e:
            logger.exception("Unexpected error while reading config.")
            raise InvalidCatalogYAMLError(
                f"Unexpected error reading config: {e}"
            ) from e

        catalogs = data.get("catalogs")
        if not isinstance(catalogs, dict):
            logger.error(
                f"Missing or invalid 'catalogs' section in config file: {path}"
            )
            raise InvalidCatalogSchemaError(
                f"Missing or invalid 'catalogs' section in config file: {path}"
            )

        logger.info(f"Found {len(catalogs)} catalogs to register.")
        for name, spec in catalogs.items():
            try:
                self.register_catalog(
                    name=name,
                    url=spec.get("url"),
                    is_private=spec.get("is_private", False),
                    summary=spec.get("summary"),
                    auth=AuthInfo(**spec["auth"]) if "auth" in spec else None,
                )
            except Exception as e:
                logger.warning(f"Failed to register catalog '{name}': {e}")
        logger.info("All catalogs loaded and registered.")
        return self.catalogs


## TEST


if __name__ == "__main__":
    # cm = CatalogManager()
    # cm.register_catalog(name="My Catalog", url="https://example.com/stac")
    # print(cm.load_catalogs_from_config())
    # print(cm.get_available_catalogs())
    ...
