use crate::utils::pythonize_obj;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pythonize::pythonize;
use superstac_search::response::SearchResponse;

/// Returned by `Client.search(...)` and `await AsyncClient.search(...)`.
///
/// Mirrors the parts of pystac-client's `ItemSearch` users actually iterate
/// over: `.items()`, `.matched()`, `.item_collection_as_dict()`. Superstac-
/// specific provenance (per-catalog failures, dedupe stats, etc.) is on
/// `.metadata`.
#[pyclass(name = "Search", module = "superstac._superstac")]
pub struct PySearch {
    pub response: SearchResponse,
    pub cached_items: Option<Py<PyList>>,
}

#[pymethods]
impl PySearch {
    fn ensure_items<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        if let Some(ref items) = self.cached_items {
            return Ok(items.bind(py).clone());
        }

        let list = PyList::empty(py);

        for item in &self.response.items {
            let d = pythonize(py, &item.item)
                .map_err(|e| PyRuntimeError::new_err(format!("serialize item: {}", e)))?;

            list.append(d)?;
        }

        let owned = list.clone().unbind();
        self.cached_items = Some(owned);

        Ok(list)
    }

    /// Total items matched. Equal to `len(list(self.items()))` until
    /// pagination is added.
    pub fn matched(&self) -> usize {
        self.response.metadata.total_items
    }

    /// Iterable of items as STAC item JSON dicts. Returns a list (Python
    /// lists are iterable, so `for item in search.items()` works as expected).
    /// The per-item provenance lives on `.metadata`.
    pub fn items<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        self.ensure_items(py)
    }

    /// GeoJSON FeatureCollection-shaped dict containing every item.
    pub fn item_collection_as_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("type", "FeatureCollection")?;
        let features = PyList::empty(py);
        for item in &self.response.items {
            let f = pythonize(py, &item.item)
                .map_err(|e| PyRuntimeError::new_err(format!("serialize item: {}", e)))?;
            features.append(f)?;
        }
        dict.set_item("features", features)?;
        Ok(dict)
    }

    /// Alias for `item_collection_as_dict()`.
    /// Returns a GeoJSON FeatureCollection dict.
    pub fn to_geojson<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.item_collection_as_dict(py)
    }

    /// Superstac-specific stats: catalogs queried/succeeded/failed, dedupe
    /// count, per-catalog failures, etc. Schema mirrors `SearchMetadata`.
    #[getter]
    pub fn metadata(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        pythonize_obj(py, &self.response.metadata)
    }

    pub fn __len__(&self) -> usize {
        self.response.items.len()
    }

    fn __iter__<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        Ok(self.ensure_items(py)?.into_any())
    }

    fn __repr__(&self) -> String {
        format!(
            "SuperSTACSearch(items={}, catalogs={}, failed={})",
            self.response.items.len(),
            self.response.metadata.catalogs_queried,
            self.response.metadata.catalogs_failed,
        )
    }
}
