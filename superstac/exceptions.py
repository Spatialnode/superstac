"""SuperSTAC Exceptions"""


class InvalidCatalogSchemaError(Exception):
    """Raised when an invalid catalog schema is provided."""


class ParametersError(Exception):
    """Raised when invalid parameters are used in a query."""


class CatalogConfigFileNotFound(FileNotFoundError):
    """Raised when the provided catalog config file is not found."""


class InvalidCatalogYAMLError(Exception):
    """Raised when invalid catalog yaml is passed or if does not conform to the schema."""
