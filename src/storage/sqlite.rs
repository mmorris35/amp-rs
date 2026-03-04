use crate::error::Result;
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
