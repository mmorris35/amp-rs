use super::{AgentStatus, Checkpoint};
use crate::embedding::pool::EmbeddingPool;
use crate::error::Result;
use crate::storage::Storage;
use std::sync::Arc;

/// Checkpoint service layer composing storage and embeddings
pub struct CheckpointService {
    storage: Arc<dyn Storage>,
    embedding_pool: Arc<EmbeddingPool>,
}

impl CheckpointService {
    /// Create a new checkpoint service
    pub fn new(storage: Arc<dyn Storage>, embedding_pool: Arc<EmbeddingPool>) -> Self {
        Self {
            storage,
            embedding_pool,
        }
    }

    /// Add a new checkpoint with embedding
    pub async fn add_checkpoint(
        &self,
        agent: &str,
        working_on: &str,
        state: &serde_json::Value,
    ) -> Result<Checkpoint> {
        let conn = self.storage.connection();
        let checkpoint_storage = super::storage::CheckpointStorage::new(conn);

        // Add the checkpoint first
        let checkpoint = checkpoint_storage.add(agent, working_on, state)?;

        // Generate and store embedding for the working_on description
        let embedding = self.embedding_pool.embed(working_on.to_string()).await?;

        checkpoint_storage.store_embedding(&checkpoint.id, &embedding)?;

        Ok(checkpoint)
    }

    /// Get recent checkpoints for an agent
    pub async fn get_recent(&self, agent: &str, limit: usize) -> Result<Vec<Checkpoint>> {
        let conn = self.storage.connection();
        let checkpoint_storage = super::storage::CheckpointStorage::new(conn);
        checkpoint_storage.get_recent(agent, limit)
    }

    /// Search checkpoints by semantic similarity to a query
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        agent_filter: Option<&str>,
    ) -> Result<Vec<(Checkpoint, f64)>> {
        let conn = self.storage.connection();
        let checkpoint_storage = super::storage::CheckpointStorage::new(conn);

        // Generate embedding for the query
        let query_embedding = self.embedding_pool.embed(query.to_string()).await?;

        // Search by embedding
        checkpoint_storage.search_by_embedding(&query_embedding, limit, agent_filter)
    }

    /// Get aggregated status for an agent
    pub async fn get_agent_status(&self, agent: &str) -> Result<AgentStatus> {
        let conn = self.storage.connection();
        let checkpoint_storage = super::storage::CheckpointStorage::new(conn);
        checkpoint_storage.get_agent_status(agent)
    }

    /// List all unique agents
    pub async fn list_agents(&self) -> Result<Vec<String>> {
        let conn = self.storage.connection();
        let checkpoint_storage = super::storage::CheckpointStorage::new(conn);
        checkpoint_storage.list_agents()
    }
}
