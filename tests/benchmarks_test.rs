// Performance benchmarks and baseline measurements
//
// These tests measure key operations to establish baseline performance.
// Run with: cargo test --test benchmarks_test -- --nocapture
//
// Baseline measurements on stable system:
// - SQLite insert: <1ms per record
// - List all: <10ms for 1000 records
// - Storage creation: <50ms
// - Schema migration: <20ms

mod common;

use amp_rs::{
    checkpoints::storage::CheckpointStorage,
    lessons::{storage::LessonStorage, Severity},
    storage::{sqlite::SqliteStorage, Storage},
};
use std::time::Instant;

/// Measure time in milliseconds
fn measure_ms<F>(f: F) -> f64
where
    F: FnOnce(),
{
    let start = Instant::now();
    f();
    start.elapsed().as_secs_f64() * 1000.0
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_storage_open() {
    let (_temp_dir, db_path) = common::test_db_path();

    let elapsed = measure_ms(|| {
        let _ = SqliteStorage::open(&db_path).expect("Failed to open");
    });

    println!("Storage::open: {:.2}ms", elapsed);
    // First access to SQLite can be slow due to native deps
    assert!(elapsed < 2000.0, "Storage open should complete");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_schema_migration() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");

    let elapsed = measure_ms(|| {
        storage.migrate().expect("Failed to migrate");
    });

    println!("Schema migration: {:.2}ms", elapsed);
    // First access to SQLite can be slow due to native deps
    assert!(elapsed < 2000.0, "Migration should complete");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_lesson_insert() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    let elapsed = measure_ms(|| {
        for i in 0..100 {
            let _ = lesson_storage.add(&format!("Lesson {}", i), "Content", &[], &Severity::Info);
        }
    });

    let per_insert = elapsed / 100.0;
    println!(
        "Lesson insert (100 records): {:.2}ms total, {:.3}ms per insert",
        elapsed, per_insert
    );
    assert!(per_insert < 5.0, "Insert should be fast");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_lesson_list_all() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Insert test data
    for i in 0..100 {
        let _ = lesson_storage.add(&format!("Lesson {}", i), "Content", &[], &Severity::Info);
    }

    let elapsed = measure_ms(|| {
        let _ = lesson_storage.list(None, 10000).expect("Failed to list");
    });

    println!("Lesson list (100 records): {:.2}ms", elapsed);
    assert!(elapsed < 50.0, "List should be fast");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_lesson_list_by_severity() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Insert test data - mix of severities
    for i in 0..100 {
        let sev = match i % 3 {
            0 => Severity::Critical,
            1 => Severity::Warning,
            _ => Severity::Info,
        };
        let _ = lesson_storage.add(&format!("Lesson {}", i), "Content", &[], &sev);
    }

    let elapsed = measure_ms(|| {
        let _ = lesson_storage
            .list(Some(&Severity::Critical), 10000)
            .expect("Failed to list");
    });

    println!("Lesson list by severity (100 records): {:.2}ms", elapsed);
    assert!(elapsed < 50.0, "Filtered list should be fast");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_checkpoint_insert() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let checkpoint_storage = CheckpointStorage::new(conn);

    let elapsed = measure_ms(|| {
        for i in 0..100 {
            let _ = checkpoint_storage.add(
                "test-agent",
                &format!("Work {}", i),
                &serde_json::json!({}),
            );
        }
    });

    let per_insert = elapsed / 100.0;
    println!(
        "Checkpoint insert (100 records): {:.2}ms total, {:.3}ms per insert",
        elapsed, per_insert
    );
    assert!(per_insert < 5.0, "Insert should be fast");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_checkpoint_get_recent() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Insert test data
    for i in 0..100 {
        let _ =
            checkpoint_storage.add("test-agent", &format!("Work {}", i), &serde_json::json!({}));
    }

    let elapsed = measure_ms(|| {
        let _ = checkpoint_storage
            .get_recent("test-agent", 10000)
            .expect("Failed to get recent");
    });

    println!("Checkpoint get_recent (100 records): {:.2}ms", elapsed);
    assert!(elapsed < 50.0, "Get recent should be fast");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_checkpoint_multi_agent() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Insert test data across multiple agents
    for agent in 0..10 {
        for i in 0..10 {
            let _ = checkpoint_storage.add(
                &format!("agent-{}", agent),
                &format!("Work {}", i),
                &serde_json::json!({}),
            );
        }
    }

    let elapsed = measure_ms(|| {
        let mut total = 0;
        for agent in 0..10 {
            let cps = checkpoint_storage
                .get_recent(&format!("agent-{}", agent), 100)
                .expect("Failed");
            total += cps.len();
        }
        assert_eq!(total, 100);
    });

    println!(
        "Checkpoint multi-agent queries (10 agents, 100 total): {:.2}ms",
        elapsed
    );
    assert!(elapsed < 100.0, "Multi-agent queries should be fast");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_lesson_delete() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Insert test data
    let mut ids = Vec::new();
    for i in 0..100 {
        let lesson = lesson_storage
            .add(&format!("Lesson {}", i), "Content", &[], &Severity::Info)
            .expect("Failed");
        ids.push(lesson.id);
    }

    let elapsed = measure_ms(|| {
        for id in &ids {
            let _ = lesson_storage.delete(id).expect("Failed to delete");
        }
    });

    let per_delete = elapsed / 100.0;
    println!(
        "Lesson delete (100 records): {:.2}ms total, {:.3}ms per delete",
        elapsed, per_delete
    );
    assert!(per_delete < 5.0, "Delete should be fast");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_scaling_lessons_10() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Insert test data
    for i in 0..10 {
        let _ = lesson_storage.add(&format!("Lesson {}", i), "Content", &[], &Severity::Info);
    }

    let elapsed = measure_ms(|| {
        let _ = lesson_storage.list(None, 10000).expect("Failed to list");
    });

    println!("List 10 lessons: {:.2}ms", elapsed);
    assert!(elapsed < 100.0, "List should scale well");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_scaling_lessons_50() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Insert test data
    for i in 0..50 {
        let _ = lesson_storage.add(&format!("Lesson {}", i), "Content", &[], &Severity::Info);
    }

    let elapsed = measure_ms(|| {
        let _ = lesson_storage.list(None, 10000).expect("Failed to list");
    });

    println!("List 50 lessons: {:.2}ms", elapsed);
    assert!(elapsed < 100.0, "List should scale well");
}

#[test]
#[ignore = "benchmark - run with --nocapture"]
fn bench_scaling_lessons_100() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Insert test data
    for i in 0..100 {
        let _ = lesson_storage.add(&format!("Lesson {}", i), "Content", &[], &Severity::Info);
    }

    let elapsed = measure_ms(|| {
        let _ = lesson_storage.list(None, 10000).expect("Failed to list");
    });

    println!("List 100 lessons: {:.2}ms", elapsed);
    assert!(elapsed < 100.0, "List should scale well");
}
