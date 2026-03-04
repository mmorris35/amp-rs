# Phase 3: Lessons System

**Goal**: Full lessons CRUD — add, search (semantic), list, filter, delete
**Duration**: 2–3 days
**Wave**: 2 (parallel with Phase 4 and Phase 5)
**Dependencies**: Phase 1 (storage) + Phase 2 (embeddings) complete

---

## Task 3.1: Lesson Model and Storage

**Git**: `git checkout -b feature/3-1-lesson-model`

### Subtask 3.1.1: Lesson Model and Storage Operations (Single Session)

**Prerequisites**:
- [x] 1.2.2: Storage Integration Tests
- [x] 2.2.2: Embedding Engine Tests

**Deliverables**:
- [ ] Create `src/lessons/mod.rs` with lesson model:

```rust
pub mod service;
pub mod storage;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub severity: Severity,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "critical" => Ok(Severity::Critical),
            "warning" => Ok(Severity::Warning),
            "info" => Ok(Severity::Info),
            _ => Err(format!("Invalid severity: {}", s)),
        }
    }
}
```

- [ ] Create `src/lessons/storage.rs` with DB operations:

```rust
use super::{Lesson, Severity};
use crate::error::Result;
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct LessonStorage<'a> {
    conn: &'a Connection,
}

impl<'a> LessonStorage<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn add(&self, title: &str, content: &str, tags: &[String], severity: &Severity) -> Result<Lesson> {
        let id = Uuid::new_v4().to_string();
        let tags_json = serde_json::to_string(tags)?;
        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();

        self.conn.execute(
            "INSERT INTO lessons (id, title, content, tags, severity, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, title, content, tags_json, severity.to_string(), now_str, now_str],
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

    pub fn get(&self, id: &str) -> Result<Option<Lesson>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, severity, created_at, updated_at FROM lessons WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(self.row_to_lesson(row))
        });

        match result {
            Ok(lesson) => Ok(Some(lesson?)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list(&self, severity: Option<&Severity>, limit: usize) -> Result<Vec<Lesson>> {
        let (sql, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match severity {
            Some(sev) => (
                "SELECT id, title, content, tags, severity, created_at, updated_at FROM lessons WHERE severity = ?1 ORDER BY created_at DESC LIMIT ?2".to_string(),
                vec![Box::new(sev.to_string()), Box::new(limit as i64)],
            ),
            None => (
                "SELECT id, title, content, tags, severity, created_at, updated_at FROM lessons ORDER BY created_at DESC LIMIT ?1".to_string(),
                vec![Box::new(limit as i64)],
            ),
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
        let lessons = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(self.row_to_lesson(row))
        })?;

        let mut result = Vec::new();
        for lesson in lessons {
            result.push(lesson??);
        }
        Ok(result)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        // Also delete embedding
        let _ = self.conn.execute("DELETE FROM lesson_embeddings WHERE id = ?1", params![id]);
        let rows = self.conn.execute("DELETE FROM lessons WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    fn row_to_lesson(&self, row: &rusqlite::Row) -> Result<Lesson> {
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
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(lessons): lesson model and storage operations"
```

**Success Criteria**:
- [ ] `cargo check` exits 0
- [ ] CRUD operations: add, get, list, delete
- [ ] Severity filtering in list

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A (subtask 3.1.2)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 3.1.2: Lesson Semantic Search (Single Session)

**Prerequisites**:
- [x] 3.1.1: Lesson Model and Storage Operations

**Deliverables**:
- [ ] Add embedding storage to `LessonStorage`:

```rust
// In src/lessons/storage.rs
pub fn store_embedding(&self, lesson_id: &str, embedding: &[f32]) -> Result<()> {
    self.conn.execute(
        "INSERT INTO lesson_embeddings (id, embedding) VALUES (?1, ?2)",
        params![lesson_id, embedding.as_bytes()],
    )?;
    Ok(())
}

pub fn search_by_embedding(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(Lesson, f64)>> {
    let mut stmt = self.conn.prepare(
        "SELECT l.id, l.title, l.content, l.tags, l.severity, l.created_at, l.updated_at, e.distance
         FROM lesson_embeddings e
         JOIN lessons l ON l.id = e.id
         WHERE e.embedding MATCH ?1
         ORDER BY e.distance
         LIMIT ?2"
    )?;

    let results = stmt.query_map(
        params![query_embedding.as_bytes(), limit as i64],
        |row| {
            let distance: f64 = row.get(7)?;
            Ok((self.row_to_lesson(row), distance))
        },
    )?;

    let mut lessons = Vec::new();
    for result in results {
        let (lesson_result, distance) = result?;
        lessons.push((lesson_result?, distance));
    }
    Ok(lessons)
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(lessons): semantic search via sqlite-vec embeddings"
```

