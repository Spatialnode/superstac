import uuid


def compute_catalog_id(name: str, url: str) -> str:
    """Compute a unique catalog id.

    Args:
        name (str): The name of the catalog.
        url (str): The url of the catalog.

    Returns:
        str: A unique uuid for the catalog.
    """
    uid = uuid.uuid4().hex[:10]
    return f"{name.lower().replace(' ', '_')}_{uid}"
