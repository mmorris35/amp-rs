// MCP Tool implementations — thin wrappers around service layer
// No business logic here, just delegation to service layer methods

use super::AmpMcpServer;
use serde_json::{json, Value};
use std::str::FromStr;

impl AmpMcpServer {
    pub async fn tool_add_lesson(
        &self,
        title: &str,
        content: &str,
        tags: Option<Vec<String>>,
        severity: Option<&str>,
    ) -> Result<Value, String> {
        let tags = tags.unwrap_or_default();
        let severity_level = match severity {
            Some(s) => crate::lessons::Severity::from_str(s)
                .map_err(|e| format!("Invalid severity: {}", e))?,
            None => crate::lessons::Severity::Info,
        };

        match self
            .lesson_service
            .add_lesson(title, content, &tags, &severity_level)
            .await
        {
            Ok(lesson) => serde_json::to_value(&lesson)
                .map_err(|e| format!("Serialization error: {}", e)),
            Err(e) => Err(format!("Failed to add lesson: {}", e)),
        }
    }

    pub async fn tool_search_lessons(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Value, String> {
        let limit = limit.unwrap_or(5);

        match self.lesson_service.search_lessons(query, limit).await {
            Ok(results) => {
                let formatted: Vec<Value> = results
                    .into_iter()
                    .map(|(lesson, score)| json!({"lesson": lesson, "score": score}))
                    .collect();
                Ok(Value::Array(formatted))
            }
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }

    pub async fn tool_list_lessons(
        &self,
        severity: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Value, String> {
        let severity_level = match severity {
            Some(s) => Some(
                crate::lessons::Severity::from_str(s)
                    .map_err(|e| format!("Invalid severity: {}", e))?,
            ),
            None => None,
        };
        let limit = limit.unwrap_or(50);

        match self
            .lesson_service
            .list_lessons(severity_level.as_ref(), limit)
            .await
        {
            Ok(lessons) => serde_json::to_value(&lessons)
                .map_err(|e| format!("Serialization error: {}", e)),
            Err(e) => Err(format!("Failed to list lessons: {}", e)),
        }
    }

    pub async fn tool_delete_lesson(&self, id: &str) -> Result<Value, String> {
        match self.lesson_service.delete_lesson(id).await {
            Ok(deleted) => Ok(json!({ "deleted": deleted })),
            Err(e) => Err(format!("Failed to delete lesson: {}", e)),
        }
    }

    pub async fn tool_add_checkpoint(
        &self,
        agent: &str,
        working_on: &str,
        state: Option<Value>,
    ) -> Result<Value, String> {
        let state = state.unwrap_or(Value::Object(Default::default()));

        match self
            .checkpoint_service
            .add_checkpoint(agent, working_on, &state)
            .await
        {
            Ok(checkpoint) => serde_json::to_value(&checkpoint)
                .map_err(|e| format!("Serialization error: {}", e)),
            Err(e) => Err(format!("Failed to add checkpoint: {}", e)),
        }
    }

    pub async fn tool_get_recent_checkpoints(
        &self,
        agent: &str,
        limit: Option<usize>,
    ) -> Result<Value, String> {
        let limit = limit.unwrap_or(5);

        match self
            .checkpoint_service
            .get_recent(agent, limit)
            .await
        {
            Ok(checkpoints) => serde_json::to_value(&checkpoints)
                .map_err(|e| format!("Serialization error: {}", e)),
            Err(e) => Err(format!("Failed to get recent checkpoints: {}", e)),
        }
    }

    pub async fn tool_search_checkpoints(
        &self,
        query: &str,
        agent: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Value, String> {
        let limit = limit.unwrap_or(5);

        match self.checkpoint_service.search(query, limit, agent).await {
            Ok(results) => {
                let formatted: Vec<Value> = results
                    .into_iter()
                    .map(|(checkpoint, score)| json!({"checkpoint": checkpoint, "score": score}))
                    .collect();
                Ok(Value::Array(formatted))
            }
            Err(e) => Err(format!("Search failed: {}", e)),
        }
    }

    pub async fn tool_get_agent_status(&self, agent: &str) -> Result<Value, String> {
        match self.checkpoint_service.get_agent_status(agent).await {
            Ok(status) => serde_json::to_value(&status)
                .map_err(|e| format!("Serialization error: {}", e)),
            Err(e) => Err(format!("Failed to get agent status: {}", e)),
        }
    }

    pub async fn tool_search_code(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Value, String> {
        let _limit = limit.unwrap_or(10);
        Err(format!(
            "search_code not yet fully integrated (query: '{}')",
            query
        ))
    }

    pub async fn tool_index_repo(&self, path: &str) -> Result<Value, String> {
        Err(format!("index_repo not yet fully integrated (path: '{}')", path))
    }

    pub async fn tool_diff_index(&self, path: &str) -> Result<Value, String> {
        Err(format!("diff_index not yet fully integrated (path: '{}')", path))
    }

    pub async fn tool_full_reindex(&self, path: &str) -> Result<Value, String> {
        Err(format!(
            "full_reindex not yet fully integrated (path: '{}')",
            path
        ))
    }

    pub async fn tool_get_status(&self) -> Result<Value, String> {
        Ok(json!({
            "lessons_count": 0,
            "checkpoints_count": 0,
            "chunks_count": 0,
            "indexed_repos": [],
            "uptime_seconds": 0
        }))
    }
}
