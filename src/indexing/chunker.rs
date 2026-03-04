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
        semantic_chunk(&lines, lang).unwrap_or_else(|| sliding_window_chunk(&lines))
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
        "rust" => &[
            "fn ", "pub fn ", "impl ", "struct ", "enum ", "trait ", "mod ",
        ],
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_chunk_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.rs");
        fs::write(&file_path, "").unwrap();

        let chunks = chunk_file(&file_path, temp_dir.path(), &Some("rust".to_string())).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = r#"fn main() {
    println!("Hello");
}

fn helper() {
    println!("Helper");
}"#;
        fs::write(&file_path, content).unwrap();

        let chunks = chunk_file(&file_path, temp_dir.path(), &Some("rust".to_string())).unwrap();
        assert!(!chunks.is_empty());

        // Should have semantic chunks
        for chunk in chunks {
            assert_eq!(chunk.language, Some("rust".to_string()));
            assert!(!chunk.content.is_empty());
        }
    }

    #[test]
    fn test_chunk_preserves_line_numbers() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
        fs::write(&file_path, content).unwrap();

        let chunks = chunk_file(&file_path, temp_dir.path(), &None).unwrap();
        assert!(!chunks.is_empty());

        for chunk in chunks {
            assert!(chunk.start_line > 0);
            assert!(chunk.end_line >= chunk.start_line);
        }
    }

    #[test]
    fn test_sliding_window_chunking() {
        let lines: Vec<&str> = (0..100).map(|_| "line").collect();

        let chunks = sliding_window_chunk(&lines);
        assert!(!chunks.is_empty());

        // All chunks should be within size limits
        for (start, end, _) in &chunks {
            let size = end - start;
            assert!(size <= MAX_CHUNK_LINES);
            assert!(size >= MIN_CHUNK_LINES || *end == lines.len());
        }
    }

    #[test]
    fn test_semantic_chunk_rust() {
        let lines = vec![
            "use std::io;",
            "",
            "fn main() {",
            "    println!(\"Hello\");",
            "}",
            "",
            "fn helper() {",
            "    println!(\"Helper\");",
            "}",
        ];

        let chunks = semantic_chunk(&lines, "rust");
        assert!(chunks.is_some());

        let chunks = chunks.unwrap();
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_semantic_chunk_python() {
        let lines = vec![
            "import sys",
            "",
            "def main():",
            "    print('Hello')",
            "",
            "class Helper:",
            "    def __init__(self):",
            "        pass",
        ];

        let chunks = semantic_chunk(&lines, "python");
        assert!(chunks.is_some());
    }

    #[test]
    fn test_semantic_chunk_unknown_language() {
        let lines = vec!["code", "more code"];
        let chunks = semantic_chunk(&lines, "unknown");
        assert!(chunks.is_none());
    }
}
