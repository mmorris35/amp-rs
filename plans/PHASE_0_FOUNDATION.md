# Phase 0: Foundation

**Goal**: Initialize Rust project with dependencies, module skeleton, linting, and testing infrastructure
**Duration**: 1–2 days
**Wave**: 0 (sequential — must complete before all other phases)

---

## Task 0.1: Project Initialization

**Git**: `git checkout -b feature/0-1-project-init`

### Subtask 0.1.1: Initialize Cargo Project (Single Session)

**Prerequisites**: None (first subtask)

**Deliverables**:
- [x] Run `cargo init --name amp-rs` in project root
- [x] Create `.gitignore` with Rust patterns:

```gitignore
/target
**/*.rs.bk
*.pdb
.env
*.db
*.db-wal
*.db-shm
models/*.onnx
```

- [x] Create `LICENSE` file (MIT)
- [x] Verify `cargo build` succeeds with default hello world
- [x] Git commit:
```bash
git add -A && git commit -m "chore: initialize Cargo project"
```

**Success Criteria**:
- [x] `cargo build` exits 0
- [x] `cargo run` prints "Hello, world!"
- [x] `.gitignore` exists with Rust patterns
- [x] `LICENSE` exists

**Completion Notes**:
- **Implementation**: Ran cargo init, created .gitignore with Rust patterns, created MIT LICENSE file, verified build and run
- **Files Created**: .gitignore (8 lines), LICENSE (21 lines), Cargo.toml, Cargo.lock, src/main.rs (3 lines)
- **Files Modified**: None
- **Tests**: N/A
- **Build**: PASS - cargo build and cargo run both succeed
- **Notes**: All documentation files were already present from setup, cargo init created base structure

---

### Subtask 0.1.2: Configure Dependencies in Cargo.toml (Single Session)

**Prerequisites**:
- [x] 0.1.1: Initialize Cargo Project

**Deliverables**:
- [x] Update `Cargo.toml` with all MVP dependencies:

```toml
[package]
name = "amp-rs"
version = "0.1.0"
edition = "2021"
description = "Reference implementation of Agent Memory Protocol (AMP)"
license = "MIT"
repository = "https://github.com/mmorris35/amp-rs"

[dependencies]
# MCP Protocol
rmcp = { version = "0.1", features = ["server", "transport-io"] }

# Async Runtime
tokio = { version = "1", features = ["full"] }

# HTTP Server
axum = "0.7"
tower-http = { version = "0.5", features = ["cors"] }

# Database
rusqlite = { version = "0.31", features = ["bundled", "vtab"] }

# Embeddings
ort = "2"
tokenizers = "0.19"
ndarray = "0.15"

# File Watching
notify = "6"
notify-debouncer-mini = "0.4"

# CLI
clap = { version = "4", features = ["derive"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Utilities
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
ignore = "0.4"  # .gitignore support (from ripgrep)
walkdir = "2"
tempfile = "3"

[dev-dependencies]
tempfile = "3"
tokio-test = "0.4"
assert_cmd = "2"
predicates = "3"
```

- [x] Run `cargo check` to verify dependency resolution
- [x] Git commit:
```bash
git add -A && git commit -m "chore: configure all MVP dependencies in Cargo.toml"
```

**Success Criteria**:
- [x] `cargo check` exits 0 (all deps resolve)
- [x] No version conflicts

**Completion Notes**:
- **Implementation**: Updated Cargo.toml with all MVP dependencies (rmcp, tokio, axum, rusqlite, ort, tokenizers, ndarray, notify, clap, serde, chrono, uuid, etc.). Fixed ort version to 2.0.0-rc.11 to resolve availability issue.
- **Files Created**: None
- **Files Modified**: Cargo.toml (55 lines), Cargo.lock (3055 lines)
- **Tests**: N/A
- **Build**: PASS - cargo check resolves all dependencies without conflicts
- **Notes**: ort crate is still in RC phase, specified explicit version 2.0.0-rc.11 instead of "2"

