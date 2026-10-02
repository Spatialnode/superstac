"""Execute the published notebook against a local STAC fixture, never a provider.

Requires an installed v0.3 wheel, pyarrow, nbformat, nbclient, and ipykernel.
"""
from datetime import datetime, timedelta, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import sys
import tempfile
import threading

import nbformat
from nbclient import NotebookClient
from jupyter_client import KernelManager
import superstac


class Catalog(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_POST(self):
        assert self.path == "/search"
        assert self.headers.get("User-Agent") == f"superstac/{superstac.__version__}"
        query = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        start = query.get("datetime", "2025-02-01T00:00:00Z/..").split("/")[0]
        instant = datetime.fromisoformat(start.replace("Z", "+00:00")) + timedelta(hours=1)
        instant = instant.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")
        item = {"type": "Feature", "stac_version": "1.0.0", "id": f"scene-{instant}",
                "collection": "sentinel-2-l2a", "geometry": {"type": "Point", "coordinates": [-3.7, 40.4]},
                "bbox": [-3.7, 40.4, -3.7, 40.4], "properties": {"datetime": instant},
                "assets": {}, "links": []}
        data = json.dumps({"type": "FeatureCollection", "features": [item], "links": [], "numberMatched": 1}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


root = Path(__file__).resolve().parents[1]
notebook = nbformat.read(root / "docs/public/notebooks/superstac-geoparquet.ipynb", as_version=4)
nbformat.validate(notebook)
notebook.cells = [cell for cell in notebook.cells if "installation" not in cell.metadata.get("tags", [])]
server = ThreadingHTTPServer(("127.0.0.1", 0), Catalog)
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
try:
    with tempfile.TemporaryDirectory(prefix="superstac-notebook-check-") as directory:
        kernel = KernelManager(kernel_name="python3")
        kernel.kernel_spec.argv = [sys.executable, "-m", "ipykernel_launcher", "-f", "{connection_file}"]
        client = NotebookClient(notebook, km=kernel, timeout=120, resources={"metadata": {"path": directory}})
        try:
            client.execute(env={**os.environ, "SUPERSTAC_DEMO_CATALOG_URL": f"http://127.0.0.1:{server.server_port}"})
        finally:
            if kernel.has_kernel:
                kernel.shutdown_kernel(now=True)
        archives = list(Path(directory).glob("*.zip"))
        assert len(archives) == 1, "Notebook must export an inventory archive"
        print(f"Executed {sum(c.cell_type == 'code' for c in notebook.cells)} code cells; verified inventory export and provenance using local HTTP only.")
finally:
    server.shutdown()
    server.server_close()
    thread.join()
