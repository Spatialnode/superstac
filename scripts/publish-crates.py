"""Publish missing workspace crates in dependency order; safe to rerun after partial success."""
import json
from pathlib import Path
import subprocess
import tomllib
import urllib.error
import urllib.request

root = Path(__file__).resolve().parents[1]
version = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["package"]["version"]
for crate in ("core", "config", "search", "geoparquet", "engine", "cli"):
    name = f"superstac-{crate}"
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{name}/{version}",
        headers={"User-Agent": "SuperSTAC release workflow (github.com/Spatialnode/superstac)"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            existing = json.load(response)
        if existing["version"]["yanked"]:
            raise RuntimeError(f"{name} {version} is yanked; choose a new version")
        print(f"Already published: {name} {version}", flush=True)
        continue
    except urllib.error.HTTPError as error:
        if error.code != 404:
            raise
    subprocess.run(["cargo", "publish", "--locked", "-p", name], cwd=root, check=True)