---

### Subtask 0.1.3: Create Module Skeleton (Single Session)

**Prerequisites**:
- [x] 0.1.2: Configure Dependencies

**Deliverables**:
- [x] Create module directory structure:

```bash
mkdir -p src/{storage,embedding,lessons,checkpoints,indexing,watcher,mcp,http}
mkdir -p tests/common
```

- [x] Create `src/lib.rs`:

```rust
#![deny(warnings)]

pub mod checkpoints;
pub mod config;
pub mod embedding;
pub mod error;
pub mod http;
pub mod indexing;
pub mod lessons;
pub mod mcp;
pub mod storage;
pub mod watcher;
```

- [x] Create `src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AmpError {
    #[error("Storage error: {0}")]
    Storage(#[from] rusqlite::Error),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Indexing error: {0}")]
    Indexing(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AmpError>;
```

- [ ] Create `src/config.rs`:

```rust
use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "amp-rs", version, about = "Agent Memory Protocol - Reference Implementation")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    /// Start the AMP server (MCP stdio + HTTP)
    Serve {
        /// Data directory for database and models
        #[arg(long, default_value = "~/.amp-rs")]
        data_dir: PathBuf,

        /// Directories to watch for file changes
        #[arg(long, value_delimiter = ',')]
        watch_dirs: Vec<PathBuf>,

        /// Number of embedding threads
        #[arg(long, default_value = "2")]
        embedding_threads: usize,

        /// HTTP port for health check and REST
        #[arg(long, default_value = "8080")]
        port: u16,

        /// Enable debug logging
        #[arg(long)]
        debug: bool,
    },
    /// Index a repository on demand
    Index {
        /// Path to repository
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub data_dir: PathBuf,
    pub watch_dirs: Vec<PathBuf>,
    pub embedding_threads: usize,
    pub port: u16,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: dirs_next::home_dir()
                .unwrap_or_default()
                .join(".amp-rs"),
            watch_dirs: Vec::new(),
            embedding_threads: 2,
            port: 8080,
            log_level: "info".to_string(),
        }
    }
}
```

- [x] Create stub `mod.rs` for each module:

```rust
// src/storage/mod.rs
// src/embedding/mod.rs
// src/lessons/mod.rs
// src/checkpoints/mod.rs
// src/indexing/mod.rs
// src/watcher/mod.rs
// src/mcp/mod.rs
// src/http/mod.rs
```

- [x] Create `src/main.rs`:

```rust
use anyhow::Result;
use clap::Parser;

mod config;

fn main() -> Result<()> {
    let _cli = config::Cli::parse();
    println!("amp-rs starting...");
    Ok(())
}
```

- [x] Create `tests/common/mod.rs`:

```rust
use tempfile::TempDir;
use std::path::PathBuf;

/// Create a temporary directory for test databases
pub fn test_data_dir() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = dir.path().to_path_buf();
    (dir, path)
}
```

- [x] Run `cargo check` to verify module structure
- [x] Git commit:
```bash
git add -A && git commit -m "feat: create module skeleton with config and error types"
```

**Success Criteria**:
- [x] `cargo check` exits 0
- [x] All modules importable from `lib.rs`
- [x] Error types compile
- [x] Config struct derives correctly

**Completion Notes**:
- **Implementation**: Created complete module skeleton with 8 directories (storage, embedding, lessons, checkpoints, indexing, watcher, mcp, http). Implemented lib.rs with all module declarations, error.rs with AmpError enum and Result type, config.rs with Cli and Config structs using clap and serde, main.rs with CLI parsing, and test helper in tests/common/mod.rs
- **Files Created**: src/lib.rs (12 lines), src/error.rs (45 lines), src/config.rs (66 lines), src/main.rs (10 lines), tests/common/mod.rs (9 lines), 8 stub mod.rs files (1 line each), 8 module directories
- **Files Modified**: Cargo.toml (updated import for config), src/main.rs (replaced hello world)
- **Tests**: N/A
- **Build**: PASS - cargo check exits 0 with no warnings
- **Notes**: Added #[allow(dead_code)] to Config struct since it will be used in later phases

