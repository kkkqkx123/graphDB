use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};
use serde::Serialize;

use crate::core::metadata::{IndexManager, SchemaManager};
use crate::core::mvcc::VersionManager;
use crate::core::types::{
    CompactConfig, LabelId, TableId, TableTracker, TableTrackerConfig, Timestamp,
    TransactionContextInfo, VertexId,
};
use crate::core::stats::StatsManager;
use crate::core::UserStorage;
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::edge::EdgeTable;
use crate::storage::edge::{EdgeRecord, EdgeStrategy, ExportedEdgeSnapshot};
use crate::storage::engine::background_freeze::{BackgroundFreezeConfig, BackgroundFreezeManager, FreezeStats};
use crate::storage::engine::cache_manager::CacheManager;
use crate::storage::engine::config::PropertyGraphConfig;
use crate::storage::engine::data_store::{EdgeTableKey, GraphDataStore};
use crate::storage::engine::params::CreateEdgeTypeParams;
use crate::storage::engine::paths::StoragePaths;
use crate::storage::engine::persistence_coordinator::PersistenceCoordinator;
use crate::storage::engine::{EdgeOperationParams, InsertEdgeParams};
use crate::storage::index::{
    GcStats, IndexDataManagerImpl, IndexGcConfig, IndexGcManager, IndexGcOps,
};
use crate::storage::types::StoragePropertyDef;
use crate::storage::vertex::{IdKey, VertexRecord};

type LastCompactedVertices = Arc<Mutex<Vec<(LabelId, Vec<IdKey>)>>>;
type CoreComponents = (
    Arc<GraphDataStore>,
    Arc<CacheManager>,
    Arc<TableTracker>,
    Arc<AtomicBool>,
    LastCompactedVertices,
    Arc<RwLock<IndexDataManagerImpl>>,
    Arc<SchemaManager>,
    Arc<IndexManager>,
    Arc<VersionManager>,
    Arc<UserStorage>,
);
use crate::transaction::wal::types::WalOpType;

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
    data_store: Arc<GraphDataStore>,
    cache_manager: Arc<CacheManager>,
    table_tracker: Arc<TableTracker>,
    config: PropertyGraphConfig,
    is_open: Arc<AtomicBool>,
    last_compacted_vertices: LastCompactedVertices,
    index_data_manager: Arc<RwLock<IndexDataManagerImpl>>,
    schema_manager: Arc<SchemaManager>,
    index_metadata_manager: Arc<IndexManager>,
    version_manager: Arc<VersionManager>,
    user_storage: Arc<UserStorage>,
    persistence: Option<Arc<RwLock<PersistenceCoordinator>>>,
    layout: GraphStorageLayout,
    stats_manager: Option<Arc<StatsManager>>,
}

impl GraphStoragePersistent {
    fn build_core_components() -> CoreComponents {
        let config = PropertyGraphConfig::default();
        let cache_manager = Arc::new(CacheManager::new(config.enable_cache, config.cache_memory));
        let table_tracker = Arc::new(TableTracker::with_config(TableTrackerConfig {
            flush_threshold: config.flush_config.flush_threshold,
            flush_interval: config.flush_config.flush_interval,
        }));

        (
            Arc::new(GraphDataStore::new()),
            cache_manager,
            table_tracker,
            Arc::new(AtomicBool::new(true)),
            Arc::new(Mutex::new(Vec::new())),
            Arc::new(RwLock::new(IndexDataManagerImpl::new())),
            Arc::new(SchemaManager::new()),
            Arc::new(IndexManager::new()),
            Arc::new(VersionManager::new()),
            Arc::new(UserStorage::new()),
        )
    }

    fn new_with_config(config: PropertyGraphConfig) -> Self {
        let cache_manager = CacheManager::new(config.enable_cache, config.cache_memory);
        let table_tracker = Arc::new(TableTracker::with_config(TableTrackerConfig {
            flush_threshold: config.flush_config.flush_threshold,
            flush_interval: config.flush_config.flush_interval,
        }));

        Self {
            data_store: Arc::new(GraphDataStore::new()),
            cache_manager: Arc::new(cache_manager),
            table_tracker,
            config,
            is_open: Arc::new(AtomicBool::new(true)),
            last_compacted_vertices: Arc::new(Mutex::new(Vec::new())),
            index_data_manager: Arc::new(RwLock::new(IndexDataManagerImpl::new())),
            schema_manager: Arc::new(SchemaManager::new()),
            index_metadata_manager: Arc::new(IndexManager::new()),
            version_manager: Arc::new(VersionManager::new()),
            user_storage: Arc::new(UserStorage::new()),
            persistence: None,
            layout: GraphStorageLayout::new(),
            stats_manager: None,
        }
    }

    fn new() -> Self {
        Self::new_with_config(PropertyGraphConfig::default())
    }

    fn new_with_persistence(
        path: PathBuf,
        config: crate::storage::engine::PersistenceConfig,
    ) -> crate::core::StorageResult<Self> {
        let (
            data_store,
            cache_manager,
            table_tracker,
            is_open,
            last_compacted_vertices,
            index_data_manager,
            schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
        ) = Self::build_core_components();

        let persistence = PersistenceCoordinator::new(config).map(|p| Arc::new(RwLock::new(p)))?;

        Ok(Self {
            data_store,
            cache_manager,
            table_tracker,
            config: PropertyGraphConfig::default(),
            is_open,
            last_compacted_vertices,
            index_data_manager,
            schema_manager,
            index_metadata_manager,
            version_manager,
            user_storage,
            persistence: Some(persistence),
            layout: GraphStorageLayout::new_with_path(path),
            stats_manager: None,
        })
    }
}

