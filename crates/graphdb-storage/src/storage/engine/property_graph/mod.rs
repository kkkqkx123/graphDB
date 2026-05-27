//! Property Graph Storage
//!
//! Main entry point for property graph storage combining vertex and edge tables.
//! This module acts as a facade that delegates to specialized sub-modules.

mod core_ops;
mod flush;
mod index_mvcc;
mod type_ops;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::core::types::{LabelId, Timestamp};
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::cache::RecordCacheStats;
use crate::storage::edge::{EdgeRecord, EdgeStrategy};
use crate::storage::engine::edge_params::CreateEdgeTypeParams;
use crate::storage::engine::data_store::EdgeTableKey;
use crate::storage::storage_types::{EdgeOffset, StoragePropertyDef};
use crate::storage::vertex::VertexRecord;
use crate::transaction::wal::writer::WalWriter;

use super::cache_manager::CacheManager;
use super::config::PropertyGraphConfig;
use super::data_store::GraphDataStore;
use super::wal_manager::WalManager;
use crate::storage::index::secondary::{GcStats, IndexDataManagerImpl, IndexGcOps};
use crate::core::types::{TableId, TableTracker, TableTrackerConfig, TableType};

pub(crate) const DATA_FORMAT_VERSION: u32 = 1;

pub struct PropertyGraph {
    pub(crate) data_store: GraphDataStore,
    pub(crate) cache_manager: CacheManager,
    pub(crate) wal_manager: Mutex<WalManager>,
    pub(crate) table_tracker: Arc<TableTracker>,
    pub(crate) config: PropertyGraphConfig,
    pub(crate) is_open: AtomicBool,
    pub(crate) last_compacted_vertices: Mutex<Vec<(LabelId, Vec<crate::storage::vertex::IdKey>)>>,
    pub(crate) index_data_manager: RwLock<IndexDataManagerImpl>,
}

impl std::fmt::Debug for PropertyGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vertex_tables = self.data_store.vertex_tables().read();
        let edge_tables = self.data_store.edge_tables().read();
        let vertex_label_names = self.data_store.vertex_label_names().read();
        let edge_label_names = self.data_store.edge_label_names().read();
        f.debug_struct("PropertyGraph")
            .field("vertex_tables", &vertex_tables)
            .field("edge_tables", &edge_tables)
            .field("vertex_label_names", &vertex_label_names)
            .field("edge_label_names", &edge_label_names)
            .field("vertex_label_counter", &self.data_store.vertex_label_counter().read())
            .field("edge_label_counter", &self.data_store.edge_label_counter().read())
            .field("config", &self.config)
            .field("is_open", &self.is_open.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl Default for PropertyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters for insert_edge operation
pub struct InsertEdgeParams<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: &'a str,
    pub dst_label: LabelId,
    pub dst_id: &'a str,
    pub properties: &'a [(String, Value)],
    pub ts: Timestamp,
}

/// Parameters for insert_edge operation with i64 vertex IDs
pub struct InsertEdgeParamsByI64<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: i64,
    pub dst_label: LabelId,
    pub dst_id: i64,
    pub properties: &'a [(String, Value)],
    pub ts: Timestamp,
}

/// Parameters for update_edge_property operation in PropertyGraph
pub struct PropertyGraphUpdateEdgePropertyParams<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: &'a str,
    pub dst_label: LabelId,
    pub dst_id: &'a str,
    pub prop_name: &'a str,
    pub value: &'a Value,
    pub ts: Timestamp,
}

impl PropertyGraph {
    pub fn new() -> Self {
        Self::with_config(PropertyGraphConfig::default())
    }

    pub fn with_config(config: PropertyGraphConfig) -> Self {
        let cache_manager = CacheManager::new(
            config.enable_cache,
            config.cache_memory,
        );

        let table_tracker = Arc::new(TableTracker::with_config(TableTrackerConfig {
            flush_threshold: config.flush_config.flush_threshold,
            flush_interval: config.flush_config.flush_interval,
        }));

        Self {
            data_store: GraphDataStore::new(),
            cache_manager,
            wal_manager: Mutex::new(WalManager::new()),
            table_tracker,
            config,
            is_open: AtomicBool::new(true),
            last_compacted_vertices: Mutex::new(Vec::new()),
            index_data_manager: RwLock::new(IndexDataManagerImpl::new()),
        }
    }

    pub fn with_wal(self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) -> Self {
        self.wal_manager.lock().set_wal_writer(wal_writer);
        self
    }

    pub fn set_wal_writer(&self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.wal_manager.lock().set_wal_writer(wal_writer);
    }

    pub fn wal_enabled(&self) -> bool {
        self.wal_manager.lock().is_enabled()
    }

