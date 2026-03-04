use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary directory for test databases
pub fn test_data_dir() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = dir.path().to_path_buf();
    (dir, path)
}

/// Create a temporary database file path
pub fn test_db_path() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    (dir, db_path)
}
