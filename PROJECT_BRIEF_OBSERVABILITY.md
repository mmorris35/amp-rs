# PROJECT_BRIEF.md

## Basic Information

- **Project Name**: amp-rs-observability
- **Project Type**: library
- **Primary Goal**: Add tool-level observability to amp-rs MCP server: per-tool usage counters, latency histograms, agent attribution, and token savings estimation — adding prometheus metrics infrastructure from scratch
- **Target Users**: Mike (project owner) monitoring AMP usage and value across agents
- **Timeline**: 1 day
- **Team Size**: 1

## Functional Requirements

### Key Features (MVP)

- Per-MCP-tool invocation counters (search_lessons, add_lesson, add_checkpoint, search_code, etc.) using prometheus crate — new infrastructure for amp-rs
- Per-tool latency histograms with p50/p95/p99 percentiles via prometheus HistogramVec with tool name labels
- Agent attribution — extract agent name from MCP tool arguments and tag tool metrics with an agent label, enabling per-agent usage breakdowns
- Token savings estimation — measure character count of each tool response payload, apply a tokens-per-char heuristic, and expose cumulative estimated tokens saved as a prometheus counter
- Structured tracing spans on all 13 MCP tool handlers with tool name, agent, and query fields
- `/metrics` endpoint on the existing axum HTTP server exposing Prometheus text format

### Nice-to-Have Features (v2)

- Grafana dashboard template for AMP metrics
- Cumulative token savings counter persisted to SQLite across restarts
- Cache hit rate tracking for embedding and search caches

## Technical Constraints

### Must Add

- prometheus 0.13 (NEW dependency — amp-rs has no metrics infrastructure yet)
- once_cell 1.0 (NEW dependency — for lazy static metric registration)

### Already Available

- tracing 0.1 (already a dependency — structured logging and spans)
- axum 0.7 (existing HTTP server — /health and /status endpoints exist)

### Cannot Use

- metrics crate (would conflict with prometheus approach used in nellie-rs)
- metrics-exporter-prometheus (same reason)

## Existing Infrastructure

- `src/http/routes.rs` — `/health` and `/status` endpoints on axum 0.7 (NO /metrics endpoint yet)
- `src/http/mod.rs` — `start_http_server()` function
- `src/mcp/tools.rs` — 13 MCP tool methods on `AmpMcpServer` struct (individual async methods, not a central dispatch match)
- `src/mcp/mod.rs` — MCP module (placeholder — not fully wired up yet)
- `src/main.rs` — tracing init with env_filter, HTTP server spawn, MCP placeholder
- `src/lib.rs` — `#![deny(warnings)]` — stricter than nellie-rs

## Architecture Differences from nellie-rs

| Aspect | nellie-rs | amp-rs |
|--------|-----------|--------|
| Prometheus | Already a dependency | Must add |
| once_cell | Already a dependency | Must add |
| /metrics endpoint | Already exists | Must create |
| Dispatch paths | 3 (HTTP, SSE, rmcp) | 1 (individual tool methods) |
| Tool count | 17 | 13 |
| Tool pattern | Central match dispatch | Individual async methods on struct |
| axum version | 0.8 | 0.7 |
| Warning level | `#![warn(...)]` | `#![deny(warnings)]` |
| MCP server | Fully wired | Placeholder (not fully connected) |

## Other Constraints

- Two new crate dependencies required: `prometheus` and `once_cell`
- No new ports or listeners — /metrics endpoint goes on existing axum server
- Must not break existing HTTP endpoints or test suite
- `#![deny(warnings)]` means ALL clippy warnings must be resolved
- Token estimation uses character count on response JSON payloads with a configurable tokens-per-char heuristic (default ~0.25 tokens per char for JSON)

## Success Criteria

- All MVP features implemented and working
- `curl localhost:<port>/metrics` shows per-tool counters, latency histograms, agent labels, and token savings
- Code passes `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- Existing tests continue to pass