    pub fn table_tracker(&self) -> &Arc<TableTracker> {
        &self.table_tracker
    }

    pub fn should_flush(&self) -> bool {
        self.table_tracker.should_flush()
    }

    pub fn get_modified_table_count(&self) -> usize {
        self.table_tracker.get_modified_count()
    }

    pub fn mark_table_modified(&self, table_type: TableType, label_id: u32) {
        let table_id = TableId::new(table_type, label_id);
        self.table_tracker.mark_modified(table_id);
    }

    pub fn mark_vertex_modified(&self, label: LabelId) {
        self.table_tracker.mark_modified(TableId::vertex(label));
    }

    pub fn mark_edge_modified(&self, label: LabelId) {
        self.table_tracker.mark_modified(TableId::edge(label));
    }

    pub fn mark_vertex_modified_since_checkpoint(&self, label: LabelId) {
        self.table_tracker
            .mark_modified_since_checkpoint(TableId::vertex(label));
    }

    pub fn mark_edge_modified_since_checkpoint(&self, label: LabelId) {
        self.table_tracker
            .mark_modified_since_checkpoint(TableId::edge(label));
    }

    pub fn take_last_compacted_vertices(
        &self,
    ) -> Vec<(LabelId, Vec<crate::storage::vertex::IdKey>)> {
        std::mem::take(&mut *self.last_compacted_vertices.lock())
    }

    pub fn record_cache(&self) -> Option<&crate::storage::cache::SharedRecordCache> {
        self.cache_manager.record_cache()
    }

    pub fn record_cache_stats(&self) -> Option<RecordCacheStats> {
        self.cache_manager.record_cache_stats()
    }

    pub fn clear_cache(&self) {
        self.cache_manager.clear_cache();
    }