---

### Task 0.1 Complete — Squash Merge
- [x] All subtasks 0.1.1–0.1.3 complete
- [x] `cargo check` passes
- [x] Squash merge to main:
```bash
git checkout main && git merge --squash feature/0-1-project-init
git commit -m "chore: complete task 0.1 - project initialization"
git branch -d feature/0-1-project-init
git push origin main
```

---

## Task 0.2: Development Tools

**Git**: `git checkout -b feature/0-2-dev-tools`

### Subtask 0.2.1: Linting and Formatting Setup (Single Session)

**Prerequisites**:
- [x] 0.1.3: Create Module Skeleton

**Deliverables**:
- [x] Create `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
```

- [x] Create `clippy.toml`:

```toml
too-many-arguments-threshold = 8
```

- [x] Verify linting passes:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings
```
- [x] Git commit:
```bash
git add -A && git commit -m "chore: configure rustfmt and clippy"
```

**Success Criteria**:
- [x] `cargo fmt --check` exits 0
- [x] `cargo clippy --workspace -- -D warnings` exits 0

**Completion Notes**:
- **Implementation**: Created rustfmt.toml with max_width=100 and shorthand settings. Created clippy.toml with too-many-arguments-threshold=8. Applied cargo fmt to ensure all code follows formatting rules, then verified lint passes.
- **Files Created**: rustfmt.toml (4 lines), clippy.toml (1 line)
- **Files Modified**: src/config.rs (reformatted for 100 char width), tests/common/mod.rs (import order fixed)
- **Tests**: N/A
- **Build**: PASS - cargo fmt --check and cargo clippy both pass
- **Notes**: Had to apply cargo fmt once to reformat config.rs attribute macros to stay within 100 char limit

---

### Subtask 0.2.2: Testing Infrastructure (Single Session)

**Prerequisites**:
- [x] 0.2.1: Linting and Formatting Setup

**Deliverables**:
- [x] Add a placeholder integration test `tests/integration_test.rs`:

```rust
mod common;

#[test]
fn placeholder_test() {
    let (_dir, path) = common::test_data_dir();
    assert!(path.exists());
}
```

- [x] Add a unit test to `src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AmpError::Other("test error".to_string());
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let amp_err = AmpError::from(io_err);
        assert!(amp_err.to_string().contains("not found"));
    }
}
```

- [x] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [x] Git commit:
```bash
git add -A && git commit -m "test: add testing infrastructure and placeholder tests"
```

**Success Criteria**:
- [x] `cargo test --workspace` discovers and runs tests
- [x] At least 3 tests pass
- [x] Full verification chain passes

**Completion Notes**:
- **Implementation**: Created tests/integration_test.rs with placeholder_test that validates temp directory creation. src/error.rs already contained two unit tests (test_error_display and test_error_from_io). Ran full verification chain: cargo fmt --check (pass), cargo clippy (pass), cargo test (3 tests pass).
- **Files Created**: tests/integration_test.rs (7 lines)
- **Files Modified**: tests/common/mod.rs (minor import order formatting)
- **Tests**: 3 tests passing (2 unit tests in error.rs + 1 integration test in integration_test.rs)
- **Build**: PASS - full verification chain completes successfully in ~36 seconds
- **Notes**: First full test run compiles dev-dependencies (assert_cmd, predicates, etc.) which adds compilation time. Subsequent runs are much faster. All 3 tests pass with no warnings or errors.

---

### Task 0.2 Complete — Squash Merge
- [x] All subtasks 0.2.1–0.2.2 complete
- [x] Full verification passes: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [x] Squash merge to main:
```bash
git checkout main && git merge --squash feature/0-2-dev-tools
git commit -m "chore: complete task 0.2 - development tools setup"
git branch -d feature/0-2-dev-tools
git push origin main
```

---

*Phase 0 complete when both tasks merged to main.*
