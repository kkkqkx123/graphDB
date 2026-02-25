//! GraphDB API 模块
//!
//! 提供多种访问方式：
//! - `core` - 核心 API（与传输层无关）
//! - `server` - 网络服务 API（HTTP）
//! - `embedded` - 嵌入式 API（单机使用）

use anyhow::Result;
use log::info;
use std::sync::Arc;

pub mod core;
pub mod server;
pub mod embedded;

// 便捷导出
pub use core::{QueryApi, TransactionApi, SchemaApi, CoreError, CoreResult};
pub use server::{HttpServer, session};
pub use embedded::{GraphDb, EmbeddedConfig};

use crate::api::server::GraphService;
use crate::config::Config;
use crate::storage::redb_storage::DefaultStorage;
use crate::transaction::{TransactionManager, SavepointManager, TransactionManagerConfig};

/// 使用配置文件路径启动服务（已弃用，请使用 start_service_with_config）
pub fn start_service(config_path: String) -> Result<()> {
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
    start_service_with_config(config)
}

/// 使用配置对象启动服务
pub fn start_service_with_config(config: Config) -> Result<()> {
    println!("Initializing GraphDB service...");
    println!("Configuration loaded: {:?}", config);

    info!("日志系统已初始化: {}/{}", config.log_dir(), config.log_file());

    let storage = Arc::new(DefaultStorage::new()?);
    println!("Storage initialized (memory mode)");

    // 创建事务管理器
    let db = storage.get_db().clone();
    let txn_config = TransactionManagerConfig {
        default_timeout: std::time::Duration::from_secs(config.transaction.default_timeout),
        max_concurrent_transactions: config.transaction.max_concurrent_transactions,
        enable_2pc: config.transaction.enable_2pc,
        deadlock_detection_interval: std::time::Duration::from_secs(5),
        auto_cleanup: config.transaction.auto_cleanup,
        cleanup_interval: std::time::Duration::from_secs(config.transaction.cleanup_interval),
    };
    let transaction_manager = Arc::new(TransactionManager::new(db, txn_config));
    let savepoint_manager = Arc::new(SavepointManager::new());
    println!("Transaction managers initialized");

    let _graph_service = GraphService::<DefaultStorage>::new_with_transaction_managers(
        config.clone(),
        storage,
        transaction_manager,
        savepoint_manager,
    );
    println!("Graph service initialized with session and transaction management");

    println!("Starting HTTP server on {}:{}", config.host(), config.port());

    shutdown_signal();

    println!("Shutting down GraphDB service...");
    Ok(())
}

pub fn execute_query(query_str: &str) -> Result<()> {
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
            return Err(anyhow::anyhow!("Failed to create session: {}", e));
        }
    };

    let session_id = session.id();

    match graph_service.execute(session_id, query_str) {
        Ok(result) => {
            println!("Query executed successfully: {}", result);
        }
        Err(e) => {
            eprintln!("Query execution error: {}", e);
        }
    }

    Ok(())
}

/// 等待关闭信号（同步实现）
pub fn shutdown_signal() {
    use signal_hook::flag;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    
    let term = Arc::new(AtomicBool::new(false));
    flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))
        .expect("Failed to register SIGTERM handler");
    flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))
        .expect("Failed to register SIGINT handler");
    
    println!("Waiting for shutdown signal (Ctrl+C or SIGTERM)...");
    
    // 主循环等待信号
    while !term.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    println!("Received shutdown signal");
}

// Additional API endpoints can be added here
// For example:
// - HTTP API for graph queries
// - Metrics and health check endpoints
// - Admin operations
