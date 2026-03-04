use super::CodeChunk;
use crate::embedding::EmbeddingGenerator;
use crate::error::Result;
use rusqlite::{params, Connection};
use std::path::Path;
use tracing::debug;

/// Chunk storage operations
pub struct ChunkStorage<'a> {
    conn: &'a Connection,
}

impl<'a> ChunkStorage<'a> {
    /// Create a new chunk storage instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Store code chunks with embeddings
    pub fn store_chunks(
        &self,
        chunks: &[CodeChunk],
        embedding_gen: &dyn EmbeddingGenerator,
    ) -> Result<usize> {
        if chunks.is_empty() {
            return Ok(0);
        }

        let mut insert_count = 0;

        for chunk in chunks {
            // Insert chunk
            self.conn.execute(
                "INSERT OR REPLACE INTO code_chunks (id, file_path, repo_path, content, language, start_line, end_line, chunk_type)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    chunk.id,
                    chunk.file_path,
                    chunk.repo_path,
                    chunk.content,
                    chunk.language,
                    chunk.start_line as i64,
                    chunk.end_line as i64,
                    chunk.chunk_type
                ],
            )?;

            // Generate embedding
            let embedding = embedding_gen.embed(&chunk.content)?;

            // Insert embedding into vector table
            let embedding_json = serde_json::to_string(&embedding)?;
            self.conn.execute(
                "INSERT OR REPLACE INTO chunk_embeddings (id, embedding) VALUES (?, ?)",
                params![chunk.id, embedding_json],
            )?;

            insert_count += 1;
        }

