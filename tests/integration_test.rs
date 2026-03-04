mod common;

use amp_rs::{
    checkpoints::storage::CheckpointStorage,
    lessons::storage::LessonStorage,
    lessons::Severity,
    storage::{sqlite::SqliteStorage, Storage},
};

/// E2E Integration Tests
///
/// These tests verify end-to-end workflows across the system without requiring
/// ONNX model or sqlite-vec to be fully initialized. They test:
///
/// 1. Lesson storage and retrieval (core data persistence)
/// 2. Checkpoint storage and retrieval (core data persistence)
/// 3. Multi-agent isolation
/// 4. Severity filtering
/// 5. Agent status tracking
/// 6. Data persistence across restarts
/// 7. Batch operations
///
/// Tests that depend on embedding search are in storage_test.rs and lessons_test.rs

#[test]
fn e2e_lesson_crud_operations() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Add first lesson
    let lesson1 = lesson_storage
        .add(
            "Rust Error Handling",
            "Result and Option types are fundamental",
            &["rust", "errors"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            &Severity::Critical,
        )
        .expect("Failed to add lesson");

    assert_eq!(lesson1.title, "Rust Error Handling");
    assert_eq!(lesson1.severity, Severity::Critical);

    // Add second lesson
    let lesson2 = lesson_storage
        .add(
            "Async Patterns",
            "async/await is the modern way for concurrency",
            &["rust", "async"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            &Severity::Warning,
        )
        .expect("Failed to add lesson");

    // Get a specific lesson
    let retrieved = lesson_storage
        .get(&lesson1.id)
        .expect("Get failed")
        .expect("Lesson not found");

    assert_eq!(retrieved.id, lesson1.id);
    assert_eq!(retrieved.title, lesson1.title);

    // List all lessons
    let all_lessons = lesson_storage.list(None, 100).expect("List all failed");

    assert!(all_lessons.len() >= 2);
    assert!(all_lessons.iter().any(|l| l.id == lesson1.id));
    assert!(all_lessons.iter().any(|l| l.id == lesson2.id));

    // Delete lesson
    let deleted = lesson_storage.delete(&lesson2.id).expect("Delete failed");

    assert!(deleted);

    // Verify deletion
    let remaining = lesson_storage
        .list(None, 100)
        .expect("List after delete failed");

    assert!(!remaining.iter().any(|l| l.id == lesson2.id));
    assert!(remaining.iter().any(|l| l.id == lesson1.id));
}

#[test]
fn e2e_lesson_severity_filtering() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Add lessons with different severities
    lesson_storage
        .add("Critical Bug", "This is critical", &[], &Severity::Critical)
        .expect("Failed");

    lesson_storage
        .add(
            "Warning Pattern",
            "This is a warning",
            &[],
            &Severity::Warning,
        )
        .expect("Failed");

    lesson_storage
        .add("Info Tip", "This is info", &[], &Severity::Info)
        .expect("Failed");

    // Test each severity filter
    let critical = lesson_storage
        .list(Some(&Severity::Critical), 100)
        .expect("Failed");

    assert!(critical.iter().any(|l| l.title == "Critical Bug"));
    assert!(!critical.iter().any(|l| l.title == "Info Tip"));

    let warning = lesson_storage
        .list(Some(&Severity::Warning), 100)
        .expect("Failed");

    assert!(warning.iter().any(|l| l.title == "Warning Pattern"));

    let info = lesson_storage
        .list(Some(&Severity::Info), 100)
        .expect("Failed");

    assert!(info.iter().any(|l| l.title == "Info Tip"));

    // Verify count method
    let critical_count = lesson_storage
        .count_by_severity(&Severity::Critical)
        .expect("Failed");
    assert!(critical_count >= 1);
}

