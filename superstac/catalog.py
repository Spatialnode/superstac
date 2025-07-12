"""SuperSTAC Catalog Manager"""

from pathlib import Path
import attr
from typing import Any, Dict, List, Optional, Union

from pystac_client import Client

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
        logger.debug(f"Params - url: {url}, is_private: {is_private},  auth: {auth}")
        if is_private and auth is None:
            logger.error(
                f"Private catalog '{name}' requires authentication but none was provided."
            )
            raise InvalidCatalogSchemaError(
                f"Authentication parameters is required for private catalogs. If this is a mistake, you can set 'is_private' to False or provide the {AuthInfo.__annotations__} parameters."
            )
        client = None
        try:
            # todo - add auth parameters and config...
            client = Client.open(url)
            metadata = {
                "client": client,
                "catalog": client.get_root(),
                "is_available": True,
            }
            logger.info(f"Catalog '{name}' is reachable and valid.")
        except Exception as e:
            logger.warning(f"Catalog '{name}' could not be reached or parsed: {e}")
            metadata = {
                "client": None,
                "catalog": None,
                "is_available": False,
            }
        entry = CatalogEntry(
            name=name,
            url=url,
            is_private=is_private,
            auth=AuthInfo(**auth.__dict__) if auth and not is_private else None,
            **metadata,
        )
        self.catalogs[name] = entry
        logger.info(f"Catalog '{name}' registered successfully.")
        return entry

    def get_catalogs(
        self,
        format: Union[str, CatalogOutputFormat] = CatalogOutputFormat.DICT,
        available: bool = False,
    ) -> list[Union[dict[str, Any], str]]:
        """Get the STAC catalogs.

        Args:
            format (Union[str, CatalogOutputFormat]): Output format, dict or json string.
            available (bool): If True, return only available catalogs; else return all.

        Raises:
            ValueError: When an invalid format is provided.

        Returns:
            list[CatalogEntry]: List of catalogs in the requested format.
        """
        logger.info(f"Retrieving {'available' if available else 'all'} catalogs.")
        if isinstance(format, str):
            try:
                format = CatalogOutputFormat(format.lower())
            except ValueError:
                logger.error(f"Invalid output format: {format}")
                raise ValueError(f"Invalid format: {format}")

        catalogs = [
            c.as_dict() if format == CatalogOutputFormat.DICT else c.as_json()
            for c in self.catalogs.values()
            if (c.is_available if available else True)
        ]

        logger.info(f"{len(catalogs)} catalogs retrieved in format '{format.value}'.")
        return catalogs

    def get_all_collections(self, available: bool = False) -> Dict[str, List[str]]:
        """
        Returns a dictionary mapping catalog names to a list of collection IDs
        available in each catalog.

        Args:
            available (bool): If True, only include catalogs that are available.

        Returns:
            Dict[str, List[str]]: Catalog name -> list of collection IDs
        """
        logger.info(f"Getting collections with available={available}")
        collections = {}
        for name, entry in self.catalogs.items():
            if (entry.is_available if available else True) and entry.catalog:
                logger.debug(
                    f"Processing catalog '{name}' (available={entry.is_available})"
                )
                try:
                    all_collections = entry.catalog.get_all_collections()
                    if all_collections:
                        collection_ids = []
                        for c in all_collections:
                            if hasattr(c, "id"):
                                collection_ids.append(c.id)
                            elif isinstance(c, dict) and "id" in c:
                                collection_ids.append(c["id"])
                            else:
                                collection_ids.append(str(c))
                        collections[name] = collection_ids
                        logger.info(
                            f"Found {len(collection_ids)} collections in catalog '{name}'"
                        )
                    else:
                        logger.info(f"No collections found in catalog '{name}'")
                except Exception as e:
                    logger.warning(
                        f"Failed to get collections for catalog '{name}': {e}"
                    )
                    continue
            else:
                logger.debug(f"Skipping catalog '{name}' due to availability filter")
        logger.info(f"Total catalogs with collections returned: {len(collections)}")
        return collections

    def load_catalogs_from_config(
        self, config: Union[str, Path, None] = None
    ) -> Dict[str, CatalogEntry]:
        """Load catalogs from configuration file.

        Args:
            config (Union[str, Path, None], optional): Path to the configuration file. Defaults to None.

        Raises:
            CatalogConfigFileNotFound: Raised when the catalog config file is not founds.
            InvalidCatalogYAMLError: Raised when the yaml file is invalid.
            InvalidCatalogSchemaError: Raised when there is a schema error in the provided config file.

        Returns:
            Dict[str, CatalogEntry]: The registered catalogs.
        """
        logger.info("Loading catalogs from configuration file.")
        if config is None:
            # todo. - make public and environment variable
            base_dir = Path(__file__).parent
            config = base_dir / ".superstac.yml"

        path = Path(config).expanduser().resolve()
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
                    auth=AuthInfo(**spec["auth"]) if "auth" in spec else None,
                )
            except Exception as e:
                logger.warning(f"Failed to register catalog '{name}': {e}")
        logger.info("All catalogs loaded and registered.")
        return self.catalogs
