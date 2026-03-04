# Phase 5: Code Indexing

**Goal**: File scanner with .gitignore support, language-aware chunking, and semantic code search
**Duration**: 3–4 days
**Wave**: 2 (parallel with Phase 3 and Phase 4)
**Dependencies**: Phase 1 (storage) + Phase 2 (embeddings) complete

---

## Task 5.1: Indexing Pipeline

**Git**: `git checkout -b feature/5-1-indexing-pipeline`

### Subtask 5.1.1: File Scanner with .gitignore Support (Single Session)

**Prerequisites**:
- [x] 1.2.2: Storage Integration Tests
- [x] 2.2.2: Embedding Engine Tests

**Deliverables**:
- [ ] Create `src/indexing/mod.rs`:

```rust
pub mod chunker;
pub mod scanner;
pub mod storage;

use std::path::PathBuf;

/// A file discovered by the scanner
#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub repo_path: PathBuf,
    pub size: u64,
    pub mtime: u64,
    pub language: Option<String>,
}

/// A chunk of code extracted from a file
#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub id: String,
    pub file_path: String,
    pub repo_path: String,
    pub content: String,
    pub language: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: String,
}
```

- [ ] Create `src/indexing/scanner.rs`:

```rust
use super::ScannedFile;
use crate::error::Result;
use ignore::WalkBuilder;
use std::path::Path;
use tracing::debug;

/// Scan a directory for indexable files, respecting .gitignore
pub fn scan_directory(repo_path: &Path) -> Result<Vec<ScannedFile>> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(repo_path)
        .hidden(true)           // Skip hidden files
        .git_ignore(true)       // Respect .gitignore
        .git_global(true)       // Respect global gitignore
        .git_exclude(true)      // Respect .git/info/exclude
        .build();

    for entry in walker {
        let entry = entry.map_err(|e| crate::error::AmpError::Indexing(e.to_string()))?;

        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let language = detect_language(&path);

        // Skip binary/non-text files
        if !is_indexable(&path, &language) {
            continue;
        }

        let metadata = std::fs::metadata(&path)?;
        let mtime = metadata.modified()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0);

        files.push(ScannedFile {
            path: path.clone(),
            repo_path: repo_path.to_path_buf(),
            size: metadata.len(),
            mtime,
            language,
        });
    }

    debug!("Scanned {} indexable files in {:?}", files.len(), repo_path);
    Ok(files)
}

/// Detect programming language from file extension
pub fn detect_language(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "tsx" => "typescript",
            "jsx" => "javascript",
            "go" => "go",
            "java" => "java",
            "c" | "h" => "c",
            "cpp" | "hpp" | "cc" => "cpp",
            "rb" => "ruby",
            "sh" | "bash" => "shell",
            "toml" => "toml",
            "yaml" | "yml" => "yaml",
            "json" => "json",
            "md" => "markdown",
            "sql" => "sql",
            "html" => "html",
            "css" => "css",
            _ => return None,
        })
        .map(String::from)
}

/// Check if a file should be indexed
fn is_indexable(path: &Path, language: &Option<String>) -> bool {
    // Must have a recognized language
    if language.is_none() {
        return false;
    }

    // Skip files that are too large (> 1MB)
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.len() > 1_000_000 {
            return false;
        }
    }

    // Skip known binary/generated patterns
    let path_str = path.to_string_lossy();
    let skip_patterns = [
        "node_modules", "target/", ".git/", "__pycache__",
        "vendor/", "dist/", "build/", ".min.", "package-lock",
        "Cargo.lock",
    ];

    !skip_patterns.iter().any(|p| path_str.contains(p))
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(indexing): file scanner with .gitignore and language detection"
```

**Success Criteria**:
- [ ] Scanner respects .gitignore via `ignore` crate
- [ ] Language detection covers common languages
- [ ] Binary/generated files skipped

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 5.1.2: Language-Aware Chunking (Single Session)

**Prerequisites**:
- [x] 5.1.1: File Scanner

**Deliverables**:
- [ ] Create `src/indexing/chunker.rs`:

