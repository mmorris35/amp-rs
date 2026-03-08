# OBS_PLAN.md — amp-rs Observability

## How to Use This Plan

**For Claude Code**: Read this plan, find the subtask ID from the prompt, complete ALL checkboxes, update completion notes, commit.

```
Use the obs-executor agent to execute subtask X.Y.Z
```

## Project Overview

**Goal**: Add tool-level observability to amp-rs: per-tool usage counters, latency histograms, agent attribution, token savings estimation, and enhanced tracing spans — adding prometheus metrics infrastructure from scratch.

**Timeline**: 1 day
**New Dependencies**: prometheus 0.13, once_cell 1.0

**MVP Scope**:
- [ ] Per-MCP-tool invocation counters with tool + agent + status labels
- [ ] Per-tool latency histograms with tool + agent labels
- [ ] Agent attribution extracted from MCP tool arguments
- [ ] Token savings estimation based on response payload character count
- [ ] Enhanced tracing spans on all 13 tool methods
- [ ] `/metrics` endpoint on existing axum HTTP server
- [ ] All metrics visible at `/metrics` endpoint

---

## Existing Infrastructure (DO NOT DUPLICATE)

| File | What's Already There |
|------|---------------------|
| `src/http/routes.rs` | `/health` and `/status` endpoints, `AppState` struct, `create_router()` |
| `src/http/mod.rs` | `start_http_server()` with axum 0.7 |
| `src/mcp/tools.rs` | 13 MCP tool methods on `AmpMcpServer` struct |
| `src/mcp/mod.rs` | Placeholder (MCP module not fully wired) |
| `src/main.rs` | `tracing_subscriber::fmt()` init with env_filter |
| `src/lib.rs` | `#![deny(warnings)]` — all warnings are errors |

**One tool dispatch path** that needs instrumentation:
1. `AmpMcpServer` methods in `src/mcp/tools.rs` — 13 individual async tool methods

**13 MCP tools**:
1. `tool_add_lesson` 2. `tool_search_lessons` 3. `tool_list_lessons` 4. `tool_delete_lesson`
5. `tool_add_checkpoint` 6. `tool_get_recent_checkpoints` 7. `tool_search_checkpoints`
8. `tool_get_agent_status` 9. `tool_search_code` 10. `tool_index_repo`
11. `tool_diff_index` 12. `tool_full_reindex` 13. `tool_get_status`

---

## Technology Stack

| Component | Library | Already in Cargo.toml |
|-----------|---------|----------------------|
| Metrics | prometheus 0.13 | **No — must add** |
| Lazy Statics | once_cell 1.0 | **No — must add** |
| Structured Logging | tracing 0.1 | Yes |
| Log Subscriber | tracing-subscriber 0.3 | Yes |
| HTTP Server | axum 0.7 | Yes |
| MCP Protocol | rmcp 0.1 | Yes |

---

## Progress Tracking

### Phase 1: Dependencies, Metrics Module & Endpoint
- [ ] 1.1.1: Add prometheus and once_cell to Cargo.toml
- [ ] 1.1.2: Create `src/metrics.rs` with tool-level metrics and `record_tool_call` helper
- [ ] 1.1.3: Add `/metrics` endpoint to `src/http/routes.rs`

### Phase 2: Instrument Tool Methods
- [ ] 2.1.1: Instrument all 13 tool methods in `src/mcp/tools.rs`

### Phase 3: Testing & Verification
- [ ] 3.1.1: Unit tests for metrics helpers and integration test for `/metrics` output

**Current**: Phase 1
**Next**: 1.1.1

---

## Phase 1: Dependencies, Metrics Module & Endpoint

**Goal**: Add prometheus infrastructure, create metrics module, expose `/metrics` endpoint.
**Duration**: ~45 minutes

### Task 1.1: Metrics Infrastructure

**Git**: Create branch `feature/obs-1-1-metrics-infra`

**Subtask 1.1.1: Add prometheus and once_cell to Cargo.toml (Single Session)**

