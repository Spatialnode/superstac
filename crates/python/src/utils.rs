use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pythonize::{depythonize, pythonize};
use superstac_core::{errors::SuperSTACError, models::storage::Storage};

/// Convert a SuperSTACError to a Python exception with a helpful error message.
pub fn err_to_py(e: SuperSTACError) -> PyErr {
    PyRuntimeError::new_err(e.to_string())
}

/// Parse a storage backend from a string. Only "memory" is supported in v0.1, but this is where other backends would be added in the future.
pub fn parse_storage_kind(kind: &str) -> PyResult<Storage> {
    match kind.to_ascii_lowercase().as_str() {
        "memory" => Ok(Storage::Memory),
        other => Err(PyValueError::new_err(format!(
            "unknown storage backend '{}'. Only 'memory' is supported in v0.1.",
            other
        ))),
    }
}

/// Convert a Rust type to a Python object, with a helpful error message on failure.
pub fn pythonize_obj<T: serde::Serialize>(py: Python<'_>, value: &T) -> PyResult<Py<PyAny>> {
    pythonize(py, value)
        .map(|b| b.unbind())
        .map_err(|e| PyRuntimeError::new_err(format!("serialize failed: {}", e)))
}

/// Convert a Python object to a Rust type, with a helpful error message on failure. 
pub fn depythonize_into<'py, T: serde::Deserialize<'py>>(
    obj: &'py Bound<'py, PyAny>,
    label: &str,
) -> PyResult<T> {
    depythonize(obj).map_err(|e| PyValueError::new_err(format!("invalid {}: {}", label, e)))
}

/// Derive a stable catalog id from a URL by taking the first dot-separated
/// label of the hostname. `https://earth-search.aws.element84.com/v1` -> `earth-search`.
pub fn derive_id_from_url(url: &str) -> PyResult<String> {
    let after_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = after_scheme.split('/').next().unwrap_or("");
    let host = host.split(':').next().unwrap_or(host);
    let first = host.split('.').next().unwrap_or("");
    if first.is_empty() {
        return Err(PyValueError::new_err(format!(
            "could not derive catalog id from URL '{}'; pass `id=` explicitly",
            url
        )));
    }
    Ok(first.to_string())
}
