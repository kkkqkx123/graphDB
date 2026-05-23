//! Graph Storage Context
//!
//! Contains all shared references and configuration for the graph storage engine.
//! This module provides a centralized context for all storage operations.

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::api::server::auth::UserStorage;
use crate::core::mvcc::VersionManager;
use crate::core::types::TransactionContextInfo;
use crate::storage::engine::persistence_coordinator::PersistenceCoordinator;
use crate::storage::engine::PropertyGraph;
use crate::storage::index::secondary::{IndexGcConfig, IndexGcManager};
use crate::storage::metadata::{IndexManager, SchemaManager};

#[derive(Clone)]
pub struct GraphStorageContext {
    pub graph: Arc<PropertyGraph>,
    pub schema_manager: Arc<SchemaManager>,
    pub index_metadata_manager: Arc<IndexManager>,
    pub version_manager: Arc<VersionManager>,
    pub user_storage: Arc<UserStorage>,
    pub current_txn_context: Arc<RwLock<Option<Arc<TransactionContextInfo>>>>,
    pub persistence: Option<Arc<RwLock<PersistenceCoordinator>>>,
    pub index_gc_manager: Option<Arc<IndexGcManager>>,
    pub work_dir: Option<PathBuf>,
    pub db_path: String,
}

impl GraphStorageContext {
    pub fn new() -> Self {
        let graph = Arc::new(PropertyGraph::new());
        let schema_manager = Arc::new(SchemaManager::new());
        let index_metadata_manager = Arc::new(IndexManager::new());
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        Self {
            graph,
            schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(RwLock::new(None)),
            persistence: None,
            index_gc_manager: None,
            work_dir: None,
            db_path: String::new(),
        }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        use crate::storage::engine::PersistenceConfig;

        let graph = Arc::new(PropertyGraph::new());
        let schema_manager = Arc::new(SchemaManager::new());
        let index_metadata_manager = Arc::new(IndexManager::new());
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        let persistence_config = PersistenceConfig {
            data_dir: path.join("data"),
            wal_dir: path.join("wal"),
            checkpoint_dir: path.join("checkpoint"),
            snapshot_dir: path.join("snapshots"),
            ..Default::default()
        };

        let persistence = PersistenceCoordinator::new(persistence_config)
            .map(|p| Arc::new(RwLock::new(p)))
            .ok();

        Self {
            graph,
            schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(RwLock::new(None)),
            persistence,
            index_gc_manager: None,
            work_dir: Some(path.clone()),
            db_path: path.to_string_lossy().to_string(),
        }
    }

    pub fn new_with_persistence(
        path: PathBuf,
        config: crate::storage::engine::PersistenceConfig,
    ) -> crate::core::StorageResult<Self> {
        let graph = Arc::new(PropertyGraph::new());
        let schema_manager = Arc::new(SchemaManager::new());
        let index_metadata_manager = Arc::new(IndexManager::new());
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        let persistence = PersistenceCoordinator::new(config).map(|p| Arc::new(RwLock::new(p)))?;

        Ok(Self {
            graph,
            schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(RwLock::new(None)),
            persistence: Some(persistence),
            index_gc_manager: None,
            work_dir: Some(path.clone()),
            db_path: path.to_string_lossy().to_string(),
        })
    }

    pub fn with_index_gc(mut self, config: IndexGcConfig) -> Self {
        let index_data_manager = self.graph.index_data_manager().read().clone();
        let gc_manager =
            IndexGcManager::new(index_data_manager, self.version_manager.clone(), config);

        self.index_gc_manager = Some(Arc::new(gc_manager));
        self
    }

    pub fn get_read_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.current_txn_context.write().clone() {
            txn_ctx.timestamp
        } else {
            self.version_manager.read_timestamp()
        }
    }

    pub fn get_write_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.current_txn_context.write().clone() {
            txn_ctx.timestamp
        } else {
            self.version_manager.write_timestamp()
        }
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContextInfo>> {
        self.current_txn_context.write().clone()
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContextInfo>>) {
        *self.current_txn_context.write() = context;
    }

    pub fn start_index_gc(&self) -> Option<std::thread::JoinHandle<()>> {
        self.index_gc_manager
            .as_ref()
            .map(|gc: &Arc<IndexGcManager>| gc.start_background_gc())
    }

    pub fn stop_index_gc(&self) {
        if let Some(ref gc) = self.index_gc_manager {
            gc.stop();
        }
    }

    pub fn is_index_gc_running(&self) -> bool {
        self.index_gc_manager
            .as_ref()
            .map(|g: &Arc<IndexGcManager>| g.is_running())
            .unwrap_or(false)
    }

    pub fn is_persistence_enabled(&self) -> bool {
        self.persistence.is_some()
    }

}

impl std::fmt::Debug for GraphStorageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStorageContext")
            .field("work_dir", &self.work_dir)
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl Default for GraphStorageContext {
    fn default() -> Self {
        Self::new()
    }
}