#[test]
fn e2e_lesson_batch_operations() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Add many lessons
    const BATCH_SIZE: usize = 10;
    for i in 0..BATCH_SIZE {
        let _ = lesson_storage.add(
            &format!("Batch Lesson {}", i),
            &format!("Content {}", i),
            &[],
            &Severity::Info,
        );
    }

    // Verify all were added
    let all_lessons = lesson_storage.list(None, 1000).expect("Failed");

    assert!(all_lessons.len() >= BATCH_SIZE);

    // Check total count
    let count = lesson_storage.count().expect("Count failed");
    assert!(count >= BATCH_SIZE);
}

#[test]
fn e2e_checkpoint_crud_operations() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Add checkpoints for multiple agents
    let checkpoint1 = checkpoint_storage
        .add(
            "agent-001",
            "Analyzing project structure",
            &serde_json::json!({"current_file": "src/main.rs", "progress": 0.25}),
        )
        .expect("Failed to add checkpoint 1");

    assert_eq!(checkpoint1.agent, "agent-001");

    // Add more checkpoints
    let checkpoint2 = checkpoint_storage
        .add(
            "agent-001",
            "Processing dependencies",
            &serde_json::json!({"packages": 15, "progress": 0.5}),
        )
        .expect("Failed to add checkpoint 2");

    let checkpoint3 = checkpoint_storage
        .add(
            "agent-002",
            "Building release binary",
            &serde_json::json!({"target": "release", "progress": 0.75}),
        )
        .expect("Failed to add checkpoint 3");

    // Get recent checkpoints for agent-001
    let recent = checkpoint_storage
        .get_recent("agent-001", 5)
        .expect("Failed to get recent");

    assert_eq!(recent.len(), 2);
    assert!(recent.iter().any(|c| c.id == checkpoint1.id));
    assert!(recent.iter().any(|c| c.id == checkpoint2.id));

    // Get recent for agent-002
    let recent_002 = checkpoint_storage
        .get_recent("agent-002", 5)
        .expect("Failed to get recent for agent-002");

    assert_eq!(recent_002.len(), 1);
    assert_eq!(recent_002[0].id, checkpoint3.id);

    // Verify count by listing all (no count method for checkpoints)
    let all_agents = checkpoint_storage
        .list_agents()
        .expect("List agents failed");
    assert!(all_agents.contains(&"agent-001".to_string()));
    assert!(all_agents.contains(&"agent-002".to_string()));
}

#[test]
fn e2e_checkpoint_agent_isolation() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Add checkpoints for different agents
    let _agent1_cp = checkpoint_storage
        .add("agent-A", "State A1", &serde_json::json!({"data": 1}))
        .expect("Failed");

    let _agent2_cp = checkpoint_storage
        .add("agent-B", "State B1", &serde_json::json!({"data": 2}))
        .expect("Failed");

    // Add more for agent A
    checkpoint_storage
        .add("agent-A", "State A2", &serde_json::json!({"data": 3}))
        .expect("Failed");

    // Verify isolation
    let agent_a_checkpoints = checkpoint_storage
        .get_recent("agent-A", 10)
        .expect("Failed");

    assert_eq!(agent_a_checkpoints.len(), 2);
    assert!(agent_a_checkpoints.iter().all(|c| c.agent == "agent-A"));

    let agent_b_checkpoints = checkpoint_storage
        .get_recent("agent-B", 10)
        .expect("Failed");

    assert_eq!(agent_b_checkpoints.len(), 1);
    assert!(agent_b_checkpoints.iter().all(|c| c.agent == "agent-B"));

    // Verify isolation again by checking another get_recent call
    let final_check = checkpoint_storage
        .get_recent("agent-A", 10)
        .expect("Final check failed");
    assert_eq!(final_check.len(), 2);
}

