//! A cap on Parquet bytes retained under runs/, including old and partial generations.
use crate::error;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};
use superstac_core::errors::SuperSTACError;

pub(crate) fn used_bytes(directory: &Path) -> Result<u64, SuperSTACError> {
    let mut total = 0u64;
    for entry in fs::read_dir(directory).map_err(error)? {
        let entry = entry.map_err(error)?;
        let kind = entry.file_type().map_err(error)?;
        if kind.is_symlink() {
            return Err(error("dataset run directories must not contain symlinks"));
        }
        let size = if kind.is_dir() {
            used_bytes(&entry.path())?
        } else {
            entry.metadata().map_err(error)?.len()
        };
        total = total
            .checked_add(size)
            .ok_or_else(|| error("dataset byte count overflow"))?;
    }
    Ok(total)
}

pub(crate) struct BudgetWriter<'a> {
    file: &'a File,
    remaining: &'a mut u64,
}
impl<'a> BudgetWriter<'a> {
    pub fn new(file: &'a File, remaining: &'a mut u64) -> Self {
        Self { file, remaining }
    }
}
impl Write for BudgetWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() as u64 > *self.remaining {
            return Err(io::Error::other(
                "dataset storage budget exceeded; raise --max-dataset-mib and resume",
            ));
        }
        let count = self.file.write(bytes)?;
        *self.remaining -= count as u64;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}