**Success Criteria**:
- [ ] `cargo check` exits 0
- [ ] Embedding storage and retrieval work
- [ ] Search returns results ordered by distance

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A (subtask 3.2.2)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 3.1.3: Lesson List and Filter (Single Session)

**Prerequisites**:
- [x] 3.1.2: Lesson Semantic Search

**Deliverables**:
- [ ] Enhance list operation with additional filter options (tag filtering, date range)
- [ ] Add count method for stats
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(lessons): enhanced list and filter operations"
```

**Success Criteria**:
- [ ] List supports severity filter
- [ ] Count returns total lessons

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 3.1 Complete — Squash Merge
- [ ] All subtasks 3.1.1–3.1.3 complete
- [ ] `cargo check` passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/3-1-lesson-model
git commit -m "feat(lessons): complete task 3.1 - lesson model, storage, and search"
git branch -d feature/3-1-lesson-model
git push origin main
```

---

## Task 3.2: Lesson Service and Tests

**Git**: `git checkout -b feature/3-2-lesson-service`

### Subtask 3.2.1: Lesson Service Layer (Single Session)

**Prerequisites**:
- [x] 3.1.3: Lesson List and Filter

**Deliverables**:
- [ ] Create `src/lessons/service.rs` that composes storage + embeddings:

```rust
use super::{Lesson, Severity};
use super::storage::LessonStorage;
use crate::embedding::pool::EmbeddingPool;
use crate::error::Result;
use std::sync::Arc;
use rusqlite::Connection;

pub struct LessonService {
    conn: Arc<Connection>,
    embedding_pool: Arc<EmbeddingPool>,
}

impl LessonService {
    pub fn new(conn: Arc<Connection>, embedding_pool: Arc<EmbeddingPool>) -> Self {
        Self { conn, embedding_pool }
    }

    pub async fn add_lesson(
        &self,
        title: String,
        content: String,
        tags: Vec<String>,
        severity: Severity,
    ) -> Result<Lesson> {
        let storage = LessonStorage::new(&self.conn);
        let lesson = storage.add(&title, &content, &tags, &severity)?;

        // Generate and store embedding
        let embed_text = format!("{} {}", title, content);
        let embedding = self.embedding_pool.embed(embed_text).await?;
        storage.store_embedding(&lesson.id, &embedding)?;

        Ok(lesson)
    }

    pub async fn search_lessons(&self, query: String, limit: usize) -> Result<Vec<Lesson>> {
        let embedding = self.embedding_pool.embed(query).await?;
        let storage = LessonStorage::new(&self.conn);
        let results = storage.search_by_embedding(&embedding, limit)?;
        Ok(results.into_iter().map(|(lesson, _)| lesson).collect())
    }

    pub fn list_lessons(&self, severity: Option<Severity>, limit: usize) -> Result<Vec<Lesson>> {
        let storage = LessonStorage::new(&self.conn);
        storage.list(severity.as_ref(), limit)
    }

    pub fn delete_lesson(&self, id: &str) -> Result<bool> {
        let storage = LessonStorage::new(&self.conn);
        storage.delete(id)
    }
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(lessons): service layer composing storage and embeddings"
```

**Success Criteria**:
- [ ] Service composes storage + embedding pool
- [ ] async add generates embeddings automatically
- [ ] Search, list, delete delegated properly

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A (next subtask)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 3.2.2: Lesson Unit and Integration Tests (Single Session)

**Prerequisites**:
- [x] 3.2.1: Lesson Service Layer

**Deliverables**:
- [ ] Add unit tests in `src/lessons/storage.rs`:

```rust
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
        let lesson = ls.add("Test", "Content", &["rust".into()], &Severity::Info).unwrap();
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
        let critical = ls.list(Some(&Severity::Critical), 50).unwrap();
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].title, "B");
    }

    #[test]
    fn test_delete_lesson() {
        let storage = setup();
        let ls = LessonStorage::new(storage.connection());
        let lesson = ls.add("Delete me", "Content", &[], &Severity::Info).unwrap();
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
```

- [ ] Create `tests/lessons_test.rs` with integration tests
- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test(lessons): unit and integration tests for lessons system"
```

**Success Criteria**:
- [ ] All lesson tests pass
- [ ] CRUD operations verified
- [ ] Full verification chain passes

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 3.2 Complete — Squash Merge
- [ ] All subtasks 3.2.1–3.2.2 complete
- [ ] Full verification: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/3-2-lesson-service
git commit -m "feat(lessons): complete task 3.2 - lesson service and tests"
git branch -d feature/3-2-lesson-service
git push origin main
```

---

*Phase 3 complete when both tasks merged to main.*