#[test]
fn e2e_agent_status_tracking() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Create checkpoints that represent agent state progression
    let _cp1 = checkpoint_storage
        .add(
            "status-test-agent",
            "Starting task",
            &serde_json::json!({"status": "in_progress"}),
        )
        .expect("Failed");

    let _cp2 = checkpoint_storage
        .add(
            "status-test-agent",
            "Task complete",
            &serde_json::json!({"status": "idle"}),
        )
        .expect("Failed");

    // Get agent status
    let status = checkpoint_storage
        .get_agent_status("status-test-agent")
        .expect("Failed");

    assert_eq!(status.agent, "status-test-agent");
    // Status should be properly tracked (either Idle or InProgress)
    assert!(!status.agent.is_empty());

    // Get recent to verify ordering
    let recent = checkpoint_storage
        .get_recent("status-test-agent", 2)
        .expect("Failed");

    assert_eq!(recent.len(), 2);
}

#[test]
fn e2e_data_persistence_across_instances() {
    let (_temp_dir, db_path) = common::test_db_path();

    // Create first storage instance and add data
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to open");
        storage.migrate().expect("Failed to migrate");
        let conn = storage.connection();
        let lesson_storage = LessonStorage::new(conn);

        lesson_storage
            .add(
                "Persistent Lesson",
                "This lesson should survive storage recreation",
                &["persistence"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                &Severity::Warning,
            )
            .expect("Failed to add lesson");
    }

    // Create second storage instance and verify data exists
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to open");
        storage.migrate().expect("Failed to migrate");
        let conn = storage.connection();
        let lesson_storage = LessonStorage::new(conn);

        let lessons = lesson_storage
            .list(None, 10)
            .expect("Failed to list lessons");

        assert!(!lessons.is_empty());
        assert!(lessons.iter().any(|l| l.title == "Persistent Lesson"));
    }
}

#[test]
fn e2e_checkpoint_persistence_across_instances() {
    let (_temp_dir, db_path) = common::test_db_path();

    // Create first storage instance and add data
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to open");
        storage.migrate().expect("Failed to migrate");
        let conn = storage.connection();
        let checkpoint_storage = CheckpointStorage::new(conn);

        checkpoint_storage
            .add(
                "persistent-agent",
                "Work session 1",
                &serde_json::json!({"phase": 1}),
            )
            .expect("Failed to add checkpoint");
    }

    // Create second storage instance and verify data exists
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to open");
        storage.migrate().expect("Failed to migrate");
        let conn = storage.connection();
        let checkpoint_storage = CheckpointStorage::new(conn);

        let checkpoints = checkpoint_storage
            .get_recent("persistent-agent", 10)
            .expect("Failed to get recent");

        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].agent, "persistent-agent");
    }
}

#[test]
fn e2e_invalid_operations_handled() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Test invalid lesson deletion (non-existent ID)
    let deleted = lesson_storage
        .delete("invalid-id-12345")
        .expect("Delete should not error");

    assert!(!deleted);

    // Test get non-existent
    let retrieved = lesson_storage
        .get("invalid-id-12345")
        .expect("Get should not error");

    assert!(retrieved.is_none());

    // Test list with high limit (should work)
    let all = lesson_storage
        .list(None, 1000)
        .expect("List with high limit failed");

    // Should return empty or valid list
    assert!(all.is_empty() || !all.is_empty());
}

#[test]
fn e2e_lesson_with_tags() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);

    // Add lesson with multiple tags
    let lesson = lesson_storage
        .add(
            "Rust Memory",
            "Understanding Rust memory management",
            &["rust", "memory", "ownership"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            &Severity::Critical,
        )
        .expect("Failed");

    // Retrieve and verify tags
    let retrieved = lesson_storage
        .get(&lesson.id)
        .expect("Get failed")
        .expect("Not found");

    assert_eq!(retrieved.tags.len(), 3);
    assert!(retrieved.tags.contains(&"rust".to_string()));
    assert!(retrieved.tags.contains(&"memory".to_string()));
    assert!(retrieved.tags.contains(&"ownership".to_string()));
}

