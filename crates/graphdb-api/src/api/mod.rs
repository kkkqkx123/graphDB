//! The GraphDB API module
//!
//! Provide multiple access methods:
//! "core" refers to the core API, which is independent of the transport layer.
//! "server" refers to a network service API (HTTP).
//! "Embedded" refers to an API that is designed to be used on a standalone device (i.e., without the need for any additional servers or networks).

#[cfg(feature = "qdrant")]
use log::warn;
use log::{error, info};
use std::sync::Arc;

pub mod core;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "embedded")]
pub mod embedded;

// Convenient export options
pub use core::{
    CoreError, CoreResult, QueryApi, SchemaApi, SyncApi,
};

#[cfg(feature = "qdrant")]
pub use core::{VectorApi, VectorSearchResult};

#[cfg(feature = "server")]
pub use server::{session, HttpServer};

#[cfg(feature = "embedded")]
pub use embedded::GraphDatabase;

#[cfg(feature = "server")]
use crate::api::server::GraphService;
use crate::config::Config;
use crate::core::error::DBResult;
use crate::storage::engine::sync_wrapper::SyncWrapper;
use crate::storage::GraphStorage;
use crate::transaction::{TransactionManager, TransactionManagerConfig};

/// Start the service using the configuration file path (deprecated; please use start_service_with_config).
#[cfg(feature = "server")]
pub async fn start_service(config_path: String) -> DBResult<()> {
    let config = match Config::load(&config_path) {
        Ok(config) => config,
        Err(e) => {
            error!(
                "Failed to load config from '{}': {}, using default config",
                config_path, e
            );
            Config::default()
        }
    };
    start_service_with_config(config).await
}

/// Start the service using the configuration object.
#[cfg(feature = "server")]
pub async fn start_service_with_config(config: Config) -> DBResult<()> {
    info!("Initializing GraphDB service...");
    info!("Configuration loaded: {:?}", config);

    info!(
        "Log system has been initialized: {}/{}",
        config.log_dir(),
        config.log_file()
    );

    let inner_storage = Arc::new(GraphStorage::new()?);
    info!("Storage initialized (memory mode)");

    let storage = if config.fulltext.enabled || config.is_vector_enabled() {
        use crate::search::manager::FulltextIndexManager;
        use crate::search::FulltextConfig;
        use crate::sync::{SyncConfig, SyncManager};

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

            let sync_manager =
                SyncManager::with_sync_config(sync_coordinator.clone(), sync_config);

            #[cfg(feature = "qdrant")]
            let sync_manager = if config.is_vector_enabled() {
                use vector_client::VectorManager;
                match VectorManager::new(config.vector_config().clone()).await {
                    Ok(vm) => {
                        let vector_manager = Arc::new(vm);
                        let vector_coordinator =
                            Arc::new(crate::sync::vector_sync::VectorSyncCoordinator::new(
                                vector_manager,
                                None,
                            ));
                        info!("Vector index sync enabled");
                        sync_manager.with_vector_coordinator(vector_coordinator)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create VectorManager: {}. Vector search will be disabled.",
                            e
                        );
                        sync_manager
                    }
                }
            } else {
                sync_manager
            };

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

            let sync_manager =
                SyncManager::with_sync_config(sync_coordinator.clone(), sync_config);

            #[cfg(feature = "qdrant")]
            let sync_manager = if config.is_vector_enabled() {
                use vector_client::VectorManager;
                match VectorManager::new(config.vector_config().clone()).await {
                    Ok(vm) => {
                        let vector_manager = Arc::new(vm);
                        let vector_coordinator =
                            Arc::new(crate::sync::vector_sync::VectorSyncCoordinator::new(
                                vector_manager,
                                None,
                            ));
                        info!("Vector index sync enabled");
                        sync_manager.with_vector_coordinator(vector_coordinator)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create VectorManager: {}. Vector search will be disabled.",
                            e
                        );
                        sync_manager
                    }
                }
            } else {
                sync_manager
            };

            (sync_coordinator.clone(), Arc::new(sync_manager))
        };

        info!("SyncManager initialized");

        let sync_storage = SyncWrapper::with_sync_manager((*inner_storage).clone(), sync_manager);
        info!("Sync enabled for fulltext and vector indexes");

        Arc::new(sync_storage)
    } else {
        let sync_storage = SyncWrapper::new((*inner_storage).clone());
        Arc::new(sync_storage)
    };

    // Create a transaction manager
    let txn_config = TransactionManagerConfig {
        default_timeout: std::time::Duration::from_secs(config.transaction.default_timeout),
        max_concurrent_transactions: config.transaction.max_concurrent_transactions,
        auto_cleanup: true,
        write_lock_timeout: std::time::Duration::from_secs(10),
    };

    // Create shared StatsManager for all components (before TransactionManager to enable wiring)
    let slow_query_config = config.to_slow_query_config();
    let m = &config.monitoring;
    let stats_manager = Arc::new(
        crate::core::stats::StatsManager::with_slow_query_logger(
            m.enabled,
            m.memory_cache_size,
            m.slow_query_threshold_ms * 1000,
            slow_query_config,
        )
        .expect("Failed to create StatsManager with slow query logger"),
    );

    let transaction_manager = Arc::new(TransactionManager::with_stats_manager(
        txn_config,
        stats_manager.clone(),
    ));
    info!("Transaction manager initialized with StatsManager");

    // Create Tokio runtime for async initialization
    let graph_service =
        GraphService::<SyncWrapper<GraphStorage>>::new_with_transaction_manager_and_stats(
            config.clone(),
            storage.clone(),
            transaction_manager.clone(),
            stats_manager.clone(),
        )
        .await;
    info!("Graph service initialized with transaction management");

    // Inject StatsManager into FulltextIndexManager to enable search metrics
    if let Some(sync_api) = graph_service.sync_api() {
        let fulltext_manager = sync_api.sync_manager().fulltext_manager();
        let stats_manager = graph_service.get_stats_manager().clone();
        fulltext_manager.set_stats_manager(stats_manager);
        info!("StatsManager injected into FulltextIndexManager for search metrics");
    }

    // Create HTTP server
    let http_server = Arc::new(HttpServer::new(
        graph_service,
        Arc::new(parking_lot::RwLock::new((*storage).clone())),
        transaction_manager,
        &config,
    ));
    info!("HTTP server created");

    info!(
        "Starting HTTP server on {}:{}",
        config.host(),
        config.port()
    );

    // Start HTTP server
    if let Err(e) = start_http_server(http_server, &config).await {
        error!("HTTP server error: {}", e);
    }

    shutdown_signal().await;

    info!("Shutting down GraphDB service...");
    Ok(())
}