    pub fn cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }

    pub fn open<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let config = PropertyGraphConfig {
            work_dir: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        let graph = Self::with_config(config);
        graph.load_data()?;
        Ok(graph)
    }

    pub fn close(&self) {
        self.is_open.store(false, Ordering::Release);
        {
            let mut vertex_tables = self.data_store.vertex_tables().write();
            for table in vertex_tables.values_mut() {
                table.close();
            }
        }
        {
            let mut edge_tables = self.data_store.edge_tables().write();
            for table in edge_tables.values_mut() {
                table.close();
            }
        }
    }

    pub(crate) fn storage_size(&self) -> usize {
        let mut total = 0usize;

        {
            let vertex_tables = self.data_store.vertex_tables().read();
            for table in vertex_tables.values() {
                total += table.memory_size();
            }
        }
        {
            let edge_tables = self.data_store.edge_tables().read();
            for table in edge_tables.values() {
                total += table.memory_size();
            }
        }

        total
    }

    pub(crate) fn used_storage_size(&self) -> usize {
        let mut total = 0usize;

        {
            let vertex_tables = self.data_store.vertex_tables().read();
            for table in vertex_tables.values() {
                total += table.used_memory_size();
            }
        }
        {
            let edge_tables = self.data_store.edge_tables().read();
            for table in edge_tables.values() {
                total += table.used_memory_size();
            }
        }

        total
    }

    // ==================== Schema Operations ====================

    pub fn create_vertex_type(
        &self,
        name: &str,
        properties: Vec<StoragePropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        type_ops::create_vertex_type(self, name, properties, primary_key)
    }

    pub fn create_vertex_type_with_id(
        &self,
        name: &str,
        label_id: LabelId,
        properties: Vec<StoragePropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        type_ops::create_vertex_type_with_id(self, name, label_id, properties, primary_key)
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
        type_ops::create_edge_type(
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
        type_ops::create_edge_type_with_id(self, params, label_id)
    }

    pub fn drop_vertex_type(&self, name: &str) -> StorageResult<()> {
        type_ops::drop_vertex_type(self, name)
    }

    pub fn drop_edge_type(&self, name: &str) -> StorageResult<()> {
        type_ops::drop_edge_type(self, name)
    }

    pub fn add_vertex_property(
        &self,
        label: LabelId,
        prop: crate::storage::storage_types::StoragePropertyDef,
    ) -> StorageResult<()> {
        type_ops::add_vertex_property(self, label, prop)
    }

    pub fn add_edge_property(
        &self,
        edge_label: LabelId,
        prop: crate::storage::storage_types::StoragePropertyDef,
    ) -> StorageResult<()> {
        type_ops::add_edge_property(self, edge_label, prop)
    }

    // ==================== Vertex Operations ====================

    pub fn insert_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        core_ops::insert_vertex(self, label, external_id, properties, ts)
    }

    pub fn insert_vertex_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        core_ops::insert_vertex_by_i64(self, label, external_id, properties, ts)
    }

    pub fn get_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        core_ops::get_vertex(self, label, external_id, ts)
    }

    pub fn get_vertex_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        core_ops::get_vertex_by_i64(self, label, external_id, ts)
    }

    pub fn get_vertex_by_internal_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        core_ops::get_vertex_by_internal_id(self, label, internal_id, ts)
    }

    pub fn get_external_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<String> {
        let vertex_tables = self.data_store.vertex_tables().read();
        vertex_tables
            .get(&label)?
            .get_external_id(internal_id, ts)
            .map(|k| k.to_string())
    }

    pub fn get_external_id_any(&self, internal_id: u32, ts: Timestamp) -> Option<String> {
        let vertex_tables = self.data_store.vertex_tables().read();
        vertex_tables.values()
            .find_map(|t| t.get_external_id(internal_id, ts))
            .map(|k| k.to_string())
    }

    pub fn delete_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> StorageResult<()> {
        core_ops::delete_vertex(self, label, external_id, ts)
    }

    pub fn delete_vertex_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        ts: Timestamp,
    ) -> StorageResult<()> {
        core_ops::delete_vertex_by_i64(self, label, external_id, ts)
    }

    pub fn update_vertex_property(
        &self,
        label: LabelId,
        external_id: &str,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        core_ops::update_vertex_property(self, label, external_id, property_name, value, ts)
    }

    pub fn update_vertex_property_by_i64(
        &self,
        label: LabelId,
        external_id: i64,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        core_ops::update_vertex_property_by_i64(self, label, external_id, property_name, value, ts)
    }

    pub fn vertex_label_ids(&self) -> Vec<LabelId> {
        self.data_store.vertex_tables()
            .read()
            .keys()
            .copied()
            .collect()
    }

    // ==================== Edge Operations ====================

    pub fn insert_edge(&self, params: InsertEdgeParams) -> StorageResult<EdgeOffset> {
        core_ops::insert_edge(self, params)
    }

    pub fn insert_edge_by_i64(&self, params: InsertEdgeParamsByI64) -> StorageResult<EdgeOffset> {
        core_ops::insert_edge_by_i64(self, params)
    }

    pub fn get_edge(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
    ) -> Option<EdgeRecord> {
        core_ops::get_edge(self, edge_label, src_label, src_id, dst_label, dst_id, ts)
    }

    pub fn get_edge_by_i64(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: i64,
        dst_label: LabelId,
        dst_id: i64,
        ts: Timestamp,
    ) -> Option<EdgeRecord> {
        core_ops::get_edge_by_i64(self, edge_label, src_label, src_id, dst_label, dst_id, ts)
    }

    pub fn delete_edge(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        core_ops::delete_edge(self, edge_label, src_label, src_id, dst_label, dst_id, ts)
    }

    pub fn update_edge_property(
        &self,
        params: PropertyGraphUpdateEdgePropertyParams,
    ) -> StorageResult<bool> {
        core_ops::update_edge_property(self, params)
    }

    pub fn out_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        src_id: &str,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        core_ops::out_edges(self, edge_label, src_label, dst_label, src_id, ts)
    }

    pub fn in_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        core_ops::in_edges(self, edge_label, src_label, dst_label, dst_id, ts)
    }

    // ==================== Query Operations ====================

    pub fn scan_vertices(&self, label: LabelId, ts: Timestamp) -> Option<Vec<VertexRecord>> {
        if !self.is_open.load(Ordering::Acquire) {
            return None;
        }
        let vertex_tables = self.data_store.vertex_tables().read();
        vertex_tables.get(&label).map(|t| t.scan(ts).collect())
    }

    pub fn vertex_count(&self, label: LabelId, ts: Timestamp) -> usize {
        if !self.is_open.load(Ordering::Acquire) {
            return 0;
        }
        let vertex_tables = self.data_store.vertex_tables().read();
        vertex_tables
            .get(&label)
            .map(|t| t.vertex_count(ts))
            .unwrap_or(0)
    }

    pub fn edge_count(&self, edge_label: LabelId) -> u64 {
        self.data_store.edge_tables()
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

    // ==================== Label Access ====================

    pub fn vertex_label_names(&self) -> Vec<String> {
        self.data_store.vertex_label_names()
            .read()
            .keys()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn edge_label_names(&self) -> Vec<String> {
        self.data_store.edge_label_names()
            .read()
            .keys()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn get_vertex_label_id(&self, name: &str) -> Option<LabelId> {
        self.data_store.vertex_label_names().read().get(name).copied()
    }

    pub fn get_edge_label_id(&self, name: &str) -> Option<LabelId> {
        self.data_store.edge_label_names().read().get(name).copied()
    }

    // ==================== Table Access ====================

    pub fn get_vertex_table_opt(&self, label: LabelId) -> Option<String> {
        self.data_store.vertex_tables()
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
        self.data_store.edge_tables()
            .read()
            .get(&EdgeTableKey::new(src_label, dst_label, edge_label))
            .map(|t| t.scan(ts))
            .unwrap_or_default()
    }

    pub fn scan_edges_by_label(&self, edge_label: LabelId, ts: Timestamp) -> Vec<EdgeRecord> {
        self.data_store.edge_tables()
            .read()
            .values()
            .find(|t| t.label() == edge_label)
            .map(|t| t.scan(ts))
            .unwrap_or_default()
    }

    pub fn total_vertex_count(&self) -> usize {
        self.data_store.vertex_tables().read().values().map(|t| t.total_count()).sum()
    }

    pub fn total_edge_count(&self) -> usize {
        self.data_store.edge_tables()
            .read()
            .values()
            .map(|t| t.edge_count() as usize)
            .sum()
    }

    pub fn collect_all_edge_records(
        &self,
        ts: Timestamp,
    ) -> Vec<(LabelId, LabelId, LabelId, EdgeRecord)> {
        let edge_tables = self.data_store.edge_tables().read();
        let mut records = Vec::new();
        for (EdgeTableKey { src_label, dst_label, edge_label }, table) in &*edge_tables {
            for edge_record in table.scan(ts) {
                records.push((*src_label, *dst_label, *edge_label, edge_record));
            }
        }
        records
    }

    // ==================== Persistence Operations ====================

    pub fn flush_to_disk(&self) -> StorageResult<()> {
        flush::flush_to_disk_impl(self)
    }

    pub fn flush_incremental(&self) -> StorageResult<Vec<TableId>> {
        flush::flush_incremental(self)
    }

    pub fn flush_tables_to_dir(&self, data_dir: &Path) -> StorageResult<()> {
        flush::flush_tables_to_dir(self, data_dir)
    }

    pub fn load(&self) -> StorageResult<()> {
        self.load_data()
    }

    pub(crate) fn load_data(&self) -> StorageResult<()> {
        flush::load_data(self)
    }

    pub fn restore_from_checkpoint(&self, checkpoint_dir: &Path) -> StorageResult<()> {
        flush::restore_from_checkpoint(self, checkpoint_dir)
    }

    // ==================== Compaction Operations ====================

    pub fn compact_vertex_table(&self, label: LabelId) -> StorageResult<()> {
        if !self.is_open.load(Ordering::Acquire) {
            return Err(StorageError::storage_not_open());
        }

        {
            let mut vertex_tables = self.data_store.vertex_tables().write();
            if let Some(table) = vertex_tables.get_mut(&label) {
                table.compact();
            }
        }
        self.cache_manager.invalidate_vertices_by_label(label);

        Ok(())
    }

    pub fn compact_vertex_table_with_ts(
        &self,
        label: LabelId,
        ts: Timestamp,
    ) -> Vec<crate::storage::vertex::IdKey> {
        let removed = {
            let mut vertex_tables = self.data_store.vertex_tables().write();
            vertex_tables
                .get_mut(&label)
                .map(|table| table.compact_with_ts_collect(ts))
                .unwrap_or_default()
        };
        if !removed.is_empty() {
            self.last_compacted_vertices
                .lock()
                .push((label, removed.clone()));
        }
        self.cache_manager.invalidate_vertices_by_label(label);
        removed
    }

    // ==================== Index Operations ====================

    pub fn index_data_manager(&self) -> &RwLock<IndexDataManagerImpl> {
        &self.index_data_manager
    }

    pub fn update_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
        props: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        index_mvcc::update_vertex_indexes_mvcc(self, space_id, vertex_id, index_name, props, ts)
    }

    pub fn delete_vertex_indexes_mvcc(
        &self,
        space_id: u64,
        vertex_id: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        index_mvcc::delete_vertex_indexes_mvcc(self, space_id, vertex_id, ts)
    }

    pub fn update_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
        props: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        index_mvcc::update_edge_indexes_mvcc(self, space_id, src, dst, index_name, props, ts)
    }

    pub fn delete_edge_indexes_mvcc(
        &self,
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_names: &[String],
        ts: Timestamp,
    ) -> StorageResult<()> {
        index_mvcc::delete_edge_indexes_mvcc(self, space_id, src, dst, index_names, ts)
    }

    pub fn gc_index_tombstones(&self, ts: Timestamp) -> StorageResult<GcStats> {
        self.index_data_manager.write().gc_tombstones(ts)
    }

    pub fn gc_index_tombstones_incremental(
        &self,
        ts: Timestamp,
        batch_size: usize,
    ) -> StorageResult<GcStats> {
        self.index_data_manager
            .read()
            .gc_tombstones_incremental(ts, batch_size)
    }
}
