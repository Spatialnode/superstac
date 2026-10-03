---
title: Installation
description: Install SuperSTAC for Python, Rust, or the command line.
---

Choose Python for notebooks and scripts, Rust to add search to an application, or the CLI to work from your terminal.

## Python

```bash
python -m pip install "superstac>=0.3,<0.4"
```

You need **Python 3.9 or newer**.

Verify the import:

```bash
python -c "from superstac import Client, AsyncClient; print(Client, AsyncClient)"
```

To build from source, you need **Rust 1.88 or newer** and Python development tools. Use this if no wheel is available for your platform:

```bash
git clone https://github.com/spatialnode/superstac
cd superstac/crates/python
git checkout v0.3.0
python -m venv .venv
source .venv/bin/activate
python -m pip install maturin
maturin develop --release
```

On Windows, activate the environment with `.venv\Scripts\Activate.ps1` in PowerShell.

## Rust

Add the crates that your application uses:

```bash
cargo add superstac-core@0.3 superstac-config@0.3 superstac-search@0.3 superstac-engine@0.3
cargo add tokio --features macros,rt-multi-thread
```

These docs target SuperSTAC 0.3. See the [Rust guide](/docs/v0.3/rust/overview/).

## Command-line tool

Build and install the CLI from the repository:

```bash
git clone https://github.com/spatialnode/superstac
cd superstac
git checkout v0.3.0
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

[Run a first search](/docs/v0.3/start/quickstart/) against two catalogs, or read the [CLI reference](/docs/v0.3/cli/overview/) for flags and output formats.
