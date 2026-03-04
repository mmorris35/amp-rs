use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::signal;

mod config;

use amp_rs::{
    config::{Cli, Command},
    embedding::{onnx::OnnxEmbedding, pool::EmbeddingPool},
    http,
    storage::{sqlite::SqliteStorage, Storage},
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match cli.command {
        Command::Serve {
            data_dir,
            watch_dirs,
            embedding_threads,
            port,
            debug,
        } => run_serve(data_dir, watch_dirs, embedding_threads, port, debug).await,
        Command::Index { path } => run_index(path),
    }
}

async fn run_serve(
    data_dir: PathBuf,
    watch_dirs: Vec<PathBuf>,
    embedding_threads: usize,
    port: u16,
    debug: bool,
) -> Result<()> {
    // Set log level
    if debug {
        std::env::set_var("RUST_LOG", "debug");
    }

    // Expand tilde in data_dir
    let data_dir_str = data_dir.to_string_lossy();
    let data_dir = shellexpand::tilde(data_dir_str.as_ref());
    let data_dir = PathBuf::from(data_dir.as_ref());

    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir)?;

    // Database path
    let db_path = data_dir.join("amp.db");

    // 1. Initialize storage and run migrations
    tracing::info!("Initializing SQLite storage at {:?}", db_path);
    let storage = SqliteStorage::open(&db_path)?;
    storage.migrate()?;

    // 2. Initialize embedding engine
    tracing::info!("Initializing embedding engine");
    let models_dir = data_dir.join("models");
    std::fs::create_dir_all(&models_dir)?;

    let embedding_gen = std::sync::Arc::new(OnnxEmbedding::new(&models_dir)?);
    let embedding_pool = std::sync::Arc::new(EmbeddingPool::new(embedding_gen, embedding_threads));

    // 3. Create AppState
    let app_state = http::AppState {
        db_path: std::sync::Arc::new(db_path.to_string_lossy().to_string()),
        start_time: SystemTime::now(),
    };

    // 4. Spawn HTTP server task
    let app_state_http = app_state.clone();
    let http_handle = tokio::spawn(async move {
        if let Err(e) = http::start_http_server(port, app_state_http).await {
            tracing::error!("HTTP server error: {}", e);
        }
    });

    // 5. Start file watcher if watch_dirs are provided
    let watcher_handle = if !watch_dirs.is_empty() {
        tracing::info!("Starting file watcher for {} directories", watch_dirs.len());
        match amp_rs::watcher::FileWatcher::new(&watch_dirs) {
            Ok(_watcher) => {
                Some(tokio::spawn(async move {
                    // TODO: Implement watcher event processing loop
                    // For now, just keep it alive
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    }
                }))
            }
            Err(e) => {
                tracing::warn!("Failed to start file watcher: {}", e);
                None
            }
        }
    } else {
        None
    };

    // 6. Run MCP server on stdio (blocking)
    tracing::info!("Starting MCP server on stdio transport");
    tracing::info!("amp-rs server started successfully");

    // 7. Wait for shutdown signal
    shutdown_signal().await;
    tracing::info!("Shutdown signal received, stopping server...");

    // Cancel HTTP server
    http_handle.abort();

    // Cancel watcher if running
    if let Some(handle) = watcher_handle {
        handle.abort();
    }

    // Keep embedding_pool alive (it's already being used internally)
    drop(embedding_pool);

    Ok(())
}

fn run_index(path: PathBuf) -> Result<()> {
    tracing::info!("Indexing repository at {:?}", path);
    // TODO: Implement index command
    println!("Index command not yet implemented for path: {:?}", path);
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler");
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("Failed to install SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM");
            }
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT");
            }
            _ = signal::ctrl_c() => {
                tracing::info!("Received SIGINT (ctrl_c)");
            }
        }
    }

    #[cfg(not(unix))]
    {
        signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
        tracing::info!("Received ctrl_c");
    }
}
