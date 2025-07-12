import time
import asyncio


def main():
    from superstac import get_catalog_registry, federated_search_async

    cr = get_catalog_registry()
    cr.load_catalogs_from_config()

    print("\nRunning asynchronous federated_search_async...")
    start_async = time.perf_counter()
    results_async = asyncio.run(
        federated_search_async(
            registry=cr,
            collections=["sentinel-2-l2a"],
            bbox=[6.0, 49.0, 7.0, 50.0],
            datetime="2024-01-01/2024-01-31",
            query={"eo:cloud_cover": {"lt": 20}},
            sortby=[{"field": "properties.datetime", "direction": "desc"}],
        )
    )
    end_async = time.perf_counter()
    print(
        f"Asynchronous search found {len(results_async)} items in {end_async - start_async:.2f} seconds."
    )

    for x in results_async:
        print(x.self_href)


if __name__ == "__main__":
    main()