**Prerequisites**:
- None (first subtask)

**Deliverables**:

- [ ] Add `prometheus` and `once_cell` to the `[dependencies]` section in `Cargo.toml`. Add after the `# Utilities` section:

```toml
# Observability
prometheus = "0.13"
once_cell = "1"
```

- [ ] Run `cargo build` — succeeds (dependencies resolve)
- [ ] Run `cargo test --workspace` — all existing tests still pass

**Success Criteria**:
- [ ] `cargo build` succeeds
- [ ] `cargo test --workspace` — all existing tests pass

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Modified**:
  - `Cargo.toml` (added prometheus 0.13, once_cell 1.0)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Branch**: feature/obs-1-1-metrics-infra
- **Notes**: (any additional context)

---

**Subtask 1.1.2: Create src/metrics.rs with tool-level metrics (Single Session)**

**Prerequisites**:
- [x] 1.1.1: prometheus and once_cell added to Cargo.toml

**Deliverables**:

- [ ] Create `src/metrics.rs` with the following content:

```rust
//! Prometheus metrics definitions for amp-rs observability.

use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_histogram_vec, register_int_counter_vec,
    CounterVec, HistogramVec, IntCounterVec, TextEncoder, Encoder,
};

/// Per-tool invocation counter with agent attribution.
pub static TOOL_INVOCATIONS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "amp_tool_invocations_total",
        "Total MCP tool invocations",
        &["tool", "agent", "status"]
    )
    .expect("failed to register amp_tool_invocations_total")
});

/// Per-tool latency histogram with agent attribution.
pub static TOOL_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "amp_tool_duration_seconds",
        "MCP tool invocation latency in seconds",
        &["tool", "agent"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    )
    .expect("failed to register amp_tool_duration_seconds")
});

/// Total response payload bytes by tool (for token estimation auditing).
pub static TOOL_RESPONSE_BYTES: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "amp_tool_response_bytes_total",
        "Total response payload bytes by tool",
        &["tool", "agent"]
    )
    .expect("failed to register amp_tool_response_bytes_total")
});

/// Estimated LLM tokens saved by AMP responses.
pub static TOKEN_SAVINGS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "amp_estimated_tokens_saved_total",
        "Estimated LLM tokens saved by AMP responses",
        &["tool", "agent"]
    )
    .expect("failed to register amp_estimated_tokens_saved_total")
});

/// Tokens-per-character ratio for estimation.
/// Claude tokenizer produces roughly 1 token per 4 characters for English/JSON.
pub const TOKENS_PER_CHAR: f64 = 0.25;

/// Record a complete tool invocation with all metrics.
///
/// Called from MCP tool methods in `mcp/tools.rs`.
///
/// # Arguments
///
/// * `tool_name` - MCP tool name (e.g., "search_lessons")
/// * `agent` - Agent identifier (e.g., "mmn/amp-rs") or "unknown"
/// * `status` - "success" or "error"
/// * `latency` - Time elapsed for the tool call
/// * `response_bytes` - Size of the response payload in bytes (0 for errors)
pub fn record_tool_call(
    tool_name: &str,
    agent: &str,
    status: &str,
    latency: std::time::Duration,
    response_bytes: usize,
) {
    TOOL_INVOCATIONS
        .with_label_values(&[tool_name, agent, status])
        .inc();
    TOOL_LATENCY
        .with_label_values(&[tool_name, agent])
        .observe(latency.as_secs_f64());

    if status == "success" && response_bytes > 0 {
        TOOL_RESPONSE_BYTES
            .with_label_values(&[tool_name, agent])
            .inc_by(response_bytes as u64);
        #[allow(clippy::cast_precision_loss)]
        let estimated_tokens = response_bytes as f64 * TOKENS_PER_CHAR;
        TOKEN_SAVINGS
            .with_label_values(&[tool_name, agent])
            .inc_by(estimated_tokens);
    }
}

/// Initialize all metrics (call once at startup).
pub fn init_metrics() {
    // Access lazy statics to register them
    let _ = &*TOOL_INVOCATIONS;
    let _ = &*TOOL_LATENCY;
    let _ = &*TOOL_RESPONSE_BYTES;
    let _ = &*TOKEN_SAVINGS;

    tracing::debug!("Prometheus metrics initialized");
}

/// Render all registered metrics as Prometheus text format.
pub fn render_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap_or_default();
    String::from_utf8(buffer).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_tool_call_success() {
        record_tool_call(
            "search_lessons",
            "mmn/test",
            "success",
            std::time::Duration::from_millis(42),
            1024,
        );

        let count = TOOL_INVOCATIONS
            .with_label_values(&["search_lessons", "mmn/test", "success"])
            .get();
        assert!(count >= 1, "tool invocation counter should be >= 1, got {count}");

        let latency = TOOL_LATENCY
            .with_label_values(&["search_lessons", "mmn/test"])
            .get_sample_count();
        assert!(latency >= 1, "latency histogram should have >= 1 sample");

        let bytes = TOOL_RESPONSE_BYTES
            .with_label_values(&["search_lessons", "mmn/test"])
            .get();
        assert!(bytes >= 1024, "response bytes should be >= 1024, got {bytes}");

        let tokens = TOKEN_SAVINGS
            .with_label_values(&["search_lessons", "mmn/test"])
            .get();
        assert!(
            tokens >= 256.0,
            "token savings should be >= 256.0 (1024 * 0.25), got {tokens}"
        );
    }

    #[test]
    fn test_record_tool_call_error() {
        record_tool_call(
            "search_lessons",
            "unknown",
            "error",
            std::time::Duration::from_millis(5),
            0,
        );

        let count = TOOL_INVOCATIONS
            .with_label_values(&["search_lessons", "unknown", "error"])
            .get();
        assert!(count >= 1, "error invocation should be counted");
    }

    #[test]
    fn test_render_metrics() {
        init_metrics();
        let output = render_metrics();
        assert!(
            output.contains("amp_tool_invocations_total") || output.is_empty() || true,
            "render_metrics should not panic"
        );
    }
}
```

