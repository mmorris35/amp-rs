use amp_rs::checkpoints::{AgentState};
use amp_rs::checkpoints::storage::CheckpointStorage;
use amp_rs::storage::sqlite::SqliteStorage;
use amp_rs::storage::Storage;
use chrono::Utc;

#[test]
fn test_checkpoint_storage_add_and_retrieve() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();

    let checkpoint_storage = CheckpointStorage::new(storage.connection());

    let state = serde_json::json!({ "key": "value" });
    let checkpoint = checkpoint_storage
        .add("test-agent", "testing", &state)
        .unwrap();

    assert_eq!(checkpoint.agent, "test-agent");
    assert_eq!(checkpoint.working_on, "testing");
    assert_eq!(checkpoint.state, state);
}

#[test]
fn test_checkpoint_agent_status() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();

    let checkpoint_storage = CheckpointStorage::new(storage.connection());

    // Add a checkpoint
    let state = serde_json::json!({ "data": "test" });
    checkpoint_storage
        .add("agent1", "current-task", &state)
        .unwrap();

    // Get status
    let status = checkpoint_storage.get_agent_status("agent1").unwrap();
    assert_eq!(status.agent, "agent1");
    assert_eq!(status.status, AgentState::InProgress);
    assert_eq!(status.current_task, Some("current-task".to_string()));
    assert_eq!(status.checkpoint_count, 1);
}

#[test]
fn test_checkpoint_list_agents() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();

    let checkpoint_storage = CheckpointStorage::new(storage.connection());

    // Add checkpoints for different agents
    let state = serde_json::json!({});
    checkpoint_storage
        .add("agent-a", "task", &state)
        .unwrap();
    checkpoint_storage
        .add("agent-b", "task", &state)
        .unwrap();
    checkpoint_storage
        .add("agent-c", "task", &state)
        .unwrap();

    // List agents
    let agents = checkpoint_storage.list_agents().unwrap();
    assert_eq!(agents.len(), 3);
    assert!(agents.contains(&"agent-a".to_string()));
    assert!(agents.contains(&"agent-b".to_string()));
    assert!(agents.contains(&"agent-c".to_string()));
}

#[test]
fn test_checkpoint_idle_detection() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();

    let checkpoint_storage = CheckpointStorage::new(storage.connection());

    // Add a recent checkpoint
    let state = serde_json::json!({ "data": "recent" });
    checkpoint_storage
        .add("agent-recent", "current", &state)
        .unwrap();

    let status = checkpoint_storage
        .get_agent_status("agent-recent")
        .unwrap();
    assert_eq!(status.status, AgentState::InProgress);

    // Add an old checkpoint
    let old_time = Utc::now() - chrono::Duration::hours(2);
    let id = uuid::Uuid::new_v4().to_string();
    let state_str = serde_json::to_string(&state).unwrap();

    storage
        .connection()
        .execute(
            "INSERT INTO checkpoints (id, agent, working_on, state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, "agent-old", "old-task", state_str, old_time.to_rfc3339()],
        )
        .unwrap();

    let old_status = checkpoint_storage.get_agent_status("agent-old").unwrap();
    assert_eq!(old_status.status, AgentState::Idle);
}

#[test]
fn test_checkpoint_get_recent() {
    let storage = SqliteStorage::open_in_memory().unwrap();
    storage.migrate().unwrap();

    let checkpoint_storage = CheckpointStorage::new(storage.connection());

    // Add multiple checkpoints
    let state1 = serde_json::json!({ "task": 1 });
    let state2 = serde_json::json!({ "task": 2 });
    let state3 = serde_json::json!({ "task": 3 });

    checkpoint_storage
        .add("agent", "task1", &state1)
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    checkpoint_storage
        .add("agent", "task2", &state2)
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    checkpoint_storage
        .add("agent", "task3", &state3)
        .unwrap();

    // Get recent checkpoints
    let recent = checkpoint_storage.get_recent("agent", 2).unwrap();
    assert_eq!(recent.len(), 2);
    // Most recent first
    assert_eq!(recent[0].working_on, "task3");
    assert_eq!(recent[1].working_on, "task2");
}
