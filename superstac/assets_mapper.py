from typing import Union
from superstac._logging import logger
from pystac import Item

unified_band_mapper = {
    # Sentinel-2 L2A mapping
    # Retrieved from ''
    "sentinel-2-l2a": {
        "coastal": "B01",
        "blue": "B02",
        "green": "B03",
        "red": "B04",
        "rededge1": "B05",
        "rededge2": "B06",
        "rededge3": "B07",
        "nir": "B08",
        "nir08": "B8A",
        "nir09": "B09",
        "swir16": "B11",
        "cirrus": "B10",
        "swir22": "B12",
        "aot": "AOT",
        "scl": "SCL",
        "wvp": "WVP",
    },
    # Landsat 8/9 Collection 2 Level 2 mapping
    "landsat-8-c2l2": {
        "coastal": "SR_B1",
        "blue": "SR_B2",
        "green": "SR_B3",
        "red": "SR_B4",
        "nir": "SR_B5",
        "swir16": "SR_B6",
        "swir22": "SR_B7",
        "thermal_infrared": "ST_B10",
        "pixel_qa": "QA_PIXEL",
    },
    "landsat-9-c2l2": {
        "coastal": "SR_B1",
        "blue": "SR_B2",
        "green": "SR_B3",
        "red": "SR_B4",
        "nir": "SR_B5",
        "swir16": "SR_B6",
        "swir22": "SR_B7",
        "thermal_infrared": "ST_B10",
        "pixel_qa": "QA_PIXEL",
    },
}


def get_asset_by_standard_name(
    stac_item: Item, standard_band_name: str
) -> Union[str, None]:
    """
    Retrieves a STAC asset from an item using a standardized band name.

    Args:
        stac_item (pystac.Item): The STAC item to search.
        standard_band_name (str): The common, standardized name of the band (e.g., 'red', 'nir').

    Returns:
        pystac.Asset or None: The asset object if found, otherwise None.
    """

    collection_id = stac_item.collection_id or ""

    band_mapping = unified_band_mapper.get(collection_id)

    if band_mapping is None:
        logger.info(f"Warning: No mapping found for collection '{collection_id}'.")
        return None

    asset_key = band_mapping.get(standard_band_name)

    if asset_key is None:
        logger.info(
            f"Warning: No asset key found for standard band '{standard_band_name}' in collection '{collection_id}'."
        )
        return None

    return stac_item.assets.get(asset_key)  # type: ignore
