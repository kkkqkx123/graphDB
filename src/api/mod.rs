use anyhow::Result;
use log::info;
use std::sync::Arc;
use tokio::signal;

pub mod service;
pub mod session;

use crate::api::service::GraphService;
use crate::config::Config;
use crate::storage::redb_storage::DefaultStorage;

/// 使用配置文件路径启动服务（已弃用，请使用 start_service_with_config）
pub async fn start_service(config_path: String) -> Result<()> {
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
    start_service_with_config(config).await
}

/// 使用配置对象启动服务
pub async fn start_service_with_config(config: Config) -> Result<()> {
    println!("Initializing GraphDB service...");
    println!("Configuration loaded: {:?}", config);

    info!("日志系统已初始化: {}/{}", config.log_dir, config.log_file);

    let storage = Arc::new(DefaultStorage::new()?);
    println!("Storage initialized (memory mode)");

    let _graph_service = GraphService::<DefaultStorage>::new(config.clone(), storage);
    println!("Graph service initialized with session management");

    println!("Starting HTTP server on {}:{}", config.host, config.port);

    shutdown_signal().await;

    println!("Shutting down GraphDB service...");
    Ok(())
}

pub async fn execute_query(query_str: &str) -> Result<()> {
    println!("Executing query: {}", query_str);

    let config = crate::config::Config::default();
    let storage = Arc::new(DefaultStorage::new()?);

    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

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
