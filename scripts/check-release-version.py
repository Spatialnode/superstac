"""Check the package versions and, on release runs, the Git tag. Python 3.11+."""
import os
from pathlib import Path
import tomllib

root = Path(__file__).resolve().parents[1]
workspace = tomllib.loads((root / "Cargo.toml").read_text())
version = workspace["workspace"]["package"]["version"]
if os.environ.get("GITHUB_REF_TYPE") == "tag":
    assert os.environ["GITHUB_REF_NAME"] == f"v{version}", "Release tag must match Cargo workspace version"
for path in (root / "crates").glob("*/Cargo.toml"):
    manifest = tomllib.loads(path.read_text())
    assert manifest["package"]["version"] == {"workspace": True}, path
    for name, dep in manifest.get("dependencies", {}).items():
        if name.startswith("superstac-"):
            assert dep["version"] == version, (path, name)
print(f"Release versions consistent: {version}")
