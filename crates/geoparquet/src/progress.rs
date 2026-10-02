//! Application-neutral ingestion events. Callbacks must return promptly.
use crate::manifest::Snapshot;
use serde::Serialize;
use std::{sync::Arc, time::Instant};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestPhase {
    Started,
    Fetching,
    Writing,
    Checkpointed,
    Retrying,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct IngestProgress {
    pub catalog_id: String,
    pub phase: IngestPhase,
    pub items_received: u64,
    pub items_saved: u64,
    pub pages: u64,
    pub files: usize,
    pub bytes_saved: u64,
    /// Optional provider estimate; it may change while paginating.
    pub total_items: Option<u64>,
    pub elapsed_seconds: f64,
    pub items_per_second: f64,
    pub message: Option<String>,
}

pub type ProgressCallback = Arc<dyn Fn(&IngestProgress) + Send + Sync>;

pub(crate) struct Progress {
    callback: Option<ProgressCallback>,
    started: Instant,
    initial_items: u64,
    pub event: IngestProgress,
}

impl Progress {
    pub fn new(id: &str, callback: Option<ProgressCallback>) -> Self {
        Self {
            callback,
            started: Instant::now(),
            initial_items: 0,
            event: IngestProgress {
                catalog_id: id.into(),
                phase: IngestPhase::Started,
                items_received: 0,
                items_saved: 0,
                pages: 0,
                files: 0,
                bytes_saved: 0,
                total_items: None,
                elapsed_seconds: 0.0,
                items_per_second: 0.0,
                message: None,
            },
        }
    }
    pub fn resume(&mut self, snapshot: &Snapshot) {
        self.saved(snapshot);
        self.event.items_received = snapshot.items;
        self.event.pages = snapshot.pages;
        self.initial_items = snapshot.items;
    }
    pub fn saved(&mut self, snapshot: &Snapshot) {
        self.event.items_saved = snapshot.items;
        self.event.files = snapshot.files.len();
        self.event.bytes_saved = snapshot.files.iter().map(|f| f.bytes).sum();
    }
    pub fn emit(&mut self, phase: IngestPhase, message: Option<String>) {
        self.event.phase = phase;
        self.event.message = message;
        self.event.elapsed_seconds = self.started.elapsed().as_secs_f64();
        self.event.items_per_second = self.event.items_received.saturating_sub(self.initial_items)
            as f64
            / self.event.elapsed_seconds.max(0.001);
        if let Some(callback) = &self.callback {
            callback(&self.event);
        }
    }
}
