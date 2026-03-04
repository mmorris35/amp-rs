use super::{AgentState, AgentStatus, Checkpoint};
use crate::error::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

/// Threshold for idle detection: checkpoints older than this are considered idle
const IDLE_THRESHOLD_HOURS: i64 = 1;

/// Database storage operations for checkpoints
pub struct CheckpointStorage<'a> {
    conn: &'a Connection,
}

impl<'a> CheckpointStorage<'a> {
    /// Create a new checkpoint storage instance
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Add a new checkpoint
    pub fn add(
        &self,
        agent: &str,
        working_on: &str,
        state: &serde_json::Value,
    ) -> Result<Checkpoint> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let state_str = serde_json::to_string(state)?;

        self.conn.execute(
            "INSERT INTO checkpoints (id, agent, working_on, state, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, agent, working_on, state_str, now.to_rfc3339()],
        )?;

        Ok(Checkpoint {
            id,
            agent: agent.to_string(),
            working_on: working_on.to_string(),
            state: state.clone(),
            created_at: now,
        })
    }

    /// Get recent checkpoints for an agent
    pub fn get_recent(&self, agent: &str, limit: usize) -> Result<Vec<Checkpoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, working_on, state, created_at
             FROM checkpoints WHERE agent = ?1
             ORDER BY created_at DESC LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![agent, limit as i64], |row| {
            let state_str: String = row.get(3)?;
            let created_str: String = row.get(4)?;
            Ok(Checkpoint {
                id: row.get(0)?,
                agent: row.get(1)?,
                working_on: row.get(2)?,
                state: serde_json::from_str(&state_str).unwrap_or(serde_json::json!({})),
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Store embedding vector for a checkpoint
    pub fn store_embedding(&self, checkpoint_id: &str, embedding: &[f32]) -> Result<()> {
        let embedding_bytes = bytemuck::cast_slice(embedding);
        self.conn.execute(
            "INSERT OR REPLACE INTO checkpoint_embeddings (id, embedding) VALUES (?1, ?2)",
            params![checkpoint_id, embedding_bytes],
        )?;
        Ok(())
    }

    /// Search checkpoints by embedding similarity
    pub fn search_by_embedding(
        &self,
        query_embedding: &[f32],
        limit: usize,
        agent_filter: Option<&str>,
    ) -> Result<Vec<(Checkpoint, f64)>> {
        let query_bytes = bytemuck::cast_slice(query_embedding);

        let results = if let Some(agent) = agent_filter {
            let mut stmt = self.conn.prepare(
                "SELECT c.id, c.agent, c.working_on, c.state, c.created_at, e.distance
                 FROM checkpoint_embeddings e
                 JOIN checkpoints c ON c.id = e.id
                 WHERE c.agent = ?1 AND e.embedding MATCH ?2
                 ORDER BY e.distance
                 LIMIT ?3",
            )?;

            let rows = stmt.query_map(params![agent, query_bytes, limit as i64], |row| {
                let state_str: String = row.get(3)?;
                let created_str: String = row.get(4)?;
                let distance: f64 = row.get(5)?;
                Ok((
                    Checkpoint {
                        id: row.get(0)?,
                        agent: row.get(1)?,
                        working_on: row.get(2)?,
                        state: serde_json::from_str(&state_str).unwrap_or(serde_json::json!({})),
                        created_at: DateTime::parse_from_rfc3339(&created_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    },
                    distance,
                ))
            })?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT c.id, c.agent, c.working_on, c.state, c.created_at, e.distance
                 FROM checkpoint_embeddings e
                 JOIN checkpoints c ON c.id = e.id
                 WHERE e.embedding MATCH ?1
                 ORDER BY e.distance
                 LIMIT ?2",
            )?;

            let rows = stmt.query_map(params![query_bytes, limit as i64], |row| {
                let state_str: String = row.get(3)?;
                let created_str: String = row.get(4)?;
                let distance: f64 = row.get(5)?;
                Ok((
                    Checkpoint {
                        id: row.get(0)?,
                        agent: row.get(1)?,
                        working_on: row.get(2)?,
                        state: serde_json::from_str(&state_str).unwrap_or(serde_json::json!({})),
                        created_at: DateTime::parse_from_rfc3339(&created_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    },
                    distance,
                ))
            })?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };

        Ok(results)
    }

    /// Get aggregated status for an agent with time-based idle detection
    pub fn get_agent_status(&self, agent: &str) -> Result<AgentStatus> {
        let checkpoint_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM checkpoints WHERE agent = ?1",
            params![agent],
            |row| row.get(0),
        )?;

        let latest = self.get_recent(agent, 1)?;
        let (status, current_task, last_at) = if let Some(cp) = latest.first() {
            // Check if checkpoint is within idle threshold
            let now = Utc::now();
            let time_since = now.signed_duration_since(cp.created_at);
            let is_idle = time_since.num_hours() > IDLE_THRESHOLD_HOURS;

            let agent_state = if is_idle {
                AgentState::Idle
            } else {
                AgentState::InProgress
            };

            (
                agent_state,
                Some(cp.working_on.clone()),
                Some(cp.created_at),
            )
        } else {
            (AgentState::Idle, None, None)
        };

        Ok(AgentStatus {
            agent: agent.to_string(),
            status,
            current_task,
            checkpoint_count,
            last_checkpoint_at: last_at,
        })
    }

    /// List all unique agents
    pub fn list_agents(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT agent FROM checkpoints ORDER BY agent")?;

        let rows = stmt.query_map([], |row| row.get(0))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
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
    fn test_add_checkpoint() {
        let db = setup();
        let storage = CheckpointStorage::new(db.connection());

        let state = serde_json::json!({ "key": "value" });
        let result = storage.add("test-agent", "testing", &state);

        assert!(result.is_ok());
        let checkpoint = result.unwrap();
        assert_eq!(checkpoint.agent, "test-agent");
        assert_eq!(checkpoint.working_on, "testing");
        assert_eq!(checkpoint.state, state);
    }

    #[test]
    fn test_get_recent() {
        let db = setup();
        let storage = CheckpointStorage::new(db.connection());

        let state1 = serde_json::json!({ "task": 1 });
        let state2 = serde_json::json!({ "task": 2 });

        storage.add("agent1", "task1", &state1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        storage.add("agent1", "task2", &state2).unwrap();

        let recent = storage.get_recent("agent1", 2).unwrap();
        assert_eq!(recent.len(), 2);
        // Most recent first
        assert_eq!(recent[0].working_on, "task2");
        assert_eq!(recent[1].working_on, "task1");
    }

    #[test]
    fn test_get_agent_status_in_progress() {
        let db = setup();
        let storage = CheckpointStorage::new(db.connection());

        let state = serde_json::json!({ "data": "test" });
        storage.add("agent1", "current-task", &state).unwrap();

        let status = storage.get_agent_status("agent1").unwrap();
        assert_eq!(status.agent, "agent1");
        assert_eq!(status.status, AgentState::InProgress);
        assert_eq!(status.current_task, Some("current-task".to_string()));
        assert_eq!(status.checkpoint_count, 1);
        assert!(status.last_checkpoint_at.is_some());
    }

    #[test]
    fn test_get_agent_status_idle() {
        let db = setup();
        let storage = CheckpointStorage::new(db.connection());

        // Insert a checkpoint with an old timestamp (more than 1 hour ago)
        let old_time = Utc::now() - chrono::Duration::hours(2);
        let id = Uuid::new_v4().to_string();
        let state = serde_json::json!({ "data": "old" });
        let state_str = serde_json::to_string(&state).unwrap();

        db.connection()
            .execute(
                "INSERT INTO checkpoints (id, agent, working_on, state, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, "agent2", "old-task", state_str, old_time.to_rfc3339()],
            )
            .unwrap();

        let status = storage.get_agent_status("agent2").unwrap();
        assert_eq!(status.agent, "agent2");
        assert_eq!(status.status, AgentState::Idle);
        assert_eq!(status.checkpoint_count, 1);
    }

    #[test]
    fn test_multiple_agents() {
        let db = setup();
        let storage = CheckpointStorage::new(db.connection());

        let state = serde_json::json!({});
        storage.add("agent1", "task", &state).unwrap();
        storage.add("agent2", "task", &state).unwrap();
        storage.add("agent3", "task", &state).unwrap();

        let agents = storage.list_agents().unwrap();
        assert_eq!(agents.len(), 3);
        assert_eq!(agents, vec!["agent1", "agent2", "agent3"]);
    }

    #[test]
    fn test_list_agents() {
        let db = setup();
        let storage = CheckpointStorage::new(db.connection());

        let state = serde_json::json!({});
        storage.add("agent-z", "task", &state).unwrap();
        storage.add("agent-a", "task", &state).unwrap();
        storage.add("agent-m", "task", &state).unwrap();
        storage.add("agent-a", "task2", &state).unwrap(); // duplicate agent

        let agents = storage.list_agents().unwrap();
        assert_eq!(agents.len(), 3);
        assert_eq!(agents[0], "agent-a");
        assert_eq!(agents[1], "agent-m");
        assert_eq!(agents[2], "agent-z");
    }
}
