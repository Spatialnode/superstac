use std::sync::Arc;
use std::sync::atomic::Ordering;

use pyo3::exceptions::{PyKeyError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};


use superstac_config::init_from_yaml;

use crate::search::PySearch;
use crate::shared::{
    Inner, add_catalog_impl, add_catalogs_impl, add_provider_impl, add_providers_impl,  build_inner, search_query_from_kwargs, update_catalog_impl, update_provider_impl, update_settings_impl
};
use crate::utils::{err_to_py, parse_storage_kind, pythonize_obj};
use crate::runtime::into_py;


#[pyclass(name = "AsyncClient", module = "superstac._superstac")]
pub struct PyAsyncClient {
    inner: Inner,
}

#[pymethods]
impl PyAsyncClient {
    /// Build an async client. `config` mirrors the YAML shape (see
    /// [`Client.__init__`]); every key is optional.
    #[new]
    #[pyo3(signature = (
        config = None,
        *,
        catalogs = None,
        providers = None,
        settings = None,
        storage = "memory"
    ))]
    fn new(
        config: Option<Bound<'_, PyDict>>,
        catalogs: Option<Bound<'_, PyAny>>,
        providers: Option<Bound<'_, PyAny>>,
        settings: Option<Bound<'_, PyAny>>,
        storage: &str,
    ) -> PyResult<Self> {
        Ok(Self {
            inner: build_inner(
                config,
                catalogs,
                providers,
                settings,
                storage,
            )?,
        })
    }
    

    /// pystac-client compat: connect to a single STAC catalog at `url`.
    /// Async: returns a coroutine that resolves to a fully-started client.
    #[classmethod]
    #[pyo3(signature = (url, *, id = None, storage = "memory"))]
    pub fn open<'py>(
        _cls: &Bound<'_, PyType>,
        py: Python<'py>,
        url: &str,
        id: Option<&str>,
        storage: &str,
    ) -> PyResult<Bound<'py, PyAny>> {
        let inner = Inner::from_storage_kind(storage)?;
        inner.register_open_catalog(url, id)?;
        let engine = Arc::clone(&inner.engine);
        into_py(py, async move {
            engine.start().await.map_err(err_to_py)?;
            Python::attach(|py| Py::new(py, PyAsyncClient { inner }))
        })
    }

    #[classmethod]
    #[pyo3(signature = (yaml_path, *, storage = "memory"))]
    pub fn from_yaml(_cls: &Bound<'_, PyType>, yaml_path: &str, storage: &str) -> PyResult<Self> {
        let backend_kind = parse_storage_kind(storage)?;
        let backend = init_from_yaml(backend_kind, yaml_path).map_err(err_to_py)?;
        Ok(Self {
            inner: Inner::from_backend(backend),
        })
    }

  
    fn start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move { engine.start().await.map_err(err_to_py) })
    }

    fn shutdown<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            engine.shutdown().await;
            Ok(())
        })
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

    #[pyo3(signature = (**kwargs))]
    fn search<'py>(
        &self,
        py: Python<'py>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let query = search_query_from_kwargs(kwargs)?;
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            let resp = engine.search(query).await.map_err(err_to_py)?;
            Python::attach(|py| Py::new(py, PySearch { response: resp, cached_items: None }))
        })
    }

    fn list_collections<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            let availabilities = engine.list_collections().await.map_err(err_to_py)?;
            Python::attach(|py| -> PyResult<Py<PyAny>> {
                let list = PyList::empty(py);
                for a in availabilities {
                    let dict = PyDict::new(py);
                    dict.set_item("id", a.id)?;
                    dict.set_item("catalogs", a.catalogs)?;
                    list.append(dict)?;
                }
                Ok(list.unbind().into_any())
            })
        })
    }

    fn get_collections<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            let availabilities = engine.list_collections().await.map_err(err_to_py)?;
            let mut out: Vec<serde_json::Value> = Vec::new();
            for a in availabilities {
                if let Some(cat) = a.catalogs.first() {
                    if let Some(c) = engine
                        .describe_collection(cat, &a.id)
                        .await
                        .map_err(err_to_py)?
                    {
                        let v = serde_json::to_value(&c).map_err(|e| {
                            PyRuntimeError::new_err(format!("serialize collection: {}", e))
                        })?;
                        out.push(v);
                    }
                }
            }
            Python::attach(|py| pythonize_obj(py, &out))
        })
    }

    fn get_collection<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            let supporting = engine.catalogs_supporting(&id).await.map_err(err_to_py)?;
            for cat in supporting {
                if let Some(c) = engine
                    .describe_collection(&cat, &id)
                    .await
                    .map_err(err_to_py)?
                {
                    return Python::attach(|py| pythonize_obj(py, &c));
                }
            }
            Err(PyKeyError::new_err(format!(
                "collection '{}' not found in any catalog",
                id
            )))
        })
    }

    fn catalogs_supporting<'py>(
        &self,
        py: Python<'py>,
        collection_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            engine
                .catalogs_supporting(&collection_id)
                .await
                .map_err(err_to_py)
        })
    }

    fn collections_by_catalog<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            engine.collections_by_catalog().await.map_err(err_to_py)
        })
    }

    fn describe_collection<'py>(
        &self,
        py: Python<'py>,
        catalog_id: String,
        collection_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let engine = Arc::clone(&self.inner.engine);
        into_py(py, async move {
            let result = engine
                .describe_collection(&catalog_id, &collection_id)
                .await
                .map_err(err_to_py)?;
            Python::attach(|py| -> PyResult<Py<PyAny>> {
                match result {
                    Some(c) => pythonize_obj(py, &c),
                    None => Ok(py.None()),
                }
            })
        })
    }

    fn __repr__(&self) -> String {
        let storage = self.inner.storage.lock();
        format!(
            "SuperSTACAsyncClient(catalogs={}, started={}, providers={})",
                storage
                .list_catalogs(None)
                .map(|c| c.len())
                .unwrap_or(0),
            self.inner.engine.started.load(Ordering::Relaxed),
                storage
                .list_providers(None)
                .map(|p| p.len())
                .unwrap_or(0)
        )
    }
}
