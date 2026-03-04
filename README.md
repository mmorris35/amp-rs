# amp-rs

**Reference implementation of [Agent Memory Protocol](https://github.com/mmorris35/agent-memory-protocol).** Local-first MCP server with checkpoints, lessons, semantic search, and code indexing. Built in Rust. Single binary. No cloud.

This is a reference board — a clean, opinionated starting point for building AMP into your own workflow. Take it, run it, make it yours.

## What it does

AI coding assistants start every session from zero. You spend the first 10 minutes re-explaining context, re-discovering decisions, re-learning lessons. AMP fixes this with three primitives:

| Primitive | Purpose |
|-----------|---------|
| **Checkpoints** | Where was I? What's done, what's next? Restore in seconds. |
| **Lessons** | What did I learn that I don't want to learn again? Compound over time. |
| **Code Memory** | Semantic search across your codebase. Ask questions in English, get code. |

amp-rs implements all three as an MCP server that works with Claude Code, Cursor, Aider, Cline, Windsurf — anything that speaks MCP.

## Quick start

### Install

```bash
# Build from source
git clone https://github.com/mmorris35/amp-rs.git
cd amp-rs
cargo build --release

# Binary lands at target/release/amp-rs
```

### Connect to Claude Code

```bash
claude mcp add amp-rs --launch "amp-rs serve"
```

### Connect to Cursor

Add to `.cursor/mcp.json`:
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

### First use

Once connected, your AI assistant has access to these tools:

```
search_code          Semantic code search — ask in English, get relevant code
search_lessons       Search lessons by natural language query
list_lessons         List all lessons (filter by severity)
add_lesson           Record a lesson learned
delete_lesson        Remove a lesson
add_checkpoint       Save agent working state
get_recent_checkpoints  Restore where you left off
search_checkpoints   Semantic search across checkpoints
get_agent_status     Quick status check (idle/working, current task)
index_repo           Index a repository on demand
diff_index           Incremental re-index (only changed files)
full_reindex         Nuclear option — clear and re-index from scratch
get_status           Server stats
```

## How it works

```
┌──────────────────────────────────────────────────────────┐
│                         amp-rs                            │
│                                                           │
│  ┌────────────┐   ┌──────────────┐   ┌────────────────┐ │
│  │  MCP Server │   │  Embedding   │   │  File Watcher  │ │
│  │  (stdio)    │   │  Worker Pool │   │  (notify-rs)   │ │
│  └──────┬──────┘   └──────┬───────┘   └───────┬────────┘ │
│         │                 │                    │           │
│         ▼                 ▼                    ▼           │
│  ┌────────────────────────────────────────────────────┐   │
│  │        SQLite + sqlite-vec (single file)            │   │
│  │   Chunks · Lessons · Checkpoints · File State       │   │
│  └────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────┘
```

- **MCP over stdio** — standard transport, works with every MCP client
- **all-MiniLM-L6-v2** — ONNX embeddings, 384 dimensions, runs locally on CPU
- **sqlite-vec** — vector similarity search embedded in SQLite, no external services
- **notify-rs** — watches your repos for changes, indexes incrementally
- **Single file database** — back it up by copying one file

## Configuration

```bash
# Defaults work out of the box. Override what you need:
amp-rs serve \
  --data-dir ~/.amp-rs \
  --watch-dirs ~/projects,~/work \
  --embedding-threads 2 \
  --port 8080
```

Or use a config file at `~/.amp-rs/config.toml`:

```toml
data_dir = "~/.amp-rs"
watch_dirs = ["~/projects", "~/work"]
embedding_threads = 2
port = 8080
log_level = "info"
```

## What you get out of the box

This is the reference board. Everything here works, with sensible defaults, no configuration required:

- **Semantic code search** across any repo you point it at
- **Lessons that compound** — save what you learn, search it later, stop repeating mistakes
- **Session continuity** — checkpoints let your AI pick up exactly where it left off
- **Incremental indexing** — watches for file changes, only re-indexes what changed
- **Single binary, single file** — `amp-rs` binary + `~/.amp-rs/amp.db`. That's the whole deployment.
- **Air-gap ready** — no cloud, no API keys, no phone-home. Everything runs locally.

---

## Bolt on the graph: making it fly

> The reference board gets you driving. The graph module is the turbo kit.

The base amp-rs gives you vector search — "find text similar to X." That's powerful, but it doesn't know *relationships*. It can't answer "what tool solved this problem last time?" or "what concepts relate to this error?"

The graph module adds a knowledge layer on top of vector search. It's opt-in, additive, and changes nothing about the base behavior.

### What the graph adds

**Entity-relationship memory.** Every lesson and checkpoint can create nodes and edges in an in-memory graph:

- **Entity types:** agent, tool, problem, solution, concept, person, project, chunk
- **Relationship types:** used, solved, failed_for, knows, prefers, depends_on, related_to, derived_from

**Self-improving confidence.** Edges start provisional (confidence: 0.3). When an agent reports that a suggestion worked, confidence goes up. When it didn't, confidence goes down. Over time, the graph learns what actually works.

**Hybrid search.** Vector search finds the entry point. Graph traversal expands the context — "you searched for OAuth, and here are the tools that solved OAuth problems before, the lessons learned, and the concepts involved."

### New tools with the graph

```
search_hybrid        Vector search + graph expansion for richer results
query_graph          Direct graph traversal (by entity, relationship, direction)
bootstrap_graph      Seed the graph from existing lessons and checkpoints
```

### Enriched existing tools

Existing tools gain optional graph fields — backward compatible, no breaking changes:

**add_lesson** — new optional fields:
```
solved_problem       → "what problem does this lesson address?"
used_tools           → "what tools were involved?"
related_concepts     → "what concepts does this touch?"
```

**add_checkpoint** — new optional fields:
```
tools_used           → tools used this session
problems_encountered → problems hit
solutions_found      → what worked
outcome              → "success" | "failure" | "partial"
```

When these fields are provided, amp-rs creates graph nodes and edges automatically. When omitted, behavior is identical to the base — nothing breaks.

### Enabling the graph

```bash
amp-rs serve --enable-graph
```

Or in `config.toml`:

```toml
[graph]
enabled = true
max_nodes = 100000
decay_half_life_days = 30
gc_min_confidence = 0.05
gc_orphan_days = 7
```

### How confidence works

```
New edge created               → confidence: 0.3 (provisional)
Agent reports success          → confidence: +0.2 (capped at 1.0)
Agent reports failure          → confidence: -0.15 (floored at 0.0)
Agent reports partial          → confidence: +0.05
Daily decay (on boot)          → confidence *= 0.5^(days / half_life)
Confidence < 0.05              → edge garbage collected
Orphaned node (no edges, 7d)   → node garbage collected
```

The graph gets smarter the more you use it. Stale knowledge fades. Confirmed knowledge strengthens. Bad suggestions die off.

### Schema additions (v2, additive only)

```sql
CREATE TABLE graph_nodes (
    id TEXT PRIMARY KEY,
    node_type TEXT NOT NULL,
    label TEXT NOT NULL,
    label_normalized TEXT NOT NULL,
    record_id TEXT,
    metadata TEXT,
    created_at INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0
);

CREATE TABLE graph_edges (
    id TEXT PRIMARY KEY,
    from_node TEXT NOT NULL REFERENCES graph_nodes(id),
    to_node TEXT NOT NULL REFERENCES graph_nodes(id),
    relationship TEXT NOT NULL,
    confidence REAL DEFAULT 0.3,
    provisional INTEGER DEFAULT 1,
    context TEXT,
    created_at INTEGER NOT NULL,
    last_confirmed INTEGER NOT NULL,
    access_count INTEGER DEFAULT 0,
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0
);
```

No existing tables are modified. The graph is purely additive.

---

## Architecture

| Component | Crate | Why |
|-----------|-------|-----|
| MCP Protocol | `rmcp` | Official Anthropic MCP SDK for Rust |
| Async Runtime | `tokio` | Industry standard, required by rmcp |
| HTTP Server | `axum` | Health checks, optional REST endpoints |
| Vector Storage | `rusqlite` + `sqlite-vec` | Embedded, no external process |
| Embeddings | `ort` (ONNX Runtime) | Local inference, no Python |
| File Watching | `notify` | Cross-platform, efficient |
| Graph (v2) | `petgraph` | In-memory directed graph |
| Fuzzy Match (v2) | `strsim` | Levenshtein distance for entity resolution |
| CLI | `clap` | Standard |
| Serialization | `serde` | Standard |

## Contributing

This is a reference implementation. Fork it, extend it, build something better. PRs welcome.

## About

amp-rs is part of the [Agent Memory Protocol](https://github.com/mmorris35/agent-memory-protocol) ecosystem. AMP is the spec. amp-rs is the reference board — a straightforward implementation designed to show what the protocol can do and give you a starting point.

Built by [Mike Morris](https://mikemorris.net) with Claude.

## License

MIT
