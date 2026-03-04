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

/// Create sqlite-vec virtual tables for vector search
/// Embedding dimension is 384 for all-MiniLM-L6-v2
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creates_version_table() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_migration_creates_all_tables() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        let tables = vec!["lessons", "checkpoints", "code_chunks", "indexed_files"];
        for table in tables {
            let count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "Table '{}' should exist", table);
        }
    }
}
