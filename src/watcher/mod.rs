pub mod handler;
pub mod service;

use crate::error::Result;
use notify::RecommendedWatcher;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tracing::{info, warn};

pub struct FileWatcher {
    _debouncer: Debouncer<RecommendedWatcher>,
    receiver: mpsc::Receiver<Vec<DebouncedEvent>>,
}

impl FileWatcher {
    /// Create a new file watcher for the given directories
    pub fn new(watch_dirs: &[PathBuf]) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            move |events: std::result::Result<Vec<DebouncedEvent>, _>| {
                if let Ok(events) = events {
                    let _ = tx.send(events);
                }
            },
        )
        .map_err(|e| crate::error::AmpError::Indexing(format!("Watcher error: {}", e)))?;

        for dir in watch_dirs {
            if dir.exists() {
                debouncer
                    .watcher()
                    .watch(dir, RecursiveMode::Recursive)
                    .map_err(|e| {
                        crate::error::AmpError::Indexing(format!(
                            "Failed to watch {:?}: {}",
                            dir, e
                        ))
                    })?;
                info!("Watching directory: {:?}", dir);
            } else {
                warn!("Watch directory does not exist: {:?}", dir);
            }
        }

        Ok(Self {
            _debouncer: debouncer,
            receiver: rx,
        })
    }

    /// Get the next batch of file change events (blocking)
    pub fn next_events(&self) -> Option<Vec<DebouncedEvent>> {
        self.receiver.recv().ok()
    }

    /// Try to get events without blocking
    pub fn try_next_events(&self) -> Option<Vec<DebouncedEvent>> {
        self.receiver.try_recv().ok()
    }
}