#[cfg(feature = "server")]
pub async fn execute_query(query_str: &str) -> DBResult<()> {
    info!("Executing query: {}", query_str);

    let config = crate::config::Config::default();
    let inner_storage = Arc::new(GraphStorage::new()?);

    let sync_storage = SyncWrapper::new((*inner_storage).clone());
    let storage = Arc::new(sync_storage);

    let graph_service =
        GraphService::<SyncWrapper<GraphStorage>>::new_for_test(config, storage).await;

    let session = match graph_service
        .get_session_manager()
        .create_session("anonymous".to_string(), "127.0.0.1".to_string())
        .await
    {
        Ok(session) => session,
        Err(e) => {
            error!("Failed to create session: {}", e);
            return Err(crate::core::error::DBError::from(
                crate::api::server::session::SessionError::manager_error(format!(
                    "Failed to create session: {}",
                    e
                )),
            ));
        }
    };

    let session_id = session.id();

    match graph_service.execute(session_id, query_str).await {
        Ok(result) => {
            info!("Query executed successfully: {:?}", result);
        }
        Err(e) => {
            error!("Query execution error: {}", e);
        }
    }

    Ok(())
}

/// Waiting for the shutdown signal (asynchronous implementation)
///
/// This function waits for the shutdown signal in an async context.
pub async fn shutdown_signal() {
    info!("Waiting for shutdown signal (Ctrl+C or SIGTERM)...");

    async_shutdown_signal().await;

    info!("Received shutdown signal");
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

/// Start both HTTP and gRPC servers concurrently.
#[cfg(all(feature = "server", feature = "grpc"))]
pub async fn start_http_and_grpc_servers<
    S: crate::storage::StorageClient + Clone + Send + Sync + 'static,
>(
    http_server: Arc<HttpServer<S>>,
    config: &Config,
) -> DBResult<()> {
    use axum::serve;
    use tokio::net::TcpListener;

    let http_state = crate::api::server::http::AppState::new(http_server.clone());

    // Create WebState for web management APIs
    let storage_path = format!("{}/metadata.db", config.storage_path());
    let web_router =
        match crate::api::server::web::WebState::new(&storage_path, http_state.clone()).await {
            Ok(web_state) => Some(crate::api::server::web::create_router(web_state)),
            Err(e) => {
                log::warn!(
                    "Failed to initialize web management: {}, continuing without it",
                    e
                );
                None
            }
        };

    let http_app = crate::api::server::http::router::create_router(http_state.clone(), web_router);

    // Setup gRPC address
    let grpc_addr = format!("{}:{}", config.host(), config.grpc_port())
        .parse::<std::net::SocketAddr>()
        .map_err(|e| crate::core::error::DBError::internal(e.to_string()))?;

    // Setup HTTP address
    let http_addr = format!("{}:{}", config.host(), config.port());

    info!("HTTP server listening on {}", http_addr);
    info!("gRPC server listening on {}", grpc_addr);

    // Clone state for gRPC server
    let grpc_state = http_state.clone();
    let grpc_config = config.clone();

    // Start HTTP server
    let http_future = async move {
        let http_listener = TcpListener::bind(&http_addr).await?;
        serve(http_listener, http_app)
            .with_graceful_shutdown(async_shutdown_signal())
            .await?;
        Ok::<(), crate::core::error::DBError>(())
    };

    // Start gRPC server
    let grpc_future = async move {
        crate::api::server::grpc::run_server(grpc_state, grpc_config, grpc_addr)
            .await
            .map_err(|e| crate::core::error::DBError::internal(e.to_string()))?;
        Ok::<(), crate::core::error::DBError>(())
    };

    // Run both servers concurrently
    tokio::select! {
        result = http_future => result?,
        result = grpc_future => result?,
    }

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
