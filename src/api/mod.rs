//! The GraphDB API module
//!
//! Provide multiple access methods:
//! “core” refers to the core API, which is independent of the transport layer.
//! “server” refers to a network service API (HTTP).
//! “Embedded” refers to an API that is designed to be used on a standalone device (i.e., without the need for any additional servers or networks).

use log::info;
use std::sync::Arc;

use crate::utils::output;

pub mod core;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "embedded")]
pub mod embedded;

// Convenient export options
pub use core::{CoreError, CoreResult, QueryApi, SchemaApi, SyncApi, VectorApi, VectorSearchResult};

#[cfg(feature = "server")]
pub use server::{session, HttpServer};

#[cfg(feature = "embedded")]
pub use embedded::GraphDatabase;

#[cfg(feature = "server")]
use crate::api::server::GraphService;
use crate::config::Config;
use crate::core::error::DBResult;
use crate::storage::engine::DefaultStorage;
use crate::storage::entity::SyncStorage;
use crate::transaction::{TransactionManager, TransactionManagerConfig};

/// Start the service using the configuration file path (deprecated; please use start_service_with_config).
#[cfg(feature = "server")]
pub fn start_service(config_path: String) -> DBResult<()> {
    let config = match Config::load(&config_path) {
        Ok(config) => config,
        Err(e) => {
            let _ = output::print_error(&format!(
                "Failed to load config from '{}': {}, using default config",
                config_path, e
            ));
            Config::default()
        }
    };
    start_service_with_config(config)
}

/// Start the service using the configuration object.
#[cfg(feature = "server")]
pub fn start_service_with_config(config: Config) -> DBResult<()> {
    let _ = output::print_info("Initializing GraphDB service...");
    let _ = output::print_info(&format!("Configuration loaded: {:?}", config));

    info!(
        "Log system has been initialized: {}/{}",
        config.log_dir(),
        config.log_file()
    );

    let inner_storage = Arc::new(DefaultStorage::new()?);
    let _ = output::print_success("Storage initialized (memory mode)");

    // 如果配置启用了全文索引或向量索引，初始化 SyncManager
    let storage = if config.fulltext.enabled || config.vector.enabled {
        use crate::search::manager::FulltextIndexManager;
        use crate::search::FulltextConfig;
        use crate::sync::{SyncConfig, SyncManager};
        use vector_client::VectorManager;

        let (_coordinator, sync_manager) = if config.fulltext.enabled {
            let manager = Arc::new(
                FulltextIndexManager::new(config.fulltext.clone())
                    .expect("Failed to create FulltextIndexManager"),
            );

            use crate::search::SyncFailurePolicy;

            let sync_config = SyncConfig {
                queue_size: 10000,
                commit_interval_ms: 1000,
                batch_size: 100,
                failure_policy: SyncFailurePolicy::FailOpen,
            };

            let batch_config = crate::sync::batch::BatchConfig::from(sync_config.clone());
            let sync_coordinator = Arc::new(crate::sync::coordinator::SyncCoordinator::new(
                manager.clone(),
                batch_config,
            ));

            let mut sync_manager =
                SyncManager::with_sync_config(sync_coordinator.clone(), sync_config);

            if config.vector.enabled {
                let rt = tokio::runtime::Handle::current();
                let vector_manager = Arc::new(
                    rt.block_on(VectorManager::new(config.vector.clone()))
                        .expect("Failed to create VectorManager"),
                );
                let vector_coordinator = Arc::new(
                    crate::sync::vector_sync::VectorSyncCoordinator::new(vector_manager, None),
                );
                sync_manager = sync_manager.with_vector_coordinator(vector_coordinator);
                let _ = output::print_info("Vector index sync enabled");
            }

            (sync_coordinator.clone(), Arc::new(sync_manager))
        } else {
            let manager = Arc::new(
                FulltextIndexManager::new(FulltextConfig::default())
                    .expect("Failed to create FulltextIndexManager"),
            );

            let sync_config = SyncConfig::default();
            let batch_config = crate::sync::batch::BatchConfig::from(sync_config.clone());
            let sync_coordinator = Arc::new(crate::sync::coordinator::SyncCoordinator::new(
                manager.clone(),
                batch_config,
            ));

            let mut sync_manager =
                SyncManager::with_sync_config(sync_coordinator.clone(), sync_config);

            if config.vector.enabled {
                let rt = tokio::runtime::Handle::current();
                let vector_manager = Arc::new(
                    rt.block_on(VectorManager::new(config.vector.clone()))
                        .expect("Failed to create VectorManager"),
                );
                let vector_coordinator = Arc::new(
                    crate::sync::vector_sync::VectorSyncCoordinator::new(vector_manager, None),
                );
                sync_manager = sync_manager.with_vector_coordinator(vector_coordinator);
                let _ = output::print_info("Vector index sync enabled");
            }

            (sync_coordinator.clone(), Arc::new(sync_manager))
        };

        let _ = output::print_success("SyncManager initialized");

        let sync_storage = SyncStorage::with_sync_manager((*inner_storage).clone(), sync_manager);
        let _ = output::print_success("Sync enabled for fulltext and vector indexes");

        Arc::new(sync_storage)
    } else {
        let sync_storage = SyncStorage::new((*inner_storage).clone());
        Arc::new(sync_storage)
    };

    // Create a transaction manager
    let db = storage.inner().get_db().clone();
    let txn_config = TransactionManagerConfig {
        default_timeout: std::time::Duration::from_secs(config.transaction.default_timeout),
        max_concurrent_transactions: config.transaction.max_concurrent_transactions,
        auto_cleanup: true,
    };
    let transaction_manager = Arc::new(TransactionManager::new(db, txn_config));
    let _ = output::print_success("Transaction manager initialized");

    // Create Tokio runtime for async initialization
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let graph_service =
            GraphService::<SyncStorage<DefaultStorage>>::new_with_transaction_manager(
                config.clone(),
                storage.clone(),
                transaction_manager.clone(),
            )
            .await;
        let _ = output::print_success("Graph service initialized with transaction management");

        // Create HTTP server
        let http_server = Arc::new(HttpServer::new(
            graph_service,
            Arc::new(parking_lot::Mutex::new((*storage).clone())),
            transaction_manager,
            &config,
        ));
        let _ = output::print_info("HTTP server created");

        let _ = output::print_info(&format!(
            "Starting HTTP server on {}:{}",
            config.host(),
            config.port()
        ));

        // Start HTTP server
        if let Err(e) = start_http_server(http_server, &config).await {
            let _ = output::print_error(&format!("HTTP server error: {}", e));
        }
    });

    shutdown_signal();

    let _ = output::print_info("Shutting down GraphDB service...");
    Ok(())
}

