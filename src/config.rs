use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "amp-rs",
    version,
    about = "Agent Memory Protocol - Reference Implementation"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    /// Start the AMP server (MCP stdio + HTTP)
    Serve {
        /// Data directory for database and models
        #[arg(long, default_value = "~/.amp-rs")]
        data_dir: PathBuf,

        /// Directories to watch for file changes
        #[arg(long, value_delimiter = ',')]
        watch_dirs: Vec<PathBuf>,

        /// Number of embedding threads
        #[arg(long, default_value = "2")]
        embedding_threads: usize,

        /// HTTP port for health check and REST
        #[arg(long, default_value = "8080")]
        port: u16,

        /// Enable debug logging
        #[arg(long)]
        debug: bool,
    },
    /// Index a repository on demand
    Index {
        /// Path to repository
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub data_dir: PathBuf,
    pub watch_dirs: Vec<PathBuf>,
    pub embedding_threads: usize,
    pub port: u16,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        Self {
            data_dir: home.join(".amp-rs"),
            watch_dirs: Vec::new(),
            embedding_threads: 2,
            port: 8080,
            log_level: "info".to_string(),
        }
    }
}
