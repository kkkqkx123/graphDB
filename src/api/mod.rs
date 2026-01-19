use anyhow::Result;
use std::sync::Arc;
use tokio::signal;

pub mod service;
pub mod session;

use crate::api::service::GraphService;
use crate::config::Config;
use crate::storage::MemoryStorage;
use crate::common::log::{Logger, FileWriter, LogLevel};

fn init_logger(config: &Config) -> Result<Arc<Logger>> {
    let log_level = match config.log_level.to_lowercase().as_str() {
        "trace" => LogLevel::Trace,
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };

    let file_writer = Arc::new(
        FileWriter::new(&config.log_file, log_level)?
            .with_max_file_size(config.max_log_file_size as u64)
            .with_max_files(config.max_log_files as u32)
    );

    let mut logger = Arc::new(Logger::new(log_level));
    if let Some(logger_mut) = Arc::get_mut(&mut logger) {
        logger_mut.add_writer(file_writer);
    }
    
    Ok(logger)
}

pub async fn start_service(config_path: String) -> Result<()> {
    println!("Initializing GraphDB service...");

    // Load configuration
    let config = match Config::load(&config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "Failed to load config from '{}': {}, using default config",
                config_path, e
            );
            Config::default()
        }
    };
    println!("Configuration loaded: {:?}", config);

    // Initialize logger
    let _logger = init_logger(&config)?;
    println!("Logger initialized: {}", config.log_file);

    // Initialize storage
    let storage = Arc::new(MemoryStorage::new()?);
    println!("Storage initialized (memory mode)");

    // Initialize graph service with session management and query execution
    let _graph_service = GraphService::<MemoryStorage>::new(config.clone(), storage);
    println!("Graph service initialized with session management");

    // Start HTTP server (placeholder)
    println!("Starting HTTP server on {}:{}", config.host, config.port);

    // Wait for shutdown signal
    shutdown_signal().await;

    println!("Shutting down GraphDB service...");
    Ok(())
}

pub async fn execute_query(query_str: &str) -> Result<()> {
    println!("Executing query: {}", query_str);

    // Initialize storage for this example
    let config = crate::config::Config::default();
    let _logger = init_logger(&config)?;
    let storage = Arc::new(MemoryStorage::new()?);

    // Initialize graph service for query execution
    let graph_service = GraphService::<MemoryStorage>::new(config, storage);

    // Create a temporary session for this execution
    let session = match graph_service
        .get_session_manager()
        .create_session("anonymous".to_string(), "127.0.0.1".to_string())
    {
        Ok(session) => session,
        Err(e) => {
            eprintln!("Failed to create session: {}", e);
            return Err(anyhow::anyhow!(e));
        }
    };

    let session_id = session.id();

    match graph_service.execute(session_id, query_str).await {
        Ok(result) => {
            println!("Query executed successfully: {}", result);
        }
        Err(e) => {
            eprintln!("Query execution error: {}", e);
        }
    }

    Ok(())
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Received shutdown signal");
}

// Additional API endpoints can be added here
// For example:
// - HTTP API for graph queries
// - Metrics and health check endpoints
// - Admin operations
