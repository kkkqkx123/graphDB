//! Property Graph Storage
//!
//! Main entry point for property graph storage combining vertex and edge tables.
//! This module acts as a facade that delegates to specialized sub-modules.

mod core_ops;
mod flush;
mod index_mvcc;
mod transaction_targets;
mod type_ops;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::cache::RecordCacheStats;
use crate::storage::edge::{
    EdgeId, EdgeRecord, EdgeStrategy, EdgeTable, PropertyDef as EdgePropertyDef,
};
use crate::storage::engine::edge::CreateEdgeTypeParams;
use crate::storage::memory::{MemoryTracker, SharedMemoryTracker};
use crate::storage::vertex::vertex_table::VertexIterator;
use crate::storage::vertex::{
    LabelId, PropertyDef as VertexPropertyDef, VertexRecord, VertexTable,
};
use crate::transaction::wal::types::Timestamp;
use crate::transaction::wal::writer::WalWriter;

use super::cache::CacheManager;
use super::config::PropertyGraphConfig;
use super::edge::EdgeOps;
use super::query::QueryOps;
use super::schema::SchemaOps;
use super::wal_manager::WalManager;
use crate::storage::index::secondary::{InMemoryIndexDataManager, GcStats};
use crate::storage::metadata::{TableId, TableTracker, TableTrackerConfig, TableType};

pub(crate) const DATA_FORMAT_VERSION: u32 = 1;

pub struct PropertyGraph {
    pub(crate) schema_ops: SchemaOps,
    pub(crate) edge_ops: EdgeOps,
    pub(crate) cache_manager: CacheManager,
    pub(crate) wal_manager: WalManager,
    pub(crate) table_tracker: Arc<TableTracker>,
    pub(crate) memory_tracker: SharedMemoryTracker,
    pub(crate) config: PropertyGraphConfig,
    pub(crate) is_open: bool,
    pub(crate) last_compacted_vertices: Vec<(LabelId, Vec<String>)>,
    pub(crate) index_data_manager: InMemoryIndexDataManager,
}

