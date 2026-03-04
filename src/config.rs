use clap::Parser;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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

impl Config {
    /// Load config from file, with CLI overrides
    #[allow(dead_code)]
    pub fn load(cli: &Cli) -> anyhow::Result<Self> {
        let config_path = match &cli.command {
            Command::Serve { data_dir, .. } => {
                let data_dir_str = data_dir.to_string_lossy();
                let expanded = shellexpand::tilde(data_dir_str.as_ref());
                PathBuf::from(expanded.as_ref()).join("config.toml")
            }
            Command::Index { .. } => {
                let home = std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."));
                home.join(".amp-rs").join("config.toml")
            }
        };

        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            Config::default()
        };

        // Apply CLI overrides for Serve command
        if let Command::Serve {
            data_dir,
            watch_dirs,
            embedding_threads,
            port,
            ..
        } = &cli.command
        {
            // Only override if CLI values are explicitly set (non-default)
            // Check if data_dir was explicitly provided
            let data_dir_str = data_dir.to_string_lossy();
            if !data_dir_str.ends_with("/.amp-rs") {
                let expanded = shellexpand::tilde(data_dir_str.as_ref());
                config.data_dir = PathBuf::from(expanded.as_ref());
            }

            if !watch_dirs.is_empty() {
                config.watch_dirs = watch_dirs.clone();
            }

            // Embedding threads override only if not default
            if *embedding_threads != 2 {
                config.embedding_threads = *embedding_threads;
            }

            // Port override only if not default
            if *port != 8080 {
                config.port = *port;
            }
        }

        Ok(config)
    }
}