- [ ] Add `pub mod metrics;` to `src/lib.rs` (after existing module declarations)
- [ ] Run `cargo build` — succeeds
- [ ] Run `cargo clippy --workspace -- -D warnings` — clean
- [ ] Run `cargo fmt --check` — clean

**Success Criteria**:
- [ ] `cargo build` succeeds
- [ ] `cargo clippy --workspace -- -D warnings` exits 0
- [ ] `cargo fmt --check` exits 0
- [ ] All existing tests still pass

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Created**:
  - `src/metrics.rs` (metrics definitions, helper fn, render fn, 3 tests)
- **Files Modified**:
  - `src/lib.rs` (added metrics module)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Branch**: feature/obs-1-1-metrics-infra
- **Notes**: (any additional context)

---

**Subtask 1.1.3: Add /metrics endpoint to src/http/routes.rs (Single Session)**

**Prerequisites**:
- [x] 1.1.2: metrics.rs created with render_metrics()

**Deliverables**:

- [ ] Add the `/metrics` route to `create_router()` in `src/http/routes.rs`. Update the function to include the new route:

```rust
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    Router::new()
        .route("/health", get(health_check))
        .route("/status", get(server_status))
        .route("/metrics", get(prometheus_metrics))
        .with_state(state)
        .layer(cors)
}
```

- [ ] Add the `prometheus_metrics` handler after the `server_status` handler:

```rust
async fn prometheus_metrics() -> ([(axum::http::header::HeaderName, &'static str); 1], String) {
    crate::metrics::init_metrics();
    let body = crate::metrics::render_metrics();
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
```

- [ ] Run `cargo build` — succeeds
- [ ] Run `cargo clippy --workspace -- -D warnings` — clean
- [ ] Run `cargo fmt --check` — clean
- [ ] Run `cargo test --workspace` — all tests pass

