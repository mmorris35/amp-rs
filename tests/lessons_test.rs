use amp_rs::lessons::storage::LessonStorage;
use amp_rs::lessons::{Lesson, Severity};
use amp_rs::storage::{sqlite::SqliteStorage, Storage};
use std::sync::Arc;

#[test]
fn test_lesson_crud_operations() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    let ls = LessonStorage::new(storage.connection());

    // Test Create
    let lesson = ls
        .add(
            "Rust Ownership",
            "Understanding move semantics",
            &["rust".into(), "memory".into()],
            &Severity::Critical,
        )
        .unwrap();

    assert_eq!(lesson.title, "Rust Ownership");
    assert_eq!(lesson.severity, Severity::Critical);
    assert_eq!(lesson.tags.len(), 2);

    // Test Read
    let found = ls.get(&lesson.id).unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, lesson.id);
    assert_eq!(found.content, "Understanding move semantics");

    // Test Update (simulated via add with same ID - not directly supported)
    // We'll just verify the lesson exists

    // Test Delete
    assert!(ls.delete(&lesson.id).unwrap());
    assert!(ls.get(&lesson.id).unwrap().is_none());
}

#[test]
fn test_lesson_list_and_filter() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    let ls = LessonStorage::new(storage.connection());

    // Add lessons with different severities
    ls.add("Critical Lesson", "Content 1", &[], &Severity::Critical)
        .unwrap();
    ls.add("Warning Lesson", "Content 2", &[], &Severity::Warning)
        .unwrap();
    ls.add("Info Lesson 1", "Content 3", &[], &Severity::Info)
        .unwrap();
    ls.add("Info Lesson 2", "Content 4", &[], &Severity::Info)
        .unwrap();

    // Test list all
    let all = ls.list(None, 100).unwrap();
    assert_eq!(all.len(), 4);

    // Test filter by severity
    let critical = ls.list(Some(&Severity::Critical), 100).unwrap();
    assert_eq!(critical.len(), 1);
    assert_eq!(critical[0].title, "Critical Lesson");

    let warnings = ls.list(Some(&Severity::Warning), 100).unwrap();
    assert_eq!(warnings.len(), 1);

    let info_lessons = ls.list(Some(&Severity::Info), 100).unwrap();
    assert_eq!(info_lessons.len(), 2);

    // Test limit
    let limited = ls.list(None, 2).unwrap();
    assert_eq!(limited.len(), 2);
}

#[test]
fn test_lesson_count() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    let ls = LessonStorage::new(storage.connection());

    assert_eq!(ls.count().unwrap(), 0);

    ls.add("First", "Content 1", &[], &Severity::Info).unwrap();
    assert_eq!(ls.count().unwrap(), 1);

    ls.add("Second", "Content 2", &[], &Severity::Critical)
        .unwrap();
    assert_eq!(ls.count().unwrap(), 2);

    // Count by severity
    assert_eq!(ls.count_by_severity(&Severity::Info).unwrap(), 1);
    assert_eq!(ls.count_by_severity(&Severity::Critical).unwrap(), 1);
    assert_eq!(ls.count_by_severity(&Severity::Warning).unwrap(), 0);
}

#[test]
fn test_lesson_tags() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    let ls = LessonStorage::new(storage.connection());

    let tags = vec!["rust".into(), "performance".into(), "async".into()];
    let lesson = ls
        .add(
            "Async Rust",
            "Making fast concurrent code",
            &tags,
            &Severity::Warning,
        )
        .unwrap();

    let found = ls.get(&lesson.id).unwrap().unwrap();
    assert_eq!(found.tags, tags);
}

#[test]
fn test_lesson_timestamps() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();
    let ls = LessonStorage::new(storage.connection());

    let lesson = ls.add("Test", "Content", &[], &Severity::Info).unwrap();

    assert!(lesson.created_at <= lesson.updated_at);
    assert!(lesson.created_at.timestamp() > 0);
}

#[test]
fn test_lesson_severity_parsing() {
    use std::str::FromStr;

    assert_eq!(Severity::from_str("critical").unwrap(), Severity::Critical);
    assert_eq!(Severity::from_str("warning").unwrap(), Severity::Warning);
    assert_eq!(Severity::from_str("info").unwrap(), Severity::Info);
    assert_eq!(Severity::from_str("CRITICAL").unwrap(), Severity::Critical);

    assert!(Severity::from_str("invalid").is_err());
}

#[test]
fn test_lesson_severity_display() {
    assert_eq!(Severity::Critical.to_string(), "critical");
    assert_eq!(Severity::Warning.to_string(), "warning");
    assert_eq!(Severity::Info.to_string(), "info");
}

// Note: Concurrent testing with rusqlite requires special handling
// as Connection is Send but not Sync. Threading is tested in the service layer
// with the EmbeddingPool which handles async operations safely.
