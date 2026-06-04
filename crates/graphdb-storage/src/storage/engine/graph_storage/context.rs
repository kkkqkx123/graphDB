//! Graph Storage Context
//!
//! Contains all shared references and configuration for the graph storage engine.
//! This module provides a centralized context for all storage operations.

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::metadata::{IndexManager, SchemaManager};
use crate::core::mvcc::VersionManager;
use crate::core::types::TransactionContextInfo;
use crate::core::UserStorage;
use crate::storage::engine::paths::StoragePaths;
use crate::storage::engine::persistence_coordinator::PersistenceCoordinator;
use crate::storage::engine::wal_manager::shared_local_wal_writer;
use crate::storage::engine::PropertyGraph;
use crate::storage::index::{IndexGcConfig, IndexGcManager};

#[derive(Clone)]
struct GraphStorageLayout {
    work_dir: Option<PathBuf>,
    db_path: String,
}

impl GraphStorageLayout {
    fn new() -> Self {
        Self {
            work_dir: None,
            db_path: String::new(),
        }
    }

    fn new_with_path(path: PathBuf) -> Self {
        Self {
            work_dir: Some(path.clone()),
            db_path: path.to_string_lossy().to_string(),
        }
    }

    fn work_dir(&self) -> &Option<PathBuf> {
        &self.work_dir
    }

    fn storage_paths(&self) -> Option<StoragePaths> {
        self.work_dir.as_ref().cloned().map(StoragePaths::new)
    }

    fn db_path(&self) -> &str {
        &self.db_path
    }
}

#[derive(Clone)]
struct GraphStoragePersistent {
    graph: Arc<PropertyGraph>,
    schema_manager: Arc<SchemaManager>,
    index_metadata_manager: Arc<IndexManager>,
    version_manager: Arc<VersionManager>,
    user_storage: Arc<UserStorage>,
    persistence: Option<Arc<RwLock<PersistenceCoordinator>>>,
    layout: GraphStorageLayout,
}

impl GraphStoragePersistent {
    fn build_core_components() -> (
        Arc<PropertyGraph>,
        Arc<SchemaManager>,
        Arc<IndexManager>,
        Arc<VersionManager>,
        Arc<UserStorage>,
    ) {
        (
            Arc::new(PropertyGraph::new()),
            Arc::new(SchemaManager::new()),
            Arc::new(IndexManager::new()),
            Arc::new(VersionManager::new()),
            Arc::new(UserStorage::new()),
        )
    }

    fn attach_persistence_wal(
        graph: &Arc<PropertyGraph>,
        persistence: &Arc<RwLock<PersistenceCoordinator>>,
    ) {
        let shared_writer = {
            let coordinator = persistence.read();
            let wal_manager = coordinator.wal_manager();
            let writer = wal_manager.read().writer();
            writer.map(shared_local_wal_writer)
        };

        if let Some(shared_writer) = shared_writer {
            graph.set_wal_writer(shared_writer);
        }
    }

    fn new() -> Self {
        let (graph, schema_manager, index_metadata_manager, version_manager, user_storage) =
            Self::build_core_components();

        Self {
            graph,
            schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            persistence: None,
            layout: GraphStorageLayout::new(),
        }
    }

    fn new_with_persistence(
        path: PathBuf,
        config: crate::storage::engine::PersistenceConfig,
    ) -> crate::core::StorageResult<Self> {
        let (graph, schema_manager, index_metadata_manager, version_manager, user_storage) =
            Self::build_core_components();

        let persistence = PersistenceCoordinator::new(config).map(|p| Arc::new(RwLock::new(p)))?;
        Self::attach_persistence_wal(&graph, &persistence);

        Ok(Self {
            graph,
            schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            persistence: Some(persistence),
            layout: GraphStorageLayout::new_with_path(path),
        })
    }

    fn graph(&self) -> &Arc<PropertyGraph> {
        &self.graph
    }

    fn schema_manager(&self) -> &Arc<SchemaManager> {
        &self.schema_manager
    }

    fn index_metadata_manager(&self) -> &Arc<IndexManager> {
        &self.index_metadata_manager
    }

    fn version_manager(&self) -> &Arc<VersionManager> {
        &self.version_manager
    }

    fn user_storage(&self) -> &Arc<UserStorage> {
        &self.user_storage
    }

    fn persistence(&self) -> &Option<Arc<RwLock<PersistenceCoordinator>>> {
        &self.persistence
    }

    fn work_dir(&self) -> &Option<PathBuf> {
        self.layout.work_dir()
    }

    fn storage_paths(&self) -> Option<StoragePaths> {
        self.layout.storage_paths()
    }

    fn db_path(&self) -> &str {
        self.layout.db_path()
    }

    fn is_persistence_enabled(&self) -> bool {
        self.persistence.is_some()
    }
}

#[derive(Clone, Default)]
struct GraphStorageRuntime {
    current_txn_context: Arc<RwLock<Option<Arc<TransactionContextInfo>>>>,
    index_gc_manager: Option<Arc<IndexGcManager>>,
}

