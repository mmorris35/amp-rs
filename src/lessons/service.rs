use super::{Lesson, Severity};
use crate::embedding::pool::EmbeddingPool;
use crate::error::Result;
use crate::storage::Storage;
use std::sync::Arc;

/// Lesson service layer composing storage and embeddings
pub struct LessonService {
    storage: Arc<dyn Storage>,
    embedding_pool: Arc<EmbeddingPool>,
}

impl LessonService {
    /// Create a new lesson service
    pub fn new(storage: Arc<dyn Storage>, embedding_pool: Arc<EmbeddingPool>) -> Self {
        Self {
            storage,
            embedding_pool,
        }
    }

    /// Add a new lesson with embedding
    pub async fn add_lesson(
        &self,
        title: &str,
        content: &str,
        tags: &[String],
        severity: &Severity,
    ) -> Result<Lesson> {
        let conn = self.storage.connection();
        let lesson_storage = super::storage::LessonStorage::new(conn);

        // Add the lesson first
        let lesson = lesson_storage.add(title, content, tags, severity)?;

        // Generate and store embedding for the content
        let embedding = self.embedding_pool.embed(content.to_string()).await?;

        lesson_storage.store_embedding(&lesson.id, &embedding)?;

        Ok(lesson)
    }

    /// Search lessons by semantic similarity to a query
    pub async fn search_lessons(&self, query: &str, limit: usize) -> Result<Vec<(Lesson, f64)>> {
        let conn = self.storage.connection();
        let lesson_storage = super::storage::LessonStorage::new(conn);

        // Generate embedding for the query
        let query_embedding = self.embedding_pool.embed(query.to_string()).await?;

        // Search by embedding
        lesson_storage.search_by_embedding(&query_embedding, limit)
    }

    /// List lessons with optional severity filter
    pub async fn list_lessons(
        &self,
        severity: Option<&Severity>,
        limit: usize,
    ) -> Result<Vec<Lesson>> {
        let conn = self.storage.connection();
        let lesson_storage = super::storage::LessonStorage::new(conn);
        lesson_storage.list(severity, limit)
    }

    /// Get a lesson by ID
    pub async fn get_lesson(&self, id: &str) -> Result<Option<Lesson>> {
        let conn = self.storage.connection();
        let lesson_storage = super::storage::LessonStorage::new(conn);
        lesson_storage.get(id)
    }

    /// Delete a lesson by ID
    pub async fn delete_lesson(&self, id: &str) -> Result<bool> {
        let conn = self.storage.connection();
        let lesson_storage = super::storage::LessonStorage::new(conn);
        lesson_storage.delete(id)
    }

    /// Count total lessons
    pub async fn count_lessons(&self) -> Result<usize> {
        let conn = self.storage.connection();
        let lesson_storage = super::storage::LessonStorage::new(conn);
        lesson_storage.count()
    }

    /// Count lessons by severity
    pub async fn count_by_severity(&self, severity: &Severity) -> Result<usize> {
        let conn = self.storage.connection();
        let lesson_storage = super::storage::LessonStorage::new(conn);
        lesson_storage.count_by_severity(severity)
    }
}
