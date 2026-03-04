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
