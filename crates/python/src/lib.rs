//! PyO3 bindings for SuperSTAC.
//!
//! Exposes two top-level classes that share the same engine internals:
//!
//! - `Client` — synchronous; designed as a drop-in for pystac-client users.
//! - `AsyncClient` — async; methods return Python coroutines.
//!
//! Both clients share the same underlying engine state, allowing
//! catalogs, providers, and settings to be configured dynamically
//! from Python.
//! Responses are returned as native Python dicts/lists via `pythonize`.


use pyo3::prelude::*;

use crate::{{async_client::PyAsyncClient, sync_client::PyClient}, search::PySearch};
mod search;
mod geoparquet;
mod shared;
mod utils;
mod runtime;
mod async_client;
mod sync_client;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");


#[pymodule]
fn _superstac(m: &Bound<'_, PyModule>) -> PyResult<()> {
    
    m.add_class::<PyClient>()?;
    m.add_class::<PyAsyncClient>()?;
    m.add_class::<PySearch>()?;
    m.add("__version__", VERSION)?;
    m.add("geoparquet_available", cfg!(feature = "geoparquet"))?;
    Ok(())
}


