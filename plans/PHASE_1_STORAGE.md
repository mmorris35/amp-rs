# Phase 1: Storage Layer

**Goal**: SQLite connection manager with WAL mode, schema, migrations, and sqlite-vec extension
**Duration**: 2–3 days
**Wave**: 1 (parallel with Phase 2)
**Dependencies**: Phase 0 complete

---

## Task 1.1: SQLite Core

**Git**: `git checkout -b feature/1-1-sqlite-core`

### Subtask 1.1.1: SQLite Connection Manager with WAL (Single Session)

**Prerequisites**:
- [x] 0.2.2: Testing Infrastructure

**Deliverables**:
- [ ] Create `src/storage/mod.rs` with storage trait and connection manager:

```rust
pub mod schema;
pub mod sqlite;

use crate::error::Result;

/// Trait for database operations
pub trait Storage: Send + Sync {
    /// Get a reference to the underlying connection for raw queries
    fn connection(&self) -> &rusqlite::Connection;

    /// Run migrations
    fn migrate(&self) -> Result<()>;
}
```

- [ ] Create `src/storage/sqlite.rs` with connection manager:

```rust
use crate::error::{AmpError, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use tracing::info;

pub struct SqliteStorage {
    conn: Connection,
}

impl SqliteStorage {
    /// Open or create database at path with WAL mode
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        // Enable WAL mode
        conn.pragma_update(None, "journal_mode", "wal")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        info!("SQLite database opened at {:?} with WAL mode", path);

        let storage = Self { conn };
        Ok(storage)
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let storage = Self { conn };
        Ok(storage)
    }
}

impl super::Storage for SqliteStorage {
    fn connection(&self) -> &Connection {
        &self.conn
    }

    fn migrate(&self) -> Result<()> {
        super::schema::run_migrations(&self.conn)
    }
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(storage): SQLite connection manager with WAL mode"
```

**Success Criteria**:
- [ ] `cargo check` exits 0
- [ ] Connection opens with WAL mode
- [ ] In-memory variant works for tests

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A (next subtask)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 1.1.2: Schema and Migrations (Single Session)

**Prerequisites**:
- [x] 1.1.1: SQLite Connection Manager

**Deliverables**:
- [ ] Create `src/storage/schema.rs` with all MVP tables:

```rust
use crate::error::Result;
use rusqlite::Connection;
use tracing::info;

const SCHEMA_VERSION: i32 = 1;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create version tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        apply_v1(conn)?;
    }

    info!("Schema at version {}", SCHEMA_VERSION);
    Ok(())
}

fn apply_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        -- Lessons table
        CREATE TABLE IF NOT EXISTS lessons (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            tags TEXT NOT NULL DEFAULT '[]',
            severity TEXT NOT NULL DEFAULT 'info',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Lesson embeddings (sqlite-vec virtual table created separately)

        -- Checkpoints table
        CREATE TABLE IF NOT EXISTS checkpoints (
            id TEXT PRIMARY KEY,
            agent TEXT NOT NULL,
            working_on TEXT NOT NULL,
            state TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Checkpoint embeddings (sqlite-vec virtual table created separately)

        -- Code chunks table
        CREATE TABLE IF NOT EXISTS code_chunks (
            id TEXT PRIMARY KEY,
            file_path TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            content TEXT NOT NULL,
            language TEXT,
            start_line INTEGER NOT NULL,
            end_line INTEGER NOT NULL,
            chunk_type TEXT NOT NULL DEFAULT 'code',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Code chunk embeddings (sqlite-vec virtual table created separately)

        -- Indexed files tracking
        CREATE TABLE IF NOT EXISTS indexed_files (
            file_path TEXT PRIMARY KEY,
            repo_path TEXT NOT NULL,
            mtime INTEGER NOT NULL,
            size INTEGER NOT NULL,
            indexed_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Indexes
        CREATE INDEX IF NOT EXISTS idx_lessons_severity ON lessons(severity);
        CREATE INDEX IF NOT EXISTS idx_checkpoints_agent ON checkpoints(agent);
        CREATE INDEX IF NOT EXISTS idx_checkpoints_created ON checkpoints(created_at);
        CREATE INDEX IF NOT EXISTS idx_code_chunks_file ON code_chunks(file_path);
        CREATE INDEX IF NOT EXISTS idx_code_chunks_repo ON code_chunks(repo_path);
        CREATE INDEX IF NOT EXISTS idx_indexed_files_repo ON indexed_files(repo_path);

        -- Record migration
        INSERT INTO schema_version (version) VALUES (1);
        "
    )?;

    info!("Applied schema v1");
    Ok(())
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(storage): schema and migrations for all MVP tables"
```

