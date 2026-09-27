---
title: Installation
description: Install the Python bindings, Rust crates, or command-line tool and choose the right interface.
---

SuperSTAC has three interfaces over the same Rust search engine. Choose Python for notebooks and pipelines, Rust for embedding, or the CLI for terminal workflows.

## Python

```bash
python -m pip install superstac
```

Use **Python 3.9 or newer** for the current bindings: the native extension targets the `abi3-py39` ABI. The package metadata currently declares an older minimum; the extension's ABI is the practical constraint.

Verify the import:

```bash
python -c "from superstac import Client, AsyncClient; print(Client, AsyncClient)"
```

If no wheel is available for your platform or the registry release differs from this checkout, build the bindings from source. You need a Rust toolchain compatible with the workspace's **Rust 1.88 minimum** and Python development tools.

```bash
git clone https://github.com/spatialnode/superstac
cd superstac/crates/python
python -m venv .venv
source .venv/bin/activate
python -m pip install maturin
maturin develop --release
```

On Windows, activate the environment with `.venv\Scripts\Activate.ps1` in PowerShell.

## Rust

Add the crates that your application uses:

```bash
cargo add superstac-core superstac-config superstac-search superstac-engine
cargo add tokio --features macros,rt-multi-thread
```

These docs describe the working tree's 0.1 API. For unreleased changes, use path dependencies to a local checkout instead of assuming that a registry release has the same API. See the [Rust guide](/docs/rust/overview/).

## Command-line tool

Build and install the CLI from the repository:

```bash
git clone https://github.com/spatialnode/superstac
cd superstac
cargo install --path crates/cli
superstac --help
```

Or build without installing:

```bash
cargo build --release -p superstac-cli
./target/release/superstac --help
```

The executable is named `superstac`; the Rust package is `superstac-cli`.

## Next step

[Run a first search](/docs/start/quickstart/) against two catalogs, or read the [CLI reference](/docs/cli/overview/) for flags and output formats.
