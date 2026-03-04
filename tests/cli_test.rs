use amp_rs::config::{Cli, Command, Config};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_cli_parse_serve_defaults() {
    let args = vec!["amp-rs", "serve"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Command::Serve {
            port,
            embedding_threads,
            debug,
            ..
        } => {
            assert_eq!(port, 8080);
            assert_eq!(embedding_threads, 2);
            assert!(!debug);
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parse_serve_with_port() {
    let args = vec!["amp-rs", "serve", "--port", "9000"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Command::Serve { port, .. } => {
            assert_eq!(port, 9000);
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parse_serve_with_embedding_threads() {
    let args = vec!["amp-rs", "serve", "--embedding-threads", "4"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Command::Serve {
            embedding_threads, ..
        } => {
            assert_eq!(embedding_threads, 4);
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parse_serve_with_debug() {
    let args = vec!["amp-rs", "serve", "--debug"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Command::Serve { debug, .. } => {
            assert!(debug);
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parse_serve_with_watch_dirs() {
    let args = vec!["amp-rs", "serve", "--watch-dirs", "/path/one,/path/two"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Command::Serve { watch_dirs, .. } => {
            assert_eq!(watch_dirs.len(), 2);
            assert_eq!(watch_dirs[0], PathBuf::from("/path/one"));
            assert_eq!(watch_dirs[1], PathBuf::from("/path/two"));
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_cli_parse_index() {
    let args = vec!["amp-rs", "index", "/home/user/my-repo"];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Command::Index { path } => {
            assert_eq!(path, PathBuf::from("/home/user/my-repo"));
        }
        _ => panic!("Expected Index command"),
    }
}

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.port, 8080);
    assert_eq!(config.embedding_threads, 2);
    assert_eq!(config.log_level, "info");
    assert!(config.watch_dirs.is_empty());
}

#[test]
fn test_config_load_from_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // Create a test config file
    let config_content = r#"
data_dir = "/tmp/amp-data"
watch_dirs = ["/home/user/repo1", "/home/user/repo2"]
embedding_threads = 4
port = 9000
log_level = "debug"
"#;

    fs::write(&config_path, config_content).unwrap();

    // Create a temp data dir with config.toml
    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();
    fs::write(data_dir.join("config.toml"), config_content).unwrap();

    // Create CLI with path pointing to temp dir
    let args = vec!["amp-rs", "serve", "--data-dir", data_dir.to_str().unwrap()];
    let cli = Cli::try_parse_from(args).unwrap();

    // Load config
    let config = Config::load(&cli).unwrap();

    assert_eq!(config.embedding_threads, 4);
    assert_eq!(config.port, 9000);
    assert_eq!(config.log_level, "debug");
    assert_eq!(config.watch_dirs.len(), 2);
}

#[test]
fn test_config_cli_overrides_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create a test config file
    let config_content = r#"
data_dir = "/tmp/amp-data"
watch_dirs = []
embedding_threads = 4
port = 9000
log_level = "debug"
"#;

    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();
    fs::write(data_dir.join("config.toml"), config_content).unwrap();

    // Create CLI with overrides
    let args = vec![
        "amp-rs",
        "serve",
        "--data-dir",
        data_dir.to_str().unwrap(),
        "--port",
        "8888",
        "--embedding-threads",
        "8",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    // Load config
    let config = Config::load(&cli).unwrap();

    // CLI overrides should win
    assert_eq!(config.embedding_threads, 8);
    assert_eq!(config.port, 8888);
    // log_level from file should be preserved
    assert_eq!(config.log_level, "debug");
}

#[test]
fn test_config_load_missing_file_uses_defaults() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path().join("nonexistent");

    // Config file doesn't exist
    let args = vec!["amp-rs", "serve", "--data-dir", data_dir.to_str().unwrap()];
    let cli = Cli::try_parse_from(args).unwrap();

    // Load config should return defaults
    let config = Config::load(&cli).unwrap();

    assert_eq!(config.port, 8080);
    assert_eq!(config.embedding_threads, 2);
    assert_eq!(config.log_level, "info");
}

#[test]
fn test_cli_parse_serve_multiple_options() {
    let args = vec![
        "amp-rs",
        "serve",
        "--port",
        "7000",
        "--embedding-threads",
        "3",
        "--watch-dirs",
        "/repo1",
        "--debug",
    ];
    let cli = Cli::try_parse_from(args).unwrap();

    match cli.command {
        Command::Serve {
            port,
            embedding_threads,
            watch_dirs,
            debug,
            ..
        } => {
            assert_eq!(port, 7000);
            assert_eq!(embedding_threads, 3);
            assert_eq!(watch_dirs.len(), 1);
            assert!(debug);
        }
        _ => panic!("Expected Serve command"),
    }
}

#[test]
fn test_config_serialize_deserialize() {
    let config = Config {
        data_dir: PathBuf::from("/tmp/data"),
        watch_dirs: vec![PathBuf::from("/repo1"), PathBuf::from("/repo2")],
        embedding_threads: 4,
        port: 9000,
        log_level: "debug".to_string(),
    };

    // Serialize
    let toml_str = toml::to_string(&config).unwrap();

    // Deserialize
    let config2: Config = toml::from_str(&toml_str).unwrap();

    assert_eq!(config.data_dir, config2.data_dir);
    assert_eq!(config.watch_dirs, config2.watch_dirs);
    assert_eq!(config.embedding_threads, config2.embedding_threads);
    assert_eq!(config.port, config2.port);
    assert_eq!(config.log_level, config2.log_level);
}
