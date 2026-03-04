# Phase 9: Integration & Release

**Goal**: End-to-end tests, CI/CD, documentation, and release preparation
**Duration**: 3–4 days
**Wave**: 4 (sequential — depends on all other phases)
**Dependencies**: Phases 6, 7, 8 complete

---

## Task 9.1: Integration Testing

**Git**: `git checkout -b feature/9-1-integration-tests`

### Subtask 9.1.1: End-to-End MCP Protocol Tests (Single Session)

**Prerequisites**:
- [x] 7.2.2: MCP Server Integration Tests
- [x] 6.2.2: File Watcher Tests
- [x] 8.2.2: CLI and Config Tests

**Deliverables**:
- [ ] Create `tests/integration_test.rs` (replace placeholder):
  - Start amp-rs server in test
  - Send MCP protocol messages via stdio
  - Verify complete tool lifecycle:
    1. add_lesson → search_lessons → list_lessons → delete_lesson
    2. add_checkpoint → get_recent_checkpoints → search_checkpoints → get_agent_status
    3. index_repo → search_code → diff_index → full_reindex
    4. get_status returns correct counts
  - Verify error handling for invalid inputs
- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test: end-to-end MCP protocol integration tests"
```

**Success Criteria**:
- [ ] Full tool lifecycle tests pass
- [ ] Error handling verified
- [ ] All tests pass

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 9.1.2: Cross-Feature Integration Tests (Single Session)

**Prerequisites**:
- [x] 9.1.1: E2E MCP Tests

**Deliverables**:
- [ ] Test concurrent operations:
  - Multiple agents saving checkpoints simultaneously
  - Indexing while lessons are being added
  - Search during active indexing
- [ ] Test data persistence:
  - Create data, restart server, verify data still there
- [ ] Test config variations:
  - Different data dirs
  - Different port numbers
  - Missing config file (defaults work)
- [ ] Git commit:
```bash
git add -A && git commit -m "test: cross-feature integration tests"
```

**Success Criteria**:
- [ ] Concurrent operations work correctly
- [ ] Data persists across restarts
- [ ] Config variations tested

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 9.1.3: Performance Benchmarks (Single Session)

**Prerequisites**:
- [x] 9.1.2: Cross-Feature Tests

**Deliverables**:
- [ ] Create `benches/` directory with criterion benchmarks (or simple timed tests):
  - Embedding generation throughput
  - SQLite insert/query performance
  - Vector search latency at various scales (100, 1000, 10000 vectors)
  - Chunking throughput (files per second)
- [ ] Document baseline performance numbers in README
- [ ] Git commit:
```bash
git add -A && git commit -m "perf: performance benchmarks and baseline numbers"
```

**Success Criteria**:
- [ ] Benchmarks run and produce numbers
- [ ] Baseline documented

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (benchmarks run)
- **Build**: (pass/fail)
- **Notes**: baseline numbers

---

### Task 9.1 Complete — Squash Merge
- [ ] All subtasks 9.1.1–9.1.3 complete
- [ ] Full verification passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/9-1-integration-tests
git commit -m "test: complete task 9.1 - integration tests and benchmarks"
git branch -d feature/9-1-integration-tests
git push origin main
```

---

## Task 9.2: CI/CD and Release

**Git**: `git checkout -b feature/9-2-cicd-release`

### Subtask 9.2.1: CI/CD with GitHub Actions (Single Session)

**Prerequisites**:
- [x] 9.1.3: Performance Benchmarks

**Deliverables**:
- [ ] Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo check --workspace

  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace -- -D warnings

  test:
    name: Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace

  build:
    name: Build Release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release
      - uses: actions/upload-artifact@v4
        with:
          name: amp-rs-${{ matrix.os }}
          path: target/release/amp-rs
```

- [ ] Git commit:
```bash
git add -A && git commit -m "ci: GitHub Actions CI/CD pipeline"
```

**Success Criteria**:
- [ ] CI checks: format, clippy, test, build
- [ ] Builds on Linux and macOS
- [ ] Artifacts uploaded

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 9.2.2: Documentation and Release Prep (Single Session)

**Prerequisites**:
- [x] 9.2.1: CI/CD

**Deliverables**:
- [ ] Update `README.md`:
  - Verify installation instructions work
  - Verify all MCP tools documented
  - Add "Building from source" section
  - Add configuration reference
  - Update architecture diagram if needed
- [ ] Verify `cargo doc` generates documentation
- [ ] Ensure `LICENSE` is MIT
- [ ] Create `.gitignore` additions if needed
- [ ] Final full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Verify single binary:
```bash
cargo build --release && ls -la target/release/amp-rs
```
- [ ] Git commit:
```bash
git add -A && git commit -m "docs: documentation and release preparation"
```

**Success Criteria**:
- [ ] README complete and accurate
- [ ] `cargo doc` succeeds
- [ ] Single binary builds
- [ ] All tests pass
- [ ] LICENSE exists

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (all passing)
- **Build**: (release binary size)
- **Notes**: (context)

---

### Task 9.2 Complete — Squash Merge
- [ ] All subtasks 9.2.1–9.2.2 complete
- [ ] Full verification passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/9-2-cicd-release
git commit -m "chore: complete task 9.2 - CI/CD and release prep"
git branch -d feature/9-2-cicd-release
git push origin main
```

---

## MVP Complete Checklist

After Phase 9, verify all MVP features:

- [ ] `amp-rs serve` starts MCP (stdio) + HTTP server
- [ ] `amp-rs --help` shows usage
- [ ] `amp-rs --version` shows version
- [ ] Health check: `curl localhost:8080/health`
- [ ] Status: `curl localhost:8080/status`
- [ ] All 13 MCP tools work (use amp-rs-verifier agent)
- [ ] Single binary: `ls -la target/release/amp-rs`
- [ ] Config file loads from `~/.amp-rs/config.toml`
- [ ] CI passes on GitHub

**Run the verifier agent**:
```
Use the amp-rs-verifier agent to validate the application against PROJECT_BRIEF.md
```

---

*Phase 9 complete = MVP complete.*
