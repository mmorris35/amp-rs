use super::{Lesson, Severity};
use crate::error::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

/// Database operations for lessons
pub struct LessonStorage<'a> {
    conn: &'a Connection,
}

impl<'a> LessonStorage<'a> {
    /// Create a new lesson storage instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Add a new lesson to storage
    pub fn add(
        &self,
        title: &str,
        content: &str,
        tags: &[String],
        severity: &Severity,
    ) -> Result<Lesson> {
        let id = Uuid::new_v4().to_string();
        let tags_json = serde_json::to_string(tags)?;
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        self.conn.execute(
            "INSERT INTO lessons (id, title, content, tags, severity, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                title,
                content,
                tags_json,
                severity.to_string(),
                now_str,
                now_str
            ],
        )?;

        Ok(Lesson {
            id,
            title: title.to_string(),
            content: content.to_string(),
            tags: tags.to_vec(),
            severity: severity.clone(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Get a lesson by ID
    pub fn get(&self, id: &str) -> Result<Option<Lesson>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, severity, created_at, updated_at FROM lessons WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], |row| {
            let tags_str: String = row.get(3)?;
            let severity_str: String = row.get(4)?;
            let created_str: String = row.get(5)?;
            let updated_str: String = row.get(6)?;

            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                tags_str,
                severity_str,
                created_str,
                updated_str,
            ))
        });

        match result {
            Ok((id, title, content, tags_str, severity_str, created_str, updated_str)) => {
                Ok(Some(Lesson {
                    id,
                    title,
                    content,
                    tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                    severity: severity_str.parse().unwrap_or(Severity::Info),
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List lessons with optional severity filter
    pub fn list(&self, severity: Option<&Severity>, limit: usize) -> Result<Vec<Lesson>> {
        let mut lessons = Vec::new();

        if let Some(sev) = severity {
            let mut stmt = self.conn.prepare(
                "SELECT id, title, content, tags, severity, created_at, updated_at FROM lessons WHERE severity = ?1 ORDER BY created_at DESC LIMIT ?2"
            )?;
            let rows = stmt.query_map(params![sev.to_string(), limit as i64], |row| {
                Ok(self.parse_lesson_row(row))
            })?;
            for row in rows {
                lessons.push(row??);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, title, content, tags, severity, created_at, updated_at FROM lessons ORDER BY created_at DESC LIMIT ?1"
            )?;
            let rows = stmt.query_map(params![limit as i64], |row| {
                Ok(self.parse_lesson_row(row))
            })?;
            for row in rows {
                lessons.push(row??);
            }
        }

        Ok(lessons)
    }

    /// Count total lessons
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM lessons",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Count lessons by severity
    pub fn count_by_severity(&self, severity: &Severity) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM lessons WHERE severity = ?1",
            params![severity.to_string()],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Delete a lesson and its embedding
    pub fn delete(&self, id: &str) -> Result<bool> {
        // Delete embedding if it exists
        let _ = self.conn.execute(
            "DELETE FROM lesson_embeddings WHERE id = ?1",
            params![id],
        );

        let rows = self.conn.execute("DELETE FROM lessons WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Store embedding for a lesson
    pub fn store_embedding(&self, lesson_id: &str, embedding: &[f32]) -> Result<()> {
        let embedding_bytes = bytemuck::cast_slice(embedding);
        self.conn.execute(
            "INSERT INTO lesson_embeddings (id, embedding) VALUES (?1, ?2)",
            params![lesson_id, embedding_bytes],
        )?;
        Ok(())
    }

    /// Search lessons by embedding similarity
    pub fn search_by_embedding(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(Lesson, f64)>> {
        let query_bytes = bytemuck::cast_slice(query_embedding);
        let mut stmt = self.conn.prepare(
            "SELECT l.id, l.title, l.content, l.tags, l.severity, l.created_at, l.updated_at, e.distance
             FROM lesson_embeddings e
             JOIN lessons l ON l.id = e.id
             WHERE e.embedding MATCH ?1
             ORDER BY e.distance
             LIMIT ?2",
        )?;

        let results = stmt.query_map(params![query_bytes, limit as i64], |row| {
            let distance: f64 = row.get(7)?;
            let lesson = Lesson {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags: serde_json::from_str::<Vec<String>>(&row.get::<_, String>(3)?)
                    .unwrap_or_default(),
                severity: row
                    .get::<_, String>(4)?
                    .parse()
                    .unwrap_or(Severity::Info),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            };
            Ok((lesson, distance))
        })?;

        let mut lessons = Vec::new();
        for result in results {
            let (lesson, distance) = result?;
            lessons.push((lesson, distance));
        }
        Ok(lessons)
    }

    /// Parse a lesson from database row
    fn parse_lesson_row(&self, row: &rusqlite::Row) -> Result<Lesson> {
        let tags_str: String = row.get(3)?;
        let severity_str: String = row.get(4)?;
        let created_str: String = row.get(5)?;
        let updated_str: String = row.get(6)?;

        Ok(Lesson {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            severity: severity_str.parse().unwrap_or(Severity::Info),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{sqlite::SqliteStorage, Storage};

    fn setup() -> SqliteStorage {
        let storage = SqliteStorage::open_in_memory().unwrap();
        storage.migrate().unwrap();
        storage
    }

    #[test]
    fn test_add_lesson() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        let lesson = ls
            .add("Test", "Content", &["rust".into()], &Severity::Info)
            .unwrap();
        assert_eq!(lesson.title, "Test");
        assert_eq!(lesson.tags, vec!["rust"]);
    }

    #[test]
    fn test_get_lesson() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        let lesson = ls.add("Test", "Content", &[], &Severity::Warning).unwrap();
        let found = ls.get(&lesson.id).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test");
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        let found = ls.get("nonexistent-id").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_list_lessons() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        ls.add("A", "Content A", &[], &Severity::Info).unwrap();
        ls.add("B", "Content B", &[], &Severity::Critical).unwrap();
        let all = ls.list(None, 50).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_list_filter_severity() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        ls.add("A", "Content A", &[], &Severity::Info).unwrap();
        ls.add("B", "Content B", &[], &Severity::Critical).unwrap();
        ls.add("C", "Content C", &[], &Severity::Warning).unwrap();

        let critical = ls.list(Some(&Severity::Critical), 50).unwrap();
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].title, "B");

        let info = ls.list(Some(&Severity::Info), 50).unwrap();
        assert_eq!(info.len(), 1);
        assert_eq!(info[0].title, "A");
    }

    #[test]
    fn test_count() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        ls.add("A", "Content A", &[], &Severity::Info).unwrap();
        ls.add("B", "Content B", &[], &Severity::Critical).unwrap();
        assert_eq!(ls.count().unwrap(), 2);
    }

    #[test]
    fn test_count_by_severity() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        ls.add("A", "Content A", &[], &Severity::Info).unwrap();
        ls.add("B", "Content B", &[], &Severity::Critical).unwrap();
        ls.add("C", "Content C", &[], &Severity::Warning).unwrap();

        assert_eq!(ls.count_by_severity(&Severity::Critical).unwrap(), 1);
        assert_eq!(ls.count_by_severity(&Severity::Info).unwrap(), 1);
        assert_eq!(ls.count_by_severity(&Severity::Warning).unwrap(), 1);
    }

    #[test]
    fn test_delete_lesson() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        let lesson = ls
            .add("Delete me", "Content", &[], &Severity::Info)
            .unwrap();
        assert!(ls.delete(&lesson.id).unwrap());
        assert!(ls.get(&lesson.id).unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        assert!(!ls.delete("nonexistent-id").unwrap());
    }
}