#[derive(Clone)]
/// Deferred WAL operations for two-phase recovery.
/// Used to handle edge operations that depend on vertex existence.
struct DeferredWalOps {
    /// Deferred edge insertions (InsertEdgeRedo, Timestamp)
    edges: Arc<Mutex<Vec<(crate::transaction::wal::types::InsertEdgeRedo, Timestamp)>>>,
    /// Deferred edge deletions (DeleteEdgeRedo, Timestamp)
    deletes: Arc<Mutex<Vec<(crate::transaction::wal::types::DeleteEdgeRedo, Timestamp)>>>,
}

impl DeferredWalOps {
    fn new() -> Self {
        Self {
            edges: Arc::new(Mutex::new(Vec::new())),
            deletes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn push_edge(&self, edge: crate::transaction::wal::types::InsertEdgeRedo, ts: Timestamp) {
        self.edges.lock().push((edge, ts));
    }

    fn push_delete(&self, delete: crate::transaction::wal::types::DeleteEdgeRedo, ts: Timestamp) {
        self.deletes.lock().push((delete, ts));
    }

    fn drain_edges(&self) -> Vec<(crate::transaction::wal::types::InsertEdgeRedo, Timestamp)> {
        self.edges.lock().drain(..).collect()
    }

    fn drain_deletes(&self) -> Vec<(crate::transaction::wal::types::DeleteEdgeRedo, Timestamp)> {
        self.deletes.lock().drain(..).collect()
    }

}

#[derive(Clone)]
struct GraphStorageRuntime {
    current_txn_context: Arc<RwLock<Option<Arc<TransactionContextInfo>>>>,
    index_gc_manager: Option<Arc<IndexGcManager>>,
    background_freeze_manager: Option<Arc<BackgroundFreezeManager>>,
    deferred_wal_ops: DeferredWalOps,
}

impl GraphStorageRuntime {
    fn new() -> Self {
        Self {
            current_txn_context: Arc::new(RwLock::new(None)),
            index_gc_manager: None,
            background_freeze_manager: None,
            deferred_wal_ops: DeferredWalOps::new(),
        }
    }

    fn with_index_gc(
        &self,
        index_data_manager: &Arc<RwLock<IndexDataManagerImpl>>,
        version_manager: &Arc<VersionManager>,
        config: IndexGcConfig,
    ) -> Self {
        let index_data = index_data_manager.read().clone();
        let gc_manager = IndexGcManager::new(index_data, version_manager.clone(), config);

        Self {
            current_txn_context: self.current_txn_context.clone(),
            index_gc_manager: Some(Arc::new(gc_manager)),
            background_freeze_manager: self.background_freeze_manager.clone(),
            deferred_wal_ops: self.deferred_wal_ops.clone(),
        }
    }

    fn with_background_freeze(
        &self,
        manager: Arc<BackgroundFreezeManager>,
    ) -> Self {
        Self {
            current_txn_context: self.current_txn_context.clone(),
            index_gc_manager: self.index_gc_manager.clone(),
            background_freeze_manager: Some(manager),
            deferred_wal_ops: self.deferred_wal_ops.clone(),
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

/// Context for resolving edge table keys with label lookup.
#[derive(Clone, Copy)]
struct EdgeLabelLookupCtx<'a> {
    vertex_tables: &'a HashMap<LabelId, crate::storage::vertex::VertexTable>,
    src_id: &'a VertexId,
    src_label: LabelId,
    dst_id: &'a VertexId,
    dst_label: LabelId,
    edge_label: LabelId,
    ts: Timestamp,
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
            &self.persistent.index_data_manager,
            &self.persistent.version_manager,
            config,
        );
        self.runtime = runtime;
        self
    }

    pub fn with_background_freeze(
        &self,
        manager: Arc<BackgroundFreezeManager>,
    ) -> Self {
        let runtime = self.runtime.with_background_freeze(manager);
        Self {
            persistent: self.persistent.clone(),
            runtime,
        }
    }

    /// Set the StatsManager for recording MVCC metrics to EdgeTable instances.
    ///
    /// This should be called once after creating the GraphStorageContext,
    /// typically at startup time. The stats manager will be injected into all
    /// EdgeTable instances for automatic metrics recording.
    pub fn set_stats_manager(&mut self, stats: Arc<StatsManager>) {
        self.persistent.stats_manager = Some(stats);
    }

    // ── Internal accessors for sub-modules ──

    pub(crate) fn data_store(&self) -> &GraphDataStore {
        &self.persistent.data_store
    }

    pub(crate) fn data_store_arc(&self) -> Arc<GraphDataStore> {
        Arc::clone(&self.persistent.data_store)
    }

    pub(crate) fn freeze_config(&self) -> &BackgroundFreezeConfig {
        &self.persistent.config.background_freeze
    }

    pub(crate) fn append_wal_redo<T: Serialize>(
        &self,
        op_type: WalOpType,
        timestamp: Timestamp,
        redo: &T,
    ) -> StorageResult<()> {
        if let Some(persistence) = self.persistent.persistence.as_ref() {
            let wal_manager = {
                let coordinator = persistence.read();
                coordinator.wal_manager()
            };
            if let Some(wal) = wal_manager {
                return wal.read().append_redo(op_type, timestamp, redo);
            }
        }

        Ok(())
    }

    /// Defer an edge insertion for phase-2 recovery.
    pub(crate) fn defer_edge_insert(
        &self,
        edge: crate::transaction::wal::types::InsertEdgeRedo,
        ts: Timestamp,
    ) {
        self.runtime.deferred_wal_ops.push_edge(edge, ts);
    }

    /// Defer an edge deletion for phase-2 recovery.
    pub(crate) fn defer_edge_delete(
        &self,
        delete: crate::transaction::wal::types::DeleteEdgeRedo,
        ts: Timestamp,
    ) {
        self.runtime.deferred_wal_ops.push_delete(delete, ts);
    }

    /// Get all deferred edge insertions for phase-2 recovery.
    pub(crate) fn take_deferred_edge_inserts(
        &self,
    ) -> Vec<(crate::transaction::wal::types::InsertEdgeRedo, Timestamp)> {
        self.runtime.deferred_wal_ops.drain_edges()
    }

    /// Get all deferred edge deletions for phase-2 recovery.
    pub(crate) fn take_deferred_edge_deletes(
        &self,
    ) -> Vec<(crate::transaction::wal::types::DeleteEdgeRedo, Timestamp)> {
        self.runtime.deferred_wal_ops.drain_deletes()
    }


    pub(crate) fn is_open_flag(&self) -> &AtomicBool {
        &self.persistent.is_open
    }

    pub(crate) fn index_data_manager(&self) -> &RwLock<IndexDataManagerImpl> {
        &self.persistent.index_data_manager
    }

    pub(crate) fn schema_manager(&self) -> &Arc<SchemaManager> {
        &self.persistent.schema_manager
    }

    pub(crate) fn index_metadata_manager(&self) -> &Arc<IndexManager> {
        &self.persistent.index_metadata_manager
    }

    pub(crate) fn version_manager(&self) -> &Arc<VersionManager> {
        &self.persistent.version_manager
    }

    pub(crate) fn user_storage(&self) -> &Arc<UserStorage> {
        &self.persistent.user_storage
    }

    pub(crate) fn persistence(&self) -> &Option<Arc<RwLock<PersistenceCoordinator>>> {
        &self.persistent.persistence
    }

    pub(crate) fn stats_manager(&self) -> Option<&Arc<StatsManager>> {
        self.persistent.stats_manager.as_ref()
    }

    pub(crate) fn work_dir(&self) -> &Option<PathBuf> {
        self.persistent.layout.work_dir()
    }

    pub(crate) fn storage_paths(&self) -> Option<StoragePaths> {
        self.persistent.layout.storage_paths()
    }

    pub(crate) fn db_path(&self) -> &str {
        self.persistent.layout.db_path()
    }

    pub(crate) fn is_persistence_enabled(&self) -> bool {
        self.persistent.persistence.is_some()
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
            self.persistent.version_manager.next_write_timestamp()
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

    // ── PropertyGraph-compatible API ──

    pub fn mark_vertex_modified(&self, label: LabelId) {
        self.persistent
            .table_tracker
            .mark_modified(TableId::vertex(label));
    }

    pub fn mark_edge_modified(&self, label: LabelId) {
        self.persistent
            .table_tracker
            .mark_modified(TableId::edge(label));
    }

    pub(crate) fn storage_size(&self) -> usize {
        let mut total = 0usize;
        {
            let vertex_tables = self.persistent.data_store.vertex_tables().read();
            for table in vertex_tables.values() {
                total += table.memory_size();
            }
        }
        {
            let edge_tables = self.persistent.data_store.edge_tables().read();
            for table in edge_tables.values() {
                total += table.memory_size();
            }
        }
        total
    }

    pub(crate) fn used_storage_size(&self) -> usize {
        let mut total = 0usize;
        {
            let vertex_tables = self.persistent.data_store.vertex_tables().read();
            for table in vertex_tables.values() {
                total += table.used_memory_size();
            }
        }
        {
            let edge_tables = self.persistent.data_store.edge_tables().read();
            for table in edge_tables.values() {
                total += table.used_memory_size();
            }
        }
        total
    }

    // ── Schema Operations ──

    pub fn create_vertex_type(
        &self,
        name: &str,
        properties: Vec<StoragePropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        super::schema_engine::create_vertex_type(self, name, properties, primary_key)
    }

    pub fn create_vertex_type_with_id(
        &self,
        name: &str,
        label_id: LabelId,
        properties: Vec<StoragePropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        super::schema_engine::create_vertex_type_with_id(
            self,
            name,
            label_id,
            properties,
            primary_key,
        )
    }

    pub fn create_edge_type(
        &self,
        name: &str,
        src_label: LabelId,
        dst_label: LabelId,
        properties: Vec<StoragePropertyDef>,
        oe_strategy: EdgeStrategy,
        ie_strategy: EdgeStrategy,
    ) -> StorageResult<LabelId> {
        super::schema_engine::create_edge_type(
            self,
            name,
            src_label,
            dst_label,
            properties,
            oe_strategy,
            ie_strategy,
        )
    }

    pub fn create_edge_type_with_id(
        &self,
        params: CreateEdgeTypeParams,
        label_id: LabelId,
    ) -> StorageResult<LabelId> {
        super::schema_engine::create_edge_type_with_id(self, params, label_id)
    }

    pub fn drop_vertex_type(&self, name: &str) -> StorageResult<()> {
        super::schema_engine::drop_vertex_type(self, name)
    }

    pub fn drop_edge_type(&self, name: &str) -> StorageResult<()> {
        super::schema_engine::drop_edge_type(self, name)
    }

    pub fn add_vertex_property(
        &self,
        label: LabelId,
        prop: StoragePropertyDef,
    ) -> StorageResult<()> {
        super::schema_engine::add_vertex_property(self, label, prop)
    }

    pub fn delete_vertex_property(&self, label: LabelId, prop_name: &str) -> StorageResult<()> {
        super::schema_engine::delete_vertex_property(self, label, prop_name)
    }

    pub fn rename_vertex_property(
        &self,
        label: LabelId,
        old_name: &str,
        new_name: &str,
    ) -> StorageResult<()> {
        super::schema_engine::rename_vertex_property(self, label, old_name, new_name)
    }

    pub fn add_edge_property(
        &self,
        edge_label: LabelId,
        prop: StoragePropertyDef,
    ) -> StorageResult<()> {
        super::schema_engine::add_edge_property(self, edge_label, prop)
    }

    pub fn delete_edge_property(&self, edge_label: LabelId, prop_name: &str) -> StorageResult<()> {
        super::schema_engine::delete_edge_property(self, edge_label, prop_name)
    }

    pub fn rename_edge_property(
        &self,
        edge_label: LabelId,
        old_name: &str,
        new_name: &str,
    ) -> StorageResult<()> {
        super::schema_engine::rename_edge_property(self, edge_label, old_name, new_name)
    }

    // ── Vertex Operations ──

    pub fn insert_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }
        let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
        let table = vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        let internal_id = table.insert(external_id, properties, ts)?;

        self.persistent
            .cache_manager
            .cache_vertex_id(label, external_id, internal_id, ts);
        self.mark_vertex_modified(label);

        Ok(internal_id)
    }

    pub fn insert_vertex_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }
        let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
        let table = vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        let internal_id = table.insert_by_i64(external_id, properties, ts)?;