impl GraphStorageRuntime {
    fn new() -> Self {
        Self {
            current_txn_context: Arc::new(RwLock::new(None)),
            index_gc_manager: None,
        }
    }

    fn with_index_gc(
        &self,
        graph: &Arc<PropertyGraph>,
        version_manager: &Arc<VersionManager>,
        config: IndexGcConfig,
    ) -> Self {
        let index_data_manager = graph.index_data_manager().read().clone();
        let gc_manager = IndexGcManager::new(index_data_manager, version_manager.clone(), config);

        Self {
            current_txn_context: self.current_txn_context.clone(),
            index_gc_manager: Some(Arc::new(gc_manager)),
        }
    }

    fn get_transaction_context(&self) -> Option<Arc<TransactionContextInfo>> {
        self.current_txn_context.read().clone()
    }

    fn set_transaction_context(&self, context: Option<Arc<TransactionContextInfo>>) {
        *self.current_txn_context.write() = context;
    }

    fn start_index_gc(&self) -> Option<std::thread::JoinHandle<()>> {
        self.index_gc_manager
            .as_ref()
            .map(|gc: &Arc<IndexGcManager>| gc.start_background_gc())
    }

    fn stop_index_gc(&self) {
        if let Some(ref gc) = self.index_gc_manager {
            gc.stop();
        }
    }

    fn is_index_gc_running(&self) -> bool {
        self.index_gc_manager
            .as_ref()
            .map(|g: &Arc<IndexGcManager>| g.is_running())
            .unwrap_or(false)
    }
}

#[derive(Clone)]
pub struct GraphStorageContext {
    persistent: GraphStoragePersistent,
    runtime: GraphStorageRuntime,
}

impl GraphStorageContext {
    pub fn new() -> Self {
        Self {
            persistent: GraphStoragePersistent::new(),
            runtime: GraphStorageRuntime::new(),
        }
    }

    pub fn new_with_path(path: PathBuf) -> crate::core::StorageResult<Self> {
        let config = crate::storage::engine::PersistenceConfig::for_work_dir(&path);
        Self::new_with_persistence(path, config)
    }

    pub fn new_with_persistence(
        path: PathBuf,
        config: crate::storage::engine::PersistenceConfig,
    ) -> crate::core::StorageResult<Self> {
        GraphStoragePersistent::new_with_persistence(path, config).map(|persistent| Self {
            persistent,
            runtime: GraphStorageRuntime::new(),
        })
    }

    pub fn with_index_gc(mut self, config: IndexGcConfig) -> Self {
        let runtime = self.runtime.with_index_gc(
            self.persistent.graph(),
            self.persistent.version_manager(),
            config,
        );
        self.runtime = runtime;
        self
    }

    pub(crate) fn graph(&self) -> &Arc<PropertyGraph> {
        self.persistent.graph()
    }

    pub(crate) fn schema_manager(&self) -> &Arc<SchemaManager> {
        self.persistent.schema_manager()
    }

    pub(crate) fn index_metadata_manager(&self) -> &Arc<IndexManager> {
        self.persistent.index_metadata_manager()
    }

    pub(crate) fn version_manager(&self) -> &Arc<VersionManager> {
        self.persistent.version_manager()
    }

    pub(crate) fn user_storage(&self) -> &Arc<UserStorage> {
        self.persistent.user_storage()
    }

    pub(crate) fn persistence(&self) -> &Option<Arc<RwLock<PersistenceCoordinator>>> {
        self.persistent.persistence()
    }

    pub(crate) fn work_dir(&self) -> &Option<PathBuf> {
        self.persistent.work_dir()
    }

    pub(crate) fn storage_paths(&self) -> Option<StoragePaths> {
        self.persistent.storage_paths()
    }

    pub(crate) fn db_path(&self) -> &str {
        self.persistent.db_path()
    }

    pub fn get_read_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.runtime.get_transaction_context() {
            txn_ctx.timestamp
        } else {
            self.persistent.version_manager.read_timestamp()
        }
    }

    pub fn get_write_timestamp(&self) -> u32 {
        if let Some(txn_ctx) = self.runtime.get_transaction_context() {
            txn_ctx.timestamp
        } else {
            self.persistent.version_manager.write_timestamp()
        }
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContextInfo>> {
        self.runtime.get_transaction_context()
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContextInfo>>) {
        self.runtime.set_transaction_context(context);
    }

    pub fn start_index_gc(&self) -> Option<std::thread::JoinHandle<()>> {
        self.runtime.start_index_gc()
    }

    pub fn stop_index_gc(&self) {
        self.runtime.stop_index_gc();
    }

    pub fn is_index_gc_running(&self) -> bool {
        self.runtime.is_index_gc_running()
    }

    pub fn is_persistence_enabled(&self) -> bool {
        self.persistent.is_persistence_enabled()
    }
}

impl std::fmt::Debug for GraphStorageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStorageContext")
            .field("work_dir", self.persistent.work_dir())
            .field("db_path", &self.persistent.db_path())
            .finish()
    }
}

impl Default for GraphStorageContext {
    fn default() -> Self {
        Self::new()
    }
}
