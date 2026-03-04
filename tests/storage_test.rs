mod common;

use amp_rs::storage::{sqlite::SqliteStorage, Storage};

#[test]
fn test_open_in_memory() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    // Verify tables exist
    let count: i32 = storage
        .connection()
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
    // In-memory doesn't support WAL, but file-based should
    let (_dir, path) = common::test_data_dir();
    let db_path = path.join("test.db");
    let file_storage = SqliteStorage::open(&db_path).unwrap();
    let mode: String = file_storage
        .connection()
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
    let tables = vec![
        "lessons",
        "checkpoints",
        "code_chunks",
        "indexed_files",
        "schema_version",
    ];
    for table in tables {
        let exists: bool = storage
            .connection()
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .unwrap();
        assert!(exists, "Table '{}' should exist", table);
    }
}

#[test]
fn test_storage_trait_send() {
    // Compile-time check that Storage trait is Send
    fn assert_send<T: Send>() {}
    assert_send::<SqliteStorage>();
}

#[test]
fn test_foreign_keys_enabled() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    let enabled: i32 = storage
        .connection()
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap();
    assert_eq!(enabled, 1, "Foreign keys should be enabled");
}

#[test]
fn test_schema_version_tracked() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();

    let version: i32 = storage
        .connection()
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, 1, "Schema version should be 1");
}

#[test]
fn test_indexes_created() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();

    let indexes = vec![
        "idx_lessons_severity",
        "idx_checkpoints_agent",
        "idx_checkpoints_created",
        "idx_code_chunks_file",
        "idx_code_chunks_repo",
        "idx_indexed_files_repo",
    ];

    for idx in indexes {
        let exists: bool = storage
            .connection()
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?1",
                [idx],
                |row| row.get(0),
            )
            .unwrap();
        assert!(exists, "Index '{}' should exist", idx);
    }
}
