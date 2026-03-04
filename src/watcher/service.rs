use crate::embedding::EmbeddingGenerator;
use crate::error::Result;
use crate::indexing::IndexingCoordinator;
use crate::storage::Storage;
use std::path::Path;
use std::time::Instant;
use tracing::info;

use super::handler;

/// Result of an index operation
#[derive(Debug)]
pub struct IndexResult {
    pub files_indexed: usize,
    pub files_total: usize,
    pub new_files: usize,
    pub changed_files: usize,
    pub deleted_files: usize,
    pub duration_secs: f64,
}

/// Index a repository from scratch
pub fn index_repo(
    repo_path: &Path,
    storage: &dyn Storage,
    embedding_gen: &dyn EmbeddingGenerator,
) -> Result<IndexResult> {
    let start = Instant::now();

    // Full initial index
    let stats = IndexingCoordinator::index_repository(repo_path, storage, embedding_gen)?;

    let duration = start.elapsed().as_secs_f64();
    info!(
        "Indexed {} files with {} chunks in {:.2}s",
        stats.files_indexed, stats.total_chunks, duration
    );

    Ok(IndexResult {
        files_indexed: stats.files_indexed,
        files_total: stats.files_scanned,
        new_files: stats.files_indexed,
        changed_files: 0,
        deleted_files: 0,
        duration_secs: duration,
    })
}

/// Incrementally update index based on file changes (mtime-based)
pub fn diff_index_repo(
    repo_path: &Path,
    storage: &dyn Storage,
    embedding_gen: &dyn EmbeddingGenerator,
) -> Result<IndexResult> {
    let start = Instant::now();
    let conn = storage.connection();

    // Get diff
    let diff_result = handler::diff_index(conn, repo_path)?;
    let new_count = diff_result.new_files.len();
    let changed_count = diff_result.changed_files.len();
    let deleted_count = diff_result.deleted_files.len();

    // Index new and changed files
    let chunk_storage = crate::indexing::storage::ChunkStorage::new(conn);
    for file in diff_result
        .new_files
        .iter()
        .chain(diff_result.changed_files.iter())
    {
        // Delete old chunks for this file
        let _ = chunk_storage.delete_file_chunks(file.path.to_string_lossy().as_ref());

        // Chunk and store
        let chunks =
            crate::indexing::chunker::chunk_file(&file.path, &file.repo_path, &file.language)?;
        chunk_storage.store_chunks(&chunks, embedding_gen)?;

        // Update file tracking
        chunk_storage.update_indexed_file(
            file.path.to_string_lossy().as_ref(),
            file.repo_path.to_string_lossy().as_ref(),
            file.mtime,
            file.size,
        )?;
    }

    // Delete chunks for deleted files
    for file_path in &diff_result.deleted_files {
        let _ = chunk_storage.delete_file_chunks(file_path);
    }

    let duration = start.elapsed().as_secs_f64();
    info!(
        "Diff indexed: {} new, {} changed, {} deleted in {:.2}s",
        new_count, changed_count, deleted_count, duration
    );

    Ok(IndexResult {
        files_indexed: new_count + changed_count,
        files_total: new_count + changed_count + deleted_count,
        new_files: new_count,
        changed_files: changed_count,
        deleted_files: deleted_count,
        duration_secs: duration,
    })
}

/// Full reindex — clear everything and re-index
pub fn full_reindex_repo(
    repo_path: &Path,
    storage: &dyn Storage,
    embedding_gen: &dyn EmbeddingGenerator,
) -> Result<IndexResult> {
    let start = Instant::now();
    let conn = storage.connection();

    // Clear all data for this repo
    handler::full_reindex(conn, repo_path)?;

    // Perform full reindex
    let stats = IndexingCoordinator::index_repository(repo_path, storage, embedding_gen)?;

    let duration = start.elapsed().as_secs_f64();
    info!(
        "Full reindex complete: {} files, {} chunks in {:.2}s",
        stats.files_indexed, stats.total_chunks, duration
    );

    Ok(IndexResult {
        files_indexed: stats.files_indexed,
        files_total: stats.files_scanned,
        new_files: stats.files_indexed,
        changed_files: 0,
        deleted_files: 0,
        duration_secs: duration,
    })
}
