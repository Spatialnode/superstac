---
title: Build from source
description: Build and test the WebAssembly binding when contributing to SuperSTAC.
---

This guide is for changing the Rust binding or producing your own package. To use SuperSTAC in a JavaScript app, start with the [browser guide](/docs/v0.3/wasm/overview).

## Build the browser package

Install Rust 1.88 or newer, then run these commands from the repository root:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.13.1 --locked --jobs 1
wasm-pack build crates/wasm --scope spatialnode --target web --release --locked --jobs 1
```

The generated package is in `crates/wasm/pkg`. Builds use one Cargo worker to limit resource use. The package contains JavaScript, WebAssembly, TypeScript declarations, and npm metadata.

Install the generated package from your app directory:

```bash
npm install /path/to/superstac/crates/wasm/pkg
```

Your application imports `@spatialnode/superstac-wasm`, just as it would with an npm release.

## Try a local build

From the repository root:

```bash
python3 -m http.server 8000 --directory crates/wasm
```

Open `http://localhost:8000/examples/` for a small browser example. For the full docs playground, `node docs/scripts/prepare-wasm.mjs` copies the existing package into the docs assets without compiling Rust.

The [WASM contributor README](https://github.com/spatialnode/superstac/tree/main/crates/wasm#tests) describes API checks and browser tests. The **WASM checks** workflow also builds the package and uploads the **wasm-web** artifact for use without a local Rust toolchain.