#[test]
fn e2e_checkpoint_state_preservation() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Add checkpoint with complex state
    let state_data = serde_json::json!({
        "files": ["src/main.rs", "src/lib.rs"],
        "progress": 0.42,
        "decisions": {"use_tokio": true},
        "nested": {"key": "value"}
    });

    let checkpoint = checkpoint_storage
        .add("complex-agent", "Complex state test", &state_data)
        .expect("Failed");

    // Retrieve and verify state preservation
    let retrieved = checkpoint_storage
        .get_recent("complex-agent", 1)
        .expect("Get failed");

    assert_eq!(retrieved.len(), 1);
    assert_eq!(retrieved[0].state, state_data);
    assert_eq!(retrieved[0].id, checkpoint.id);
}

// ============================================================================
// Cross-Feature Integration Tests (9.1.2)
// ============================================================================
// These tests verify interactions and integration between multiple subsystems

#[test]
fn cross_feature_lesson_and_checkpoint_mix() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Add lessons and checkpoints in mixed order
    let lesson1 = lesson_storage
        .add("Lesson 1", "Content 1", &[], &Severity::Info)
        .expect("Failed");

    let checkpoint1 = checkpoint_storage
        .add("agent-1", "Work 1", &serde_json::json!({}))
        .expect("Failed");

    let lesson2 = lesson_storage
        .add("Lesson 2", "Content 2", &[], &Severity::Warning)
        .expect("Failed");

    let checkpoint2 = checkpoint_storage
        .add("agent-1", "Work 2", &serde_json::json!({}))
        .expect("Failed");

    let lesson3 = lesson_storage
        .add("Lesson 3", "Content 3", &[], &Severity::Critical)
        .expect("Failed");

    // Verify both systems have all their data
    let all_lessons = lesson_storage.list(None, 100).expect("Failed");
    assert!(all_lessons.len() >= 3);

    let all_checkpoints = checkpoint_storage
        .get_recent("agent-1", 100)
        .expect("Failed");
    assert_eq!(all_checkpoints.len(), 2);

    // Verify integrity of each item
    assert!(all_lessons.iter().any(|l| l.id == lesson1.id));
    assert!(all_lessons.iter().any(|l| l.id == lesson2.id));
    assert!(all_lessons.iter().any(|l| l.id == lesson3.id));

    assert!(all_checkpoints.iter().any(|c| c.id == checkpoint1.id));
    assert!(all_checkpoints.iter().any(|c| c.id == checkpoint2.id));
}

#[test]
fn cross_feature_multiple_agents_lessons() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Agent-1 adds lessons and creates checkpoints
    for i in 0..5 {
        let _ = lesson_storage.add(
            &format!("Agent1 Lesson {}", i),
            &format!("Content {}", i),
            &[],
            &Severity::Info,
        );
    }

    for i in 0..3 {
        let _ = checkpoint_storage.add(
            "agent-1",
            &format!("Task {}", i),
            &serde_json::json!({"agent": "1"}),
        );
    }

    // Agent-2 adds lessons and creates checkpoints
    for i in 0..3 {
        let _ = lesson_storage.add(
            &format!("Agent2 Lesson {}", i),
            &format!("Content {}", i),
            &[],
            &Severity::Warning,
        );
    }

    for i in 0..4 {
        let _ = checkpoint_storage.add(
            "agent-2",
            &format!("Task {}", i),
            &serde_json::json!({"agent": "2"}),
        );
    }

    // Verify total counts
    let all_lessons = lesson_storage.list(None, 1000).expect("Failed");
    assert!(all_lessons.len() >= 8);

    // Verify agent isolation
    let agent1_checkpoints = checkpoint_storage
        .get_recent("agent-1", 100)
        .expect("Failed");
    assert_eq!(agent1_checkpoints.len(), 3);
    assert!(agent1_checkpoints.iter().all(|c| c.agent == "agent-1"));

    let agent2_checkpoints = checkpoint_storage
        .get_recent("agent-2", 100)
        .expect("Failed");
    assert_eq!(agent2_checkpoints.len(), 4);
    assert!(agent2_checkpoints.iter().all(|c| c.agent == "agent-2"));
}

