use std::sync::atomic::Ordering;
use std::sync::Arc;

use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};

use superstac_config::init_from_yaml;
use superstac_core::errors::SuperSTACError;

use crate::runtime::block_on;
use crate::search::PySearch;
use crate::shared::{
    add_catalog_impl, add_catalogs_impl, add_provider_impl, add_providers_impl, build_inner,
    search_query_from_kwargs, update_catalog_impl, update_provider_impl, update_settings_impl,
    Inner,
};
use crate::utils::{err_to_py, parse_storage_kind, pythonize_obj};

#[pyclass(name = "Client", module = "superstac._superstac")]
pub struct PyClient {
    inner: Inner,
}

#[pymethods]
impl PyClient {
    /// Build an async client. `config` mirrors the YAML shape (see
    /// [`Client.__init__`]); every key is optional.
    #[new]
    #[pyo3(signature = (
        config = None,
        *,
        catalogs = None,
        providers = None,
        settings = None,
        storage = "memory",
        mode = "live",
        dataset = None,
        max_snapshot_age_seconds = 86400
    ))]
    fn new(
        config: Option<Bound<'_, PyDict>>,
        catalogs: Option<Bound<'_, PyAny>>,
        providers: Option<Bound<'_, PyAny>>,
        settings: Option<Bound<'_, PyAny>>,
        storage: &str,
        mode: &str,
        dataset: Option<&str>,
        max_snapshot_age_seconds: u64,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: build_inner(config, catalogs, providers, settings, storage)?
                .with_search_backend(mode, dataset, max_snapshot_age_seconds)?,
        })
    }

    /// pystac-client compat: connect to a single STAC catalog at `url`. The
    /// catalog id defaults to the first dot-separated label of the hostname.
    /// Calls `start()` before returning so the result is ready to `search()`.
    #[classmethod]
    #[pyo3(signature = (url, *, id = None, storage = "memory", mode = "live", dataset = None, max_snapshot_age_seconds = 86400))]
    fn open(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        url: &str,
        id: Option<&str>,
        storage: &str,
        mode: &str,
        dataset: Option<&str>,
        max_snapshot_age_seconds: u64,
    ) -> PyResult<Self> {
        let inner = Inner::from_storage_kind(storage)?;

        inner.register_open_catalog(url, id)?;
        let inner = inner.with_search_backend(mode, dataset, max_snapshot_age_seconds)?;

        let engine = Arc::clone(&inner.engine);

        block_on(py, async move { engine.start().await }).map_err(err_to_py)?;

        Ok(Self { inner })
    }

    /// Load catalogs/providers/settings from a `superstac.yml` file.
    #[classmethod]
    #[pyo3(signature = (yaml_path, *, storage = "memory", mode = "live", dataset = None, max_snapshot_age_seconds = 86400))]
    fn from_yaml(
        _cls: &Bound<'_, PyType>,
        yaml_path: &str,
        storage: &str,
        mode: &str,
        dataset: Option<&str>,
        max_snapshot_age_seconds: u64,
    ) -> PyResult<Self> {
        let backend_kind = parse_storage_kind(storage)?;
        let backend = init_from_yaml(backend_kind, yaml_path).map_err(err_to_py)?;
        Ok(Self {
            inner: Inner::from_backend(backend).with_search_backend(
                mode,
                dataset,
                max_snapshot_age_seconds,
            )?,
        })
    }

    /// Run health checks + `/collections` introspection. Idempotent.
    fn start(&self, py: Python<'_>) -> PyResult<()> {
        let engine = Arc::clone(&self.inner.engine);

        block_on(py, async move { engine.start().await }).map_err(err_to_py)
    }

    /// Cancel background health-monitor tasks.
    fn shutdown(&self, py: Python<'_>) {
        let engine = Arc::clone(&self.inner.engine);

        block_on(py, async move { engine.shutdown().await });
    }

    #[pyo3(signature = (catalog, *, provider = None))]
    fn add_catalog(
        &self,
        py: Python<'_>,
        catalog: Bound<'_, PyAny>,
        provider: Option<&str>,
    ) -> PyResult<Py<PyAny>> {
        add_catalog_impl(&self.inner, py, catalog, provider)
    }

    /// Bulk equivalent of `add_catalog`. Accepts a list of catalog dicts.
    /// Each entry's optional `provider` field is honored.
    fn add_catalogs(&self, py: Python<'_>, catalogs: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        add_catalogs_impl(&self.inner, py, catalogs)
    }

    fn get_catalog(&self, py: Python<'_>, id: &str) -> PyResult<Py<PyAny>> {
        let storage = self.inner.storage.lock();
        pythonize_obj(py, storage.get_catalog(id).map_err(err_to_py)?)
    }

    fn list_catalogs(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let catalogs = self
            .inner
            .storage
            .lock()
            .list_catalogs(None)
            .map_err(err_to_py)?;
        pythonize_obj(py, &catalogs)
    }

    fn update_catalog(
        &self,
        py: Python<'_>,
        id: &str,
        update: Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        update_catalog_impl(&self.inner, py, id, update)
    }

    fn delete_catalog(&self, id: &str) -> PyResult<()> {
        self.inner
            .storage
            .lock()
            .delete_catalog(id)
            .map_err(err_to_py)
    }

    fn add_provider(&self, py: Python<'_>, provider: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        add_provider_impl(&self.inner, py, provider)
    }

    /// Bulk equivalent of `add_provider`.
    fn add_providers(&self, py: Python<'_>, providers: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        add_providers_impl(&self.inner, py, providers)
    }

    fn get_provider(&self, py: Python<'_>, id: &str) -> PyResult<Py<PyAny>> {
        let storage = self.inner.storage.lock();
        pythonize_obj(py, storage.get_provider(id).map_err(err_to_py)?)
    }

    fn list_providers(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let providers = self
            .inner
            .storage
            .lock()
            .list_providers(None)
            .map_err(err_to_py)?;
        pythonize_obj(py, &providers)
    }

    fn update_provider(
        &self,
        py: Python<'_>,
        id: &str,
        update: Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        update_provider_impl(&self.inner, py, id, update)
    }

    fn delete_provider(&self, id: &str) -> PyResult<()> {
        self.inner
            .storage
            .lock()
            .delete_provider(id)
            .map_err(err_to_py)
    }

    fn get_settings(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let s = self.inner.storage.lock().get_settings();
        pythonize_obj(py, &s)
    }

    fn update_settings(&self, update: Bound<'_, PyAny>) -> PyResult<()> {
        update_settings_impl(&self.inner, update)
    }

    /// pystac-client style: `client.search(collections=[...], bbox=[...], ...)`.
    /// Accepts kwargs matching the STAC item-search schema: `collections`,
    /// `ids`, `intersects`, `bbox`, `datetime`, `limit`, `sortby`.
    #[pyo3(signature = (**kwargs))]
    pub fn search(&self, py: Python<'_>, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<PySearch> {
        let query = search_query_from_kwargs(kwargs)?;

        let engine = Arc::clone(&self.inner.engine);

        let resp = block_on(py, async move { engine.search(query).await }).map_err(err_to_py)?;

        Ok(PySearch {
            response: resp,
            cached_items: None,
        })
    }

    /// Aggregated view: every collection ID known across catalogs, with the
    /// catalogs that serve each. Returns a list of `{"id", "catalogs"}` dicts.
    pub fn list_collections(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        let availabilities =
            block_on(py, async move { engine.list_collections().await }).map_err(err_to_py)?;
        let list = PyList::empty(py);
        for a in availabilities {
            let dict = PyDict::new(py);
            dict.set_item("id", a.id)?;
            dict.set_item("catalogs", a.catalogs)?;
            list.append(dict)?;
        }
        Ok(list.unbind().into_any())
    }

    /// pystac-client compat: full STAC Collection JSONs across catalogs.
    /// For each unique collection id, fetches the full JSON from the first
    /// catalog that serves it. Can be slow — issues one HTTP request per
    /// distinct collection id.
    pub fn get_collections(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        let result = block_on(py, async move {
            let availabilities = engine.list_collections().await?;
            let mut out: Vec<serde_json::Value> = Vec::new();
            for a in availabilities {
                if let Some(cat) = a.catalogs.first() {
                    if let Some(c) = engine.describe_collection(cat, &a.id).await? {
                        let v = serde_json::to_value(&c).map_err(|e| {
                            SuperSTACError::SearchFailed(format!("serialize collection: {}", e))
                        })?;
                        out.push(v);
                    }
                }
            }
            Ok::<_, SuperSTACError>(out)
        })
        .map_err(err_to_py)?;
        pythonize_obj(py, &result)
    }

    /// pystac-client compat: full STAC Collection JSON for `id`. Searches
    /// every catalog that claims to serve it; returns the first hit. Raises
    /// `KeyError` if no catalog has it.
    pub fn get_collection(&self, py: Python<'_>, id: &str) -> PyResult<Py<PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        let id_owned = id.to_string();
        let id_for_err = id_owned.clone();
        let result = block_on(py, async move {
            let supporting = engine.catalogs_supporting(&id_owned).await?;
            for cat in supporting {
                if let Some(c) = engine.describe_collection(&cat, &id_owned).await? {
                    let v = serde_json::to_value(&c).map_err(|e| {
                        SuperSTACError::SearchFailed(format!("serialize collection: {}", e))
                    })?;
                    return Ok::<Option<serde_json::Value>, SuperSTACError>(Some(v));
                }
            }
            Ok(None)
        })
        .map_err(err_to_py)?;
        match result {
            Some(v) => pythonize_obj(py, &v),
            None => Err(PyKeyError::new_err(format!(
                "collection '{}' not found in any catalog",
                id_for_err
            ))),
        }
    }

    pub fn catalogs_supporting(
        &self,
        py: Python<'_>,
        collection_id: &str,
    ) -> PyResult<Vec<String>> {
        let engine = Arc::clone(&self.inner.engine);

        let id = collection_id.to_string();

        block_on(py, async move { engine.catalogs_supporting(&id).await }).map_err(err_to_py)
    }

    pub fn collections_by_catalog(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let engine = Arc::clone(&self.inner.engine);

        let m = block_on(py, async move { engine.collections_by_catalog().await })
            .map_err(err_to_py)?;

        pythonize_obj(py, &m)
    }
    /// Explicit single-catalog lookup. Returns the full STAC Collection
    /// JSON, or `None` if the catalog returns 404.
    pub fn describe_collection(
        &self,
        py: Python<'_>,
        catalog_id: &str,
        collection_id: &str,
    ) -> PyResult<Py<PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        let cat = catalog_id.to_string();
        let cid = collection_id.to_string();
        let result = block_on(
            py,
            async move { engine.describe_collection(&cat, &cid).await },
        )
        .map_err(err_to_py)?;
        match result {
            Some(c) => pythonize_obj(py, &c),
            None => Ok(py.None()),
        }
    }

    /// Download a complete scoped metadata inventory. Progress callbacks receive dicts.
    #[pyo3(signature = (catalog_id, output, *, progress = None, **kwargs))]
    fn ingest(
        &self,
        py: Python<'_>,
        catalog_id: &str,
        output: &str,
        progress: Option<Py<PyAny>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let job = crate::geoparquet::prepare(&self.inner, catalog_id, output, progress, kwargs)?;
        let result = block_on(py, crate::geoparquet::run(job))?;
        pythonize_obj(py, &result)
    }

    #[pyo3(signature = (dataset, *, apply = false))]
    fn cleanup_dataset(&self, py: Python<'_>, dataset: String, apply: bool) -> PyResult<Py<PyAny>> {
        let result = py.detach(move || crate::geoparquet::cleanup(&dataset, apply))?;
        pythonize_obj(py, &result)
    }

    #[pyo3(signature = (dataset, *, items_per_file = 10000, max_dataset_mib = 1024))]
    fn compact_dataset(
        &self,
        py: Python<'_>,
        dataset: String,
        items_per_file: usize,
        max_dataset_mib: u64,
    ) -> PyResult<Py<PyAny>> {
        let result = py.detach(move || {
            crate::geoparquet::compact(&dataset, items_per_file, max_dataset_mib)
        })?;
        pythonize_obj(py, &result)
    }

    fn __repr__(&self) -> String {
        let storage = self.inner.storage.lock();
        format!(
            "SuperSTACSyncClient(catalogs={}, started={}, providers={})",
            storage.list_catalogs(None).map(|c| c.len()).unwrap_or(0),
            self.inner.engine.started.load(Ordering::Relaxed),
            storage.list_providers(None).map(|p| p.len()).unwrap_or(0)
        )
    }
}