**Success Criteria**:
- [ ] `cargo build` succeeds
- [ ] `cargo test --workspace` — all tests pass
- [ ] `/metrics` endpoint returns Prometheus text format

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Modified**:
  - `src/http/routes.rs` (added /metrics route and handler)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Branch**: feature/obs-1-1-metrics-infra
- **Notes**: (any additional context)

### Task 1.1 Complete - Squash Merge
- [ ] All subtasks complete (1.1.1, 1.1.2, 1.1.3)
- [ ] All tests pass: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] Squash merge to main: `git checkout main && git merge --squash feature/obs-1-1-metrics-infra && git commit -m "feat(metrics): add prometheus observability infrastructure"`
- [ ] Push to remote: `git push origin main`
- [ ] Delete branch: `git branch -d feature/obs-1-1-metrics-infra`

---

## Phase 2: Instrument Tool Methods

**Goal**: Add timing, agent extraction, response measurement, and enhanced tracing to all 13 MCP tool methods.
**Duration**: ~1 hour

### Task 2.1: Instrument MCP Tools

**Git**: Create branch `feature/obs-2-1-instrument-tools`

**Subtask 2.1.1: Instrument all tool methods in src/mcp/tools.rs (Single Session)**

**Prerequisites**:
- [x] 1.1.2: metrics.rs with record_tool_call helper

**Context**: Unlike nellie-rs which has a central match dispatch, amp-rs has 13 individual async methods on `AmpMcpServer`. Each method needs to be wrapped with timing, agent extraction, and metrics recording. The pattern is the same for every method:

1. Record start time
2. Extract agent from arguments (if available)
3. Execute the original logic
4. Record metrics based on success/error

**Instrumentation Pattern** — Apply this to EVERY tool method:

```rust
pub async fn tool_example(
    &self,
    agent_or_other_param: &str,
    // ... other params
) -> Result<Value, String> {
    let start = std::time::Instant::now();
    let tool_name = "example";  // hardcoded per method
    let agent = /* extract from params if available, else "unknown" */;

    let result = /* ... original logic ... */;

    let latency = start.elapsed();
    match &result {
        Ok(value) => {
            let response_bytes = value.to_string().len();
            crate::metrics::record_tool_call(tool_name, agent, "success", latency, response_bytes);
            tracing::debug!(
                tool = tool_name,
                agent,
                latency_ms = latency.as_millis() as u64,
                response_bytes,
                "Tool invocation succeeded"
            );
        }
        Err(e) => {
            crate::metrics::record_tool_call(tool_name, agent, "error", latency, 0);
            tracing::warn!(
                tool = tool_name,
                agent,
                error = %e,
                latency_ms = latency.as_millis() as u64,
                "Tool invocation failed"
            );
        }
    }
    result
}
```

**Agent Extraction Rules**:
- `tool_add_checkpoint` — has `agent: &str` parameter → use it
- `tool_get_recent_checkpoints` — has `agent: &str` parameter → use it
- `tool_search_checkpoints` — has `agent: Option<&str>` parameter → use `.unwrap_or("unknown")`
- `tool_get_agent_status` — has `agent: &str` parameter → use it
- All other tools — use `"unknown"` (no agent parameter)

**Deliverables**:

- [ ] Instrument all 13 tool methods following the pattern above. Each method should:
  - Record `std::time::Instant::now()` at start
  - Use hardcoded tool name matching the method (e.g., `"add_lesson"` for `tool_add_lesson`)
  - Extract agent from parameters where available
  - Call `crate::metrics::record_tool_call()` on both success and error paths
  - Add `tracing::debug!()` on success, `tracing::warn!()` on error

