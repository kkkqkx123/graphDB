//! Server startup functions
//!
//! Orchestrates storage, sync, transaction manager, and graph service initialization.

use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "qdrant")]
use log::warn;
#[cfg(feature = "qdrant")]
use vector_client::EmbeddingService;
use log::{error, info};

use crate::api::server::{GraphService, HttpServer};
use crate::config::Config;
use crate::core::error::DBResult;
use crate::storage::{GraphStorage, MetricsStorage, SyncWrapper};
use crate::transaction::{TransactionManager, TransactionManagerConfig};

/// Helper: attach vector sync coordinator to an existing SyncManager (if qdrant is enabled)
#[cfg(feature = "qdrant")]
async fn setup_vector_sync(
    sync_manager: crate::sync::SyncManager,
    config: &Config,
) -> crate::sync::SyncManager {
    if config.is_vector_enabled() {
        use vector_client::VectorManager;
        match VectorManager::new(config.vector_config().clone()).await {
            Ok(vm) => {
                let vector_manager = Arc::new(vm);
                let handle = tokio::runtime::Handle::current();
                // Create optional embedding service
                let embedding_service = config.vector_config().embedding.as_ref().map(|ec| {
                    EmbeddingService::from_config(ec.clone())
                        .map_err(|e| format!("Failed to create embedding service: {}", e))
                }).transpose();

                let embedding_service = match embedding_service {
                    Ok(es) => es.map(Arc::new),
                    Err(e) => {
                        warn!("Failed to create embedding service: {}", e);
                        None
                    }
                };
                
                let vector_coordinator = Arc::new(
                    crate::sync::vector_sync::VectorSyncCoordinator::new(vector_manager, embedding_service, handle),
                );
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
    }
}

/// Start the service using the configuration file path (deprecated; please use start_service_with_config).
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
pub async fn start_service_with_config(config: Config) -> DBResult<()> {
    info!("Initializing GraphDB service...");
    info!("Configuration loaded: {:?}", config);

    info!(
        "Log system has been initialized: {}/{}",
        config.log_dir(),
        config.log_file()
    );

    // Create shared StatsManager for all components before wiring storage decorators.
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

    let storage_path = PathBuf::from(config.storage_path());
    let inner_storage = Arc::new(MetricsStorage::new(
        GraphStorage::open(storage_path)?,
        stats_manager.clone(),
    ));
    info!(
        "Storage initialized (persistent mode at {}, metrics enabled)",
        config.storage_path()
    );

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

            let sync_manager = SyncManager::with_sync_config(sync_coordinator.clone(), sync_config);

            #[cfg(feature = "qdrant")]
            let sync_manager = setup_vector_sync(sync_manager, &config).await;

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

            let sync_manager = SyncManager::with_sync_config(sync_coordinator.clone(), sync_config);

            #[cfg(feature = "qdrant")]
            let sync_manager = setup_vector_sync(sync_manager, &config).await;

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

    let transaction_manager = Arc::new(TransactionManager::with_stats_manager(
        txn_config,
        stats_manager.clone(),
    ));
    info!("Transaction manager initialized with StatsManager");

    // Create Tokio runtime for async initialization
    let graph_service = GraphService::new_with_transaction_manager_and_stats(
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
    if let Err(e) = super::start_http_server(http_server, &config).await {
        error!("HTTP server error: {}", e);
    }

    super::shutdown_signal().await;

    info!("Shutting down GraphDB service...");
    Ok(())
}

/// Execute a single query directly (for CLI / quick testing).
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
