# Package releases

`Publish packages` runs when a GitHub release is published. Rust tests, five
native wheel builds (Linux x86-64/ARM64, macOS Intel/Apple Silicon, Windows x86-64),
installed-wheel tests, and a source-archive rebuild must pass before publication.
Wheels use the CPython 3.9+ stable ABI. Linux wheels require glibc 2.28+; other
platforms can build from the source distribution.

## Credentials

- `CARGO_REGISTRY_TOKEN`: repository Actions secret (or `crates-io` environment),
  with publishing rights for the five existing SuperSTAC crates.
- PyPI Trusted Publisher on the existing `superstac` project: owner `Spatialnode`,
  repository `superstac`, workflow `publish.yml`, environment `pypi`.
  Alternatively, set a project-scoped `PYPI_API_TOKEN` Actions secret.
- GitHub environments may inherit repository secrets. Keep tokens out of files.

## Releasing

1. Update the workspace version, internal Cargo dependency constraints, Cargo.lock,
   changelog, migration notes, and docs version labels. Python reads its version
   from Cargo.
2. Merge the release PR after `Package checks` passes. That workflow also supports
   manual runs, producing artifacts without publishing.
3. Tag the checked commit `v<version>` and publish its GitHub release. The workflow
   checks the tag against the workspace version.
4. Check `Publish packages` and the independently triggered `Deploy docs` workflow.
   Confirm versions and documentation links on PyPI and crates.io before announcing.

Crates publish in dependency order: core, config, search, engine, cli. The Python
extension's Cargo package has `publish = false` and goes to PyPI only. Publication
cannot be atomic across registries. Rerun after fixing credentials or registry
availability; existing crate versions and Python files are skipped. Source
changes after publication require a new version and tag.

Docs may deploy before package publication. The package remains alpha software
even when the numeric version has no `a` suffix.

## Local verification

```bash
cargo test --workspace --exclude superstac-python --locked
maturin build --release --locked --manifest-path crates/python/Cargo.toml --out dist
maturin sdist --manifest-path crates/python/Cargo.toml --out dist
# In a clean environment with Python >=3.9:
python -m pip install --no-deps dist/*.whl
python -m unittest discover -s crates/python/tests -v
```

Tests use local HTTP fixtures and no public STAC services. Always test the installed
wheel, not a checkout's extension module.
