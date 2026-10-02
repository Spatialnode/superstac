//! Python adapters keep blocking IO and HTTP outside the GIL.
use crate::shared::Inner;
#[cfg(feature = "geoparquet")]
use crate::utils::{depythonize_into, err_to_py, pythonize_obj};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};
use serde_json::Value;

#[cfg(feature = "geoparquet")]
pub struct Job(
    superstac_core::models::catalog::Catalog,
    superstac_geoparquet::ingest::IngestOptions,
);
#[cfg(not(feature = "geoparquet"))]
pub struct Job;
#[cfg(not(feature = "geoparquet"))]
fn unavailable() -> PyErr {
    PyValueError::new_err("rebuild superstac with the geoparquet feature")
}

pub fn prepare(
    inner: &Inner,
    catalog_id: &str,
    output: &str,
    progress: Option<Py<PyAny>>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Job> {
    #[cfg(feature = "geoparquet")]
    {
        #[derive(serde::Deserialize, Default)]
        #[serde(default, deny_unknown_fields)]
        struct Options {
            collections: Vec<String>,
            bbox: Option<Value>,
            datetime: Option<String>,
            name: Option<String>,
            resume: bool,
            incremental_since: Option<String>,
            page_size: Option<usize>,
            items_per_file: Option<usize>,
            max_dataset_mib: Option<u64>,
            timeout_seconds: Option<u64>,
            all: bool,
        }
        let input: Options = match kwargs {
            Some(kw) => depythonize_into(kw.as_any(), "ingest options")?,
            None => Options::default(),
        };
        let filtered =
            !input.collections.is_empty() || input.bbox.is_some() || input.datetime.is_some();
        if input.all == filtered {
            return Err(PyValueError::new_err(
                "provide collections, bbox or datetime, or explicitly all=True",
            ));
        }
        let scope = superstac_geoparquet::manifest::IngestScope {
            collections: input.collections,
            datetime: input.datetime,
            bbox: input
                .bbox
                .map(serde_json::from_value)
                .transpose()
                .map_err(|e| PyValueError::new_err(e.to_string()))?,
        };
        let mut options = superstac_geoparquet::ingest::IngestOptions::new(output, scope);
        options.name = input.name;
        options.resume = input.resume;
        options.incremental_since = input.incremental_since;
        if let Some(n) = input.page_size {
            options.page_size = n;
        }
        if let Some(n) = input.items_per_file {
            options.items_per_file = n;
        }
        if let Some(n) = input.timeout_seconds {
            options.request_timeout = std::time::Duration::from_secs(n);
        }
        options.max_dataset_bytes = budget(input.max_dataset_mib.unwrap_or(1024))?;
        if let Some(callback) = progress {
            Python::attach(|py| {
                if callback.bind(py).is_callable() {
                    Ok(())
                } else {
                    Err(PyValueError::new_err("progress must be callable"))
                }
            })?;
            options.progress = Some(std::sync::Arc::new(move |event| {
                Python::attach(|py| {
                    let result =
                        pythonize_obj(py, event).and_then(|event| callback.call1(py, (event,)));
                    if let Err(error) = result {
                        error.write_unraisable(py, Some(callback.bind(py)));
                    }
                })
            }));
        }
        let catalog = inner
            .storage
            .lock()
            .get_catalog(catalog_id)
            .map_err(err_to_py)?
            .clone();
        Ok(Job(catalog, options))
    }
    #[cfg(not(feature = "geoparquet"))]
    {
        let _ = (inner, catalog_id, output, progress, kwargs);
        Err(unavailable())
    }
}
#[cfg(feature = "geoparquet")]
fn budget(mib: u64) -> PyResult<Option<u64>> {
    if mib == 0 {
        Ok(None)
    } else {
        mib.checked_mul(1048576)
            .map(Some)
            .ok_or_else(|| PyValueError::new_err("budget too large"))
    }
}
pub async fn run(job: Job) -> PyResult<Value> {
    #[cfg(feature = "geoparquet")]
    {
        let result = superstac_geoparquet::ingest::ingest_catalog(&job.0, job.1)
            .await
            .map_err(err_to_py)?;
        serde_json::to_value(result).map_err(|e| PyValueError::new_err(e.to_string()))
    }
    #[cfg(not(feature = "geoparquet"))]
    {
        let _ = job;
        Err(unavailable())
    }
}
pub fn cleanup(dataset: &str, apply: bool) -> PyResult<Value> {
    #[cfg(feature = "geoparquet")]
    {
        serde_json::to_value(
            superstac_geoparquet::maintenance::cleanup(dataset, apply).map_err(err_to_py)?,
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))
    }
    #[cfg(not(feature = "geoparquet"))]
    {
        let _ = (dataset, apply);
        Err(unavailable())
    }
}
pub fn compact(dataset: &str, items: usize, mib: u64) -> PyResult<Value> {
    #[cfg(feature = "geoparquet")]
    {
        serde_json::to_value(
            superstac_geoparquet::maintenance::compact(dataset, items, budget(mib)?)
                .map_err(err_to_py)?,
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))
    }
    #[cfg(not(feature = "geoparquet"))]
    {
        let _ = (dataset, items, mib);
        Err(unavailable())
    }
}
