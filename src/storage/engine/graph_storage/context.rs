//! Graph Storage Context
//!
//! Contains all shared references and configuration for the graph storage engine.
//! This module provides a centralized context for all storage operations.

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::api::server::auth::UserStorage;
use crate::storage::engine::persistence_coordinator::PersistenceCoordinator;
use crate::storage::engine::PropertyGraph;
use crate::storage::extend::FulltextStorage;
use crate::storage::index::secondary::{IndexGcConfig, IndexGcManager};
use crate::storage::metadata::{
    ExtendedSchemaManager, IndexManager, SchemaManager,
};
use crate::transaction::context::TransactionContext;
use crate::transaction::undo_log::UndoLogManager;
use crate::transaction::version_manager::VersionManager;

use super::transaction_support::TransactionSupport;

#[derive(Clone)]
pub struct GraphStorageContext {
    pub graph: Arc<RwLock<PropertyGraph>>,
    pub schema_manager: Arc<SchemaManager>,
    pub extended_schema_manager: Arc<ExtendedSchemaManager>,
    pub index_metadata_manager: Arc<IndexManager>,
    pub version_manager: Arc<VersionManager>,
    pub user_storage: Arc<UserStorage>,
    pub current_txn_context: Arc<Mutex<Option<Arc<TransactionContext>>>>,
    pub persistence: Option<Arc<RwLock<PersistenceCoordinator>>>,
    pub index_gc_manager: Option<Arc<IndexGcManager>>,
    pub fulltext_storage: Option<Arc<FulltextStorage>>,
    pub work_dir: Option<PathBuf>,
    pub db_path: String,
    /// Transaction support for undo log management
    pub txn_support: Arc<Mutex<TransactionSupport>>,
    /// Undo log manager for transaction rollback
    pub undo_logs: Arc<Mutex<UndoLogManager>>,
}

impl GraphStorageContext {
    pub fn new() -> Self {
        let graph = Arc::new(RwLock::new(PropertyGraph::new()));
        let schema_manager = Arc::new(SchemaManager::new());
        let extended_schema_manager = Arc::new(ExtendedSchemaManager::new());
        let index_metadata_manager = Arc::new(IndexManager::new());
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        Self {
            graph,
            schema_manager,
            extended_schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(Mutex::new(None)),
            persistence: None,
            index_gc_manager: None,
            fulltext_storage: None,
            work_dir: None,
            db_path: String::new(),
            txn_support: Arc::new(Mutex::new(TransactionSupport::new())),
            undo_logs: Arc::new(Mutex::new(UndoLogManager::new())),
        }
    }

    pub fn new_with_path(path: PathBuf) -> Self {
        use crate::storage::engine::PersistenceConfig;

        let graph = Arc::new(RwLock::new(PropertyGraph::new()));
        let schema_manager = Arc::new(SchemaManager::new());
        let extended_schema_manager = Arc::new(ExtendedSchemaManager::new());
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
            extended_schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(Mutex::new(None)),
            persistence,
            index_gc_manager: None,
            fulltext_storage: None,
            work_dir: Some(path.clone()),
            db_path: path.to_string_lossy().to_string(),
            txn_support: Arc::new(Mutex::new(TransactionSupport::new())),
            undo_logs: Arc::new(Mutex::new(UndoLogManager::new())),
        }
    }

    pub fn new_with_persistence(
        path: PathBuf,
        config: crate::storage::engine::PersistenceConfig,
    ) -> crate::core::StorageResult<Self> {
        let graph = Arc::new(RwLock::new(PropertyGraph::new()));
        let schema_manager = Arc::new(SchemaManager::new());
        let extended_schema_manager = Arc::new(ExtendedSchemaManager::new());
        let index_metadata_manager = Arc::new(IndexManager::new());
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        let persistence = PersistenceCoordinator::new(config).map(|p| Arc::new(RwLock::new(p)))?;

        Ok(Self {
            graph,
            schema_manager,
            extended_schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            current_txn_context: Arc::new(Mutex::new(None)),
            persistence: Some(persistence),
            index_gc_manager: None,
            fulltext_storage: None,
            work_dir: Some(path.clone()),
            db_path: path.to_string_lossy().to_string(),
            txn_support: Arc::new(Mutex::new(TransactionSupport::new())),
            undo_logs: Arc::new(Mutex::new(UndoLogManager::new())),
        })
    }

    pub fn with_index_gc(mut self, config: IndexGcConfig) -> Self {
        let graph = self.graph.read();
        let index_data_manager = graph.index_data_manager().clone();
        drop(graph);

        let gc_manager = IndexGcManager::new(index_data_manager, self.version_manager.clone(), config);

        self.index_gc_manager = Some(Arc::new(gc_manager));
        self
    }

    pub fn with_fulltext_storage(mut self, fulltext: Arc<FulltextStorage>) -> Self {
        self.fulltext_storage = Some(fulltext);
        self
    }

    pub fn with_edge_property_cache(
        self,
        config: crate::storage::cache::EdgePropertyCacheConfig,
    ) -> Self {
        self.set_edge_property_cache(config);
        self
    }

    pub fn set_edge_property_cache(
        &self,
        config: crate::storage::cache::EdgePropertyCacheConfig,
    ) {
        let mut graph = self.graph.write();
        graph.set_edge_property_cache(config);
    }

    pub fn edge_cache_stats(
        &self,
    ) -> Option<std::sync::Arc<crate::storage::cache::EdgePropertyCacheStats>> {
        self.graph.read().edge_cache_stats()
    }

    pub fn get_read_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.current_txn_context.lock().clone() {
            txn_ctx.timestamp()
        } else {
            self.version_manager.read_timestamp()
        }
    }

    pub fn get_write_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.current_txn_context.lock().clone() {
            txn_ctx.timestamp()
        } else {
            self.version_manager.write_timestamp()
        }
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.current_txn_context.lock().clone()
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContext>>) {
        *self.current_txn_context.lock() = context;
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

    pub fn is_fulltext_enabled(&self) -> bool {
        self.fulltext_storage.is_some()
    }

    /// Begin a transaction
    pub fn begin_transaction(&self) {
        self.txn_support.lock().begin();
    }

    /// Commit the current transaction
    pub fn commit_transaction(&self) {
        self.txn_support.lock().commit();
    }

    /// Rollback the current transaction
    pub fn rollback_transaction(&self, ts: u32) -> crate::core::StorageResult<()> {
        let mut graph = self.graph.write();
        self.txn_support.lock().rollback(&mut graph, ts)
    }

    /// Check if in a transaction
    pub fn is_in_transaction(&self) -> bool {
        self.txn_support.lock().is_in_transaction()
    }

    /// Get undo log count
    pub fn undo_log_count(&self) -> usize {
        self.txn_support.lock().undo_log_count()
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
