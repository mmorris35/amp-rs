# Phase 7: MCP Server

**Goal**: rmcp stdio MCP server with all tools registered (lessons, checkpoints, code search, indexing, status)
**Duration**: 3–4 days
**Wave**: 3 (parallel with Phase 6 and Phase 8)
**Dependencies**: Phase 3 (lessons) + Phase 4 (checkpoints) + Phase 5 (code indexing) complete

---

## Task 7.1: MCP Server Bootstrap

**Git**: `git checkout -b feature/7-1-mcp-server`

### Subtask 7.1.1: rmcp Server Bootstrap (stdio transport) (Single Session)

**Prerequisites**:
- [x] 3.2.2: Lesson Tests
- [x] 4.2.2: Checkpoint Tests
- [x] 5.2.2: Code Indexing Tests

**Deliverables**:
- [ ] Create `src/mcp/mod.rs` with MCP server struct:

```rust
pub mod tools;

use crate::checkpoints::service::CheckpointService;
use crate::lessons::service::LessonService;
use crate::error::Result;
use rmcp::ServerHandler;
use std::sync::Arc;

/// AMP MCP Server — implements the MCP protocol over stdio
pub struct AmpMcpServer {
    pub lesson_service: Arc<LessonService>,
    pub checkpoint_service: Arc<CheckpointService>,
    // Add indexing service, etc.
}

impl AmpMcpServer {
    pub fn new(
        lesson_service: Arc<LessonService>,
        checkpoint_service: Arc<CheckpointService>,
    ) -> Self {
        Self {
            lesson_service,
            checkpoint_service,
        }
    }
}

// Implement rmcp::ServerHandler for AmpMcpServer
// This registers all MCP tools and handles incoming requests
```

- [ ] Set up stdio transport:

```rust
/// Start the MCP server on stdio
pub async fn serve_stdio(server: AmpMcpServer) -> Result<()> {
    use rmcp::transport::stdio;

    let transport = stdio::StdioTransport::new();
    // Register server with transport
    // Run event loop

    Ok(())
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(mcp): rmcp server bootstrap with stdio transport"
```

**Success Criteria**:
- [ ] MCP server struct compiles with rmcp
- [ ] stdio transport setup compiles
- [ ] Server holds references to service layers

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: rmcp API specifics, any version compatibility issues

---

### Subtask 7.1.2: Lesson MCP Tools (Single Session)

**Prerequisites**:
- [x] 7.1.1: rmcp Server Bootstrap

**Deliverables**:
- [ ] Create `src/mcp/tools.rs` starting with lesson tools:

```rust
// MCP tool implementations — thin wrappers around service layer

// Tool: add_lesson
// Parameters: title (string, required), content (string, required),
//             tags (array of strings, optional), severity (string, optional, default "info")
// Returns: Lesson JSON

// Tool: search_lessons
// Parameters: query (string, required), limit (integer, optional, default 5)
// Returns: Array of Lesson JSON

// Tool: list_lessons
// Parameters: severity (string, optional), limit (integer, optional, default 50)
// Returns: Array of Lesson JSON

// Tool: delete_lesson
// Parameters: id (string, required)
// Returns: { deleted: boolean }
```

- [ ] Register all four tools with rmcp `#[tool]` macro or manual registration
- [ ] Each tool validates parameters and calls service layer
- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(mcp): lesson MCP tools (add, search, list, delete)"
```

**Success Criteria**:
- [ ] Four lesson tools registered
- [ ] Parameter validation on all tools
- [ ] Tools delegate to LessonService

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: rmcp tool registration pattern used

---

### Subtask 7.1.3: Checkpoint MCP Tools (Single Session)

**Prerequisites**:
- [x] 7.1.2: Lesson MCP Tools

**Deliverables**:
- [ ] Add checkpoint tools to `src/mcp/tools.rs`:

```rust
// Tool: add_checkpoint
// Parameters: agent (string, required), working_on (string, required),
//             state (object, optional, default {})
// Returns: Checkpoint JSON

// Tool: get_recent_checkpoints
// Parameters: agent (string, required), limit (integer, optional, default 5)
// Returns: Array of Checkpoint JSON

// Tool: search_checkpoints
// Parameters: query (string, required), agent (string, optional), limit (integer, optional, default 5)
// Returns: Array of Checkpoint JSON

// Tool: get_agent_status
// Parameters: agent (string, required)
// Returns: AgentStatus JSON
```

- [ ] Register all four tools
- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(mcp): checkpoint MCP tools (add, get_recent, search, get_status)"
```

**Success Criteria**:
- [ ] Four checkpoint tools registered
- [ ] All delegate to CheckpointService

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 7.1 Complete — Squash Merge
- [ ] All subtasks 7.1.1–7.1.3 complete
- [ ] `cargo check` passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/7-1-mcp-server
git commit -m "feat(mcp): complete task 7.1 - MCP server with lesson and checkpoint tools"
git branch -d feature/7-1-mcp-server
git push origin main
```

---

## Task 7.2: Search Tools and MCP Tests

**Git**: `git checkout -b feature/7-2-mcp-search-tools`

### Subtask 7.2.1: Code Search and Indexing MCP Tools (Single Session)

**Prerequisites**:
- [x] 7.1.3: Checkpoint MCP Tools

**Deliverables**:
- [ ] Add remaining MCP tools:

```rust
// Tool: search_code
// Parameters: query (string, required), limit (integer, optional, default 10)
// Returns: Array of code chunk results

// Tool: index_repo
// Parameters: path (string, required)
// Returns: { files_indexed: number, duration_ms: number }

// Tool: diff_index
// Parameters: path (string, required)
// Returns: { new: number, changed: number, deleted: number }

// Tool: full_reindex
// Parameters: path (string, required)
// Returns: { files_indexed: number, duration_ms: number }

// Tool: get_status
// Parameters: none
// Returns: { lessons_count, checkpoints_count, chunks_count, indexed_repos, uptime_seconds }
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(mcp): code search, indexing, and status MCP tools"
```

**Success Criteria**:
- [ ] All 13 MCP tools registered
- [ ] get_status returns server stats

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 7.2.2: MCP Server Integration Tests (Single Session)

**Prerequisites**:
- [x] 7.2.1: Code Search and Indexing MCP Tools

**Deliverables**:
- [ ] Create `tests/mcp_test.rs`:
  - Test tool listing (verify all 13 tools registered)
  - Test tool parameter validation
  - Test tool responses match expected schema
- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test(mcp): MCP server integration tests"
```

**Success Criteria**:
- [ ] All MCP tests pass
- [ ] Full verification passes

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 7.2 Complete — Squash Merge
- [ ] All subtasks 7.2.1–7.2.2 complete
- [ ] Full verification passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/7-2-mcp-search-tools
git commit -m "feat(mcp): complete task 7.2 - all MCP tools and tests"
git branch -d feature/7-2-mcp-search-tools
git push origin main
```

---

*Phase 7 complete when both tasks merged to main.*
