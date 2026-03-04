use axum::{extract::State, routing::get, Json, Router};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Arc;
use std::time::SystemTime;
use tower_http::cors::CorsLayer;

#[derive(Serialize, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Serialize, Clone)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub lessons_count: usize,
    pub checkpoints_count: usize,
    pub chunks_count: usize,
    pub uptime_seconds: u64,
}

/// Application state shared across HTTP handlers
#[derive(Clone)]
pub struct AppState {
    /// Database path - can be opened fresh when needed
    pub db_path: Arc<String>,
    pub start_time: SystemTime,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(server_status))
        .with_state(state)
        .layer(cors)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn server_status(State(state): State<AppState>) -> Json<StatusResponse> {
    // Try to open a connection and get counts
    let (lessons_count, checkpoints_count, chunks_count) =
        match Connection::open(state.db_path.as_str()) {
            Ok(conn) => {
                let lessons: i64 = conn
                    .query_row("SELECT COUNT(*) FROM lessons", [], |row| row.get(0))
                    .unwrap_or(0);
                let checkpoints: i64 = conn
                    .query_row("SELECT COUNT(*) FROM checkpoints", [], |row| row.get(0))
                    .unwrap_or(0);
                let chunks: i64 = conn
                    .query_row("SELECT COUNT(*) FROM code_chunks", [], |row| row.get(0))
                    .unwrap_or(0);
                (lessons as usize, checkpoints as usize, chunks as usize)
            }
            Err(_) => (0, 0, 0),
        };

    // Calculate uptime
    let uptime = state.start_time.elapsed().map(|d| d.as_secs()).unwrap_or(0);

    Json(StatusResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        lessons_count,
        checkpoints_count,
        chunks_count,
        uptime_seconds: uptime,
    })
}
