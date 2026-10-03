---
title: Installation
description: Install SuperSTAC for Python, Rust, or the command line.
---

Choose Python for notebooks and scripts, Rust to add search to an application, or the CLI to work from your terminal.

## Python

```bash
python -m pip install superstac
```

You need **Python 3.9 or newer**.

Verify the import:

```bash
python -c "from superstac import Client, AsyncClient; print(Client, AsyncClient)"
```

To build from source, you need **Rust 1.88 or newer** and Python development tools. Use this if no wheel is available for your platform or you want to try unreleased changes:

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

These docs target SuperSTAC 0.3. See the [Rust guide](/docs/rust/overview/).

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

## Run a search

[Run a first search](/docs/start/quickstart/) against two catalogs, or read the [CLI reference](/docs/cli/overview/) for flags and output formats.
