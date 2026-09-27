use std::sync::Arc;

use parking_lot::Mutex;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use superstac_config::config::{CatalogConfig, CatalogProviderConfig};
use superstac_core::{
    models::{
        catalog::{Catalog, CatalogUpdate},
        provider::{CatalogProvider, CatalogProviderUpdate},
        settings::SettingsUpdate,
    },
    storages::factory::StorageBackend,
};
use superstac_engine::{SharedStorage, SuperSTACEngine};
use superstac_search::query::SearchQuery;
use crate::utils::{err_to_py, parse_storage_kind, pythonize_obj, depythonize_into, derive_id_from_url};

#[derive(Clone)]
pub struct Inner {
    pub storage: SharedStorage,
    pub engine: Arc<SuperSTACEngine>,
}

impl Inner {
    pub fn from_backend(backend: Box<dyn StorageBackend + Send + Sync>) -> Self {
        let storage: SharedStorage = Arc::new(Mutex::new(backend));
        let engine = SuperSTACEngine::from_shared(Arc::clone(&storage));
        Self {
            storage,
            engine: Arc::new(engine),
        }
    }

    pub fn from_storage_kind(kind: &str) -> PyResult<Self> {
        Ok(Self::from_backend(parse_storage_kind(kind)?.init()))
    }

    pub fn register_open_catalog(&self, url: &str, id: Option<&str>) -> PyResult<()> {
        let catalog_id = match id {
            Some(s) => s.to_string(),
            None => derive_id_from_url(url)?,
        };
        let catalog = Catalog::new(&catalog_id, None::<String>, url, None::<String>, None)
            .map_err(err_to_py)?;
        self.storage
            .lock()
            .create_catalog(catalog, None)
            .map_err(err_to_py)?;
        Ok(())
    }
}

pub fn add_catalog_impl(
    inner: &Inner,
    py: Python<'_>,
    catalog: Bound<'_, PyAny>,
    provider: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let cfg: CatalogConfig = depythonize_into(&catalog, "catalog")?;
    let model: Catalog = Catalog::try_from(cfg).map_err(err_to_py)?;
    let created = inner
        .storage
        .lock()
        .create_catalog(model, provider)
        .map_err(err_to_py)?;
    pythonize_obj(py, &created)
}

pub fn add_catalogs_impl(
    inner: &Inner,
    py: Python<'_>,
    catalogs: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let cfgs: Vec<CatalogConfig> = depythonize_into(&catalogs, "catalogs")?;
    let mut storage = inner.storage.lock();
    let list = PyList::empty(py);
    for cfg in cfgs {
        let provider_ref = cfg.provider.clone();
        let model: Catalog = Catalog::try_from(cfg).map_err(err_to_py)?;
        let created = storage
            .create_catalog(model, provider_ref.as_deref())
            .map_err(err_to_py)?;
        list.append(pythonize_obj(py, &created)?)?;
    }
    Ok(list.unbind().into_any())
}

pub fn add_providers_impl(
    inner: &Inner,
    py: Python<'_>,
    providers: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let cfgs: Vec<CatalogProviderConfig> = depythonize_into(&providers, "providers")?;
    let mut storage = inner.storage.lock();
    let list = PyList::empty(py);
    for cfg in cfgs {
        let model: CatalogProvider = CatalogProvider::try_from(cfg).map_err(err_to_py)?;
        let created = storage.create_provider(model).map_err(err_to_py)?;
        list.append(pythonize_obj(py, &created)?)?;
    }
    Ok(list.unbind().into_any())
}

