use amp_rs::indexing::{chunker, scanner};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_scanner_detects_languages() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create files with different extensions
    fs::write(repo_path.join("test.rs"), "fn main() {}").unwrap();
    fs::write(repo_path.join("test.py"), "def main():").unwrap();
    fs::write(repo_path.join("test.js"), "function main() {}").unwrap();

    let files = scanner::scan_directory(repo_path).unwrap();

    assert!(!files.is_empty(), "Should find indexable files");
    let languages: Vec<_> = files.iter().map(|f| f.language.clone()).collect();
    assert!(languages.contains(&Some("rust".to_string())));
    assert!(languages.contains(&Some("python".to_string())));
    assert!(languages.contains(&Some("javascript".to_string())));
}

#[test]
fn test_chunker_basic_chunking() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create a file with multiple lines to get chunked
    let content = (0..100)
        .map(|i| format!("line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&file_path, content).unwrap();

    let chunks = chunker::chunk_file(&file_path, temp_dir.path(), &None).unwrap();

    assert!(
        !chunks.is_empty(),
        "Should create chunks from multi-line file"
    );
    for chunk in chunks {
        assert!(chunk.start_line > 0);
        assert!(chunk.end_line >= chunk.start_line);
    }
}

#[test]
fn test_chunker_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.rs");

    fs::write(&file_path, "").unwrap();

    let chunks =
        chunker::chunk_file(&file_path, temp_dir.path(), &Some("rust".to_string())).unwrap();

    assert!(chunks.is_empty(), "Empty files should produce no chunks");
}

#[test]
fn test_scanner_skips_large_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create a large file (>1MB)
    let large_content = "x".repeat(2_000_000);
    fs::write(repo_path.join("large.rs"), large_content).unwrap();

    // Create a normal file
    fs::write(repo_path.join("small.rs"), "fn main() {}").unwrap();

    let files = scanner::scan_directory(repo_path).unwrap();

    // Should only find the small file
    let file_names: Vec<_> = files.iter().map(|f| f.path.file_name()).collect();
    assert!(file_names
        .iter()
        .any(|n| n.map_or(false, |n| n == "small.rs")));
    assert!(!file_names
        .iter()
        .any(|n| n.map_or(false, |n| n == "large.rs")));
}

#[test]
fn test_scanner_skips_binary_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create files that should be skipped
    fs::write(repo_path.join("package-lock.json"), "{}").unwrap();
    fs::write(repo_path.join("Cargo.lock"), "").unwrap();
    fs::write(repo_path.join("minified.min.js"), "var x = 1;").unwrap();

    let files = scanner::scan_directory(repo_path).unwrap();

    // Should find no files (all are skip patterns)
    assert!(
        files.is_empty(),
        "Should skip lock files and minified files"
    );
}

#[test]
fn test_scanner_scans_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Create multiple indexable files
    fs::write(repo_path.join("main.rs"), "fn main() {}").unwrap();
    fs::write(repo_path.join("lib.rs"), "pub mod lib;").unwrap();
    fs::write(repo_path.join("utils.rs"), "pub fn util() {}").unwrap();

    let files = scanner::scan_directory(repo_path).unwrap();

    assert_eq!(files.len(), 3, "Should find all three Rust files");
    for file in &files {
        assert_eq!(file.language, Some("rust".to_string()));
        assert!(file.size > 0);
    }
}

#[test]
fn test_chunker_line_count_bounds() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    // Create a file with 200 lines (should create multiple chunks)
    let content = (0..200)
        .map(|i| format!("line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&file_path, content).unwrap();

    let chunks = chunker::chunk_file(&file_path, temp_dir.path(), &None).unwrap();

    assert!(!chunks.is_empty());

    // Check that chunks don't exceed max size
    for chunk in chunks {
        let chunk_lines = chunk.content.lines().count();
        // Chunks should be reasonable size (less than 100 lines to account for line counting differences)
        assert!(
            chunk_lines > 0 && chunk_lines < 100,
            "Chunk size {} is out of bounds",
            chunk_lines
        );
    }
}
