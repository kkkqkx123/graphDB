//! The GraphDB API module
//!
//! Provide multiple access methods:
//! “core” refers to the core API, which is independent of the transport layer.
//! “server” refers to a network service API (HTTP).
//! “Embedded” refers to an API that is designed to be used on a standalone device (i.e., without the need for any additional servers or networks).

use log::info;
use std::sync::Arc;

pub mod core;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "embedded")]
pub mod embedded;

// Convenient export options
pub use core::{CoreError, CoreResult, QueryApi, SchemaApi};

#[cfg(feature = "server")]
pub use server::{session, HttpServer};

#[cfg(feature = "embedded")]
pub use embedded::GraphDatabase;

#[cfg(feature = "server")]
use crate::api::server::GraphService;
use crate::config::Config;
use crate::core::error::DBResult;
use crate::storage::redb_storage::DefaultStorage;
use crate::transaction::{TransactionManager, TransactionManagerConfig};

/// Start the service using the configuration file path (deprecated; please use start_service_with_config).
#[cfg(feature = "server")]
pub fn start_service(config_path: String) -> DBResult<()> {
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

/// Start the service using the configuration object.
#[cfg(feature = "server")]
pub fn start_service_with_config(config: Config) -> DBResult<()> {
    println!("Initializing GraphDB service...");
    println!("Configuration loaded: {:?}", config);

    info!(
        "Log system has been initialized: {}/{}",
        config.log_dir(),
        config.log_file()
    );

    let storage = Arc::new(DefaultStorage::new()?);
    println!("Storage initialized (memory mode)");

    // Create a transaction manager
    let db = storage.get_db().clone();
    let txn_config = TransactionManagerConfig {
        default_timeout: std::time::Duration::from_secs(config.transaction.default_timeout),
        max_concurrent_transactions: config.transaction.max_concurrent_transactions,
        auto_cleanup: true,
    };
    let transaction_manager = Arc::new(TransactionManager::new(db, txn_config));
    println!("Transaction manager initialized");

    let _graph_service = GraphService::<DefaultStorage>::new_with_transaction_manager(
        config.clone(),
        storage,
        transaction_manager,
    );
    println!("Graph service initialized with transaction management");

    println!(
        "Starting HTTP server on {}:{}",
        config.host(),
        config.port()
    );

    shutdown_signal();

    println!("Shutting down GraphDB service...");
    Ok(())
}

#[cfg(feature = "server")]
pub async fn execute_query(query_str: &str) -> DBResult<()> {
    println!("Executing query: {}", query_str);

    let config = crate::config::Config::default();
    let storage = Arc::new(DefaultStorage::new()?);

    let graph_service = GraphService::<DefaultStorage>::new(config, storage);

    let session = match graph_service
        .get_session_manager()
        .create_session("anonymous".to_string(), "127.0.0.1".to_string())
        .await
    {
        Ok(session) => session,
        Err(e) => {
            eprintln!("Failed to create session: {}", e);
            return Err(crate::core::error::DBError::Session(
                crate::core::error::SessionError::ManagerError(format!(
                    "Failed to create session: {}",
                    e
                )),
            ));
        }
    };

    let session_id = session.id();

    match graph_service.execute(session_id, query_str) {
        Ok(result) => {
            println!("Query executed successfully: {:?}", result);
        }
        Err(e) => {
            eprintln!("Query execution error: {}", e);
        }
    }

    Ok(())
}

/// Waiting for the shutdown signal (synchronous implementation)
///
/// This function is used externally when running asynchronously; it blocks the current thread in order to wait for the signal.
/// The internal implementation uses `tokio::signal`, which requires a brief initialization of the runtime.
pub fn shutdown_signal() {
    use tokio::runtime::Runtime;

    println!("Waiting for shutdown signal (Ctrl+C or SIGTERM)...");

    // Create a temporary runtime to wait for asynchronous signals.
    let rt = Runtime::new().expect("Failed to create temporary runtime");
    rt.block_on(async {
        async_shutdown_signal().await;
    });

    println!("Received shutdown signal");
}

// Additional API endpoints can be added here
// For example:
// - HTTP API for graph queries
// - Metrics and health check endpoints
// - Admin operations

/// Start an HTTP server using an asynchronous runtime.
#[cfg(feature = "server")]
pub async fn start_http_server<S: crate::storage::StorageClient + Clone + Send + Sync + 'static>(
    server: Arc<HttpServer<S>>,
    config: &Config,
) -> DBResult<()> {
    use axum::serve;
    use tokio::net::TcpListener;

    let state = crate::api::server::http::AppState::new(server.clone());

    // Create WebState for web management APIs
    let storage_path = format!("{}/metadata.db", config.storage_path());
    let web_state = crate::api::server::WebState::new(
        crate::api::server::http::AppState::new(server),
        &storage_path,
    )
    .await
    .ok();

    let app = crate::api::server::http::router::create_router(state, web_state);

    let addr = format!("{}:{}", config.host(), config.port());
    let listener = TcpListener::bind(&addr).await?;

    info!("HTTP server listening on {}", addr);

    serve(listener, app)
        .with_graceful_shutdown(async_shutdown_signal())
        .await?;

    Ok(())
}

/// Asynchronous shutdown signal
async fn async_shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received, starting graceful shutdown");
}
