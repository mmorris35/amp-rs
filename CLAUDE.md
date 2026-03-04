# CLAUDE.md — Project Rules for amp-rs

> Read at the start of every session. These rules override defaults.

## 1. Project Context

**amp-rs** is the reference implementation of Agent Memory Protocol (AMP) — a local-first MCP server with checkpoints, lessons, semantic search, and code indexing. Built in Rust. Single binary. No cloud.

## 2. Project Structure

```
amp-rs/
├── src/
│   ├── main.rs              # Entry point, CLI
│   ├── lib.rs               # Library root, re-exports
│   ├── config.rs            # Configuration (CLI + TOML)
│   ├── error.rs             # Error types (thiserror)
│   ├── storage/
│   │   ├── mod.rs           # Storage trait
│   │   ├── sqlite.rs        # SQLite implementation
│   │   └── schema.rs        # Schema + migrations
│   ├── embedding/
│   │   ├── mod.rs           # Embedding trait
│   │   ├── onnx.rs          # ONNX Runtime session
│   │   └── pool.rs          # Thread pool for embeddings
│   ├── lessons/
│   │   ├── mod.rs           # Lesson model
│   │   ├── service.rs       # Lesson business logic
│   │   └── storage.rs       # Lesson DB operations
│   ├── checkpoints/
│   │   ├── mod.rs           # Checkpoint model
│   │   ├── service.rs       # Checkpoint business logic
│   │   └── storage.rs       # Checkpoint DB operations
│   ├── indexing/
│   │   ├── mod.rs           # Indexing coordinator
│   │   ├── scanner.rs       # File scanner (.gitignore)
│   │   ├── chunker.rs       # Language-aware chunking
│   │   └── storage.rs       # Chunk DB operations
│   ├── watcher/
│   │   ├── mod.rs           # File watcher (notify)
│   │   └── handler.rs       # Change event handler
│   ├── mcp/
│   │   ├── mod.rs           # MCP server (rmcp)
│   │   └── tools.rs         # Tool implementations
│   └── http/
│       ├── mod.rs           # Axum HTTP server
│       └── routes.rs        # Health + status routes
├── tests/
│   ├── common/
│   │   └── mod.rs           # Test helpers, fixtures
│   ├── storage_test.rs
│   ├── embedding_test.rs
│   ├── lessons_test.rs
│   ├── checkpoints_test.rs
│   ├── indexing_test.rs
│   ├── mcp_test.rs
│   └── integration_test.rs
├── models/                   # ONNX model files (git-lfs or download)
├── plans/                    # Phase plan files
├── .claude/agents/           # Executor + verifier agents
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── CLAUDE.md
├── DEVELOPMENT_PLAN.md
└── PROJECT_BRIEF.md
```

## 3. Build Commands

```bash
# CRITICAL: Never run cargo commands in parallel. Always chain with &&.
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace

# Build
cargo build
cargo build --release

# Test
cargo test --workspace
cargo test --workspace -- --nocapture  # with output
cargo test test_name                   # specific test

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --check      # check only
cargo fmt              # apply
```

**NEVER run cargo commands in parallel Bash calls. Always `&&` chain.**

## 4. First Build Warning

The first `cargo build` will compile ONNX Runtime and sqlite-vec native dependencies. This takes several minutes and is CPU-intensive. **Do not start additional cargo processes or CPU-intensive work during first build.** This is a one-time cost — subsequent builds are fast.

## 5. Session Protocol

### Starting a Session
1. Read `DEVELOPMENT_PLAN.md` — find your subtask
2. Read the relevant phase plan in `plans/PHASE_X_*.md`
3. Verify prerequisite subtasks are `[x]` complete
4. Read prerequisite completion notes for context
5. Check git state — correct branch for the TASK

### Ending a Session
1. All subtask deliverable checkboxes `[x]` checked
2. All tests pass: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace`
3. Completion notes written in phase plan
4. Git commit with semantic message
5. If task complete: squash merge → push → delete branch

## 6. Git Discipline

### Branch per TASK (not subtask)
```bash
# Starting a task — create branch
git checkout -b feature/{phase}-{task}-{description}

# After each subtask — commit
git add -A && git commit -m "feat(scope): subtask X.Y.Z - description"

# Task complete — squash merge and push
git checkout main && git merge --squash feature/{branch}
git commit -m "feat(scope): complete task X.Y - description"
git branch -d feature/{branch}
git push origin main
```

### Worktrees for Parallel Phases
```bash
git worktree add ../amp-rs-phase-X -b feature/X-1-description
# Work in worktree, merge back, remove worktree
git worktree remove ../amp-rs-phase-X
```

### Commit Convention
- `feat(scope):` — new feature
- `fix(scope):` — bug fix
- `refactor(scope):` — code restructuring
- `test(scope):` — test additions
- `docs(scope):` — documentation
- `chore(scope):` — tooling, CI, deps

## 7. Code Standards

### Rust Style
- `#![deny(warnings)]` in lib.rs
- Doc comments (`///`) on all public items
- `Result<T, Error>` for all fallible operations — use `anyhow::Result` in main, `thiserror` for library errors
- Prefer `&str` parameters, return owned `String`
- Use `#[derive(Debug, Clone, Serialize, Deserialize)]` on models
- Module-level error enums with `thiserror`

### Naming
- Modules: `snake_case`
- Types/Traits: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`

### Testing
- Unit tests in `#[cfg(test)] mod tests` within source files
- Integration tests in `tests/` directory
- Test helpers in `tests/common/mod.rs`
- Use `tempfile` crate for test databases
- Target: 80% coverage minimum

## 8. Error Handling

If blocked during a subtask:
1. Do NOT commit broken code
2. Do NOT mark subtask `[x]`
3. Write in completion notes:
   ```
   **Status**: ❌ BLOCKED
   **Error**: [message]
   **Attempted**: [what was tried]
   **Root Cause**: [analysis]
   ```
4. Commit WIP to feature branch if partial progress exists
5. Report immediately

## 9. Architecture Rules

- **Storage trait abstraction**: All database access goes through traits in `storage/mod.rs`. No raw SQL outside `storage/` modules.
- **Embedding trait abstraction**: Embedding generation goes through trait in `embedding/mod.rs`. Implementation details (ONNX) are private.
- **Service layer pattern**: Business logic in `service.rs` files. Storage operations in `storage.rs` files. Models in `mod.rs`.
- **MCP tools are thin wrappers**: MCP tool functions call service layer. No business logic in MCP tool handlers.
- **HTTP routes are thin wrappers**: Same pattern — routes call services.
- **No blocking on async runtime**: Embedding inference runs on `spawn_blocking` or dedicated thread pool. File I/O uses tokio's async file ops.

## 10. Dependency Notes

| Crate | Notes |
|-------|-------|
| `rmcp` | Use `#[tool]` macro for MCP tool registration. stdio transport via `rmcp::transport::stdio`. |
| `rusqlite` | Use `Connection::open_with_flags` with WAL mode. Load sqlite-vec at connection time. |
| `ort` | Session creation is expensive — create once, reuse. Use `SessionBuilder::new()` with model path. |
| `notify` | Use `RecommendedWatcher` with `RecursiveMode::Recursive`. Debounce events. |
| `axum` | Share state via `Extension` or `State`. Run on separate tokio task from MCP. |
| `clap` | Use derive API: `#[derive(Parser)]`. Subcommands for `serve`, `index`, etc. |

---

**Version**: 1.0
**Project**: amp-rs