**Success Criteria**:
- [ ] `cargo check` exits 0
- [ ] Schema creates all tables: lessons, checkpoints, code_chunks, indexed_files
- [ ] Migration is idempotent

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A (next subtask)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 1.1.3: sqlite-vec Extension Loading (Single Session)

**Prerequisites**:
- [x] 1.1.2: Schema and Migrations

**Deliverables**:
- [ ] Add sqlite-vec virtual table creation to schema (embedding dimension = 384 for all-MiniLM-L6-v2):

```rust
// Add to src/storage/schema.rs after the regular tables
// Note: sqlite-vec tables are created after extension is loaded

pub fn create_vector_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE VIRTUAL TABLE IF NOT EXISTS lesson_embeddings USING vec0(
            id TEXT PRIMARY KEY,
            embedding float[384]
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS checkpoint_embeddings USING vec0(
            id TEXT PRIMARY KEY,
            embedding float[384]
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS chunk_embeddings USING vec0(
            id TEXT PRIMARY KEY,
            embedding float[384]
        );
        "
    )?;

    info!("Created sqlite-vec virtual tables (384 dimensions)");
    Ok(())
}
```

- [ ] Add sqlite-vec loading to `SqliteStorage::open()`:

```rust
// In sqlite.rs, after opening connection:
// Load sqlite-vec extension
// Note: sqlite-vec may be loaded via rusqlite's bundled feature
// or as a loadable extension depending on build configuration
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(storage): sqlite-vec virtual tables for vector search"
```

**Success Criteria**:
- [ ] `cargo check` exits 0
- [ ] Vector tables use 384 dimensions (matches all-MiniLM-L6-v2)
- [ ] Three vector tables: lesson_embeddings, checkpoint_embeddings, chunk_embeddings

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A (next subtask)
- **Build**: (pass/fail)
- **Notes**: sqlite-vec integration approach (bundled vs loadable)

---

### Task 1.1 Complete — Squash Merge
- [ ] All subtasks 1.1.1–1.1.3 complete
- [ ] `cargo check` passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/1-1-sqlite-core
git commit -m "feat(storage): complete task 1.1 - SQLite core with WAL and sqlite-vec"
git branch -d feature/1-1-sqlite-core
git push origin main
```

---

## Task 1.2: Storage Abstraction and Tests

**Git**: `git checkout -b feature/1-2-storage-tests`

### Subtask 1.2.1: Storage Trait and Error Types (Single Session)

**Prerequisites**:
- [x] 1.1.3: sqlite-vec Extension Loading

**Deliverables**:
- [ ] Enhance storage trait with concrete operations needed by lessons, checkpoints, and indexing
- [ ] Add helper methods for common patterns (insert, query, vector search)
- [ ] Ensure all storage errors map to `AmpError::Storage`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(storage): enhanced storage trait with CRUD helpers"
```

**Success Criteria**:
- [ ] Storage trait is generic enough for all modules
- [ ] Error mapping is clean

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A (next subtask)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 1.2.2: Storage Integration Tests (Single Session)

**Prerequisites**:
- [x] 1.2.1: Storage Trait and Error Types

**Deliverables**:
- [ ] Create `tests/storage_test.rs`:

```rust
mod common;

use amp_rs::storage::{sqlite::SqliteStorage, Storage};

#[test]
fn test_open_in_memory() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    // Verify tables exist
    let count: i32 = storage.connection()
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='lessons'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_open_file_based() {
    let (_dir, path) = common::test_data_dir();
    let db_path = path.join("test.db");
    let storage = SqliteStorage::open(&db_path).unwrap();
    storage.migrate().unwrap();
    assert!(db_path.exists());
}

#[test]
fn test_wal_mode() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    // In-memory doesn't support WAL, but file-based should
    let (_dir, path) = common::test_data_dir();
    let db_path = path.join("test.db");
    let file_storage = SqliteStorage::open(&db_path).unwrap();
    let mode: String = file_storage.connection()
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal");
}

#[test]
fn test_migration_idempotent() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    storage.migrate().unwrap(); // Should not error
}

#[test]
fn test_all_tables_created() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    let tables = vec!["lessons", "checkpoints", "code_chunks", "indexed_files", "schema_version"];
    for table in tables {
        let exists: bool = storage.connection()
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        assert!(exists, "Table '{}' should exist", table);
    }
}
```

- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test(storage): integration tests for SQLite storage layer"
```

**Success Criteria**:
- [ ] All storage tests pass
- [ ] `cargo test --workspace` exits 0
- [ ] Full verification chain passes

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 1.2 Complete — Squash Merge
- [ ] All subtasks 1.2.1–1.2.2 complete
- [ ] Full verification: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/1-2-storage-tests
git commit -m "feat(storage): complete task 1.2 - storage abstraction and tests"
git branch -d feature/1-2-storage-tests
git push origin main
```

---

*Phase 1 complete when both tasks merged to main.*