#[cfg(feature = "server")]
pub async fn execute_query(query_str: &str) -> DBResult<()> {
    let _ = output::print_info(&format!("Executing query: {}", query_str));

    let config = crate::config::Config::default();
    let inner_storage = Arc::new(DefaultStorage::new()?);

    // 初始化存储（简化版本，不启用全文索引）
    let sync_storage = SyncStorage::new((*inner_storage).clone());
    let storage = Arc::new(sync_storage);

    let graph_service =
        GraphService::<SyncStorage<DefaultStorage>>::new_for_test(config, storage).await;

    let session = match graph_service
        .get_session_manager()
        .create_session("anonymous".to_string(), "127.0.0.1".to_string())
        .await
    {
        Ok(session) => session,
        Err(e) => {
            let _ = output::print_error(&format!("Failed to create session: {}", e));
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
            let _ = output::print_success(&format!("Query executed successfully: {:?}", result));
        }
        Err(e) => {
            let _ = output::print_error(&format!("Query execution error: {}", e));
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

    let _ = output::print_info("Waiting for shutdown signal (Ctrl+C or SIGTERM)...");

    // Create a temporary runtime to wait for asynchronous signals.
    let rt = Runtime::new().expect("Failed to create temporary runtime");
    rt.block_on(async {
        async_shutdown_signal().await;
    });

    let _ = output::print_info("Received shutdown signal");
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
    let web_router =
        match crate::api::server::web::WebState::new(&storage_path, state.clone()).await {
            Ok(web_state) => Some(crate::api::server::web::create_router(web_state)),
            Err(e) => {
                log::warn!(
                    "Failed to initialize web management: {}, continuing without it",
                    e
                );
                None
            }
        };

    let app = crate::api::server::http::router::create_router(state, web_router);

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
