//! Published dataset metadata. Only completed snapshots appear in the manifest.
use std::{
    collections::BTreeMap,
    fs::File,
    io::Write,
    path::{Component, Path, PathBuf},
};

use crate::error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stac::{api::Search, Bbox};
use superstac_core::errors::SuperSTACError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct IngestScope {
    /// Source-local collection IDs in stored manifests; canonical IDs on ingest input.
    pub collections: Vec<String>,
    pub bbox: Option<Bbox>,
    pub datetime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFile {
    /// Relative to the dataset directory.
    pub path: PathBuf,
    pub collection: Option<String>,
    pub items: u64,
    pub bytes: u64,
    pub bbox: Option<Bbox>,
    pub datetime_min: Option<DateTime<Utc>>,
    pub datetime_max: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default = "legacy_name")]
    pub name: String,
    pub catalog_id: String,
    pub source_url: String,
    pub scope: IngestScope,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    /// True means pagination exhausted for the scope, not a provider-wide archive.
    pub complete: bool,
    /// Delta overlays require newest-first deduplication before filtering.
    #[serde(default)]
    pub overlays: bool,
    pub pages: u64,
    pub items: u64,
    pub files: Vec<DataFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetManifest {
    pub version: u32,
    pub catalogs: BTreeMap<String, BTreeMap<String, Snapshot>>,
}

impl Default for DatasetManifest {
    fn default() -> Self {
        Self {
            version: 2,
            catalogs: BTreeMap::new(),
        }
    }
}

impl DatasetManifest {
    pub fn read(root: &Path) -> Result<Self, SuperSTACError> {
        let value: serde_json::Value =
            serde_json::from_reader(File::open(root.join("manifest.json")).map_err(error)?)
                .map_err(error)?;
        let manifest: Self = match value.get("version").and_then(|v| v.as_u64()) {
            Some(1) => {
                #[derive(Deserialize)]
                struct Legacy {
                    catalogs: BTreeMap<String, Snapshot>,
                }
                let old: Legacy = serde_json::from_value(value).map_err(error)?;
                Self {
                    version: 2,
                    catalogs: old
                        .catalogs
                        .into_iter()
                        .map(|(id, mut snapshot)| {
                            snapshot.name = legacy_name();
                            (id, BTreeMap::from([(snapshot.name.clone(), snapshot)]))
                        })
                        .collect(),
                }
            }
            Some(2) => serde_json::from_value(value).map_err(error)?,
            _ => return Err(error("unsupported dataset manifest version")),
        };
        for (id, scopes) in &manifest.catalogs {
            for (name, snapshot) in scopes {
                if id != &snapshot.catalog_id
                    || name != &snapshot.name
                    || !snapshot.complete
                    || snapshot.completed_at.is_none()
                {
                    return Err(error(
                        "manifest contains an incomplete or inconsistent snapshot",
                    ));
                }
            }
        }
        Ok(manifest)
    }
}

impl Snapshot {
    /// Conservatively reject a query whose completeness cannot be established.
    pub(crate) fn ensure_coverage(&self, search: &Search) -> Result<(), SuperSTACError> {
        let fail = || {
            error(format!(
            "query exceeds recorded coverage for catalog '{}'; narrow the query or ingest a broader scope",
            self.catalog_id
        ))
        };
        if !self.scope.collections.is_empty()
            && (search.collections.is_empty()
                || search
                    .collections
                    .iter()
                    .any(|c| !self.scope.collections.contains(c)))
        {
            return Err(fail());
        }
        if let Some(bounds) = self.scope.bbox {
            let requested = if let Some(bbox) = search.items.bbox {
                Some(bbox)
            } else if let Some(geometry) = &search.intersects {
                let mut item = stac::Item::new("coverage");
                item.set_geometry(geometry.clone()).map_err(error)?;
                item.bbox
            } else {
                None
            };
            let Some(requested) = requested else {
                return Err(fail());
            };
            // Ingestion currently accepts 2D, non-antimeridian bounds only.
            if requested.xmin() < bounds.xmin()
                || requested.ymin() < bounds.ymin()
                || requested.xmax() > bounds.xmax()
                || requested.ymax() > bounds.ymax()
            {
                return Err(fail());
            }
        }
        if let Some(datetime) = &self.scope.datetime {
            let Some(requested) = &search.items.datetime else {
                return Err(fail());
            };
            let (start, end) = stac::datetime::parse(datetime).map_err(error)?;
            let (qstart, qend) = stac::datetime::parse(requested).map_err(error)?;
            if start.is_some_and(|s| qstart.is_none_or(|q| q < s))
                || end.is_some_and(|e| qend.is_none_or(|q| q > e))
            {
                return Err(fail());
            }
        }
        Ok(())
    }
}

impl DataFile {
    pub(crate) fn may_match(&self, search: &Search) -> Result<bool, SuperSTACError> {
        if !search.collections.is_empty()
            && self
                .collection
                .as_ref()
                .is_none_or(|c| !search.collections.contains(c))
        {
            return Ok(false);
        }
        if let (Some(bounds), Some(q)) = (self.bbox, search.items.bbox) {
            if q.xmin() > bounds.xmax()
                || q.xmax() < bounds.xmin()
                || q.ymin() > bounds.ymax()
                || q.ymax() < bounds.ymin()
            {
                return Ok(false);
            }
        }
        if let Some(datetime) = &search.items.datetime {
            let (start, end) = stac::datetime::parse(datetime).map_err(error)?;
            if start.is_some_and(|s| self.datetime_max.is_some_and(|m| m < s))
                || end.is_some_and(|e| self.datetime_min.is_some_and(|m| m > e))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

pub(crate) fn resolve_file(root: &Path, relative: &Path) -> Result<PathBuf, SuperSTACError> {
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(error(
            "manifest file paths must stay within the dataset directory",
        ));
    }
    let path = root.join(relative).canonicalize().map_err(error)?;
    if !path.starts_with(root.canonicalize().map_err(error)?) || !path.is_file() {
        return Err(error(
            "manifest references a file outside the dataset or a non-file",
        ));
    }
    Ok(path)
}

pub(crate) fn atomic_json(path: &Path, value: &impl Serialize) -> Result<(), SuperSTACError> {
    let parent = path
        .parent()
        .ok_or_else(|| error("missing parent directory"))?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(error)?;
    serde_json::to_writer_pretty(temp.as_file_mut(), value).map_err(error)?;
    temp.as_file_mut().write_all(b"\n").map_err(error)?;
    temp.as_file().sync_all().map_err(error)?;
    temp.persist(path).map_err(error)?;
    #[cfg(unix)]
    File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(error)?;
    Ok(())
}

fn legacy_name() -> String {
    "legacy".to_owned()
}
