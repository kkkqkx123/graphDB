use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::core::metadata::{IndexManager, SchemaManager};
use crate::core::mvcc::VersionManager;
use crate::core::types::{
    CompactConfig, LabelId, TableId, TableTracker, TableTrackerConfig, TableType, Timestamp,
    TransactionContextInfo, VertexId,
};
use crate::core::UserStorage;
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::cache::{RecordCacheStats, SharedRecordCache};
use crate::storage::edge::{EdgeRecord, EdgeStrategy};
use crate::storage::engine::cache_manager::CacheManager;
use crate::storage::engine::config::PropertyGraphConfig;
use crate::storage::engine::data_store::{EdgeTableKey, GraphDataStore};
use crate::storage::engine::params::CreateEdgeTypeParams;
use crate::storage::engine::paths::StoragePaths;
use crate::storage::engine::persistence_coordinator::PersistenceCoordinator;
use crate::storage::engine::wal_manager::WalManager;
use crate::storage::engine::{
    EdgeOperationParams, InsertEdgeParams, PropertyGraphUpdateEdgePropertyParams,
};
use crate::storage::index::{
    GcStats, IndexDataManagerImpl, IndexGcConfig, IndexGcManager, IndexGcOps,
};
use crate::storage::types::{EdgeOffset, StoragePropertyDef};
use crate::storage::vertex::{IdKey, VertexRecord};

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
    wal_manager: Arc<Mutex<WalManager>>,
    table_tracker: Arc<TableTracker>,
    config: PropertyGraphConfig,
    is_open: Arc<AtomicBool>,
    last_compacted_vertices: Arc<Mutex<Vec<(LabelId, Vec<IdKey>)>>>,
    index_data_manager: Arc<RwLock<IndexDataManagerImpl>>,
    schema_manager: Arc<SchemaManager>,
    index_metadata_manager: Arc<IndexManager>,
    version_manager: Arc<VersionManager>,
    user_storage: Arc<UserStorage>,
    persistence: Option<Arc<RwLock<PersistenceCoordinator>>>,
    layout: GraphStorageLayout,
}

impl GraphStoragePersistent {
    fn build_core_components() -> (
        Arc<GraphDataStore>,
        Arc<CacheManager>,
        Arc<Mutex<WalManager>>,
        Arc<TableTracker>,
        Arc<AtomicBool>,
        Arc<Mutex<Vec<(LabelId, Vec<IdKey>)>>>,
        Arc<RwLock<IndexDataManagerImpl>>,
        Arc<SchemaManager>,
        Arc<IndexManager>,
        Arc<VersionManager>,
        Arc<UserStorage>,
    ) {
        let config = PropertyGraphConfig::default();
        let cache_manager = Arc::new(CacheManager::new(config.enable_cache, config.cache_memory));
        let table_tracker = Arc::new(TableTracker::with_config(TableTrackerConfig {
            flush_threshold: config.flush_config.flush_threshold,
            flush_interval: config.flush_config.flush_interval,
        }));

        (
            Arc::new(GraphDataStore::new()),
            cache_manager,
            Arc::new(Mutex::new(WalManager::new())),
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
            wal_manager: Arc::new(Mutex::new(WalManager::new())),
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
            wal_manager,
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
            wal_manager,
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
        })
    }
}

impl GraphStoragePersistent {
    fn data_store(&self) -> &GraphDataStore {
        &self.data_store
    }