        self.persistent.cache_manager.cache_vertex_id(
            label,
            &external_id.to_string(),
            internal_id,
            ts,
        );
        self.mark_vertex_modified(label);

        Ok(internal_id)
    }

    pub fn get_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        let internal_id = self
            .persistent
            .cache_manager
            .get_cached_vertex_id(label, external_id, ts)
            .or_else(|| {
                let id = {
                    let vertex_tables = self.persistent.data_store.vertex_tables().read();
                    vertex_tables.get(&label)?.get_internal_id(external_id, ts)
                };
                if let Some(id) = id {
                    self.persistent
                        .cache_manager
                        .cache_vertex_id(label, external_id, id, ts);
                }
                id
            })?;

        if let Some(cached) =
            self.persistent
                .cache_manager
                .get_cached_vertex(label, internal_id, ts)
        {
            return Some(VertexRecord {
                internal_id: cached.internal_id,
                vid: cached
                    .external_id
                    .parse::<i64>()
                    .map(crate::core::types::VertexId::from_int64)
                    .unwrap_or_else(|_| {
                        crate::core::types::VertexId::from_string(&cached.external_id)
                    }),
                properties: cached.properties,
            });
        }

        let record = {
            let vertex_tables = self.persistent.data_store.vertex_tables().read();
            vertex_tables
                .get(&label)?
                .get_by_internal_id(internal_id, ts)?
        };

        self.persistent.cache_manager.cache_vertex(
            label,
            internal_id,
            external_id.to_string(),
            record.properties.clone(),
            ts,
        );

        Some(record)
    }

    pub fn get_vertex_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        let external_id_str = external_id.to_string();
        let internal_id = self
            .persistent
            .cache_manager
            .get_cached_vertex_id(label, &external_id_str, ts)
            .or_else(|| {
                let id = {
                    let vertex_tables = self.persistent.data_store.vertex_tables().read();
                    vertex_tables
                        .get(&label)?
                        .get_internal_id_by_i64(external_id, ts)
                };
                if let Some(id) = id {
                    self.persistent
                        .cache_manager
                        .cache_vertex_id(label, &external_id_str, id, ts);
                }
                id
            })?;

        if let Some(cached) =
            self.persistent
                .cache_manager
                .get_cached_vertex(label, internal_id, ts)
        {
            return Some(VertexRecord {
                internal_id: cached.internal_id,
                vid: crate::core::types::VertexId::from_int64(external_id),
                properties: cached.properties,
            });
        }

        let record = {
            let vertex_tables = self.persistent.data_store.vertex_tables().read();
            vertex_tables
                .get(&label)?
                .get_by_internal_id(internal_id, ts)?
        };

        self.persistent.cache_manager.cache_vertex(
            label,
            internal_id,
            external_id_str,
            record.properties.clone(),
            ts,
        );

        Some(record)
    }

    pub fn get_vertex_by_internal_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        if let Some(cached) =
            self.persistent
                .cache_manager
                .get_cached_vertex(label, internal_id, ts)
        {
            return Some(VertexRecord {
                internal_id: cached.internal_id,
                vid: cached
                    .external_id
                    .parse::<i64>()
                    .map(crate::core::types::VertexId::from_int64)
                    .unwrap_or_else(|_| {
                        crate::core::types::VertexId::from_string(&cached.external_id)
                    }),
                properties: cached.properties,
            });
        }

        let record = {
            let vertex_tables = self.persistent.data_store.vertex_tables().read();
            vertex_tables
                .get(&label)?
                .get_by_internal_id(internal_id, ts)?
        };

        let external_id = {
            let vertex_tables = self.persistent.data_store.vertex_tables().read();
            vertex_tables
                .get(&label)?
                .get_external_id(internal_id, ts)
                .map(|k| k.to_string())
                .unwrap_or_default()
        };

        if !external_id.is_empty() {
            self.persistent
                .cache_manager
                .cache_vertex_id(label, &external_id, internal_id, ts);
        }

        self.persistent.cache_manager.cache_vertex(
            label,
            internal_id,
            external_id,
            record.properties.clone(),
            ts,
        );

        Some(record)
    }

    pub fn get_external_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<String> {
        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        vertex_tables
            .get(&label)?
            .get_external_id(internal_id, ts)
            .map(|k| k.to_string())
    }

    pub fn get_external_id_any(&self, internal_id: u32, ts: Timestamp) -> Option<String> {
        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        vertex_tables
            .values()
            .find_map(|t| t.get_external_id(internal_id, ts))
            .map(|k| k.to_string())
    }

    /// Get external ID by internal ID without timestamp check.
    /// Returns the external ID even for deleted vertices.
    pub fn get_external_id_by_internal_id(
        &self,
        label: LabelId,
        internal_id: u32,
    ) -> Option<VertexId> {
        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        let table = vertex_tables.get(&label)?;
        let key = table.get_external_id_raw(internal_id)?;
        Some(match key {
            crate::storage::vertex::IdKey::Int(i) => VertexId::from_int64(i),
            crate::storage::vertex::IdKey::Text(s) => VertexId::from_string(s),
        })
    }

    pub fn delete_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
        let table = vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        let internal_id = table.get_internal_id(external_id, ts);
        table.delete(external_id, ts)?;

        self.persistent
            .cache_manager
            .remove_cached_vertex_id(label, external_id);
        if let Some(id) = internal_id {
            self.persistent
                .cache_manager
                .remove_cached_vertex(label, id);
        }
        self.mark_vertex_modified(label);

        Ok(())
    }

    pub fn delete_vertex_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
        let table = vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        let internal_id = table.get_internal_id_by_i64(external_id, ts);
        let external_id_str = external_id.to_string();
        table.delete_by_i64(external_id, ts)?;

        self.persistent
            .cache_manager
            .remove_cached_vertex_id(label, &external_id_str);
        if let Some(id) = internal_id {
            self.persistent
                .cache_manager
                .remove_cached_vertex(label, id);
        }
        self.mark_vertex_modified(label);

        Ok(())
    }

    pub fn update_vertex_property(
        &self,
        label: LabelId,
        external_id: &str,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
        let table = vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        let internal_id = table
            .get_internal_id(external_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        table.update_property(internal_id, property_name, value, ts)?;

        self.persistent
            .cache_manager
            .remove_cached_vertex(label, internal_id);
        self.mark_vertex_modified(label);

        Ok(())
    }

    pub fn update_vertex_property_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
        let table = vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

        let internal_id = table
            .get_internal_id_by_i64(external_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        table.update_property(internal_id, property_name, value, ts)?;

        self.persistent
            .cache_manager
            .remove_cached_vertex(label, internal_id);
        self.mark_vertex_modified(label);

        Ok(())
    }

    // ── Edge Operations ──

    pub fn insert_edge(&self, params: InsertEdgeParams) -> StorageResult<()> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();

        let src_internal = self
            .resolve_internal_id(&vertex_tables, params.src_label, params.src_id, params.ts)
            .ok_or(StorageError::vertex_not_found())?;

        let dst_internal = self
            .resolve_internal_id(&vertex_tables, params.dst_label, params.dst_id, params.ts)
            .ok_or(StorageError::vertex_not_found())?;

        // For unconstrained edge types (params.src_label == 0), resolve the actual
        // vertex table label so edges go into per-label tables. This prevents internal
        // ID collisions when different vertex tables use the same internal ID.
        let actual_src_label = if params.src_label == 0 {
            Self::resolve_internal_id_label(&vertex_tables, &params.src_id, params.ts)
                .ok_or(StorageError::vertex_not_found())?
        } else {
            params.src_label
        };
        let actual_dst_label = if params.dst_label == 0 {
            Self::resolve_internal_id_label(&vertex_tables, &params.dst_id, params.ts)
                .ok_or(StorageError::vertex_not_found())?
        } else {
            params.dst_label
        };
        drop(vertex_tables);

        // Look up or create the per-label edge table
        let key = EdgeTableKey::new(actual_src_label, actual_dst_label, params.edge_label);
        let mut edge_tables = self.persistent.data_store.edge_tables().write();

        // The (0, 0, edge_label) table always exists (created by create_edge_type).
        // Clone its schema when creating new per-label tables.
        let edge_table = if edge_tables.contains_key(&key) {
            edge_tables.get_mut(&key).unwrap()
        } else {
            let edge_schema = {
                let original_key = EdgeTableKey::new(0, 0, params.edge_label);
                let orig = edge_tables.get(&original_key).ok_or_else(|| {
                    StorageError::label_not_found(format!("edge label {}", params.edge_label))
                })?;
                let mut s = orig.schema().clone();
                s.src_label = actual_src_label;
                s.dst_label = actual_dst_label;
                s
            };
            let mut new_table = EdgeTable::new(edge_schema)?;
            if let Some(stats) = &self.persistent.stats_manager {
                new_table.set_stats_manager(stats.clone());
            }
            edge_tables.insert(key, new_table);
            edge_tables.get_mut(&key).unwrap()
        };

        let mut rank = params.rank;
        loop {
            match edge_table.insert_edge(
                src_internal,
                dst_internal,
                rank,
                params.properties,
                params.ts,
            ) {
                Ok(()) => {
                    self.mark_edge_modified(params.edge_label);
                    return Ok(());
                }
                Err(ref e)
                    if e.kind()
                        == crate::core::error::storage::StorageErrorKind::EdgeAlreadyExists
                        && rank == params.rank =>
                {
                    rank += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Resolve the actual edge table key for unconstrained edge types.
    /// For constrained edge types (src_label != 0 and dst_label != 0), returns
    /// the direct key. For unconstrained types, resolves the actual vertex labels.
    fn resolve_edge_table_key(ctx: EdgeLabelLookupCtx) -> EdgeTableKey {
        let actual_src_label = if ctx.src_label == 0 {
            Self::resolve_internal_id_label(ctx.vertex_tables, ctx.src_id, ctx.ts)
                .unwrap_or(ctx.src_label)
        } else {
            ctx.src_label
        };
        let actual_dst_label = if ctx.dst_label == 0 {
            Self::resolve_internal_id_label(ctx.vertex_tables, ctx.dst_id, ctx.ts)
                .unwrap_or(ctx.dst_label)
        } else {
            ctx.dst_label
        };
        EdgeTableKey::new(actual_src_label, actual_dst_label, ctx.edge_label)
    }

    pub fn get_edge(&self, params: &EdgeOperationParams, ts: Timestamp) -> Option<EdgeRecord> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();

        let src_internal =
            self.resolve_internal_id(&vertex_tables, params.src_label, params.src_id, ts)?;

        let dst_internal =
            self.resolve_internal_id(&vertex_tables, params.dst_label, params.dst_id, ts)?;
        let key = Self::resolve_edge_table_key(EdgeLabelLookupCtx {
            vertex_tables: &vertex_tables,
            src_id: &params.src_id,
            src_label: params.src_label,
            dst_id: &params.dst_id,
            dst_label: params.dst_label,
            edge_label: params.edge_label,
            ts,
        });
        let edge_tables = self.persistent.data_store.edge_tables().read();
        let edge_table = edge_tables.get(&key)?;

        edge_table.get_edge(src_internal, dst_internal, params.rank, ts)
    }

    pub fn delete_edge(&self, params: &EdgeOperationParams, ts: Timestamp) -> StorageResult<bool> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();

        let src_internal = self
            .resolve_internal_id(&vertex_tables, params.src_label, params.src_id, ts)
            .or_else(|| {
                self.resolve_internal_id_any(&vertex_tables, params.src_label, params.src_id)
            })
            .ok_or(StorageError::vertex_not_found())?;

        let dst_internal = self
            .resolve_internal_id(&vertex_tables, params.dst_label, params.dst_id, ts)
            .or_else(|| {
                self.resolve_internal_id_any(&vertex_tables, params.dst_label, params.dst_id)
            })
            .ok_or(StorageError::vertex_not_found())?;

        let key = Self::resolve_edge_table_key(EdgeLabelLookupCtx {
            vertex_tables: &vertex_tables,
            src_id: &params.src_id,
            src_label: params.src_label,
            dst_id: &params.dst_id,
            dst_label: params.dst_label,
            edge_label: params.edge_label,
            ts,
        });
        drop(vertex_tables);

        let mut edge_tables = self.persistent.data_store.edge_tables().write();
        let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
            StorageError::label_not_found(format!("edge label {}", params.edge_label))
        })?;

        let deleted = edge_table.delete_edge(src_internal, dst_internal, params.rank, ts)?;
        if deleted {
            self.mark_edge_modified(params.edge_label);
        }

        Ok(deleted)
    }

    pub fn out_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        _dst_label: LabelId,
        src_id: VertexId,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        let src_internal = self.resolve_internal_id(&vertex_tables, src_label, src_id, ts)?;
        let actual_src = if src_label == 0 {
            Self::resolve_internal_id_label(&vertex_tables, &src_id, ts).unwrap_or(src_label)
        } else {
            src_label
        };
        drop(vertex_tables);

        let edge_tables = self.persistent.data_store.edge_tables().read();
        let mut records = Vec::new();
        for table in edge_tables
            .values()
            .filter(|t| t.label() == edge_label && t.src_label() == actual_src)
        {
            records.extend(table.out_edges(src_internal, ts));
        }
        Some(records)
    }

    pub fn in_edges(
        &self,
        edge_label: LabelId,
        _src_label: LabelId,
        dst_label: LabelId,
        dst_id: VertexId,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        let dst_internal = self.resolve_internal_id(&vertex_tables, dst_label, dst_id, ts)?;
        let actual_dst = if dst_label == 0 {
            Self::resolve_internal_id_label(&vertex_tables, &dst_id, ts).unwrap_or(dst_label)
        } else {
            dst_label
        };
        drop(vertex_tables);

        let edge_tables = self.persistent.data_store.edge_tables().read();
        let mut records = Vec::new();
        for table in edge_tables
            .values()
            .filter(|t| t.label() == edge_label && t.dst_label() == actual_dst)
        {
            records.extend(table.in_edges(dst_internal, ts));
        }
        Some(records)
    }

    // ── Query Operations ──

    pub fn scan_vertices(&self, label: LabelId, ts: Timestamp) -> Option<Vec<VertexRecord>> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }
        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        vertex_tables.get(&label).map(|t| t.scan(ts).collect())
    }

    // ── Label Access ──

    pub fn scan_edges(
        &self,
        src_label: LabelId,
        dst_label: LabelId,
        edge_label: LabelId,
        ts: Timestamp,
    ) -> Vec<EdgeRecord> {
        self.persistent
            .data_store
            .edge_tables()
            .read()
            .get(&EdgeTableKey::new(src_label, dst_label, edge_label))
            .map(|t| t.scan(ts))
            .unwrap_or_default()
    }

    pub fn scan_edges_by_label(&self, edge_label: LabelId, ts: Timestamp) -> Vec<EdgeRecord> {
        let edge_tables = self.persistent.data_store.edge_tables().read();
        let mut records = Vec::new();

        for table in edge_tables
            .values()
            .filter(|table| table.label() == edge_label)
        {
            records.extend(table.scan(ts));
        }

        records
    }

    pub fn total_vertex_count(&self) -> usize {
        self.persistent
            .data_store
            .vertex_tables()
            .read()
            .values()
            .map(|t| t.total_count())
            .sum()
    }

    pub fn total_edge_count(&self) -> usize {
        self.persistent
            .data_store
            .edge_tables()
            .read()
            .values()
            .map(|t| t.edge_count() as usize)
            .sum()
    }

    pub fn collect_all_edge_records(
        &self,
        ts: Timestamp,
    ) -> Vec<(LabelId, LabelId, LabelId, EdgeRecord)> {
        let edge_tables = self.persistent.data_store.edge_tables().read();
        let mut records = Vec::new();
        for (
            EdgeTableKey {
                src_label,
                dst_label,
                edge_label,
            },
            table,
        ) in &*edge_tables
        {
            for edge_record in table.scan(ts) {
                records.push((*src_label, *dst_label, *edge_label, edge_record));
            }
        }
        records
    }

    // ── Persistence Operations ──

    pub(crate) fn flush_tables_to_dir(&self, data_dir: &Path) -> StorageResult<()> {
        use std::fs;

        let compression = self.persistent.config.flush_config.compression;
        let vertex_dir = data_dir.join("vertices");
        fs::create_dir_all(&vertex_dir)?;

        {
            let vertex_tables = self.persistent.data_store.vertex_tables().read();
            for (label_id, table) in &*vertex_tables {
                let table_dir = vertex_dir.join(format!("label_{}", label_id));
                table.flush(&table_dir, compression)?;
            }
        }

        let edge_dir = data_dir.join("edges");
        fs::create_dir_all(&edge_dir)?;

        {
            let ts = self.get_read_timestamp();
            let mut edge_tables = self.persistent.data_store.edge_tables().write();
            for (
                EdgeTableKey {
                    src_label,
                    dst_label,
                    edge_label,
                },
                table,
            ) in edge_tables.iter_mut()
            {
                let table_dir =
                    edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
                table.maybe_compact_for_flush(ts, 2.0);
                table.flush(&table_dir, compression)?;
            }
        }

        let index_dir = data_dir.join("indexes");
        fs::create_dir_all(&index_dir)?;
        self.persistent
            .index_data_manager
            .read()
            .flush(&index_dir)?;

        if let Some(persistence) = self.persistent.persistence.as_ref() {
            persistence
                .read()
                .wal_manager()
                .and_then(|w| w.read().sync().ok());
        }

        Ok(())
    }

    pub(crate) fn restore_from_checkpoint(&self, checkpoint_dir: &Path) -> StorageResult<()> {
        use std::fs;

        let checkpoint_paths = crate::storage::engine::paths::StoragePaths::new(checkpoint_dir);

        let vertex_dir = checkpoint_paths.vertices_dir();
        if vertex_dir.exists() {
            let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
            for entry in fs::read_dir(&vertex_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name() {
                        if let Some(name_str) = dir_name.to_str() {
                            if let Some(label_str) = name_str.strip_prefix("label_") {
                                if let Ok(label_id) = label_str.parse::<LabelId>() {
                                    if let Some(table) = vertex_tables.get_mut(&label_id) {
                                        table.load(&path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let edge_dir = checkpoint_paths.edges_dir();
        if edge_dir.exists() {
            let mut edge_tables = self.persistent.data_store.edge_tables().write();
            for entry in fs::read_dir(&edge_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name() {
                        if let Some(name_str) = dir_name.to_str() {
                            let parts: Vec<&str> = name_str.splitn(3, '_').collect();
                            if parts.len() == 3 {
                                if let (Ok(src_label), Ok(dst_label), Ok(edge_label)) = (
                                    parts[0].parse::<LabelId>(),
                                    parts[1].parse::<LabelId>(),
                                    parts[2].parse::<LabelId>(),
                                ) {
                                    let key = EdgeTableKey::new(src_label, dst_label, edge_label);
                                    if let Some(table) = edge_tables.get_mut(&key) {
                                        table.load(&path)?;
                                        if let Some(stats) = &self.persistent.stats_manager {
                                            table.set_stats_manager(stats.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let index_dir = checkpoint_paths.indexes_dir();
        if index_dir.exists() {
            self.persistent
                .index_data_manager
                .write()
                .load(&index_dir)?;
        }

        Ok(())
    }

    // ── Compaction ──

    pub(crate) fn compact_maintenance(
        &self,
        config: &CompactConfig,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let mut last_compacted_vertices = self.persistent.last_compacted_vertices.lock();
        last_compacted_vertices.clear();

        let vertex_labels: Vec<LabelId>;
        {
            let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
            vertex_labels = vertex_tables.keys().copied().collect();

            for &label_id in &vertex_labels {
                let table = vertex_tables.get_mut(&label_id).expect("label must exist");
                let removed = table.compact_with_ts_collect(ts);
                if !removed.is_empty() {
                    last_compacted_vertices.push((label_id, removed));
                }
            }
        }

        for &label_id in &vertex_labels {
            self.mark_vertex_modified(label_id);
        }

        let total_vertices_removed: usize = last_compacted_vertices
            .iter()
            .map(|(_, removed)| removed.len())
            .sum();

        log::info!(
            "Compacted vertex tables: {} vertices removed",
            total_vertices_removed
        );

        let edge_keys: Vec<EdgeTableKey>;
        let mut total_edges_removed = 0usize;
        {
            let mut edge_tables = self.persistent.data_store.edge_tables().write();
            edge_keys = edge_tables.keys().copied().collect();

            if config.enable_structure_compaction {
                for &key in &edge_keys {
                    let table = edge_tables.get_mut(&key).expect("edge key must exist");
                    let removed = table.compact_and_freeze_with_config(ts, config);
                    total_edges_removed += removed;
                }

                log::info!(
                    "Compacted CSR structures: {} edges removed",
                    total_edges_removed
                );
            } else {
                // If not compacting structure, still freeze mutable CSR for consistency
                for &key in &edge_keys {
                    let table = edge_tables.get_mut(&key).expect("edge key must exist");
                    table.freeze_csr_only(ts);
                    table.compact_properties(ts);
                }
            }
        }

        for &key in &edge_keys {
            self.mark_edge_modified(key.edge_label);
        }

        match self.gc_index_tombstones(ts) {
            Ok(index_gc_stats) if index_gc_stats.total_removed() > 0 => {
                log::info!(
                    "Index GC during compaction: removed {} vertex entries",
                    index_gc_stats.vertex_entries_removed,
                );
            }
            Ok(_) => {}
            Err(err) => {
                log::warn!("Index GC during compaction failed: {}", err);
            }
        }

        self.persistent.cache_manager.clear_cache();

        log::info!(
            "Compaction completed: {} vertices, {} edges removed",
            total_vertices_removed,
            total_edges_removed
        );

        Ok(())
    }

    // ── Cache Operations ──

    pub(crate) fn invalidate_vertex_cache(&self, label: LabelId) {
        self.persistent
            .cache_manager
            .invalidate_vertices_by_label(label);
    }

    // ── Index Operations ──

    pub(crate) fn update_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        super::index_engine::update_vertex_indexes_mvcc(
            self, space_id, vertex_id, index_name, props, ts,
        )
    }

    pub(crate) fn delete_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_names: &[String],
        ts: Timestamp,
    ) -> StorageResult<()> {
        super::index_engine::delete_vertex_indexes_mvcc(self, space_id, vertex_id, index_names, ts)
    }

    pub(crate) fn gc_index_tombstones(&self, ts: Timestamp) -> StorageResult<GcStats> {
        self.persistent.index_data_manager.write().gc_tombstones(ts)
    }

    // ── Snapshot Export ──

    pub fn export_snapshot(&self, ts: Timestamp) -> StorageResult<Vec<ExportedEdgeSnapshotRecord>> {
        let edge_tables = self.persistent.data_store.edge_tables().read();
        let mut results = Vec::with_capacity(edge_tables.len());
        for (
            EdgeTableKey {
                src_label,
                dst_label,
                edge_label,
            },
            table,
        ) in edge_tables.iter()
        {
            let snapshot = table.export_snapshot(ts)?;
            results.push(ExportedEdgeSnapshotRecord {
                src_label: *src_label,
                dst_label: *dst_label,
                edge_label: *edge_label,
                snapshot,
            });
        }
        Ok(results)
    }

    // ── Freeze Stats ──

    pub fn get_freeze_stats(&self) -> Option<FreezeStats> {
        self.runtime
            .background_freeze_manager
            .as_ref()
            .map(|m| m.get_stats())
    }

    pub fn trigger_background_freeze(&self) -> StorageResult<()> {
            let config = CompactConfig::with_fixed_ratio(true, 2.0)
                .enable_segment_merge(1000);  // Default: merge segments within 1000 timestamp units
        let ts = u32::MAX;
        let mut total_frozen = 0u64;
        let mut any_frozen = false;
        let start = std::time::Instant::now();

        {
            let mut edge_tables = self.persistent.data_store.edge_tables().write();
            for table in edge_tables.values_mut() {
                let delta_edges = table.delta_edge_count();
                if delta_edges >= self.persistent.config.background_freeze.delta_edge_threshold {
                    let frozen = table.compact_and_freeze_with_config(ts, &config);
                    total_frozen += frozen as u64;
                    any_frozen = true;
                }
            }
        }

        if any_frozen {
            let duration_ms = start.elapsed().as_millis() as u64;
            if let Some(ref manager) = self.runtime.background_freeze_manager {
                manager.record_freeze(total_frozen, duration_ms);
            }
        }

        Ok(())
    }

    // ── Helper Methods ──

    fn resolve_internal_id(
        &self,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
        label: LabelId,
        id: VertexId,
        ts: Timestamp,
    ) -> Option<u32> {
        if let Some(int_id) = id.as_int64() {
            self.resolve_internal_id_from_i64(vertex_tables, label, int_id, ts)
        } else if let Some(str_id) = id.as_str() {
            self.resolve_internal_id_from_str(vertex_tables, label, str_id, ts)
        } else {
            None
        }
    }

    /// Resolve internal ID without timestamp check (works for deleted vertices).
    /// Used for dangling edge repair where the endpoint vertex may be deleted.
    fn resolve_internal_id_any(
        &self,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
        label: LabelId,
        id: VertexId,
    ) -> Option<u32> {
        if let Some(int_id) = id.as_int64() {
            if label == 0 {
                vertex_tables
                    .values()
                    .find_map(|t| t.get_internal_id_by_i64_raw(int_id))
            } else {
                vertex_tables
                    .get(&label)?
                    .get_internal_id_by_i64_raw(int_id)
            }
        } else if let Some(str_id) = id.as_str() {
            if label == 0 {
                vertex_tables
                    .values()
                    .find_map(|t| t.get_internal_id_raw(str_id))
            } else {
                vertex_tables.get(&label)?.get_internal_id_raw(str_id)
            }
        } else {
            None
        }
    }

    fn resolve_internal_id_from_i64(
        &self,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
        label: LabelId,
        id: i64,
        ts: Timestamp,
    ) -> Option<u32> {
        if label == 0 {
            vertex_tables
                .values()
                .find_map(|t| t.get_internal_id_by_i64(id, ts))
        } else {
            vertex_tables.get(&label)?.get_internal_id_by_i64(id, ts)
        }
    }

    fn resolve_internal_id_from_str(
        &self,
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
        label: LabelId,
        id: &str,
        ts: Timestamp,
    ) -> Option<u32> {
        if label == 0 {
            vertex_tables
                .values()
                .find_map(|t| t.get_internal_id(id, ts))
        } else {
            vertex_tables.get(&label)?.get_internal_id(id, ts)
        }
    }

    /// Resolve the vertex table label for an external vertex ID.
    /// Searches all vertex tables and returns the label of the matching table.
    fn resolve_internal_id_label(
        vertex_tables: &HashMap<LabelId, crate::storage::vertex::VertexTable>,
        id: &VertexId,
        ts: Timestamp,
    ) -> Option<LabelId> {
        if let Some(int_id) = id.as_int64() {
            vertex_tables
                .iter()
                .find_map(|(lbl, t)| t.get_internal_id_by_i64(int_id, ts).map(|_| *lbl))
        } else if let Some(str_id) = id.as_str() {
            vertex_tables
                .iter()
                .find_map(|(lbl, t)| t.get_internal_id(str_id, ts).map(|_| *lbl))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for GraphStorageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStorageContext")
            .field("work_dir", &self.persistent.layout.work_dir())
            .field("db_path", &self.persistent.layout.db_path())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ExportedEdgeSnapshotRecord {
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub edge_label: LabelId,
    pub snapshot: ExportedEdgeSnapshot,
}

impl Default for GraphStorageContext {
    fn default() -> Self {
        Self::new()
    }
}