impl std::fmt::Debug for PropertyGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyGraph")
            .field("vertex_tables", &self.schema_ops.vertex_tables)
            .field("edge_tables", &self.edge_ops.edge_tables)
            .field("vertex_label_names", &self.schema_ops.vertex_label_names)
            .field("edge_label_names", &self.edge_ops.edge_label_names)
            .field("vertex_label_counter", &self.schema_ops.vertex_label_counter)
            .field("edge_label_counter", &self.edge_ops.edge_label_counter)
            .field("config", &self.config)
            .field("is_open", &self.is_open)
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
        let memory_tracker = Arc::new(MemoryTracker::new(config.memory_config.clone()));

        let cache_manager = CacheManager::new(
            config.enable_cache,
            config.cache_memory,
            memory_tracker.clone(),
        );

        let table_tracker = Arc::new(TableTracker::with_config(
            TableTrackerConfig {
                flush_threshold: config.flush_config.flush_threshold,
                flush_interval: config.flush_config.flush_interval,
            },
        ));

        Self {
            schema_ops: SchemaOps::new(),
            edge_ops: EdgeOps::new(),
            cache_manager,
            wal_manager: WalManager::new(),
            table_tracker,
            memory_tracker,
            config,
            is_open: true,
            last_compacted_vertices: Vec::new(),
            index_data_manager: InMemoryIndexDataManager::new(),
        }
    }

    pub fn with_wal(mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) -> Self {
        self.wal_manager.set_wal_writer(wal_writer);
        self
    }

    pub fn set_wal_writer(&mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.wal_manager.set_wal_writer(wal_writer);
    }

    pub fn wal_enabled(&self) -> bool {
        self.wal_manager.is_enabled()
    }

    pub fn memory_tracker(&self) -> &SharedMemoryTracker {
        &self.memory_tracker
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
        self.table_tracker.mark_modified_since_checkpoint(TableId::vertex(label));
    }

    pub fn mark_edge_modified_since_checkpoint(&self, label: LabelId) {
        self.table_tracker.mark_modified_since_checkpoint(TableId::edge(label));
    }

    pub fn take_last_compacted_vertices(&mut self) -> Vec<(LabelId, Vec<String>)> {
        std::mem::take(&mut self.last_compacted_vertices)
    }

    pub fn record_cache(&self) -> Option<&crate::storage::cache::SharedRecordCache> {
        self.cache_manager.record_cache()
    }

    pub fn record_cache_stats(&self) -> Option<RecordCacheStats> {
        self.cache_manager.record_cache_stats()
    }

    pub fn memory_stats(&self) -> Option<crate::storage::memory::MemoryStats> {
        Some(self.memory_tracker.stats())
    }

    pub fn clear_cache(&self) {
        self.cache_manager.clear_cache();
    }

    pub fn with_edge_property_cache(
        mut self,
        config: crate::storage::cache::EdgePropertyCacheConfig,
    ) -> Self {
        self.cache_manager = self.cache_manager.with_edge_property_cache(config);
        self
    }

    pub fn set_edge_property_cache(
        &mut self,
        config: crate::storage::cache::EdgePropertyCacheConfig,
    ) {
        self.cache_manager.set_edge_property_cache(config);
    }

    pub fn edge_cache_stats(
        &self,
    ) -> Option<std::sync::Arc<crate::storage::cache::EdgePropertyCacheStats>> {
        self.cache_manager.edge_cache_stats()
    }

    pub fn cache_manager(&self) -> &CacheManager {
        &self.cache_manager
    }

    pub fn open<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let config = PropertyGraphConfig {
            work_dir: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        let mut graph = Self::with_config(config);
        graph.load_data()?;
        Ok(graph)
    }

    pub fn close(&mut self) {
        self.is_open = false;
        for table in self.schema_ops.vertex_tables.values_mut() {
            table.close();
        }
        for table in self.edge_ops.edge_tables.values_mut() {
            table.close();
        }
    }

    // ==================== Schema Operations ====================

    pub fn create_vertex_type(
        &mut self,
        name: &str,
        properties: Vec<VertexPropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        type_ops::create_vertex_type(self, name, properties, primary_key)
    }

    pub fn create_vertex_type_with_id(
        &mut self,
        name: &str,
        label_id: LabelId,
        properties: Vec<VertexPropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        type_ops::create_vertex_type_with_id(self, name, label_id, properties, primary_key)
    }

    pub fn create_edge_type(
        &mut self,
        name: &str,
        src_label: LabelId,
        dst_label: LabelId,
        properties: Vec<EdgePropertyDef>,
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
        &mut self,
        params: CreateEdgeTypeParams,
        label_id: LabelId,
    ) -> StorageResult<LabelId> {
        type_ops::create_edge_type_with_id(self, params, label_id)
    }

    pub fn drop_vertex_type(&mut self, name: &str) -> StorageResult<()> {
        type_ops::drop_vertex_type(self, name)
    }

    pub fn drop_edge_type(&mut self, name: &str) -> StorageResult<()> {
        type_ops::drop_edge_type(self, name)
    }

    // ==================== Vertex Operations ====================

    pub fn insert_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        core_ops::insert_vertex(self, label, external_id, properties, ts)
    }

    pub fn get_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        core_ops::get_vertex(self, label, external_id, ts)
    }

    pub fn get_vertex_by_internal_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        core_ops::get_vertex_by_internal_id(self, label, internal_id, ts)
    }

    pub fn delete_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> StorageResult<()> {
        core_ops::delete_vertex(self, label, external_id, ts)
    }

    pub fn update_vertex_property(
        &mut self,
        label: LabelId,
        external_id: &str,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        core_ops::update_vertex_property(self, label, external_id, property_name, value, ts)
    }

    pub fn vertex_label_ids(&self) -> Vec<LabelId> {
        self.schema_ops.vertex_tables.keys().copied().collect()
    }

    // ==================== Edge Operations ====================

    pub fn insert_edge(&mut self, params: InsertEdgeParams) -> StorageResult<EdgeId> {
        core_ops::insert_edge(self, params)
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

    pub fn delete_edge(
        &mut self,
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
        &mut self,
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

    pub fn scan_vertices(&self, label: LabelId, ts: Timestamp) -> Option<VertexIterator<'_>> {
        if !self.is_open {
            return None;
        }
        QueryOps::scan_vertices(&self.schema_ops.vertex_tables, label, ts)
    }

    pub fn vertex_count(&self, label: LabelId, ts: Timestamp) -> usize {
        if !self.is_open {
            return 0;
        }
        QueryOps::vertex_count(&self.schema_ops.vertex_tables, label, ts)
    }

    pub fn edge_count(&self, edge_label: LabelId) -> u64 {
        self.edge_ops.edge_count(edge_label)
    }

    // ==================== Label Access ====================

    pub fn vertex_label_names(&self) -> Vec<&str> {
        self.schema_ops.vertex_label_names()
    }

    pub fn edge_label_names(&self) -> Vec<&str> {
        self.edge_ops.edge_label_names()
    }

    pub fn get_vertex_label_id(&self, name: &str) -> Option<LabelId> {
        self.schema_ops.get_vertex_label_id(name)
    }

    pub fn get_edge_label_id(&self, name: &str) -> Option<LabelId> {
        self.edge_ops.get_edge_label_id(name)
    }

    // ==================== Table Access ====================

    pub fn get_vertex_table(&self, label: LabelId) -> Option<&VertexTable> {
        self.schema_ops.get_vertex_table(label)
    }

    pub fn get_edge_table(
        &self,
        src_label: LabelId,
        dst_label: LabelId,
        edge_label: LabelId,
    ) -> Option<&EdgeTable> {
        self.edge_ops.get_edge_table(src_label, dst_label, edge_label)
    }

    pub fn get_edge_table_by_label(&self, edge_label: LabelId) -> Option<&EdgeTable> {
        self.edge_ops.get_edge_table_by_label(edge_label)
    }

    pub fn edge_tables(&self) -> impl Iterator<Item = (&(LabelId, LabelId, LabelId), &EdgeTable)> {
        self.edge_ops.edge_tables()
    }

    pub fn vertex_tables(&self) -> &HashMap<LabelId, VertexTable> {
        self.schema_ops.vertex_tables()
    }

    // ==================== Persistence Operations ====================

    pub fn flush(&self) -> StorageResult<()> {
        flush::flush(self)
    }

    pub fn flush_incremental(&self) -> StorageResult<Vec<TableId>> {
        flush::flush_incremental(self)
    }

    pub fn flush_tables_to_dir(&self, data_dir: &Path) -> StorageResult<()> {
        flush::flush_tables_to_dir(self, data_dir)
    }

    pub fn load(&mut self) -> StorageResult<()> {
        self.load_data()
    }

    pub(crate) fn load_data(&mut self) -> StorageResult<()> {
        flush::load_data(self)
    }

    pub fn restore_from_checkpoint(&mut self, checkpoint_dir: &Path) -> StorageResult<()> {
        flush::restore_from_checkpoint(self, checkpoint_dir)
    }

    // ==================== Compaction Operations ====================

    pub fn compact_vertex_table(&mut self, label: LabelId) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if let Some(table) = self.schema_ops.vertex_tables.get_mut(&label) {
            table.compact();
            self.cache_manager.invalidate_vertices_by_label(label);
        }

        Ok(())
    }

    pub fn compact_vertex_table_with_ts(&mut self, label: LabelId, ts: Timestamp) -> Vec<String> {
        let removed = self
            .schema_ops
            .vertex_tables
            .get_mut(&label)
            .map(|table| table.compact_with_ts_collect(ts))
            .unwrap_or_default();
        if !removed.is_empty() {
            self.last_compacted_vertices.push((label, removed.clone()));
        }
        self.cache_manager.invalidate_vertices_by_label(label);
        removed
    }

    // ==================== Index Operations ====================

    pub fn index_data_manager(&self) -> &InMemoryIndexDataManager {
        &self.index_data_manager
    }

    pub fn index_data_manager_mut(&mut self) -> &mut InMemoryIndexDataManager {
        &mut self.index_data_manager
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

    pub fn gc_index_tombstones(&mut self, ts: Timestamp) -> StorageResult<GcStats> {
        index_mvcc::gc_index_tombstones(self, ts)
    }

    pub fn gc_index_tombstones_incremental(
        &self,
        ts: Timestamp,
        batch_size: usize,
    ) -> StorageResult<GcStats> {
        index_mvcc::gc_index_tombstones_incremental(self, ts, batch_size)
    }
}