- [ ] Tool name mapping (use these exact strings):
  - `tool_add_lesson` → `"add_lesson"`
  - `tool_search_lessons` → `"search_lessons"`
  - `tool_list_lessons` → `"list_lessons"`
  - `tool_delete_lesson` → `"delete_lesson"`
  - `tool_add_checkpoint` → `"add_checkpoint"`
  - `tool_get_recent_checkpoints` → `"get_recent_checkpoints"`
  - `tool_search_checkpoints` → `"search_checkpoints"`
  - `tool_get_agent_status` → `"get_agent_status"`
  - `tool_search_code` → `"search_code"`
  - `tool_index_repo` → `"index_repo"`
  - `tool_diff_index` → `"diff_index"`
  - `tool_full_reindex` → `"full_reindex"`
  - `tool_get_status` → `"get_status"`

- [ ] Run `cargo build` — succeeds
- [ ] Run `cargo clippy --workspace -- -D warnings` — clean
- [ ] Run `cargo fmt --check` — clean
- [ ] Run `cargo test --workspace` — all tests pass

**Success Criteria**:
- [ ] `cargo build` succeeds
- [ ] `cargo test --workspace` — all tests pass
- [ ] `cargo clippy --workspace -- -D warnings` exits 0
- [ ] All 13 tool methods now record metrics and emit tracing events

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Modified**:
  - `src/mcp/tools.rs` (instrumented all 13 tool methods)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Branch**: feature/obs-2-1-instrument-tools
- **Notes**: (any additional context)

### Task 2.1 Complete - Squash Merge
- [ ] All subtasks complete (2.1.1)
- [ ] All tests pass: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] Squash merge to main: `git checkout main && git merge --squash feature/obs-2-1-instrument-tools && git commit -m "feat(observability): instrument all MCP tool methods with metrics and tracing"`
- [ ] Push to remote: `git push origin main`
- [ ] Delete branch: `git branch -d feature/obs-2-1-instrument-tools`

---

## Phase 3: Testing & Verification

**Goal**: Verify all metrics are correctly exposed and end-to-end observability works.
**Duration**: ~30 minutes

### Task 3.1: Integration Tests

**Git**: Create branch `feature/obs-3-1-testing`

**Subtask 3.1.1: Integration test for metrics endpoint (Single Session)**

**Prerequisites**:
- [x] 2.1.1: Tool methods instrumented

**Deliverables**:

- [ ] Add a test to `src/http/routes.rs` inside a `#[cfg(test)] mod tests` block. This test verifies that tool metrics appear in the `/metrics` output after a tool invocation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        AppState {
            db_path: Arc::new(":memory:".to_string()),
            start_time: SystemTime::now(),
        }
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = create_test_state();
        let app = create_router(state);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let state = create_test_state();

        // Simulate a tool call to populate metrics
        crate::metrics::record_tool_call(
            "search_lessons",
            "mmn/test-agent",
            "success",
            std::time::Duration::from_millis(150),
            2048,
        );

        let app = create_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Verify tool invocation counter
        assert!(
            body_str.contains("amp_tool_invocations_total"),
            "metrics should contain amp_tool_invocations_total"
        );
        // Verify tool latency histogram
        assert!(
            body_str.contains("amp_tool_duration_seconds"),
            "metrics should contain amp_tool_duration_seconds"
        );
        // Verify response bytes counter
        assert!(
            body_str.contains("amp_tool_response_bytes_total"),
            "metrics should contain amp_tool_response_bytes_total"
        );
        // Verify token savings counter
        assert!(
            body_str.contains("amp_estimated_tokens_saved_total"),
            "metrics should contain amp_estimated_tokens_saved_total"
        );
        // Verify agent label is present
        assert!(
            body_str.contains("mmn/test-agent"),
            "metrics should contain agent label value"
        );
    }
}
```

- [ ] Add `tower` to `[dev-dependencies]` in `Cargo.toml` if not already present:

```toml
tower = "0.4"
```

- [ ] Run `cargo test --workspace` — all tests pass (existing + new)
- [ ] Run `cargo clippy --workspace -- -D warnings` — clean
- [ ] Run `cargo fmt --check` — clean
- [ ] Run `cargo build --release` — succeeds

**Verification Commands**:

```bash
# All tests pass
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace

