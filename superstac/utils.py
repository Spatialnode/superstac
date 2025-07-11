"""SuperSTAC utils"""

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


# Band map between Element 84's STAC and Planetary Computer
# allow users to provide band map ?
BAND_MAP = {
    "sentinel-2-l2a": {"red": "B04", "green": "B03", "blue": "B02", "nir": "B08"},
    "landsat-8": {"red": "SR_B4", "green": "SR_B3", "blue": "SR_B2", "nir": "SR_B5"},
    "modis": {"red": "sur_refl_b01", "nir": "sur_refl_b02"},
}
