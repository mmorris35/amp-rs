pub mod service;
pub mod storage;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single agent checkpoint (snapshot of state at a point in time)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique identifier for this checkpoint
    pub id: String,
    /// Agent identifier (e.g., "mmn/my-project")
    pub agent: String,
    /// Description of what the agent was working on
    pub working_on: String,
    /// Arbitrary state data (decisions, flags, file paths, etc.)
    pub state: serde_json::Value,
    /// Timestamp when checkpoint was created
    pub created_at: DateTime<Utc>,
}

/// Aggregated status of an agent based on recent checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Agent identifier
    pub agent: String,
    /// Current status (idle or in_progress)
    pub status: AgentState,
    /// What the agent is currently working on (if in progress)
    pub current_task: Option<String>,
    /// Total number of checkpoints for this agent
    pub checkpoint_count: usize,
    /// Timestamp of the most recent checkpoint
    pub last_checkpoint_at: Option<DateTime<Utc>>,
}

/// Agent operational state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    /// Agent is idle (no recent checkpoint activity)
    Idle,
    /// Agent is actively working (recent checkpoint within idle threshold)
    InProgress,
}
