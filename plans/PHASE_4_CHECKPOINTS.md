# Phase 4: Checkpoint System

**Goal**: Agent checkpoint save/restore, semantic search, and agent status tracking
**Duration**: 2–3 days
**Wave**: 2 (parallel with Phase 3 and Phase 5)
**Dependencies**: Phase 1 (storage) + Phase 2 (embeddings) complete

---

## Task 4.1: Checkpoint Model and Storage

**Git**: `git checkout -b feature/4-1-checkpoint-model`

### Subtask 4.1.1: Checkpoint Model and Storage Operations (Single Session)

**Prerequisites**:
- [x] 1.2.2: Storage Integration Tests
- [x] 2.2.2: Embedding Engine Tests

**Deliverables**:
- [ ] Create `src/checkpoints/mod.rs`:

```rust
pub mod service;
pub mod storage;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub agent: String,
    pub working_on: String,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent: String,
    pub status: AgentState,
    pub current_task: Option<String>,
    pub checkpoint_count: usize,
    pub last_checkpoint_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    InProgress,
}
```

- [ ] Create `src/checkpoints/storage.rs`:

```rust
use super::{AgentState, AgentStatus, Checkpoint};
use crate::error::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

pub struct CheckpointStorage<'a> {
    conn: &'a Connection,
}

impl<'a> CheckpointStorage<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

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

    pub fn get_recent(&self, agent: &str, limit: usize) -> Result<Vec<Checkpoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, agent, working_on, state, created_at
             FROM checkpoints WHERE agent = ?1
             ORDER BY created_at DESC LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![agent, limit as i64], |row| {
            let state_str: String = row.get(3)?;
            let created_str: String = row.get(4)?;
            Ok(Checkpoint {
                id: row.get(0)?,
                agent: row.get(1)?,
                working_on: row.get(2)?,
                state: serde_json::from_str(&state_str).unwrap_or_default(),
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;

        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn store_embedding(&self, checkpoint_id: &str, embedding: &[f32]) -> Result<()> {
        self.conn.execute(
            "INSERT INTO checkpoint_embeddings (id, embedding) VALUES (?1, ?2)",
            params![checkpoint_id, embedding.as_bytes()],
        )?;
        Ok(())
    }

    pub fn search_by_embedding(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<(Checkpoint, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.agent, c.working_on, c.state, c.created_at, e.distance
             FROM checkpoint_embeddings e
             JOIN checkpoints c ON c.id = e.id
             WHERE e.embedding MATCH ?1
             ORDER BY e.distance
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(
            params![query_embedding.as_bytes(), limit as i64],
            |row| {
                let state_str: String = row.get(3)?;
                let created_str: String = row.get(4)?;
                let distance: f64 = row.get(5)?;
                Ok((
                    Checkpoint {
                        id: row.get(0)?,
                        agent: row.get(1)?,
                        working_on: row.get(2)?,
                        state: serde_json::from_str(&state_str).unwrap_or_default(),
                        created_at: DateTime::parse_from_rfc3339(&created_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    },
                    distance,
                ))
            },
        )?;

        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_agent_status(&self, agent: &str) -> Result<AgentStatus> {
        let checkpoint_count: usize = self.conn.query_row(
            "SELECT COUNT(*) FROM checkpoints WHERE agent = ?1",
            params![agent],
            |row| row.get(0),
        )?;

        let latest = self.get_recent(agent, 1)?;
        let (status, current_task, last_at) = if let Some(cp) = latest.first() {
            (AgentState::InProgress, Some(cp.working_on.clone()), Some(cp.created_at))
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
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(checkpoints): checkpoint model and storage operations"
```

**Success Criteria**:
- [ ] `cargo check` exits 0
- [ ] CRUD: add, get_recent, search, agent_status
- [ ] Agent status tracks idle/in_progress

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 4.1.2: Checkpoint Semantic Search (Single Session)

**Prerequisites**:
- [x] 4.1.1: Checkpoint Model and Storage Operations

**Deliverables**:
- [ ] Verify search_by_embedding and store_embedding work with sqlite-vec
- [ ] Add optional agent filter to search
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(checkpoints): semantic search with optional agent filter"
```

**Success Criteria**:
- [ ] Search returns results ordered by distance
- [ ] Agent filter works

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 4.1.3: Agent Status Tracking (Single Session)

**Prerequisites**:
- [x] 4.1.2: Checkpoint Semantic Search

**Deliverables**:
- [ ] Enhance agent status with time-based idle detection (if last checkpoint > 1 hour ago, status = idle)
- [ ] Add `list_agents` to return all known agents
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(checkpoints): enhanced agent status tracking"
```

**Success Criteria**:
- [ ] Time-based idle detection works
- [ ] List agents returns all unique agents

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 4.1 Complete — Squash Merge
- [ ] All subtasks 4.1.1–4.1.3 complete
- [ ] `cargo check` passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/4-1-checkpoint-model
git commit -m "feat(checkpoints): complete task 4.1 - checkpoint model, storage, and status"
git branch -d feature/4-1-checkpoint-model
git push origin main
```

---

## Task 4.2: Checkpoint Service and Tests

**Git**: `git checkout -b feature/4-2-checkpoint-service`

### Subtask 4.2.1: Checkpoint Service Layer (Single Session)

**Prerequisites**:
- [x] 4.1.3: Agent Status Tracking

**Deliverables**:
- [ ] Create `src/checkpoints/service.rs` composing storage + embeddings (same pattern as LessonService)
- [ ] Async methods: add_checkpoint, get_recent, search, get_agent_status
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(checkpoints): service layer composing storage and embeddings"
```

**Success Criteria**:
- [ ] Service composes storage + embedding pool
- [ ] All async methods work

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 4.2.2: Checkpoint Unit and Integration Tests (Single Session)

**Prerequisites**:
- [x] 4.2.1: Checkpoint Service Layer

**Deliverables**:
- [ ] Unit tests in `src/checkpoints/storage.rs`:
  - test_add_checkpoint
  - test_get_recent
  - test_get_agent_status_idle
  - test_get_agent_status_in_progress
  - test_multiple_agents
  - test_list_agents

- [ ] Create `tests/checkpoints_test.rs` with integration tests
- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test(checkpoints): unit and integration tests for checkpoint system"
```

**Success Criteria**:
- [ ] All checkpoint tests pass
- [ ] Full verification chain passes

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 4.2 Complete — Squash Merge
- [ ] All subtasks 4.2.1–4.2.2 complete
- [ ] Full verification passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/4-2-checkpoint-service
git commit -m "feat(checkpoints): complete task 4.2 - checkpoint service and tests"
git branch -d feature/4-2-checkpoint-service
git push origin main
```

---

*Phase 4 complete when both tasks merged to main.*
