# amp-rs User Manual

> Agent Memory Protocol — local-first memory for AI coding assistants.
> Single binary. No cloud. No API keys.

---

## Table of Contents

1. [Installation](#installation)
2. [First Run](#first-run)
3. [Configuration](#configuration)
4. [Connecting to AI Clients](#connecting-to-ai-clients)
5. [MCP Tools Reference](#mcp-tools-reference)
6. [HTTP API](#http-api)
7. [Working with Lessons](#working-with-lessons)
8. [Working with Checkpoints](#working-with-checkpoints)
9. [Code Indexing](#code-indexing)
10. [Knowledge Graph](#knowledge-graph)
11. [Troubleshooting](#troubleshooting)

---

## Installation

### Requirements

- **Rust 1.70+** (stable toolchain)
- **Git**
- **C compiler** (for ONNX Runtime and SQLite native dependencies)

On macOS, install build tools with:

```bash
xcode-select --install
```

On Debian/Ubuntu:

```bash
sudo apt install build-essential pkg-config
```

### Build from source

```
$ git clone https://github.com/mmorris35/amp-rs.git
$ cd amp-rs
$ cargo build --release

   Compiling amp-rs v0.1.0 (/home/user/amp-rs)
    Finished `release` profile [optimized] target(s) in 2m 47s
```

> **First build warning:** The initial build compiles ONNX Runtime and SQLite
> extensions from source. This takes 2-3 minutes and is CPU-intensive. Do not
> run other cargo processes simultaneously. Subsequent builds complete in <30s.

### Install the binary

```
$ cp target/release/amp-rs ~/.local/bin/
$ amp-rs --version

amp-rs 0.1.0
```

Or add `target/release` to your `PATH`.

### Verify the build

```
$ cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace

running 24 tests
test checkpoints::tests::test_add_checkpoint ... ok
test checkpoints::tests::test_get_recent ... ok
test lessons::tests::test_add_lesson ... ok
test lessons::tests::test_search_lessons ... ok
test storage::tests::test_open_and_migrate ... ok
...
test result: ok. 24 passed; 0 failed; 0 ignored
```

---

## First Run

Start the server with default settings:

```
$ amp-rs serve

2026-03-05T10:00:01Z  INFO amp_rs: Initializing SQLite storage at "/home/user/.amp-rs/amp.db"
2026-03-05T10:00:01Z  INFO amp_rs: Initializing embedding engine
2026-03-05T10:00:02Z  INFO amp_rs: Starting MCP server on stdio transport
2026-03-05T10:00:02Z  INFO amp_rs: amp-rs server started successfully
```

On first run, amp-rs will:

1. Create `~/.amp-rs/` data directory
2. Create `~/.amp-rs/amp.db` SQLite database
3. Download the `all-MiniLM-L6-v2` ONNX model to `~/.amp-rs/models/`
4. Start the MCP server on stdio and HTTP health server on port 8080

Everything lives in `~/.amp-rs/`. Back it up by copying that directory.

---

## Configuration

### CLI flags

```
$ amp-rs serve --help

Start the AMP server (MCP stdio + HTTP)

Usage: amp-rs serve [OPTIONS]

Options:
      --data-dir <DATA_DIR>
          Data directory for database and models
          [default: ~/.amp-rs]

      --watch-dirs <WATCH_DIRS>
          Directories to watch for file changes
          (comma-separated)

      --embedding-threads <EMBEDDING_THREADS>
          Number of embedding threads
          [default: 2]

      --port <PORT>
          HTTP port for health check and REST
          [default: 8080]

      --debug
          Enable debug logging

  -h, --help
          Print help
```

### Example: custom data directory and watched repos

```
$ amp-rs serve \
    --data-dir ~/my-amp-data \
    --watch-dirs ~/projects/backend,~/projects/frontend \
    --embedding-threads 4 \
    --port 9090

2026-03-05T10:00:01Z  INFO amp_rs: Initializing SQLite storage at "/home/user/my-amp-data/amp.db"
2026-03-05T10:00:01Z  INFO amp_rs: Initializing embedding engine
2026-03-05T10:00:02Z  INFO amp_rs: Starting file watcher for 2 directories
2026-03-05T10:00:02Z  INFO amp_rs: Starting MCP server on stdio transport
2026-03-05T10:00:02Z  INFO amp_rs: amp-rs server started successfully
```

### Config file

Instead of CLI flags, you can use `~/.amp-rs/config.toml`:

```toml
data_dir = "~/.amp-rs"
watch_dirs = ["~/projects", "~/work"]
embedding_threads = 2
port = 8080
log_level = "info"
```

CLI flags override config file values.

### Environment variables

Set `RUST_LOG` for fine-grained log control:

```
$ RUST_LOG=debug amp-rs serve
$ RUST_LOG=amp_rs=debug,rmcp=info amp-rs serve
```

---

## Connecting to AI Clients

amp-rs speaks MCP over stdio. Any MCP-compatible client can connect.

### Claude Code

```
$ claude mcp add amp-rs -- amp-rs serve

Added amp-rs MCP server
```

Verify the connection:

```
$ claude mcp list

amp-rs: amp-rs serve (stdio)
```

To pass custom flags:

```
$ claude mcp add amp-rs -- amp-rs serve --watch-dirs ~/projects --debug
```

### Cursor

Add to `.cursor/mcp.json` in your project root (or globally at `~/.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "amp-rs": {
      "command": "amp-rs",
      "args": ["serve"]
    }
  }
}
```

With custom flags:

```json
{
  "mcpServers": {
    "amp-rs": {
      "command": "amp-rs",
      "args": ["serve", "--watch-dirs", "/home/user/projects", "--debug"]
    }
  }
}
```

### Windsurf / Cline / Other MCP Clients

The pattern is the same for any MCP client: point it at the `amp-rs serve` command using stdio transport. Consult your client's docs for the exact config format.

---

## MCP Tools Reference

Once connected, your AI assistant gains these tools:

### Lessons

| Tool | Description |
|------|-------------|
| `add_lesson` | Record a lesson learned |
| `search_lessons` | Semantic search across lessons |
| `list_lessons` | List lessons, optionally filtered by severity |
| `delete_lesson` | Remove a lesson by ID |

### Checkpoints

| Tool | Description |
|------|-------------|
| `add_checkpoint` | Save agent working state |
| `get_recent_checkpoints` | Get most recent checkpoints for an agent |
| `search_checkpoints` | Semantic search across checkpoints |
| `get_agent_status` | Quick status: what an agent is working on |

### Code Indexing

| Tool | Description |
|------|-------------|
| `search_code` | Semantic code search — ask in English, get code |
| `index_repo` | Index a repository on demand |
| `diff_index` | Incremental re-index (only changed files) |
| `full_reindex` | Clear and re-index from scratch |

### Server

| Tool | Description |
|------|-------------|
| `get_status` | Server stats (counts, uptime) |

### Graph (when enabled)

| Tool | Description |
|------|-------------|
| `search_hybrid` | Vector search + graph expansion |
| `query_graph` | Direct graph traversal |
| `bootstrap_graph` | Seed graph from existing data |

---

## HTTP API

amp-rs runs a lightweight HTTP server alongside the MCP server. Use it for health checks, monitoring, and debugging.

### Health check

```
$ curl http://localhost:8080/health

{
  "status": "ok",
  "version": "0.1.0"
}
```

### Server status

```
$ curl http://localhost:8080/status

{
  "status": "ok",
  "version": "0.1.0",
  "lessons_count": 47,
  "checkpoints_count": 132,
  "chunks_count": 8491,
  "uptime_seconds": 86412
}
```

---

## Working with Lessons

Lessons are persistent notes that your AI assistant can search semantically. They survive across sessions and compound over time.

### What makes a good lesson

- **Gotchas:** "rusqlite bundled feature requires C compiler on this platform"
- **Patterns:** "Use spawn_blocking for embedding inference, not block_in_place"
- **Corrections:** "API returns 404 not 403 when token is expired"
- **Preferences:** "Always use bun, not npm, in this project"

### Severity levels

| Level | Use for |
|-------|---------|
| `info` | General knowledge, patterns, preferences |
| `warning` | Gotchas, footguns, things that waste time |
| `critical` | Showstoppers, data loss risks, security issues |

### How it looks in practice

Your AI assistant interacts with lessons through natural conversation:

```
You:    "Remember that SQLite WAL mode needs to be enabled at connection
         time, not after."

Claude: I'll save that as a lesson.
        [calls add_lesson(
            title: "SQLite WAL mode timing",
            content: "WAL mode must be enabled at connection time via
                      PRAGMA journal_mode=WAL, not after tables are
                      created. Setting it later may silently fail.",
            severity: "warning",
            tags: ["sqlite", "WAL", "rusqlite"]
        )]
        Saved. I'll remember this for future sessions.
```

Later, in a new session:

```
Claude: [calls search_lessons(query: "sqlite connection setup")]

        Found a relevant lesson from a previous session:
        "SQLite WAL mode must be enabled at connection time..."
        I'll make sure to set the pragma before any table operations.
```

### Listing and filtering

```
You:    "Show me all critical lessons."

Claude: [calls list_lessons(severity: "critical", limit: 50)]

        Found 3 critical lessons:
        1. "Never force-push to main" (2026-02-15)
        2. "ONNX model path must be absolute" (2026-02-20)
        3. "sqlite-vec load order matters" (2026-03-01)
```

### Optional graph fields

When the knowledge graph is enabled, lessons can include relationship metadata:

```
add_lesson(
    title: "OAuth refresh token rotation",
    content: "When the OAuth provider rotates refresh tokens...",
    severity: "warning",
    tags: ["oauth", "auth"],
    solved_problem: "silent auth failures after token rotation",
    used_tools: ["reqwest", "oauth2-rs"],
    related_concepts: ["token rotation", "refresh tokens", "OAuth 2.0"]
)
```

These fields create nodes and edges in the knowledge graph, enabling richer search via `search_hybrid`.

---

## Working with Checkpoints

Checkpoints save your AI assistant's working context — what it was doing, what decisions were made, what's next. They're the "session resume" mechanism.

### Saving a checkpoint

```
You:    "Save a checkpoint before we switch to the frontend."

Claude: [calls add_checkpoint(
            agent: "mmn/amp-rs",
            working_on: "Implementing lesson search with vector similarity",
            state: {
                "decisions": ["Using cosine similarity", "384-dim embeddings"],
                "current_file": "src/lessons/service.rs",
                "next_steps": ["Add pagination", "Wire up MCP tool"],
                "blockers": []
            }
        )]
        Checkpoint saved. You can switch contexts now.
```

### Resuming in a new session

```
Claude: [calls get_recent_checkpoints(agent: "mmn/amp-rs", limit: 3)]

        Last checkpoint (2 hours ago):
        "Implementing lesson search with vector similarity"
        - Using cosine similarity with 384-dim embeddings
        - Working in src/lessons/service.rs
        - Next: Add pagination, wire up MCP tool

        Ready to pick up where we left off.
```

### Searching across checkpoints

```
You:    "When did we last work on the embedding pipeline?"

Claude: [calls search_checkpoints(
            query: "embedding pipeline ONNX",
            agent: "mmn/amp-rs",
            limit: 5
        )]

        Found 2 relevant checkpoints:
        1. "Set up ONNX embedding pool" (2026-02-18)
           - Decided on 2 worker threads
           - Model: all-MiniLM-L6-v2
        2. "Embedding integration tests" (2026-02-19)
           - All tests passing
           - Throughput: ~50 embeddings/sec
```

### Agent status

```
You:    "What's the status of the frontend agent?"

Claude: [calls get_agent_status(agent: "mmn/frontend")]

        Agent: mmn/frontend
        Status: idle (last active 3 days ago)
        Last task: "Migrating to React 19"
```

### Optional graph fields

```
add_checkpoint(
    agent: "mmn/amp-rs",
    working_on: "Fix WAL lock contention",
    state: { ... },
    tools_used: ["rusqlite", "lsof"],
    problems_encountered: ["WAL lock contention under concurrent writes"],
    solutions_found: ["Use WAL2 mode with BEGIN CONCURRENT"],
    outcome: "success"
)
```

---

## Code Indexing

amp-rs can index your codebase for semantic search — ask questions in English, get relevant code.

### Index a repository

```
$ amp-rs index ~/projects/my-app

Indexing repository at "/home/user/projects/my-app"
Scanning files... 847 files found
Chunking... 3,291 chunks created
Embedding... done (3,291 embeddings in 42s)
Indexing complete.
```

### Automatic watching

Start the server with `--watch-dirs` to automatically re-index on file changes:

```
$ amp-rs serve --watch-dirs ~/projects/my-app,~/projects/my-lib
```

The watcher respects `.gitignore` rules and only re-indexes changed files.

### Searching code

```
You:    "Find where we handle database migrations"

Claude: [calls search_code(query: "database migration schema upgrade")]

        Found 3 relevant chunks:

        1. src/storage/schema.rs:15-42 (score: 0.89)
           pub fn migrate(&self) -> Result<()> {
               let version = self.get_schema_version()?;
               match version {
                   0 => self.migrate_v0_to_v1()?,
                   1 => self.migrate_v1_to_v2()?,
                   ...
               }
           }

        2. src/storage/sqlite.rs:8-20 (score: 0.82)
           impl SqliteStorage {
               pub fn open(path: &Path) -> Result<Self> { ... }
               pub fn migrate(&self) -> Result<()> { ... }
           }

        3. tests/storage_test.rs:45-60 (score: 0.71)
           #[test]
           fn test_migration_from_v0() { ... }
```

---

## Knowledge Graph

The knowledge graph is an optional layer that adds entity-relationship memory on top of vector search.

### Enabling

Via CLI flag:

```
$ amp-rs serve --enable-graph
```

Via config file (`~/.amp-rs/config.toml`):

```toml
[graph]
enabled = true
max_nodes = 100000
decay_half_life_days = 30
gc_min_confidence = 0.05
gc_orphan_days = 7
```

### How it works

The graph tracks entities (tools, problems, solutions, concepts) and relationships between them. Confidence scores on edges evolve over time:

```
New edge created               --> confidence: 0.3 (provisional)
Agent reports success (outcome) --> confidence: +0.2 (capped at 1.0)
Agent reports failure           --> confidence: -0.15 (floored at 0.0)
Agent reports partial           --> confidence: +0.05
Daily decay (on boot)           --> confidence *= 0.5^(days / half_life)
Confidence < 0.05              --> edge garbage collected
Orphaned node (7 days)         --> node garbage collected
```

### Hybrid search

`search_hybrid` combines vector similarity with graph traversal:

```
You:    "What tools have we used to solve OAuth issues?"

Claude: [calls search_hybrid(query: "OAuth authentication problems")]

        Vector matches:
        - Lesson: "OAuth refresh token rotation" (score: 0.91)
        - Checkpoint: "Implementing OAuth flow" (score: 0.84)

        Graph expansion:
        - Tool "reqwest" solved "OAuth token refresh" (confidence: 0.85)
        - Tool "oauth2-rs" solved "PKCE flow" (confidence: 0.72)
        - Concept "token rotation" related to "refresh tokens" (confidence: 0.90)
```

### Seeding the graph

If you have existing lessons and checkpoints, seed the graph from them:

```
You:    "Bootstrap the knowledge graph from existing data."

Claude: [calls bootstrap_graph()]

        Bootstrapped graph from existing data:
        - Created 142 nodes (38 tools, 27 problems, 31 solutions, 46 concepts)
        - Created 203 edges
        - All edges start at confidence 0.3 (provisional)
```

### Querying the graph directly

```
You:    "What do we know about rusqlite in the graph?"

Claude: [calls query_graph(label: "rusqlite", limit: 10)]

        Node: rusqlite (type: tool)

        Relationships:
        --> solved "WAL lock contention" (confidence: 0.85)
        --> solved "schema migration ordering" (confidence: 0.72)
        --> related_to "sqlite-vec" (confidence: 0.90)
        --> used_by agent "mmn/amp-rs" (confidence: 0.95)
        --> failed_for "concurrent write throughput" (confidence: 0.20)
```

---

## Troubleshooting

### Server won't start

**Symptom:** `amp-rs serve` exits immediately.

```
$ amp-rs serve 2>&1 | head

Error: Failed to initialize ONNX Runtime
```

**Fix:** Ensure the models directory is writable. amp-rs downloads the embedding model on first run:

```
$ ls -la ~/.amp-rs/models/
total 90M
-rw-r--r-- 1 user user 90M Mar  5 10:00 all-MiniLM-L6-v2.onnx
-rw-r--r-- 1 user user 711K Mar  5 10:00 tokenizer.json
```

If the model files are missing or corrupted, delete and restart:

```
$ rm -rf ~/.amp-rs/models/
$ amp-rs serve
```

### Port already in use

```
$ amp-rs serve

Error: Address already in use (port 8080)
```

**Fix:** Use a different port:

```
$ amp-rs serve --port 9090
```

Or find and stop the conflicting process:

```
$ lsof -i :8080
$ kill <PID>
```

### MCP client can't connect

**Symptom:** Claude Code or Cursor doesn't see amp-rs tools.

1. Verify amp-rs runs standalone:
   ```
   $ amp-rs serve
   # Should print startup messages without errors
   # Ctrl+C to stop
   ```

2. Check the MCP registration:
   ```
   # Claude Code
   $ claude mcp list

   # Cursor — check .cursor/mcp.json syntax
   $ cat .cursor/mcp.json | python3 -m json.tool
   ```

3. Check that `amp-rs` is on your PATH:
   ```
   $ which amp-rs
   /home/user/.local/bin/amp-rs
   ```

### Debug logging

Enable verbose output to diagnose issues:

```
$ amp-rs serve --debug

2026-03-05T10:00:01Z DEBUG amp_rs::storage: Opening SQLite connection
2026-03-05T10:00:01Z DEBUG amp_rs::storage: Running migration v0 -> v1
2026-03-05T10:00:01Z DEBUG amp_rs::embedding: Loading ONNX model
2026-03-05T10:00:02Z DEBUG amp_rs::embedding: Model loaded, 384 dimensions
2026-03-05T10:00:02Z DEBUG amp_rs::mcp: Registering 13 MCP tools
...
```

Or with fine-grained control:

```
$ RUST_LOG=amp_rs::embedding=debug,amp_rs::storage=trace amp-rs serve
```

### Database issues

**Reset the database** (deletes all data):

```
$ rm ~/.amp-rs/amp.db
$ amp-rs serve
# Fresh database created on startup
```

**Back up the database:**

```
$ cp ~/.amp-rs/amp.db ~/.amp-rs/amp.db.backup
```

The entire state is in one file. Copy it, move it, restore it.

### High CPU usage

Embedding generation is CPU-intensive. If amp-rs is consuming too many resources:

```
$ amp-rs serve --embedding-threads 1
```

The default is 2 threads. Reduce to 1 on constrained machines.

---

## Quick Reference

```
amp-rs serve                          # Start with defaults
amp-rs serve --debug                  # Verbose logging
amp-rs serve --port 9090              # Custom HTTP port
amp-rs serve --watch-dirs ~/proj      # Watch for file changes
amp-rs serve --embedding-threads 1    # Limit CPU usage
amp-rs index ~/my-repo                # One-shot index a repo

curl localhost:8080/health            # Health check
curl localhost:8080/status            # Server stats
```

### Data directory layout

```
~/.amp-rs/
├── amp.db              # SQLite database (lessons, checkpoints, chunks, graph)
├── config.toml         # Optional config file
└── models/
    ├── all-MiniLM-L6-v2.onnx    # Embedding model (~90MB)
    └── tokenizer.json            # Tokenizer config
```
