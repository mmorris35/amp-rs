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
- [ ] Run `cargo init --name amp-rs` in project root
- [ ] Create `.gitignore` with Rust patterns:

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

- [ ] Create `LICENSE` file (MIT)
- [ ] Verify `cargo build` succeeds with default hello world
- [ ] Git commit:
```bash
git add -A && git commit -m "chore: initialize Cargo project"
```

**Success Criteria**:
- [ ] `cargo build` exits 0
- [ ] `cargo run` prints "Hello, world!"
- [ ] `.gitignore` exists with Rust patterns
- [ ] `LICENSE` exists

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (any additional context)

---

### Subtask 0.1.2: Configure Dependencies in Cargo.toml (Single Session)

**Prerequisites**:
- [x] 0.1.1: Initialize Cargo Project

**Deliverables**:
- [ ] Update `Cargo.toml` with all MVP dependencies:

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

- [ ] Run `cargo check` to verify dependency resolution
- [ ] Git commit:
```bash
git add -A && git commit -m "chore: configure all MVP dependencies in Cargo.toml"
```

**Success Criteria**:
- [ ] `cargo check` exits 0 (all deps resolve)
- [ ] No version conflicts

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (any additional context)

---

### Subtask 0.1.3: Create Module Skeleton (Single Session)

**Prerequisites**:
- [x] 0.1.2: Configure Dependencies

**Deliverables**:
- [ ] Create module directory structure:

```bash
mkdir -p src/{storage,embedding,lessons,checkpoints,indexing,watcher,mcp,http}
mkdir -p tests/common
```

- [ ] Create `src/lib.rs`:

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

- [ ] Create `src/error.rs`:

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

- [ ] Create stub `mod.rs` for each module:

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

- [ ] Create `src/main.rs`:

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

- [ ] Create `tests/common/mod.rs`:

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

- [ ] Run `cargo check` to verify module structure
- [ ] Git commit:
```bash
git add -A && git commit -m "feat: create module skeleton with config and error types"
```

**Success Criteria**:
- [ ] `cargo check` exits 0
- [ ] All modules importable from `lib.rs`
- [ ] Error types compile
- [ ] Config struct derives correctly

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (any additional context)

---

### Task 0.1 Complete — Squash Merge
- [ ] All subtasks 0.1.1–0.1.3 complete
- [ ] `cargo check` passes
- [ ] Squash merge to main:
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
- [ ] Create `rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
```

- [ ] Create `clippy.toml`:

```toml
too-many-arguments-threshold = 8
```

- [ ] Verify linting passes:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings
```
- [ ] Git commit:
```bash
git add -A && git commit -m "chore: configure rustfmt and clippy"
```

**Success Criteria**:
- [ ] `cargo fmt --check` exits 0
- [ ] `cargo clippy --workspace -- -D warnings` exits 0

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (any additional context)

---

### Subtask 0.2.2: Testing Infrastructure (Single Session)

**Prerequisites**:
- [x] 0.2.1: Linting and Formatting Setup

**Deliverables**:
- [ ] Add a placeholder integration test `tests/integration_test.rs`:

```rust
mod common;

#[test]
fn placeholder_test() {
    let (_dir, path) = common::test_data_dir();
    assert!(path.exists());
}
```

- [ ] Add a unit test to `src/error.rs`:

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

- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test: add testing infrastructure and placeholder tests"
```

**Success Criteria**:
- [ ] `cargo test --workspace` discovers and runs tests
- [ ] At least 3 tests pass
- [ ] Full verification chain passes

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (any additional context)

---

### Task 0.2 Complete — Squash Merge
- [ ] All subtasks 0.2.1–0.2.2 complete
- [ ] Full verification passes: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] Squash merge to main:
```bash
git checkout main && git merge --squash feature/0-2-dev-tools
git commit -m "chore: complete task 0.2 - development tools setup"
git branch -d feature/0-2-dev-tools
git push origin main
```

---

*Phase 0 complete when both tasks merged to main.*
