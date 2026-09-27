"""Integration checks against the installed wheel, using local STAC fixtures."""
import asyncio
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from importlib.metadata import metadata, version
import json
from pathlib import Path
import threading
import unittest

import superstac
from superstac import AsyncClient, Client


class CatalogHandler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_HEAD(self):
        self.send_response(200)
        self.end_headers()

    def do_POST(self):
        self.rfile.read(int(self.headers.get("Content-Length", 0)))
        self.do_GET()

    def do_GET(self):
        path = self.path.split("?", 1)[0]
        status = 200
        if path.endswith("/collections"):
            response = {"collections": [{
                "type": "Collection", "stac_version": "1.0.0", "id": "test-scenes",
                "description": "Local test collection", "license": "proprietary", "links": [],
                "extent": {"spatial": {"bbox": [[-180, -90, 180, 90]]},
                           "temporal": {"interval": [[None, None]]}},
            }], "links": []}
        elif path.endswith("/search"):
            if path.startswith("/broken"):
                status, response = 503, {"error": "fixture unavailable"}
            else:
                catalog = path.split("/")[1]
                response = {"type": "FeatureCollection", "links": [], "features": [
                    {"type": "Feature", "stac_version": "1.0.0", "id": item_id,
                     "collection": "test-scenes", "geometry": {"type": "Point", "coordinates": [0, 0]},
                     "bbox": [0, 0, 0, 0], "properties": {"datetime": "2026-01-01T00:00:00Z"},
                     "links": [], "assets": {}}
                    for item_id in ["shared", catalog]
                ]}
        else:
            response = {"type": "Catalog", "stac_version": "1.0.0", "id": "fixture", "description": "Fixture", "links": []}
        data = json.dumps(response).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


class WheelTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), CatalogHandler)
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()

    def catalogs(self, *ids):
        return [{"id": id, "url": f"http://127.0.0.1:{self.server.server_port}/{id}",
                 "settings": {"health_check_strategy": "hourly", "healthy_status_code_range": [200, 299],
                              "enable_background_health_monitor": False}} for id in ids]

    def client(self, cls=Client, ids=("first", "second")):
        return cls(catalogs=self.catalogs(*ids), settings={
            "logging_enabled": False, "per_catalog_timeout_seconds": 2,
            "max_retry_attempts": 1, "search_healthy_catalogs_only": False,
        })

    def check_results(self, result):
        self.assertEqual({item["id"] for item in result.items()}, {"shared", "first", "second"})
        self.assertEqual(len(result), 3)
        self.assertEqual(result.matched(), 3)
        self.assertEqual(list(result), result.items())
        self.assertEqual(list(result), list(result))
        self.assertEqual(result.to_geojson(), result.item_collection_as_dict())
        self.assertEqual(result.to_geojson()["type"], "FeatureCollection")
        self.assertEqual(result.metadata["catalogs_failed"], 0)
        self.assertEqual(result.metadata["catalogs_queried"], 2)

    def test_installed_metadata(self):
        self.assertEqual(superstac.__version__, version("superstac"))
        self.assertEqual(metadata("superstac")["Requires-Python"], ">=3.9")
        self.assertFalse(metadata("superstac").get_all("Requires-Dist"))
        package = Path(superstac.__file__).parent
        self.assertTrue((package / "py.typed").is_file())
        self.assertTrue((package / "_superstac.pyi").is_file())

    def test_sync_federation_and_iteration(self):
        client = self.client()
        try:
            self.check_results(client.search(collections=["test-scenes"], limit=5))
        finally:
            client.shutdown()

    def test_async_federation(self):
        async def run():
            client = self.client(AsyncClient)
            try:
                self.check_results(await client.search(collections=["test-scenes"], limit=5))
            finally:
                await client.shutdown()
        asyncio.run(run())

    def test_partial_failure(self):
        client = self.client(ids=("first", "broken"))
        try:
            result = client.search(collections=["test-scenes"], limit=5)
            self.assertEqual(len(result), 2)
            self.assertEqual(result.metadata["catalogs_failed"], 1)
            self.assertEqual(result.metadata["failures"][0]["catalog_id"], "broken")
        finally:
            client.shutdown()

    def test_configuration_and_invalid_storage(self):
        with self.assertRaises(ValueError):
            Client(storage="sqlite")
        client = self.client()
        try:
            self.assertEqual(len(client.list_catalogs()), 2)
            client.delete_catalog("second")
            self.assertEqual(len(client.list_catalogs()), 1)
            client.update_settings({"max_items_per_catalog": 7})
            self.assertEqual(client.get_settings()["max_items_per_catalog"], 7)
        finally:
            client.shutdown()


if __name__ == "__main__":
    unittest.main()