    fn cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }

    fn wal_manager(&self) -> &Mutex<WalManager> {
        &self.wal_manager
    }

    fn table_tracker(&self) -> &Arc<TableTracker> {
        &self.table_tracker
    }

    fn config(&self) -> &PropertyGraphConfig {
        &self.config
    }

    fn is_open(&self) -> &AtomicBool {
        &self.is_open
    }

    fn last_compacted_vertices(&self) -> &Mutex<Vec<(LabelId, Vec<IdKey>)>> {
        &self.last_compacted_vertices
    }

    fn index_data_manager(&self) -> &RwLock<IndexDataManagerImpl> {
        &self.index_data_manager
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
        index_data_manager: &Arc<RwLock<IndexDataManagerImpl>>,
        version_manager: &Arc<VersionManager>,
        config: IndexGcConfig,
    ) -> Self {
        let index_data = index_data_manager.read().clone();
        let gc_manager = IndexGcManager::new(index_data, version_manager.clone(), config);

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

    pub fn with_config(config: PropertyGraphConfig) -> Self {
        Self {
            persistent: GraphStoragePersistent::new_with_config(config),
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

    // ── Internal accessors for sub-modules ──

    pub(crate) fn data_store(&self) -> &GraphDataStore {
        self.persistent.data_store()
    }

    pub(crate) fn cache_manager(&self) -> &CacheManager {
        self.persistent.cache_manager()
    }

    pub(crate) fn wal_manager(&self) -> &Mutex<WalManager> {
        self.persistent.wal_manager()
    }

    pub(crate) fn table_tracker(&self) -> &Arc<TableTracker> {
        self.persistent.table_tracker()
    }

    pub(crate) fn config(&self) -> &PropertyGraphConfig {
        self.persistent.config()
    }

    pub(crate) fn is_open_flag(&self) -> &AtomicBool {
        self.persistent.is_open()
    }

    pub(crate) fn last_compacted_vertices(&self) -> &Mutex<Vec<(LabelId, Vec<IdKey>)>> {
        self.persistent.last_compacted_vertices()
    }

    pub(crate) fn index_data_manager(&self) -> &RwLock<IndexDataManagerImpl> {
        self.persistent.index_data_manager()
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

    // ── PropertyGraph-compatible API ──

    pub fn wal_enabled(&self) -> bool {
        if let Some(persistence) = self.persistent.persistence.as_ref() {
            persistence.read().wal_manager().read().is_enabled()
        } else {
            self.persistent.wal_manager.lock().is_enabled()
        }
    }

    pub fn should_flush(&self) -> bool {
        self.persistent.table_tracker.should_flush()
    }

    pub fn get_modified_table_count(&self) -> usize {
        self.persistent.table_tracker.get_modified_count()
    }

    pub fn mark_table_modified(&self, table_type: TableType, label_id: u32) {
        let table_id = TableId::new(table_type, label_id);
        self.persistent.table_tracker.mark_modified(table_id);
    }

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

    pub fn mark_vertex_modified_since_checkpoint(&self, label: LabelId) {
        self.persistent
            .table_tracker
            .mark_modified_since_checkpoint(TableId::vertex(label));
    }

    pub fn mark_edge_modified_since_checkpoint(&self, label: LabelId) {
        self.persistent
            .table_tracker
            .mark_modified_since_checkpoint(TableId::edge(label));
    }

    pub fn take_last_compacted_vertices(&self) -> Vec<(LabelId, Vec<IdKey>)> {
        std::mem::take(&mut *self.persistent.last_compacted_vertices.lock())
    }

    pub fn record_cache(&self) -> Option<&SharedRecordCache> {
        self.persistent.cache_manager.record_cache()
    }

    pub fn record_cache_stats(&self) -> Option<RecordCacheStats> {
        self.persistent.cache_manager.record_cache_stats()
    }

    pub fn clear_cache(&self) {
        self.persistent.cache_manager.clear_cache();
    }

    pub fn close(&self) {
        self.persistent.is_open.store(false, Ordering::Release);
        {
            let mut vertex_tables = self.persistent.data_store.vertex_tables().write();
            for table in vertex_tables.values_mut() {
                table.close();
            }
        }
        {
            let mut edge_tables = self.persistent.data_store.edge_tables().write();
            for table in edge_tables.values_mut() {
                table.close();
            }
        }
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

    pub fn vertex_label_ids(&self) -> Vec<LabelId> {
        self.persistent
            .data_store
            .vertex_tables()
            .read()
            .keys()
            .copied()
            .collect()
    }

    // ── Edge Operations ──

    pub fn insert_edge(&self, params: InsertEdgeParams) -> StorageResult<EdgeOffset> {
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
        drop(vertex_tables);

        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        let mut edge_tables = self.persistent.data_store.edge_tables().write();
        let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
            StorageError::label_not_found(format!("edge label {}", params.edge_label))
        })?;

        let offset = edge_table.insert_edge(
            VertexId::from_int64(src_internal as i64),
            VertexId::from_int64(dst_internal as i64),
            params.rank,
            params.properties,
            params.ts,
        )?;
        self.mark_edge_modified(params.edge_label);

        Ok(offset)
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
        drop(vertex_tables);

        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        let edge_tables = self.persistent.data_store.edge_tables().read();
        let edge_table = edge_tables.get(&key)?;

        edge_table.get_edge(
            VertexId::from_int64(src_internal as i64),
            VertexId::from_int64(dst_internal as i64),
            params.rank,
            ts,
        )
    }

    pub fn delete_edge(&self, params: &EdgeOperationParams, ts: Timestamp) -> StorageResult<bool> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();

        let src_internal = self
            .resolve_internal_id(&vertex_tables, params.src_label, params.src_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        let dst_internal = self
            .resolve_internal_id(&vertex_tables, params.dst_label, params.dst_id, ts)
            .ok_or(StorageError::vertex_not_found())?;

        drop(vertex_tables);

        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        let mut edge_tables = self.persistent.data_store.edge_tables().write();
        let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
            StorageError::label_not_found(format!("edge label {}", params.edge_label))
        })?;

        let deleted = edge_table.delete_edge(
            VertexId::from_int64(src_internal as i64),
            VertexId::from_int64(dst_internal as i64),
            params.rank,
            ts,
        )?;
        if deleted {
            self.mark_edge_modified(params.edge_label);
        }

        Ok(deleted)
    }

    pub fn update_edge_property(
        &self,
        params: PropertyGraphUpdateEdgePropertyParams,
    ) -> StorageResult<bool> {
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

        drop(vertex_tables);

        let key = EdgeTableKey::new(params.src_label, params.dst_label, params.edge_label);
        let mut edge_tables = self.persistent.data_store.edge_tables().write();
        let edge_table = edge_tables.get_mut(&key).ok_or_else(|| {
            StorageError::label_not_found(format!("edge label {}", params.edge_label))
        })?;

        let updated = edge_table.update_edge_property(
            VertexId::from_int64(src_internal as i64),
            VertexId::from_int64(dst_internal as i64),
            params.rank,
            params.prop_name,
            params.value,
            params.ts,
        )?;
        if updated {
            self.mark_edge_modified(params.edge_label);
        }

        Ok(updated)
    }

    pub fn out_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        src_id: VertexId,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        let src_internal = self.resolve_internal_id(&vertex_tables, src_label, src_id, ts)?;
        drop(vertex_tables);

        let key = EdgeTableKey::new(src_label, dst_label, edge_label);
        let edge_tables = self.persistent.data_store.edge_tables().read();
        let edge_table = edge_tables.get(&key)?;

        Some(edge_table.out_edges(VertexId::from_int64(src_internal as i64), ts))
    }

    pub fn in_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        dst_id: VertexId,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }

        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        let dst_internal = self.resolve_internal_id(&vertex_tables, dst_label, dst_id, ts)?;
        drop(vertex_tables);

        let key = EdgeTableKey::new(src_label, dst_label, edge_label);
        let edge_tables = self.persistent.data_store.edge_tables().read();
        let edge_table = edge_tables.get(&key)?;

        Some(edge_table.in_edges(VertexId::from_int64(dst_internal as i64), ts))
    }

    // ── Query Operations ──

    pub fn scan_vertices(&self, label: LabelId, ts: Timestamp) -> Option<Vec<VertexRecord>> {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return None;
        }
        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        vertex_tables.get(&label).map(|t| t.scan(ts).collect())
    }

    pub fn vertex_count(&self, label: LabelId, ts: Timestamp) -> usize {
        if !self.persistent.is_open.load(Ordering::Acquire) {
            return 0;
        }
        let vertex_tables = self.persistent.data_store.vertex_tables().read();
        vertex_tables
            .get(&label)
            .map(|t| t.vertex_count(ts))
            .unwrap_or(0)
    }

    pub fn edge_count(&self, edge_label: LabelId) -> u64 {
        self.persistent
            .data_store
            .edge_tables()
            .read()
            .values()
            .filter_map(|t| {
                if t.label() == edge_label {
                    Some(t.edge_count())
                } else {
                    None
                }
            })
            .sum()
    }

    // ── Label Access ──

    pub fn vertex_label_names(&self) -> Vec<String> {
        self.persistent
            .data_store
            .vertex_label_names()
            .read()
            .keys()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn edge_label_names(&self) -> Vec<String> {
        self.persistent
            .data_store
            .edge_label_names()
            .read()
            .keys()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn get_vertex_label_id(&self, name: &str) -> Option<LabelId> {
        self.persistent
            .data_store
            .vertex_label_names()
            .read()
            .get(name)
            .copied()
    }

    pub fn get_edge_label_id(&self, name: &str) -> Option<LabelId> {
        self.persistent
            .data_store
            .edge_label_names()
            .read()
            .get(name)
            .copied()
    }

    // ── Table Access ──

    pub fn get_vertex_table_opt(&self, label: LabelId) -> Option<String> {
        self.persistent
            .data_store
            .vertex_tables()
            .read()
            .get(&label)
            .map(|t| t.label_name().to_string())
    }

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
        self.persistent
            .data_store
            .edge_tables()
            .read()
            .values()
            .find(|t| t.label() == edge_label)
            .map(|t| t.scan(ts))
            .unwrap_or_default()
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

        let compression = self.persistent.config().flush_config.compression;
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
            let edge_tables = self.persistent.data_store.edge_tables().read();
            for (
                EdgeTableKey {
                    src_label,
                    dst_label,
                    edge_label,
                },
                table,
            ) in &*edge_tables
            {
                let table_dir =
                    edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
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
            persistence.read().wal_manager().read().sync()?;
        } else {
            self.persistent.wal_manager.lock().sync()?;
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
                    let removed = table.compact_csr(ts, config.reserve_ratio);
                    total_edges_removed += removed;
                }

                log::info!(
                    "Compacted CSR structures: {} edges removed",
                    total_edges_removed
                );
            }

            for &key in &edge_keys {
                let table = edge_tables.get_mut(&key).expect("edge key must exist");
                table.compact_properties(ts);
            }
        }

        for &key in &edge_keys {
            self.mark_edge_modified(key.edge_label);
        }

        match self.gc_index_tombstones(ts) {
            Ok(index_gc_stats) if index_gc_stats.total_removed() > 0 => {
                log::info!(
                    "Index GC during compaction: removed {} vertex entries, {} edge entries",
                    index_gc_stats.vertex_entries_removed,
                    index_gc_stats.edge_entries_removed
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
        ts: Timestamp,
    ) -> StorageResult<()> {
        super::index_engine::delete_vertex_indexes_mvcc(self, space_id, vertex_id, ts)
    }

    pub(crate) fn update_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        super::index_engine::update_edge_indexes_mvcc(
            self, space_id, src, dst, index_name, props, ts,
        )
    }

    pub(crate) fn delete_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
        ts: Timestamp,
    ) -> StorageResult<()> {
        super::index_engine::delete_edge_indexes_mvcc(self, space_id, src, dst, index_names, ts)
    }

    pub(crate) fn gc_index_tombstones(&self, ts: Timestamp) -> StorageResult<GcStats> {
        self.persistent.index_data_manager.write().gc_tombstones(ts)
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
}

impl std::fmt::Debug for GraphStorageContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStorageContext")
            .field("work_dir", &self.persistent.layout.work_dir())
            .field("db_path", &self.persistent.layout.db_path())
            .finish()
    }
}

impl Default for GraphStorageContext {
    fn default() -> Self {
        Self::new()
    }
}
