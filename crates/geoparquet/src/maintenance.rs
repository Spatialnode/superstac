//! Explicit maintenance. Publication is atomic; cleanup refuses active readers.
use crate::{
    budget, error,
    ingest::write_files,
    manifest::{atomic_json, resolve_file, DatasetManifest},
    visit_items,
};
use fs2::FileExt;
use serde::Serialize;
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use superstac_core::errors::SuperSTACError;

pub(crate) fn lock(root: &Path, name: &str, exclusive: bool) -> Result<File, SuperSTACError> {
    let file = if !exclusive && root.join(name).exists() {
        File::open(root.join(name)).map_err(error)?
    } else {
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join(name))
            .map_err(error)?
    };
    if exclusive {
        file.try_lock_exclusive()
    } else {
        FileExt::try_lock_shared(&file)
    }
    .map_err(|e| {
        error(format!(
            "dataset is in use; retry maintenance after readers/writers finish: {e}"
        ))
    })?;
    Ok(file)
}

#[derive(Debug, Serialize)]
pub struct CleanupReport {
    pub files: Vec<PathBuf>,
    pub bytes: u64,
    pub applied: bool,
}

/// Remove only unreferenced Parquet files. Checkpoint files remain protected.
/// Dry run by default; an exclusive reader lock prevents deleting live generations.
pub fn cleanup(root: impl AsRef<Path>, apply: bool) -> Result<CleanupReport, SuperSTACError> {
    let root = root.as_ref().canonicalize().map_err(error)?;
    let _writer = lock(&root, ".writer.lock", true)?;
    let _readers = lock(&root, ".readers.lock", apply)?;
    let manifest = if root.join("manifest.json").exists() {
        DatasetManifest::read(&root)?
    } else {
        DatasetManifest::default()
    };
    let mut referenced = HashSet::new();
    for snapshot in manifest.catalogs.values().flat_map(|s| s.values()) {
        for file in &snapshot.files {
            referenced.insert(resolve_file(&root, &file.path)?);
        }
    }
    let checkpoints = root.join("checkpoints");
    if checkpoints.exists() {
        for entry in fs::read_dir(checkpoints).map_err(error)? {
            let path = entry.map_err(error)?.path();
            if path.extension().is_some_and(|e| e == "json") {
                let value: serde_json::Value =
                    serde_json::from_reader(File::open(path).map_err(error)?).map_err(error)?;
                let files = value
                    .pointer("/snapshot/files")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| error("invalid checkpoint; cleanup aborted"))?;
                for file in files {
                    let path = file
                        .get("path")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| error("invalid checkpoint file path"))?;
                    referenced.insert(resolve_file(&root, Path::new(path))?);
                }
            }
        }
    }
    fn collect(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), SuperSTACError> {
        if !path.exists() {
            return Ok(());
        }
        for entry in fs::read_dir(path).map_err(error)? {
            let entry = entry.map_err(error)?;
            let kind = entry.file_type().map_err(error)?;
            if kind.is_symlink() {
                return Err(error("cleanup refuses symlinks"));
            }
            if kind.is_dir() {
                collect(&entry.path(), out)?;
            } else if entry.path().extension().is_some_and(|e| e == "parquet") {
                out.push(entry.path());
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    collect(&root.join("runs"), &mut files)?;
    files.retain(|f| !referenced.contains(f));
    files.sort();
    let bytes = files.iter().try_fold(0u64, |total, path| {
        Ok::<_, SuperSTACError>(total + fs::metadata(path).map_err(error)?.len())
    })?;
    if apply {
        for file in &files {
            fs::remove_file(file).map_err(error)?;
        }
    }
    Ok(CleanupReport {
        files: files
            .into_iter()
            .map(|p| p.strip_prefix(&root).unwrap().to_owned())
            .collect(),
        bytes,
        applied: apply,
    })
}

/// Rewrite each scope into larger bounded files and remove duplicate item IDs.
/// Newest occurrence wins. Old files are retained until explicit cleanup.
pub fn compact(
    root: impl AsRef<Path>,
    items_per_file: usize,
    max_dataset_bytes: Option<u64>,
) -> Result<DatasetManifest, SuperSTACError> {
    if items_per_file == 0 || items_per_file > 1_000_000 {
        return Err(error("items_per_file must be 1..=1000000"));
    }
    let root = root.as_ref().canonicalize().map_err(error)?;
    let _writer = lock(&root, ".writer.lock", true)?;
    let mut manifest = DatasetManifest::read(&root)?;
    fs::create_dir_all(root.join("runs")).map_err(error)?;
    let mut remaining = max_dataset_bytes
        .unwrap_or(u64::MAX)
        .checked_sub(budget::used_bytes(&root.join("runs"))?)
        .ok_or_else(|| error("existing files exceed storage budget"))?;
    let run = tempfile::Builder::new()
        .prefix("compact-")
        .tempdir_in(root.join("runs"))
        .map_err(error)?
        .keep();
    #[cfg(unix)]
    File::open(root.join("runs"))
        .and_then(|f| f.sync_all())
        .map_err(error)?;
    for snapshot in manifest.catalogs.values_mut().flat_map(|s| s.values_mut()) {
        let mut rows = Vec::new();
        let mut files = Vec::new();
        let mut seen = HashSet::new();
        for file in &snapshot.files {
            visit_items(
                &resolve_file(&root, &file.path)?,
                None,
                &AtomicBool::new(false),
                |item| {
                    if seen.insert((item.collection.clone(), item.id.clone())) {
                        rows.push(item);
                    }
                    if rows.len() >= items_per_file {
                        let written =
                            write_files(&root, &run, std::mem::take(&mut rows), remaining)?;
                        remaining =
                            remaining.saturating_sub(written.iter().map(|f| f.bytes).sum::<u64>());
                        files.extend(written);
                    }
                    Ok(true)
                },
            )?;
        }
        if !rows.is_empty() {
            let written = write_files(&root, &run, rows, remaining)?;
            remaining = remaining.saturating_sub(written.iter().map(|f| f.bytes).sum::<u64>());
            files.extend(written);
        }
        snapshot.overlays = false;
        snapshot.items = files.iter().map(|f| f.items).sum();
        snapshot.files = files;
    }
    atomic_json(&root.join("manifest.json"), &manifest)?;
    Ok(manifest)
}
