use std::future::Future;
use std::sync::OnceLock;

use pyo3::prelude::*;
use pyo3_async_runtimes::tokio as pyo3_tokio;

static RUNTIME_INIT: OnceLock<()> =
    OnceLock::new();

pub fn ensure_runtime() {
    RUNTIME_INIT.get_or_init(|| {

        let mut builder =
            tokio::runtime::Builder::new_multi_thread();

        builder.enable_all();

        pyo3_tokio::init(builder);
    });
}


/// Execute async Rust from sync Python.
pub fn block_on<F, T>(
    py: Python<'_>,
    future: F,
) -> T
where
    F: Future<Output = T>
        + Send
        + 'static,
    T: Send + 'static,
{
    ensure_runtime();

    py.detach(|| {
        pyo3_tokio::get_runtime()
            .block_on(future)
    })
}
/// Convert async Rust into Python coroutine.
pub fn into_py<'py, F, T>(
    py: Python<'py>,
    future: F,
) -> PyResult<Bound<'py, PyAny>>
where
    F: Future<Output = PyResult<T>>
        + Send
        + 'static,

    T: for<'a> IntoPyObject<'a>,
{
    ensure_runtime();

    pyo3_tokio::future_into_py(
        py,
        async move {
            future.await
        },
    )
}