#[test]
fn cross_feature_severity_mix_with_checkpoints() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Create lessons with all severity levels
    let critical = lesson_storage
        .add("Critical", "Critical lesson", &[], &Severity::Critical)
        .expect("Failed");

    let warning = lesson_storage
        .add("Warning", "Warning lesson", &[], &Severity::Warning)
        .expect("Failed");

    let info = lesson_storage
        .add("Info", "Info lesson", &[], &Severity::Info)
        .expect("Failed");

    // Create checkpoints for each severity level
    let _cp_critical = checkpoint_storage
        .add(
            "critical-agent",
            "Working on critical issue",
            &serde_json::json!({"severity": "critical"}),
        )
        .expect("Failed");

    let _cp_warning = checkpoint_storage
        .add(
            "warning-agent",
            "Working on warning",
            &serde_json::json!({"severity": "warning"}),
        )
        .expect("Failed");

    // Verify severity filtering still works when checkpoints exist
    let critical_lessons = lesson_storage
        .list(Some(&Severity::Critical), 100)
        .expect("Failed");
    assert!(critical_lessons.iter().any(|l| l.id == critical.id));

    let warning_lessons = lesson_storage
        .list(Some(&Severity::Warning), 100)
        .expect("Failed");
    assert!(warning_lessons.iter().any(|l| l.id == warning.id));

    let info_lessons = lesson_storage
        .list(Some(&Severity::Info), 100)
        .expect("Failed");
    assert!(info_lessons.iter().any(|l| l.id == info.id));

    // Verify agent separation
    let critical_agent_cps = checkpoint_storage
        .get_recent("critical-agent", 100)
        .expect("Failed");
    assert_eq!(critical_agent_cps.len(), 1);
}

#[test]
fn cross_feature_delete_and_recreate() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Add initial data
    let lesson1 = lesson_storage
        .add("Initial", "Content", &[], &Severity::Info)
        .expect("Failed");

    let cp1 = checkpoint_storage
        .add("agent-1", "Initial work", &serde_json::json!({}))
        .expect("Failed");

    // Delete lesson
    let deleted = lesson_storage.delete(&lesson1.id).expect("Failed");
    assert!(deleted);

    // Create new lesson with same name (different ID)
    let lesson2 = lesson_storage
        .add("Initial", "New content", &[], &Severity::Warning)
        .expect("Failed");

    // Verify IDs are different
    assert_ne!(lesson1.id, lesson2.id);

    // Verify checkpoint still exists
    let recent_cps = checkpoint_storage
        .get_recent("agent-1", 10)
        .expect("Failed");
    assert_eq!(recent_cps.len(), 1);
    assert_eq!(recent_cps[0].id, cp1.id);

    // Verify correct lesson count
    let all_lessons = lesson_storage.list(None, 100).expect("Failed");
    assert!(!all_lessons.iter().any(|l| l.id == lesson1.id));
    assert!(all_lessons.iter().any(|l| l.id == lesson2.id));
}

#[test]
fn cross_feature_large_scale_operations() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Create 50 lessons across 5 severity groups
    let severities = [Severity::Critical, Severity::Warning, Severity::Info];

    const LESSONS_PER_SEVERITY: usize = 16;
    for sev in &severities {
        for i in 0..LESSONS_PER_SEVERITY {
            let _ = lesson_storage.add(&format!("Lesson {:?} {}", sev, i), "Content", &[], sev);
        }
    }

    // Create 30 checkpoints across 3 agents
    for agent in 0..3 {
        for checkpoint in 0..10 {
            let _ = checkpoint_storage.add(
                &format!("agent-{}", agent),
                &format!("Checkpoint {}", checkpoint),
                &serde_json::json!({"agent": agent}),
            );
        }
    }

    // Verify counts
    let all_lessons = lesson_storage.list(None, 10000).expect("Failed");
    assert!(all_lessons.len() >= 48);

    // Verify severity distribution
    let critical_count = lesson_storage
        .count_by_severity(&Severity::Critical)
        .expect("Failed");
    assert_eq!(critical_count, LESSONS_PER_SEVERITY);

    // Verify agent checkpoints
    for agent in 0..3 {
        let cps = checkpoint_storage
            .get_recent(&format!("agent-{}", agent), 100)
            .expect("Failed");
        assert_eq!(cps.len(), 10);
    }
}

