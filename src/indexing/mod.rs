pub mod chunker;
pub mod scanner;
pub mod storage;

use std::path::PathBuf;

/// A file discovered by the scanner
#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub repo_path: PathBuf,
    pub size: u64,
    pub mtime: u64,
    pub language: Option<String>,
}

/// A chunk of code extracted from a file
#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub id: String,
    pub file_path: String,
    pub repo_path: String,
    pub content: String,
    pub language: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: String,
}

use crate::embedding::EmbeddingGenerator;
use crate::error::Result;
use crate::storage::Storage;
use std::path::Path;
use tracing::info;

/// Orchestrate the full indexing pipeline
pub struct IndexingCoordinator;

impl IndexingCoordinator {
    /// Index a repository: scan -> chunk -> embed -> store
    pub fn index_repository(
        repo_path: &Path,
        storage: &dyn Storage,
        embedding_gen: &dyn EmbeddingGenerator,
    ) -> Result<IndexingStats> {
        let conn = storage.connection();
        let chunk_storage = storage::ChunkStorage::new(conn);

        // 1. Scan for indexable files
        let scanned_files = scanner::scan_directory(repo_path)?;
        info!("Scanned {} files in {:?}", scanned_files.len(), repo_path);

        let mut total_chunks = 0;
        let mut indexed_files = 0;

        // 2. Process each file
        let files_scanned_count = scanned_files.len();
        for file in scanned_files {
            // Check if file needs reindexing (mtime-based)
            let should_index =
                match chunk_storage.get_indexed_file(file.path.to_string_lossy().as_ref())? {
                    Some(info) => info.mtime < file.mtime as i64,
                    None => true,
                };

            if !should_index {
                continue;
            }

            // Delete old chunks for this file
            let _ = chunk_storage.delete_file_chunks(file.path.to_string_lossy().as_ref());

            // 3. Chunk the file
            let chunks = chunker::chunk_file(&file.path, &file.repo_path, &file.language)?;

            if chunks.is_empty() {
                continue;
            }

            // 4. Store chunks with embeddings
            let stored = chunk_storage.store_chunks(&chunks, embedding_gen)?;
            total_chunks += stored;

            // 5. Update file tracking
            chunk_storage.update_indexed_file(
                file.path.to_string_lossy().as_ref(),
                file.repo_path.to_string_lossy().as_ref(),
                file.mtime,
                file.size,
            )?;

            indexed_files += 1;
        }

        info!(
            "Indexed {} files with {} chunks",
            indexed_files, total_chunks
        );

        Ok(IndexingStats {
            files_scanned: files_scanned_count,
            files_indexed: indexed_files,
            total_chunks,
        })
    }

    /// Search for code chunks by semantic similarity
    pub fn search_code(
        query: &str,
        storage: &dyn Storage,
        embedding_gen: &dyn EmbeddingGenerator,
        limit: usize,
    ) -> Result<Vec<storage::SearchResult>> {
        let conn = storage.connection();
        let chunk_storage = storage::ChunkStorage::new(conn);
        chunk_storage.search_chunks(query, embedding_gen, limit)
    }
}

/// Statistics from indexing operation
#[derive(Debug, Clone)]
pub struct IndexingStats {
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub total_chunks: usize,
}
