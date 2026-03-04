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

        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let language = detect_language(&path);

        // Skip binary/non-text files
        if !is_indexable(&path, &language) {
            continue;
        }

        let metadata = std::fs::metadata(&path)?;
        let mtime = metadata
            .modified()
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })
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
        .and_then(|ext| {
            let lang = match ext {
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
            };
            Some(lang.to_string())
        })
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
        "node_modules",
        "target/",
        ".git/",
        "__pycache__",
        "vendor/",
        "dist/",
        "build/",
        ".min.",
        "package-lock",
        "Cargo.lock",
    ];

    !skip_patterns.iter().any(|p| path_str.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_language() {
        assert_eq!(
            detect_language(Path::new("test.rs")),
            Some("rust".to_string())
        );
        assert_eq!(
            detect_language(Path::new("test.py")),
            Some("python".to_string())
        );
        assert_eq!(
            detect_language(Path::new("test.js")),
            Some("javascript".to_string())
        );
        assert_eq!(
            detect_language(Path::new("test.ts")),
            Some("typescript".to_string())
        );
        assert_eq!(
            detect_language(Path::new("test.go")),
            Some("go".to_string())
        );
        assert_eq!(
            detect_language(Path::new("test.java")),
            Some("java".to_string())
        );
        assert_eq!(detect_language(Path::new("test.unknown")), None);
    }

    #[test]
    fn test_is_indexable() {
        // Language must be present
        assert!(!is_indexable(Path::new("test.bin"), &None));

        // Valid language
        assert!(is_indexable(
            Path::new("test.rs"),
            &Some("rust".to_string())
        ));

        // Skip known patterns
        let skip_paths = [
            "node_modules/test.js",
            "target/test.rs",
            ".git/test.rs",
            "__pycache__/test.py",
            "vendor/test.go",
            "dist/test.js",
            "build/test.rs",
            "test.min.js",
            "package-lock.json",
            "Cargo.lock",
        ];
        for skip_path in &skip_paths {
            assert!(!is_indexable(
                Path::new(skip_path),
                &Some("rust".to_string())
            ));
        }
    }

    #[test]
    fn test_scan_directory_respects_gitignore() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo so that ignore crate will respect .gitignore
        std::process::Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .output()
            .ok(); // Ignore if git is not available

        // Create some test files
        fs::write(repo_path.join("main.rs"), "fn main() {}").unwrap();
        fs::write(repo_path.join("lib.rs"), "pub mod test;").unwrap();

        // Create .gitignore
        fs::write(repo_path.join(".gitignore"), "ignored.rs\n").unwrap();

        // Create ignored file
        fs::write(repo_path.join("ignored.rs"), "fn ignored() {}").unwrap();

        // Scan directory
        let files = scan_directory(repo_path).unwrap();

        // Should find main.rs and lib.rs, but not ignored.rs
        let file_names: Vec<_> = files.iter().map(|f| f.path.file_name()).collect();
        assert!(file_names
            .iter()
            .any(|n| n.map_or(false, |n| n == "main.rs")));
        assert!(file_names
            .iter()
            .any(|n| n.map_or(false, |n| n == "lib.rs")));
        assert!(!file_names
            .iter()
            .any(|n| n.map_or(false, |n| n == "ignored.rs")));
    }

    #[test]
    fn test_scan_directory_returns_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        fs::write(repo_path.join("test.rs"), "fn test() {}").unwrap();

        let files = scan_directory(repo_path).unwrap();
        assert!(!files.is_empty());

        let file = &files[0];
        assert_eq!(file.language, Some("rust".to_string()));
        assert!(file.size > 0);
        assert!(file.mtime > 0);
    }
}