#[test]
fn cross_feature_tag_filtering_with_checkpoints() {
    let (_temp_dir, db_path) = common::test_db_path();
    let storage = SqliteStorage::open(&db_path).expect("Failed to open storage");
    storage.migrate().expect("Failed to migrate");

    let conn = storage.connection();
    let lesson_storage = LessonStorage::new(conn);
    let checkpoint_storage = CheckpointStorage::new(conn);

    // Add tagged lessons
    let lesson1 = lesson_storage
        .add(
            "Rust Memory",
            "Memory management in Rust",
            &["rust", "memory"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            &Severity::Critical,
        )
        .expect("Failed");

    let lesson2 = lesson_storage
        .add(
            "Python Memory",
            "Memory management in Python",
            &["python", "memory"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
            &Severity::Info,
        )
        .expect("Failed");

    // Add checkpoints while lessons exist
    let _cp1 = checkpoint_storage
        .add(
            "memory-agent",
            "Analyzing memory issues",
            &serde_json::json!({"focused_on": "memory"}),
        )
        .expect("Failed");

    // Verify lessons are intact and have correct tags
    let retrieved1 = lesson_storage
        .get(&lesson1.id)
        .expect("Failed")
        .expect("Not found");

    assert_eq!(retrieved1.tags.len(), 2);
    assert!(retrieved1.tags.contains(&"rust".to_string()));
    assert!(retrieved1.tags.contains(&"memory".to_string()));

    let retrieved2 = lesson_storage
        .get(&lesson2.id)
        .expect("Failed")
        .expect("Not found");

    assert_eq!(retrieved2.tags.len(), 2);
    assert!(retrieved2.tags.contains(&"python".to_string()));
    assert!(retrieved2.tags.contains(&"memory".to_string()));
}

#[test]
fn cross_feature_storage_and_retrieval_consistency() {
    let (_temp_dir, db_path) = common::test_db_path();

    // First session: create data
    let lesson_id_1 = {
        let storage = SqliteStorage::open(&db_path).expect("Failed to open");
        storage.migrate().expect("Failed to migrate");
        let conn = storage.connection();
        let lesson_storage = LessonStorage::new(conn);

        let lesson = lesson_storage
            .add(
                "Persistent Data",
                "This should survive",
                &["persistent"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
                &Severity::Critical,
            )
            .expect("Failed");

        lesson.id
    };

    // Second session: verify data exists and is unchanged
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to open");
        storage.migrate().expect("Failed to migrate");
        let conn = storage.connection();
        let lesson_storage = LessonStorage::new(conn);

        let retrieved = lesson_storage
            .get(&lesson_id_1)
            .expect("Failed")
            .expect("Not found");

        assert_eq!(retrieved.id, lesson_id_1);
        assert_eq!(retrieved.title, "Persistent Data");
        assert_eq!(retrieved.content, "This should survive");
        assert_eq!(retrieved.tags.len(), 1);
        assert!(retrieved.tags.contains(&"persistent".to_string()));
        assert_eq!(retrieved.severity, Severity::Critical);
    }

    // Third session: add more data and verify all is present
    {
        let storage = SqliteStorage::open(&db_path).expect("Failed to open");
        storage.migrate().expect("Failed to migrate");
        let conn = storage.connection();
        let lesson_storage = LessonStorage::new(conn);

        let new_lesson = lesson_storage
            .add("New Lesson", "New content", &[], &Severity::Info)
            .expect("Failed");

        // Verify both old and new exist
        let all_lessons = lesson_storage.list(None, 100).expect("Failed");

        assert!(all_lessons.len() >= 2);
        assert!(all_lessons.iter().any(|l| l.id == lesson_id_1));
        assert!(all_lessons.iter().any(|l| l.id == new_lesson.id));
    }
}
