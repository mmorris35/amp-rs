# Phase 6: File Watcher

**Goal**: notify-rs file watcher for automatic incremental indexing, plus on-demand indexing tools
**Duration**: 2 days
**Wave**: 3 (parallel with Phase 7 and Phase 8)
**Dependencies**: Phase 5 (code indexing) complete

---

## Task 6.1: File Watcher Core

**Git**: `git checkout -b feature/6-1-file-watcher`

### Subtask 6.1.1: notify-rs Watcher Setup (Single Session)

**Prerequisites**:
- [x] 5.2.2: Code Indexing Integration Tests

**Deliverables**:
- [ ] Create `src/watcher/mod.rs`:

```rust
pub mod handler;

use crate::error::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tracing::{info, warn};

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: mpsc::Receiver<Vec<DebouncedEvent>>,
}

impl FileWatcher {
    /// Create a new file watcher for the given directories
    pub fn new(watch_dirs: &[PathBuf]) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            move |events: Result<Vec<DebouncedEvent>, _>| {
                if let Ok(events) = events {
                    let _ = tx.send(events);
                }
            },
        ).map_err(|e| crate::error::AmpError::Indexing(format!("Watcher error: {}", e)))?;

        for dir in watch_dirs {
            if dir.exists() {
                debouncer.watcher().watch(dir, RecursiveMode::Recursive)
                    .map_err(|e| crate::error::AmpError::Indexing(
                        format!("Failed to watch {:?}: {}", dir, e)
                    ))?;
                info!("Watching directory: {:?}", dir);
            } else {
                warn!("Watch directory does not exist: {:?}", dir);
            }
        }

        Ok(Self {
            _watcher: debouncer.into(),
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
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(watcher): notify-rs file watcher with debouncing"
```

**Success Criteria**:
- [ ] Watcher watches multiple directories
- [ ] Events debounced (2 second window)
- [ ] Non-existent directories warned, not errored

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 6.1.2: Incremental Indexing (mtime-based diff) (Single Session)

**Prerequisites**:
- [x] 6.1.1: notify-rs Watcher Setup

**Deliverables**:
- [ ] Create `src/watcher/handler.rs`:

```rust
use crate::error::Result;
use crate::indexing::scanner;
use crate::indexing::storage::IndexingStorage;
use rusqlite::Connection;
use std::path::Path;
use tracing::{debug, info};

/// Diff index — only re-index files with changed mtime
pub fn diff_index(conn: &Connection, repo_path: &Path) -> Result<DiffResult> {
    let storage = IndexingStorage::new(conn);
    let scanned = scanner::scan_directory(repo_path)?;

    let mut new_files = Vec::new();
    let mut changed_files = Vec::new();
    let mut deleted_files = Vec::new();

    for file in &scanned {
        let file_path_str = file.path.to_string_lossy().to_string();
        match storage.get_indexed_file(&file_path_str)? {
            Some(indexed) => {
                if file.mtime > indexed.mtime as u64 {
                    changed_files.push(file.clone());
                }
            }
            None => {
                new_files.push(file.clone());
            }
        }
    }

    // Find deleted files
    let indexed_paths = storage.list_indexed_files(repo_path)?;
    let scanned_paths: std::collections::HashSet<String> = scanned
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    for indexed_path in indexed_paths {
        if !scanned_paths.contains(&indexed_path) {
            deleted_files.push(indexed_path);
        }
    }

    info!(
        "Diff index for {:?}: {} new, {} changed, {} deleted",
        repo_path,
        new_files.len(),
        changed_files.len(),
        deleted_files.len()
    );

    Ok(DiffResult {
        new_files,
        changed_files,
        deleted_files,
    })
}

#[derive(Debug)]
pub struct DiffResult {
    pub new_files: Vec<crate::indexing::ScannedFile>,
    pub changed_files: Vec<crate::indexing::ScannedFile>,
    pub deleted_files: Vec<String>,
}

/// Full reindex — clear all indexed data for a path and re-index
pub fn full_reindex(conn: &Connection, repo_path: &Path) -> Result<usize> {
    let storage = IndexingStorage::new(conn);

    // Delete all chunks and file records for this repo
    let repo_str = repo_path.to_string_lossy().to_string();
    storage.delete_repo_data(&repo_str)?;

    info!("Cleared index for {:?}, starting full re-index", repo_path);

    // Re-scan and index everything
    let files = scanner::scan_directory(repo_path)?;
    Ok(files.len())
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(watcher): incremental indexing with mtime-based diff"
```

**Success Criteria**:
- [ ] diff_index detects new, changed, and deleted files
- [ ] full_reindex clears and rescans
- [ ] mtime comparison works correctly

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 6.1 Complete — Squash Merge
- [ ] All subtasks 6.1.1–6.1.2 complete
- [ ] `cargo check` passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/6-1-file-watcher
git commit -m "feat(watcher): complete task 6.1 - file watcher and incremental indexing"
git branch -d feature/6-1-file-watcher
git push origin main
```

---

## Task 6.2: On-Demand Tools and Tests

**Git**: `git checkout -b feature/6-2-indexing-tools`

### Subtask 6.2.1: On-Demand Indexing Tools (Single Session)

**Prerequisites**:
- [x] 6.1.2: Incremental Indexing

**Deliverables**:
- [ ] Create coordinating functions for MCP tools:
  - `index_repo(path)` — full initial index of a repository
  - `diff_index(path)` — incremental re-index
  - `full_reindex(path)` — nuclear clear + re-index
- [ ] All functions return structured results (files indexed, time taken, errors)
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(watcher): on-demand indexing tool functions"
```

**Success Criteria**:
- [ ] Three indexing modes work
- [ ] Structured result types

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 6.2.2: File Watcher Integration Tests (Single Session)

**Prerequisites**:
- [x] 6.2.1: On-Demand Indexing Tools

**Deliverables**:
- [ ] Unit tests for diff_index (using tempdir with files at known mtimes)
- [ ] Unit tests for full_reindex
- [ ] Tests for scanner .gitignore respect (create tempdir with .gitignore)
- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test(watcher): integration tests for file watcher and indexing tools"
```

**Success Criteria**:
- [ ] All watcher tests pass
- [ ] Full verification passes

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 6.2 Complete — Squash Merge
- [ ] All subtasks 6.2.1–6.2.2 complete
- [ ] Full verification passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/6-2-indexing-tools
git commit -m "feat(watcher): complete task 6.2 - on-demand tools and tests"
git branch -d feature/6-2-indexing-tools
git push origin main
```

---

*Phase 6 complete when both tasks merged to main.*