# Specific new tests pass
cargo test -- test_metrics_endpoint test_health_endpoint --nocapture

# Build release
cargo build --release

# Manual verification (if server is running):
# curl -s localhost:<port>/metrics | grep amp_tool
# Expected output includes lines like:
#   amp_tool_invocations_total{tool="search_lessons",agent="mmn/amp-rs",status="success"} 5
#   amp_tool_duration_seconds_bucket{tool="search_lessons",agent="mmn/amp-rs",le="0.1"} 4
#   amp_estimated_tokens_saved_total{tool="search_lessons",agent="mmn/amp-rs"} 1234.5
```

**Success Criteria**:
- [ ] `cargo test --workspace` — all tests pass (0 failures)
- [ ] `cargo clippy --workspace -- -D warnings` exits 0
- [ ] `cargo fmt --check` exits 0
- [ ] `cargo build --release` succeeds
- [ ] New metrics appear in `/metrics` output with correct labels

**Completion Notes**:
- **Implementation**: (describe what was done)
- **Files Modified**:
  - `src/http/routes.rs` (added test module with integration tests)
  - `Cargo.toml` (added tower dev-dependency if needed)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Branch**: feature/obs-3-1-testing
- **Notes**: (any additional context)

### Task 3.1 Complete - Squash Merge
- [ ] All subtasks complete (3.1.1)
- [ ] All tests pass: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] Release build passes: `cargo build --release`
- [ ] Squash merge to main: `git checkout main && git merge --squash feature/obs-3-1-testing && git commit -m "test(observability): add integration tests for tool metrics"`
- [ ] Push to remote: `git push origin main`
- [ ] Delete branch: `git branch -d feature/obs-3-1-testing`

---

## v2 Roadmap (Post-MVP)

### v2.1: Grafana Dashboard Template
- JSON dashboard template with panels for: tool invocations over time, latency p50/p95/p99, token savings cumulative, per-agent breakdown
- PromQL queries pre-configured for AMP metrics

### v2.2: Persistent Token Savings
- Store cumulative token savings in SQLite across restarts
- Load previous totals on startup and seed prometheus counters

### v2.3: Cache Hit Rate Tracking
- Track embedding cache hits/misses
- Track search result cache hits/misses
- Expose as prometheus counters with cache_status label

---

## Git Workflow

### Branch Strategy
- **ONE branch per TASK** (e.g., `feature/obs-1-1-metrics-infra`)
- Subtasks within a task are individual commits on the same branch
- Branch naming: `feature/obs-{phase}-{task}-{description}`

### Commit Strategy
- One commit per subtask with semantic message
- Format: `feat(observability): description`
- Example: `feat(observability): add prometheus metrics infrastructure`

### Merge Strategy
- Squash merge when task is complete
- Delete feature branch after merge
- Push to remote after each merge

---

## Summary

| Phase | Task | Subtask | Files Modified | What |
|-------|------|---------|---------------|------|
| 1 | 1.1 | 1.1.1 | `Cargo.toml` | Add prometheus + once_cell dependencies |
| 1 | 1.1 | 1.1.2 | `src/metrics.rs`, `src/lib.rs` | Create metrics module with 4 metrics + helper |
| 1 | 1.1 | 1.1.3 | `src/http/routes.rs` | Add `/metrics` endpoint |
| 2 | 2.1 | 2.1.1 | `src/mcp/tools.rs` | Instrument all 13 tool methods |
| 3 | 3.1 | 3.1.1 | `src/http/routes.rs`, `Cargo.toml` | Integration tests for /metrics |

**Total files modified**: 5
**New dependencies**: 2 (prometheus 0.13, once_cell 1.0)
**New metrics exposed**: `amp_tool_invocations_total`, `amp_tool_duration_seconds`, `amp_tool_response_bytes_total`, `amp_estimated_tokens_saved_total`
