use tempfile::TempDir;
use std::path::PathBuf;

/// Create a temporary directory for test databases
pub fn test_data_dir() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let path = dir.path().to_path_buf();
    (dir, path)
}