        debug!("Stored {} code chunks with embeddings", insert_count);
        Ok(insert_count)
    }

    /// Delete all chunks for a file
    pub fn delete_file_chunks(&self, file_path: &str) -> Result<usize> {
        // Get chunk IDs for deletion
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM code_chunks WHERE file_path = ?")?;
        let ids: Vec<String> = stmt
            .query_map(params![file_path], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .collect();

        // Delete from embeddings
        for id in &ids {
            self.conn
                .execute("DELETE FROM chunk_embeddings WHERE id = ?", params![id])?;
        }

        // Delete from chunks
        let deleted = self.conn.execute(
            "DELETE FROM code_chunks WHERE file_path = ?",
            params![file_path],
        )?;

        debug!("Deleted {} chunks for file {}", deleted, file_path);
        Ok(deleted)
    }

    /// Check if a file is indexed and get its metadata
    pub fn get_indexed_file(&self, file_path: &str) -> Result<Option<IndexedFileInfo>> {
        let result = self.conn.query_row(
            "SELECT file_path, repo_path, mtime, size FROM indexed_files WHERE file_path = ?",
            params![file_path],
            |row| {
                Ok(IndexedFileInfo {
                    file_path: row.get(0)?,
                    repo_path: row.get(1)?,
                    mtime: row.get(2)?,
                    size: row.get(3)?,
                })
            },
        );

        match result {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Update file tracking record
    pub fn update_indexed_file(
        &self,
        file_path: &str,
        repo_path: &str,
        mtime: u64,
        size: u64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO indexed_files (file_path, repo_path, mtime, size) VALUES (?, ?, ?, ?)",
            params![file_path, repo_path, mtime as i64, size as i64],
        )?;

        Ok(())
    }

    /// Search code chunks by semantic similarity
    pub fn search_chunks(
        &self,
        query: &str,
        embedding_gen: &dyn EmbeddingGenerator,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        // Generate query embedding
        let query_embedding = embedding_gen.embed(query)?;

        // Use sqlite-vec for similarity search
        let embedding_json = serde_json::to_string(&query_embedding)?;
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.file_path, c.content, c.start_line, c.end_line, c.language, c.chunk_type
             FROM code_chunks c
             JOIN chunk_embeddings ce ON c.id = ce.id
             ORDER BY vec_distance(ce.embedding, ?) ASC
             LIMIT ?"
        )?;

        let results: Vec<SearchResult> = stmt
            .query_map(params![embedding_json, limit as i64], |row| {
                Ok(SearchResult {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    content: row.get(2)?,
                    start_line: row.get(3)?,
                    end_line: row.get(4)?,
                    language: row.get(5)?,
                    chunk_type: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        debug!("Found {} matching chunks for query", results.len());
        Ok(results)
    }

    /// List all indexed file paths in a repository
    pub fn list_indexed_files(&self, repo_path: &Path) -> Result<Vec<String>> {
        let repo_str = repo_path.to_string_lossy().to_string();
        let mut stmt = self
            .conn
            .prepare("SELECT file_path FROM indexed_files WHERE repo_path = ?")?;
        let paths: Vec<String> = stmt
            .query_map([&repo_str], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .collect();

        debug!(
            "Listed {} indexed files for repo {:?}",
            paths.len(),
            repo_path
        );
        Ok(paths)
    }

    /// Delete all chunks and file records for a repository
    pub fn delete_repo_data(&self, repo_path: &str) -> Result<()> {
        // Get all chunk IDs for this repo
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM code_chunks WHERE repo_path = ?")?;
        let ids: Vec<String> = stmt
            .query_map([repo_path], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .collect();

        // Delete from embeddings
        for id in &ids {
            self.conn
                .execute("DELETE FROM chunk_embeddings WHERE id = ?", [id])?;
        }

        // Delete from chunks
        self.conn
            .execute("DELETE FROM code_chunks WHERE repo_path = ?", [repo_path])?;

        // Delete from indexed files
        self.conn
            .execute("DELETE FROM indexed_files WHERE repo_path = ?", [repo_path])?;

        debug!("Deleted all data for repo {}", repo_path);
        Ok(())
    }
}

/// Information about an indexed file
#[derive(Debug, Clone)]
pub struct IndexedFileInfo {
    pub file_path: String,
    pub repo_path: String,
    pub mtime: i64,
    pub size: i64,
}

/// Search result for a code chunk
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub file_path: String,
    pub content: String,
    pub start_line: i64,
    pub end_line: i64,
    pub language: Option<String>,
    pub chunk_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::EmbeddingGenerator;
    use crate::storage::Storage;

    #[test]
    fn test_store_and_retrieve_chunks() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        let storage = crate::storage::sqlite::SqliteStorage::open(&db_path).unwrap();
        storage.migrate().unwrap();
        // Try to create vector tables, but don't fail if sqlite-vec isn't available
        let _ = crate::storage::schema::create_vector_tables(storage.connection());

        struct MockEmbedding;
        impl EmbeddingGenerator for MockEmbedding {
            fn embed(&self, _text: &str) -> Result<Vec<f32>> {
                Ok(vec![0.1; 384])
            }

            fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
                Ok(vec![vec![0.1; 384]; texts.len()])
            }

            fn dimension(&self) -> usize {
                384
            }
        }

        let embedding = MockEmbedding;
        let chunk_storage = ChunkStorage::new(storage.connection());

        let chunks = vec![CodeChunk {
            id: "test-1".to_string(),
            file_path: "/test/file.rs".to_string(),
            repo_path: "/test".to_string(),
            content: "fn main() {}".to_string(),
            language: Some("rust".to_string()),
            start_line: 1,
            end_line: 1,
            chunk_type: "function".to_string(),
        }];

        // Storing chunks requires vector tables - skip if they're not available
        match chunk_storage.store_chunks(&chunks, &embedding) {
            Ok(stored) => {
                assert_eq!(stored, 1);
            }
            Err(_) => {
                // sqlite-vec not available, skip this part
            }
        }

        // Test indexed file tracking (doesn't require vector tables)
        let info = chunk_storage.get_indexed_file("/test/file.rs").unwrap();
        assert!(info.is_none());

        chunk_storage
            .update_indexed_file("/test/file.rs", "/test", 1234567890, 100)
            .unwrap();

        let info = chunk_storage.get_indexed_file("/test/file.rs").unwrap();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.file_path, "/test/file.rs");
        assert_eq!(info.mtime, 1234567890);
        assert_eq!(info.size, 100);
    }

    #[test]
    fn test_delete_file_chunks() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        let storage = crate::storage::sqlite::SqliteStorage::open(&db_path).unwrap();
        storage.migrate().unwrap();
        // Try to create vector tables, but don't fail if sqlite-vec isn't available
        let _ = crate::storage::schema::create_vector_tables(storage.connection());

        struct MockEmbedding;
        impl EmbeddingGenerator for MockEmbedding {
            fn embed(&self, _text: &str) -> Result<Vec<f32>> {
                Ok(vec![0.1; 384])
            }

            fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
                Ok(vec![vec![0.1; 384]; texts.len()])
            }

            fn dimension(&self) -> usize {
                384
            }
        }

        let embedding = MockEmbedding;
        let chunk_storage = ChunkStorage::new(storage.connection());

        let chunks = vec![
            CodeChunk {
                id: "test-1".to_string(),
                file_path: "/test/file.rs".to_string(),
                repo_path: "/test".to_string(),
                content: "fn main() {}".to_string(),
                language: Some("rust".to_string()),
                start_line: 1,
                end_line: 1,
                chunk_type: "function".to_string(),
            },
            CodeChunk {
                id: "test-2".to_string(),
                file_path: "/test/file.rs".to_string(),
                repo_path: "/test".to_string(),
                content: "fn helper() {}".to_string(),
                language: Some("rust".to_string()),
                start_line: 3,
                end_line: 3,
                chunk_type: "function".to_string(),
            },
        ];

        // Storing chunks requires vector tables - skip if they're not available
        match chunk_storage.store_chunks(&chunks, &embedding) {
            Ok(_) => {
                let deleted = chunk_storage.delete_file_chunks("/test/file.rs").unwrap();
                assert_eq!(deleted, 2);
            }
            Err(_) => {
                // sqlite-vec not available, skip this part
            }
        }
    }

    #[test]
    #[ignore] // requires sqlite-vec native extension
    fn test_search_chunks() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        let storage = crate::storage::sqlite::SqliteStorage::open(&db_path).unwrap();
        storage.migrate().unwrap();
        // Try to create vector tables, but don't fail if sqlite-vec isn't available
        let _ = crate::storage::schema::create_vector_tables(storage.connection());

        struct MockEmbedding;
        impl EmbeddingGenerator for MockEmbedding {
            fn embed(&self, _text: &str) -> Result<Vec<f32>> {
                Ok(vec![0.1; 384])
            }

            fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
                Ok(vec![vec![0.1; 384]; texts.len()])
            }

            fn dimension(&self) -> usize {
                384
            }
        }

        let embedding = MockEmbedding;
        let chunk_storage = ChunkStorage::new(storage.connection());

        let chunks = vec![CodeChunk {
            id: "test-1".to_string(),
            file_path: "/test/file.rs".to_string(),
            repo_path: "/test".to_string(),
            content: "fn main() {}".to_string(),
            language: Some("rust".to_string()),
            start_line: 1,
            end_line: 1,
            chunk_type: "function".to_string(),
        }];

        // Storing chunks requires vector tables - skip if they're not available
        match chunk_storage.store_chunks(&chunks, &embedding) {
            Ok(_) => {
                let results = chunk_storage
                    .search_chunks("function", &embedding, 10)
                    .unwrap();
                assert_eq!(results.len(), 1);
                assert_eq!(results[0].id, "test-1");
            }
            Err(_) => {
                // sqlite-vec not available, skip this part
            }
        }
    }
}