/// Apply a YAML-shaped dict (`{catalogs, providers, settings}`) to an empty
/// backend. Every key is optional. `settings` is treated as a partial
/// `SettingsUpdate` so users only need to set the knobs they care about.
pub fn apply_config_dict(inner: &Inner, config: &Bound<'_, PyDict>) -> PyResult<()> {
    if let Some(settings) = config.get_item("settings")? {
        let parsed: SettingsUpdate = depythonize_into(&settings, "settings")?;
        inner.storage.lock().update_settings(parsed);
    }
    if let Some(providers) = config.get_item("providers")? {
        let cfgs: Vec<CatalogProviderConfig> = depythonize_into(&providers, "providers")?;
        let mut storage = inner.storage.lock();
        for cfg in cfgs {
            let p = CatalogProvider::try_from(cfg).map_err(err_to_py)?;
            storage.create_provider(p).map_err(err_to_py)?;
        }
    }
    if let Some(catalogs) = config.get_item("catalogs")? {
        let cfgs: Vec<CatalogConfig> = depythonize_into(&catalogs, "catalogs")?;
        let mut storage = inner.storage.lock();
        for cfg in cfgs {
            let provider_ref = cfg.provider.clone();
            let c = Catalog::try_from(cfg).map_err(err_to_py)?;
            storage
                .create_catalog(c, provider_ref.as_deref())
                .map_err(err_to_py)?;
        }
    }
    Ok(())
}

pub fn update_catalog_impl(
    inner: &Inner,
    py: Python<'_>,
    id: &str,
    update: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let parsed: CatalogUpdate = depythonize_into(&update, "catalog update")?;
    let updated = inner
        .storage
        .lock()
        .update_catalog(id, parsed)
        .map_err(err_to_py)?;
    pythonize_obj(py, &updated)
}

pub fn add_provider_impl(
    inner: &Inner,
    py: Python<'_>,
    provider: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let cfg: CatalogProviderConfig = depythonize_into(&provider, "provider")?;
    let model: CatalogProvider = CatalogProvider::try_from(cfg).map_err(err_to_py)?;
    let created = inner
        .storage
        .lock()
        .create_provider(model)
        .map_err(err_to_py)?;
    pythonize_obj(py, &created)
}

pub fn update_provider_impl(
    inner: &Inner,
    py: Python<'_>,
    id: &str,
    update: Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let parsed: CatalogProviderUpdate = depythonize_into(&update, "provider update")?;
    let updated = inner
        .storage
        .lock()
        .update_provider(id, parsed)
        .map_err(err_to_py)?;
    pythonize_obj(py, &updated)
}

pub fn update_settings_impl(inner: &Inner, update: Bound<'_, PyAny>) -> PyResult<()> {
    let parsed: SettingsUpdate = depythonize_into(&update, "settings update")?;
    inner.storage.lock().update_settings(parsed);
    Ok(())
}

pub fn search_query_from_kwargs(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<SearchQuery> {
    let kw = kwargs.ok_or_else(|| {
        PyValueError::new_err(
            "search() requires keyword arguments (e.g. collections=[...], bbox=[...])",
        )
    })?;
    depythonize_into(kw.as_any(), "search query")
}


#[allow(clippy::too_many_arguments)]
pub fn build_inner(
    config: Option<Bound<'_, PyDict>>,
    catalogs: Option<Bound<'_, PyAny>>,
    providers: Option<Bound<'_, PyAny>>,
    settings: Option<Bound<'_, PyAny>>,
    storage: &str,
) -> PyResult<Inner> {
    let inner = Inner::from_storage_kind(storage)?;

    if let Some(cfg) = config {
        apply_config_dict(&inner, &cfg)?;
        return Ok(inner);
    }

    let py = catalogs
        .as_ref()
        .map(|x| x.py())
        .or_else(|| providers.as_ref().map(|x| x.py()))
        .or_else(|| settings.as_ref().map(|x| x.py()));

    if let Some(py) = py {
        let cfg = PyDict::new(py);

        if let Some(c) = catalogs {
            cfg.set_item("catalogs", c)?;
        }

        if let Some(p) = providers {
            cfg.set_item("providers", p)?;
        }

        if let Some(s) = settings {
            cfg.set_item("settings", s)?;
        }

        apply_config_dict(&inner, &cfg)?;
    }

    Ok(inner)
}

