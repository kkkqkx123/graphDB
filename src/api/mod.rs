use anyhow::Result;
use std::sync::Arc;
use tokio::signal;

pub mod service;
pub mod session;

use crate::api::service::GraphService;
use crate::config::Config;
use crate::storage::NativeStorage;

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

    // Initialize storage
    let storage = Arc::new(NativeStorage::new(&config.storage_path)?);
    println!("Storage initialized at: {}", config.storage_path);

    // Initialize graph service with session management and query execution
    let _graph_service = GraphService::new(config.clone(), storage);
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
    let storage = Arc::new(NativeStorage::new(&config.storage_path)?);

    // Initialize graph service for the query execution
    let graph_service = GraphService::new(config, storage);

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
