# Phase 8: HTTP & CLI

**Goal**: Axum HTTP server (health check, REST status), clap CLI, and config file support
**Duration**: 2–3 days
**Wave**: 3 (parallel with Phase 6 and Phase 7)
**Dependencies**: Phase 3 + Phase 4 + Phase 5 complete

---

## Task 8.1: HTTP Server and CLI

**Git**: `git checkout -b feature/8-1-http-cli`

### Subtask 8.1.1: Axum HTTP Server with Health Check (Single Session)

**Prerequisites**:
- [x] 3.2.2: Lesson Tests
- [x] 4.2.2: Checkpoint Tests
- [x] 5.2.2: Code Indexing Tests

**Deliverables**:
- [ ] Create `src/http/mod.rs`:

```rust
pub mod routes;

use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

pub async fn start_http_server(
    port: u16,
    state: crate::AppState,
) -> anyhow::Result<()> {
    let app = routes::create_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP server listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
```

- [ ] Create `src/http/routes.rs`:

```rust
use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub lessons_count: usize,
    pub checkpoints_count: usize,
    pub chunks_count: usize,
    pub uptime_seconds: u64,
}

pub fn create_router(state: crate::AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(server_status))
        .with_state(state)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn server_status(
    State(state): State<crate::AppState>,
) -> Result<Json<StatusResponse>, StatusCode> {
    // Query counts from services
    Ok(Json(StatusResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        lessons_count: 0,  // TODO: wire to service
        checkpoints_count: 0,
        chunks_count: 0,
        uptime_seconds: 0,
    }))
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(http): Axum HTTP server with health check and status endpoints"
```

**Success Criteria**:
- [ ] `/health` returns 200 with status "ok"
- [ ] `/status` returns server stats
- [ ] Server binds to configurable port

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 8.1.2: REST Status Endpoint (Single Session)

**Prerequisites**:
- [x] 8.1.1: Axum HTTP Server

**Deliverables**:
- [ ] Wire StatusResponse to actual service counts
- [ ] Add uptime tracking (start time stored in AppState)
- [ ] Add CORS headers via tower-http
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(http): wired status endpoint with CORS support"
```

**Success Criteria**:
- [ ] Status returns real counts
- [ ] CORS headers present

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 8.1.3: Serve Command (MCP + HTTP) (Single Session)

**Prerequisites**:
- [x] 8.1.2: REST Status Endpoint

**Deliverables**:
- [ ] Update `src/main.rs` to implement the `serve` command:
  - Initialize storage, run migrations
  - Initialize embedding engine
  - Create service layers
  - Start HTTP server on separate tokio task
  - Start MCP server on stdio
  - Start file watcher on separate tokio task
  - Handle graceful shutdown (Ctrl+C / SIGTERM)

```rust
// Pseudostructure for main.rs serve command:
async fn run_serve(config: Config) -> anyhow::Result<()> {
    // 1. Initialize tracing
    // 2. Open SQLite database
    // 3. Run migrations
    // 4. Initialize embedding engine
    // 5. Create service layers (lessons, checkpoints, indexing)
    // 6. Create AppState (Arc-wrapped shared state)
    // 7. Spawn HTTP server task
    // 8. Spawn file watcher task
    // 9. Run MCP server on stdio (blocking)
    // 10. Await shutdown signal
}
```

- [ ] Define `AppState` struct in `src/lib.rs` or `src/main.rs`
- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat: serve command combining MCP stdio + HTTP + file watcher"
```

**Success Criteria**:
- [ ] `amp-rs serve` starts all subsystems
- [ ] Graceful shutdown works

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 8.1 Complete — Squash Merge
- [ ] All subtasks 8.1.1–8.1.3 complete
- [ ] `cargo check` passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/8-1-http-cli
git commit -m "feat: complete task 8.1 - HTTP server, serve command"
git branch -d feature/8-1-http-cli
git push origin main
```

---

## Task 8.2: CLI and Config

**Git**: `git checkout -b feature/8-2-cli-config`

### Subtask 8.2.1: clap CLI with Config File Support (Single Session)

**Prerequisites**:
- [x] 8.1.3: Serve Command

**Deliverables**:
- [ ] Enhance `src/config.rs` with config file loading:

```rust
impl Config {
    /// Load config from file, with CLI overrides
    pub fn load(cli: &Cli) -> anyhow::Result<Self> {
        let config_path = match &cli.command {
            Command::Serve { data_dir, .. } => data_dir.join("config.toml"),
            Command::Index { .. } => {
                dirs_next::home_dir()
                    .unwrap_or_default()
                    .join(".amp-rs")
                    .join("config.toml")
            }
        };

        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        // Apply CLI overrides
        if let Command::Serve { data_dir, watch_dirs, embedding_threads, port, .. } = &cli.command {
            config.data_dir = data_dir.clone();
            if !watch_dirs.is_empty() {
                config.watch_dirs = watch_dirs.clone();
            }
            config.embedding_threads = *embedding_threads;
            config.port = *port;
        }

        Ok(config)
    }
}
```

- [ ] Add `--version` and `--help` via clap derive
- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(config): config file loading with CLI overrides"
```

**Success Criteria**:
- [ ] Config loads from `config.toml`
- [ ] CLI flags override config file values
- [ ] Defaults work without config file

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 8.2.2: CLI and Config Integration Tests (Single Session)

**Prerequisites**:
- [x] 8.2.1: clap CLI

**Deliverables**:
- [ ] Test config loading (from file, from CLI, defaults)
- [ ] Test CLI argument parsing
- [ ] Test `amp-rs --help` and `amp-rs --version` output
- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test: CLI and config integration tests"
```

**Success Criteria**:
- [ ] Config tests pass
- [ ] CLI tests pass
- [ ] Full verification passes

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 8.2 Complete — Squash Merge
- [ ] All subtasks 8.2.1–8.2.2 complete
- [ ] Full verification passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/8-2-cli-config
git commit -m "feat: complete task 8.2 - CLI and config"
git branch -d feature/8-2-cli-config
git push origin main
```

---

*Phase 8 complete when both tasks merged to main.*
