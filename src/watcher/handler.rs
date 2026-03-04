use crate::error::Result;
use crate::indexing::scanner;
use crate::indexing::storage::ChunkStorage;
use rusqlite::Connection;
use std::path::Path;
use tracing::info;

/// Diff index — only re-index files with changed mtime
pub fn diff_index(conn: &Connection, repo_path: &Path) -> Result<DiffResult> {
    let storage = ChunkStorage::new(conn);
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
    let storage = ChunkStorage::new(conn);

    // Delete all chunks and file records for this repo
    let repo_str = repo_path.to_string_lossy().to_string();
    storage.delete_repo_data(&repo_str)?;

    info!("Cleared index for {:?}, starting full re-index", repo_path);

    // Re-scan and index everything
    let files = scanner::scan_directory(repo_path)?;
    Ok(files.len())
}
