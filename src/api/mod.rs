//! The GraphDB API module
//!
//! Provide multiple access methods:
//! "core" refers to the core API, which is independent of the transport layer.
//! "server" refers to a network service API (HTTP).
//! "Embedded" refers to an API that is designed to be used on a standalone device (i.e., without the need for any additional servers or networks).

use log::{info, error, warn};
use std::sync::Arc;

pub mod core;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "embedded")]
pub mod embedded;

// Convenient export options
pub use core::{
    CoreError, CoreResult, QueryApi, SchemaApi, SyncApi, VectorApi, VectorSearchResult,
};

#[cfg(feature = "server")]
pub use server::{session, HttpServer};

#[cfg(feature = "embedded")]
pub use embedded::GraphDatabase;

#[cfg(feature = "server")]
use crate::api::server::GraphService;
use crate::config::Config;
use crate::core::error::DBResult;
use crate::storage::api::StorageClient;
use crate::storage::entity::SyncStorage;
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
                match VectorManager::new(config.vector.clone()).await {
                    Ok(vm) => {
                        let vector_manager = Arc::new(vm);
                        let vector_coordinator = Arc::new(
                            crate::sync::vector_sync::VectorSyncCoordinator::new(vector_manager, None),
                        );
                        sync_manager = sync_manager.with_vector_coordinator(vector_coordinator);
                        info!("Vector index sync enabled");
                    }
                    Err(e) => {
                        warn!("Failed to create VectorManager: {}. Vector search will be disabled.", e);
                    }
                }
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
                match VectorManager::new(config.vector.clone()).await {
                    Ok(vm) => {
                        let vector_manager = Arc::new(vm);
                        let vector_coordinator = Arc::new(
                            crate::sync::vector_sync::VectorSyncCoordinator::new(vector_manager, None),
                        );
                        sync_manager = sync_manager.with_vector_coordinator(vector_coordinator);
                        info!("Vector index sync enabled");
                    }
                    Err(e) => {
                        warn!("Failed to create VectorManager: {}. Vector search will be disabled.", e);
                    }
                }
            }

            (sync_coordinator.clone(), Arc::new(sync_manager))
        };

        info!("SyncManager initialized");

        let sync_storage = SyncStorage::with_sync_manager((*inner_storage).clone(), sync_manager);
        info!("Sync enabled for fulltext and vector indexes");

        Arc::new(sync_storage)
    } else {
        let sync_storage = SyncStorage::new((*inner_storage).clone());
        Arc::new(sync_storage)
    };

    // Create a transaction manager
    let txn_config = TransactionManagerConfig {
        default_timeout: std::time::Duration::from_secs(config.transaction.default_timeout),
        max_concurrent_transactions: config.transaction.max_concurrent_transactions,
        auto_cleanup: true,
        write_lock_timeout: std::time::Duration::from_secs(10),
    };
    let transaction_manager = Arc::new(TransactionManager::new(txn_config));
    info!("Transaction manager initialized");

    // Create Tokio runtime for async initialization
    // Initialize telemetry recorder and set as global
    let telemetry_recorder = Arc::new(crate::api::core::telemetry::TelemetryRecorder::new());
    if let Err(e) = crate::api::core::telemetry::set_global_recorder((*telemetry_recorder).clone())
    {
        error!("Failed to set global telemetry recorder: {}", e);
    } else {
        info!("Telemetry recorder initialized");
    }

    // Start telemetry server if enabled
    let _telemetry_handle = if config.server.telemetry.enabled {
        let telemetry_config = crate::api::server::telemetry_server::TelemetryConfig {
            bind_address: config.server.telemetry.bind_address.clone(),
            port: config.server.telemetry.port,
            max_histogram_entries: config.server.telemetry.max_histogram_entries,
            cleanup_interval_secs: config.server.telemetry.cleanup_interval_secs,
        };
        let telemetry_server = crate::api::server::telemetry_server::TelemetryServer::new(
            telemetry_config,
            telemetry_recorder.clone(),
        );
        info!(
            "Starting telemetry server on {}:{}",
            config.server.telemetry.bind_address, config.server.telemetry.port
        );
        Some(telemetry_server.spawn())
    } else {
        info!("Telemetry server disabled");
        None
    };

    let graph_service =
        GraphService::<SyncStorage<GraphStorage>>::new_with_transaction_manager(
            config.clone(),
            storage.clone(),
            transaction_manager.clone(),
        )
        .await;
    info!("Graph service initialized with transaction management");

    // Create HTTP server
    let http_server = Arc::new(HttpServer::new(
        graph_service,
        Arc::new(parking_lot::Mutex::new((*storage).clone())),
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

    // Initialize storage (simplified version without fulltext index)
    let sync_storage = SyncStorage::new((*inner_storage).clone());
    let storage = Arc::new(sync_storage);

    let graph_service =
        GraphService::<SyncStorage<GraphStorage>>::new_for_test(config, storage).await;

    let session = match graph_service
        .get_session_manager()
        .create_session("anonymous".to_string(), "127.0.0.1".to_string())
        .await
    {
        Ok(session) => session,
        Err(e) => {
            error!("Failed to create session: {}", e);
            return Err(crate::core::error::DBError::Session(
                crate::core::error::SessionError::ManagerError(format!(
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
        .map_err(|e| crate::core::error::DBError::Internal(e.to_string()))?;

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
            .map_err(|e| crate::core::error::DBError::Internal(e.to_string()))?;
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
