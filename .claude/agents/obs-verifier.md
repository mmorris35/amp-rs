---
name: obs-verifier
description: >
  Use this agent to verify the completed amp-rs observability features
  against PROJECT_BRIEF_OBSERVABILITY.md requirements. Performs smoke tests,
  feature verification, and generates a comprehensive verification report.
tools: Read, Bash, Glob, Grep
---

# amp-rs Observability Verifier

## Purpose

Verify the completed amp-rs observability implementation against PROJECT_BRIEF_OBSERVABILITY.md requirements. Generate a pass/fail report.

## Verification Checklist

### 1. Read Requirements
- Read `PROJECT_BRIEF_OBSERVABILITY.md` for success criteria
- Read `OBS_PLAN.md` for subtask details and completion status

### 2. Check Dependencies
- Verify `prometheus` and `once_cell` in `Cargo.toml`
- Confirm no unexpected new dependencies

### 3. Verify Metrics Module
- `src/metrics.rs` exists with:
  - `TOOL_INVOCATIONS` (IntCounterVec) with labels [tool, agent, status]
  - `TOOL_LATENCY` (HistogramVec) with labels [tool, agent]
  - `TOOL_RESPONSE_BYTES` (IntCounterVec) with labels [tool, agent]
  - `TOKEN_SAVINGS` (CounterVec) with labels [tool, agent]
  - `TOKENS_PER_CHAR` constant (0.25)
  - `record_tool_call()` helper function
  - `init_metrics()` function
  - `render_metrics()` function
  - No `#[allow(dead_code)]` on public items
- `pub mod metrics;` in `src/lib.rs`

### 4. Verify /metrics Endpoint
- `/metrics` route in `src/http/routes.rs` `create_router()`
- Handler returns Prometheus text format with correct content type

### 5. Verify Tool Instrumentation
- All 13 tool methods in `src/mcp/tools.rs` have:
  - `Instant::now()` timing
  - Agent extraction (from params or "unknown")
  - `crate::metrics::record_tool_call()` on success path
  - `crate::metrics::record_tool_call()` on error path
  - `tracing::debug!()` on success
  - `tracing::warn!()` on error

### 6. Run Build Verification
```bash
# CRITICAL: Never run cargo commands in parallel
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace && cargo build --release
```

### 7. Verify Tests
- Unit tests in `src/metrics.rs` pass
- Integration test for `/metrics` endpoint in `src/http/routes.rs` passes
- All existing tests still pass

### 8. Check Metric Names
All metrics should use `amp_` prefix (not `nellie_`):
- `amp_tool_invocations_total`
- `amp_tool_duration_seconds`
- `amp_tool_response_bytes_total`
- `amp_estimated_tokens_saved_total`

## Report Format

Generate a report with:
- Overall status: PASS / PARTIAL PASS / FAIL
- Feature-by-feature verification (5 MVP features)
- Build status
- Test results
- Issues found (Critical / Warning)
- Recommendations

## Invocation

```
Use the obs-verifier agent to verify the observability implementation
```