```rust
use super::CodeChunk;
use crate::error::Result;
use std::path::Path;
use uuid::Uuid;

const MAX_CHUNK_LINES: usize = 50;
const MIN_CHUNK_LINES: usize = 5;
const OVERLAP_LINES: usize = 3;

/// Chunk a file into semantic code blocks
pub fn chunk_file(
    file_path: &Path,
    repo_path: &Path,
    language: &Option<String>,
) -> Result<Vec<CodeChunk>> {
    let content = std::fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let file_path_str = file_path.to_string_lossy().to_string();
    let repo_path_str = repo_path.to_string_lossy().to_string();

    // Try semantic chunking first (by functions/classes), fall back to sliding window
    let chunks = if let Some(lang) = language {
        semantic_chunk(&lines, lang)
            .unwrap_or_else(|| sliding_window_chunk(&lines))
    } else {
        sliding_window_chunk(&lines)
    };

    Ok(chunks
        .into_iter()
        .map(|(start, end, chunk_type)| {
            let chunk_content = lines[start..end].join("\n");
            CodeChunk {
                id: Uuid::new_v4().to_string(),
                file_path: file_path_str.clone(),
                repo_path: repo_path_str.clone(),
                content: chunk_content,
                language: language.clone(),
                start_line: start + 1,
                end_line: end,
                chunk_type,
            }
        })
        .filter(|c| !c.content.trim().is_empty())
        .collect())
}

/// Attempt semantic chunking based on language patterns
fn semantic_chunk(lines: &[&str], language: &str) -> Option<Vec<(usize, usize, String)>> {
    let boundary_patterns: &[&str] = match language {
        "rust" => &["fn ", "pub fn ", "impl ", "struct ", "enum ", "trait ", "mod "],
        "python" => &["def ", "class ", "async def "],
        "javascript" | "typescript" => &["function ", "class ", "const ", "export "],
        "go" => &["func ", "type "],
        _ => return None,
    };

    let mut boundaries: Vec<usize> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if boundary_patterns.iter().any(|p| trimmed.starts_with(p)) {
            boundaries.push(i);
        }
    }

    if boundaries.is_empty() {
        return None;
    }

    let mut chunks = Vec::new();
    for (i, &start) in boundaries.iter().enumerate() {
        let end = if i + 1 < boundaries.len() {
            boundaries[i + 1]
        } else {
            lines.len()
        };

        // Split large chunks
        if end - start > MAX_CHUNK_LINES {
            let sub_chunks = sliding_window_chunk(&lines[start..end]);
            for (s, e, t) in sub_chunks {
                chunks.push((start + s, start + e, t));
            }
        } else if end - start >= MIN_CHUNK_LINES {
            chunks.push((start, end, "semantic".to_string()));
        }
    }

    // Include file header (imports, etc.) if not covered
    if !boundaries.is_empty() && boundaries[0] > MIN_CHUNK_LINES {
        chunks.insert(0, (0, boundaries[0], "header".to_string()));
    }

    Some(chunks)
}

/// Sliding window chunking with overlap
fn sliding_window_chunk(lines: &[&str]) -> Vec<(usize, usize, String)> {
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < lines.len() {
        let end = (start + MAX_CHUNK_LINES).min(lines.len());
        chunks.push((start, end, "window".to_string()));
        start = end.saturating_sub(OVERLAP_LINES);
        if start + MIN_CHUNK_LINES >= lines.len() {
            break;
        }
    }

    chunks
}
```

- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(indexing): language-aware chunking with semantic boundaries"
```

**Success Criteria**:
- [ ] Semantic chunking for Rust, Python, JS/TS, Go
- [ ] Sliding window fallback for unknown languages
- [ ] Chunk size bounded (5–50 lines)

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 5.1.3: Chunk Storage and Vector Indexing (Single Session)

**Prerequisites**:
- [x] 5.1.2: Language-Aware Chunking

**Deliverables**:
- [ ] Create `src/indexing/storage.rs` with chunk DB operations:
  - `store_chunks()` — insert chunks + embeddings
  - `delete_file_chunks()` — remove all chunks for a file
  - `get_indexed_file()` — check if file is indexed and its mtime
  - `update_indexed_file()` — update file tracking record
  - `search_chunks()` — vector search on chunk embeddings
- [ ] Run `cargo check`
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(indexing): chunk storage with vector indexing and file tracking"
```

**Success Criteria**:
- [ ] Chunks stored in code_chunks table
- [ ] Embeddings stored in chunk_embeddings virtual table
- [ ] File tracking in indexed_files table

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 5.1 Complete — Squash Merge
- [ ] All subtasks 5.1.1–5.1.3 complete
- [ ] `cargo check` passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/5-1-indexing-pipeline
git commit -m "feat(indexing): complete task 5.1 - indexing pipeline"
git branch -d feature/5-1-indexing-pipeline
git push origin main
```

---

## Task 5.2: Search and Tests

**Git**: `git checkout -b feature/5-2-code-search`

### Subtask 5.2.1: Semantic Code Search (Single Session)

**Prerequisites**:
- [x] 5.1.3: Chunk Storage

**Deliverables**:
- [ ] Create indexing coordinator that orchestrates: scan → chunk → embed → store
- [ ] Add `search_code()` method that embeds query and searches chunks
- [ ] Git commit:
```bash
git add -A && git commit -m "feat(indexing): semantic code search coordinator"
```

**Success Criteria**:
- [ ] Full pipeline: scan → chunk → embed → store
- [ ] Search returns relevant code chunks

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: N/A
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Subtask 5.2.2: Code Indexing Integration Tests (Single Session)

**Prerequisites**:
- [x] 5.2.1: Semantic Code Search

**Deliverables**:
- [ ] Unit tests for scanner (mock filesystem or use tempdir)
- [ ] Unit tests for chunker (verify chunk boundaries)
- [ ] Integration tests for full pipeline (scan → search)
- [ ] Create `tests/indexing_test.rs`
- [ ] Run full verification:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
- [ ] Git commit:
```bash
git add -A && git commit -m "test(indexing): unit and integration tests for code indexing"
```

**Success Criteria**:
- [ ] Scanner tests verify .gitignore respect
- [ ] Chunker tests verify semantic boundaries
- [ ] Full verification passes

**Completion Notes**:
- **Implementation**: (describe)
- **Files Created**: (list with line counts)
- **Files Modified**: (list)
- **Tests**: (X tests passing)
- **Build**: (pass/fail)
- **Notes**: (context)

---

### Task 5.2 Complete — Squash Merge
- [ ] All subtasks 5.2.1–5.2.2 complete
- [ ] Full verification passes
- [ ] Squash merge:
```bash
git checkout main && git merge --squash feature/5-2-code-search
git commit -m "feat(indexing): complete task 5.2 - code search and tests"
git branch -d feature/5-2-code-search
git push origin main
```

---

*Phase 5 complete when both tasks merged to main.*
