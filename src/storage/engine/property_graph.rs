//! Property Graph Storage
//!
//! Main entry point for property graph storage combining vertex and edge tables.
//! This module acts as a facade that delegates to specialized sub-modules.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::cache::{
    RecordCacheStats, SharedRecordCache,
};
use crate::storage::edge::{
    EdgeId, EdgeRecord, EdgeStrategy, EdgeTable,
    PropertyDef as EdgePropertyDef,
};
use crate::storage::memory::{MemoryTracker, SharedMemoryTracker};
use crate::storage::persistence::{
    DirtyPageTracker, FlushManager, PageId,
    TableType as PersistenceTableType,
};
use crate::storage::vertex::vertex_table::VertexIterator;
use crate::storage::vertex::{
    LabelId, PropertyDef as VertexPropertyDef,
    VertexRecord, VertexTable,
};
use crate::transaction::insert_transaction::{
    InsertTarget, InsertTransactionResult,
};
use crate::transaction::undo_log::{PropertyValue, UndoLogResult, UndoTarget};
use crate::transaction::wal::types::{
    ColumnId, LabelId as TxnLabelId, Timestamp, VertexId as TxnVertexId,
};
use crate::transaction::wal::writer::WalWriter;

use super::cache::CacheManager;
use super::config::PropertyGraphConfig;
use super::edge::EdgeOps;
use super::flush::FlushManagerWrapper;
use super::persistence::PersistenceOps;
use super::query::QueryOps;
use super::schema::SchemaOps;
use super::transaction::TransactionOps;

const DATA_FORMAT_VERSION: u32 = 1;

pub struct PropertyGraph {
    schema_ops: SchemaOps,
    edge_ops: EdgeOps,
    cache_manager: CacheManager,
    flush_manager: FlushManagerWrapper,
    config: PropertyGraphConfig,
    is_open: bool,
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
            .finish()
    }
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

        let flush_manager = FlushManagerWrapper::new(
            config.enable_incremental_flush,
            config.flush_threshold,
            config.flush_interval_secs,
            config.compression.clone(),
            config.work_dir.clone(),
        );

        Self {
            schema_ops: SchemaOps::new(),
            edge_ops: EdgeOps::new(),
            cache_manager,
            flush_manager,
            config,
            is_open: true,
        }
    }

    pub fn with_wal(mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) -> Self {
        self.flush_manager.set_wal_writer(wal_writer);
        self
    }

    pub fn set_wal_writer(&mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.flush_manager.set_wal_writer(wal_writer);
    }

    pub fn wal_enabled(&self) -> bool {
        self.flush_manager.wal_enabled()
    }

    pub fn record_cache(&self) -> Option<&SharedRecordCache> {
        self.cache_manager.record_cache()
    }

    pub fn memory_tracker(&self) -> Option<&SharedMemoryTracker> {
        self.cache_manager.memory_tracker()
    }

    pub fn record_cache_stats(&self) -> Option<RecordCacheStats> {
        self.cache_manager.record_cache_stats()
    }

    pub fn memory_stats(&self) -> Option<crate::storage::memory::MemoryStats> {
        self.cache_manager.memory_stats()
    }

    pub fn clear_cache(&self) {
        self.cache_manager.clear_cache();
    }

    pub fn dirty_tracker(&self) -> Option<&Arc<DirtyPageTracker>> {
        self.flush_manager.dirty_tracker()
    }

    pub fn flush_manager(&self) -> Option<&Arc<FlushManager>> {
        self.flush_manager.flush_manager()
    }

    pub fn get_dirty_page_count(&self) -> usize {
        self.flush_manager.get_dirty_page_count()
    }

    pub fn should_flush(&self) -> bool {
        self.flush_manager.should_flush()
    }

    pub fn flush_dirty_pages(&self) -> StorageResult<Vec<PageId>> {
        let pages = self.flush_manager.flush_dirty_pages()?;
        if !pages.is_empty() {
            self.flush_pages(&pages)?;
        }
        Ok(pages)
    }

    fn flush_pages(&self, pages: &[PageId]) -> StorageResult<()> {
        use std::fs;

        let data_dir = self.config.work_dir.join("data");
        fs::create_dir_all(&data_dir).map_err(|e| StorageError::IOError(e.to_string()))?;

        for page_id in pages {
            match page_id.table_type {
                PersistenceTableType::Vertex => {
                    let vertex_dir = data_dir.join("vertices");
                    fs::create_dir_all(&vertex_dir)
                        .map_err(|e| StorageError::IOError(e.to_string()))?;

                    let table_dir = vertex_dir.join(format!("label_{}", page_id.label_id));
                    if let Some(table) = self.schema_ops.vertex_tables.get(&page_id.label_id) {
                        table.flush(&table_dir)?;
                    }
                }
                PersistenceTableType::Edge => {
                    let edge_dir = data_dir.join("edges");
                    fs::create_dir_all(&edge_dir)
                        .map_err(|e| StorageError::IOError(e.to_string()))?;

                    let src_label = (page_id.block_number >> 32) as LabelId;
                    let dst_label = page_id.block_number as LabelId;
                    let key = (src_label, dst_label, page_id.label_id);
                    let table_dir =
                        edge_dir.join(format!("{}_{}_{}", src_label, dst_label, page_id.label_id));
                    if let Some(table) = self.edge_ops.edge_tables.get(&key) {
                        table.flush(&table_dir)?;
                    }
                }
                _ => {}
            }
        }

        self.flush_manager.sync_wal()?;

        Ok(())
    }

    pub fn start_background_flush(&self) -> StorageResult<()> {
        self.flush_manager.start_background_flush()
    }

    pub fn stop_background_flush(&self) {
        self.flush_manager.stop_background_flush();
    }

    fn write_wal(&self, _op_type: u8, _data: &[u8]) -> StorageResult<()> {
        self.flush_manager.write_wal(_data)
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

    pub fn create_vertex_type(
        &mut self,
        name: &str,
        properties: Vec<VertexPropertyDef>,
        primary_key: &str,
    ) -> StorageResult<LabelId> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        let label_id = self.schema_ops.create_vertex_type(name, properties, primary_key)?;
        self.mark_vertex_dirty(label_id);
        Ok(label_id)
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
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        let label_id = self.edge_ops.create_edge_type(
            name,
            src_label,
            dst_label,
            properties,
            oe_strategy,
            ie_strategy,
            &self.schema_ops.vertex_tables,
        )?;
        self.mark_edge_dirty(src_label, dst_label, label_id);
        Ok(label_id)
    }

    pub fn drop_vertex_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        let label_id = self
            .schema_ops
            .vertex_label_names
            .get(name)
            .copied()
            .ok_or_else(|| StorageError::LabelNotFound(name.to_string()))?;
        self.schema_ops.drop_vertex_type(name)?;
        self.edge_ops.drop_edges_for_vertex_label(label_id);
        Ok(())
    }

    pub fn drop_edge_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        self.edge_ops.drop_edge_type(name)
    }

    pub fn insert_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<u32> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        let result = self
            .schema_ops
            .insert_vertex(label, external_id, properties, ts)?;
        self.mark_vertex_dirty(label);
        Ok(result)
    }

    pub fn get_vertex(
        &self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        if !self.is_open {
            return None;
        }

        let internal_id = self
            .cache_manager
            .get_cached_vertex_id(label, external_id)
            .or_else(|| {
                let id = self
                    .schema_ops
                    .get_vertex_internal_id(label, external_id, ts)?;
                self.cache_manager
                    .cache_vertex_id(label, external_id, id);
                Some(id)
            })?;

        if let Some(cached) = self.cache_manager.get_cached_vertex(label, internal_id) {
            return Some(VertexRecord {
                internal_id: cached.internal_id,
                vid: cached.internal_id as u64,
                properties: cached.properties,
            });
        }

        let record = self
            .schema_ops
            .get_vertex_by_internal_id(label, internal_id, ts)?;

        self.cache_manager
            .cache_vertex(label, internal_id, external_id.to_string(), record.properties.clone());

        Some(record)
    }

    pub fn get_vertex_by_internal_id(
        &self,
        label: LabelId,
        internal_id: u32,
        ts: Timestamp,
    ) -> Option<VertexRecord> {
        if !self.is_open {
            return None;
        }

        if let Some(cached) = self.cache_manager.get_cached_vertex(label, internal_id) {
            return Some(VertexRecord {
                internal_id: cached.internal_id,
                vid: cached.internal_id as u64,
                properties: cached.properties,
            });
        }

        let record = self
            .schema_ops
            .get_vertex_by_internal_id(label, internal_id, ts)?;

        self.cache_manager
            .cache_vertex(label, internal_id, String::new(), record.properties.clone());

        Some(record)
    }

    pub fn delete_vertex(
        &mut self,
        label: LabelId,
        external_id: &str,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        self.schema_ops.delete_vertex(label, external_id, ts)?;
        self.mark_vertex_dirty(label);
        Ok(())
    }

    pub fn update_vertex_property(
        &mut self,
        label: LabelId,
        external_id: &str,
        property_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        self.schema_ops.update_vertex_property(
            label,
            external_id,
            property_name,
            value,
            ts,
        )?;
        self.mark_vertex_dirty(label);
        Ok(())
    }

    pub fn insert_edge(
        &mut self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        properties: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<EdgeId> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        let edge_id = self.edge_ops.insert_edge(
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
            properties,
            ts,
            &self.schema_ops.vertex_tables,
        )?;
        self.mark_edge_dirty(src_label, dst_label, edge_label);
        Ok(edge_id)
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
        if !self.is_open {
            return None;
        }
        self.edge_ops.get_edge(
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
            ts,
            &self.schema_ops.vertex_tables,
        )
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
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        let deleted = self.edge_ops.delete_edge(
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
            ts,
            &self.schema_ops.vertex_tables,
        )?;
        if deleted {
            self.mark_edge_dirty(src_label, dst_label, edge_label);
        }
        Ok(deleted)
    }

    pub fn update_edge_property(
        &mut self,
        edge_label: LabelId,
        src_label: LabelId,
        src_id: &str,
        dst_label: LabelId,
        dst_id: &str,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }
        let updated = self.edge_ops.update_edge_property(
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
            prop_name,
            value,
            ts,
            &self.schema_ops.vertex_tables,
        )?;
        if updated {
            self.mark_edge_dirty(src_label, dst_label, edge_label);
        }
        Ok(updated)
    }

    pub fn out_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        src_id: &str,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.is_open {
            return None;
        }
        self.edge_ops.out_edges(
            edge_label,
            src_label,
            dst_label,
            src_id,
            ts,
            &self.schema_ops.vertex_tables,
        )
    }

    pub fn in_edges(
        &self,
        edge_label: LabelId,
        src_label: LabelId,
        dst_label: LabelId,
        dst_id: &str,
        ts: Timestamp,
    ) -> Option<Vec<EdgeRecord>> {
        if !self.is_open {
            return None;
        }
        self.edge_ops.in_edges(
            edge_label,
            src_label,
            dst_label,
            dst_id,
            ts,
            &self.schema_ops.vertex_tables,
        )
    }

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

    pub fn flush(&self) -> StorageResult<()> {
        PersistenceOps::flush(
            &self.schema_ops.vertex_tables,
            &self.edge_ops.edge_tables,
            &self.config.work_dir,
            &self.flush_manager,
        )
    }

    pub fn flush_tables_to_dir(&self, data_dir: &Path) -> StorageResult<()> {
        PersistenceOps::flush_tables_to_dir(
            &self.schema_ops.vertex_tables,
            &self.edge_ops.edge_tables,
            data_dir,
            &self.flush_manager,
        )
    }

    pub fn load(&mut self) -> StorageResult<()> {
        self.load_data()
    }

    fn load_data(&mut self) -> StorageResult<()> {
        PersistenceOps::load(
            &mut self.schema_ops.vertex_tables,
            &mut self.edge_ops.edge_tables,
            &self.config.work_dir,
        )
    }

    pub fn restore_from_checkpoint(&mut self, checkpoint_dir: &Path) -> StorageResult<()> {
        PersistenceOps::restore_from_checkpoint(
            &mut self.schema_ops.vertex_tables,
            &mut self.edge_ops.edge_tables,
            checkpoint_dir,
        )
    }

    fn mark_vertex_dirty(&self, label_id: LabelId) {
        self.flush_manager.mark_vertex_dirty(label_id);
    }

    fn mark_edge_dirty(&self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) {
        self.flush_manager
            .mark_edge_dirty(src_label, dst_label, edge_label);
    }
}

impl InsertTarget for PropertyGraph {
    fn add_vertex(
        &mut self,
        label: TxnLabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<TxnVertexId> {
        TransactionOps::add_vertex(&mut self.schema_ops, label, oid, properties, ts)
    }

    fn add_edge(
        &mut self,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<EdgeId> {
        TransactionOps::add_edge(
            &mut self.edge_ops,
            &self.schema_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            properties,
            ts,
        )
    }

    fn get_vertex_id(&self, label: TxnLabelId, oid: &[u8], ts: Timestamp) -> Option<TxnVertexId> {
        TransactionOps::get_vertex_id(&self.schema_ops, label, oid, ts)
    }

    fn get_vertex_oid(
        &self,
        label: TxnLabelId,
        vid: TxnVertexId,
        _ts: Timestamp,
    ) -> Option<Vec<u8>> {
        TransactionOps::get_vertex_oid(&self.schema_ops, label, vid, _ts)
    }

    fn get_vertex_property_types(&self, label: TxnLabelId) -> Vec<String> {
        TransactionOps::get_vertex_property_types(&self.schema_ops, label)
    }

    fn get_edge_property_types(
        &self,
        _src_label: TxnLabelId,
        _dst_label: TxnLabelId,
        edge_label: TxnLabelId,
    ) -> Vec<String> {
        TransactionOps::get_edge_property_types(&self.edge_ops, _src_label, _dst_label, edge_label)
    }

    fn vertex_label_num(&self) -> usize {
        TransactionOps::vertex_label_num(&self.schema_ops)
    }

    fn lid_num(&self, label: TxnLabelId) -> usize {
        TransactionOps::lid_num(&self.schema_ops, label)
    }
}

impl UndoTarget for PropertyGraph {
    fn delete_vertex_type(&mut self, label: TxnLabelId) -> UndoLogResult<()> {
        TransactionOps::delete_vertex_type(&mut self.schema_ops, &mut self.edge_ops, label)
    }

    fn delete_edge_type(
        &mut self,
        src_label: TxnLabelId,
        dst_label: TxnLabelId,
        edge_label: TxnLabelId,
    ) -> UndoLogResult<()> {
        TransactionOps::delete_edge_type(&mut self.edge_ops, src_label, dst_label, edge_label)
    }

    fn delete_vertex(
        &mut self,
        label: TxnLabelId,
        vid: TxnVertexId,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::delete_vertex(&mut self.schema_ops, label, vid, ts)
    }

    fn delete_edge(
        &mut self,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        _oe_offset: i32,
        _ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::delete_edge(
            &mut self.edge_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            ts,
        )
    }

    fn undo_update_vertex_property(
        &mut self,
        label: TxnLabelId,
        vid: TxnVertexId,
        _col_id: ColumnId,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::update_vertex_property_undo(
            &mut self.schema_ops,
            label,
            vid,
            "property",
            old_value,
            ts,
        )
    }

    fn undo_update_edge_property(
        &mut self,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        _oe_offset: i32,
        _ie_offset: i32,
        _col_id: ColumnId,
        old_value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::update_edge_property_undo(
            &mut self.edge_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            "property",
            old_value,
            ts,
        )
    }

    fn revert_delete_vertex(
        &mut self,
        label: TxnLabelId,
        vid: TxnVertexId,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::delete_vertex(&mut self.schema_ops, label, vid, ts)
    }

    fn revert_delete_edge(
        &mut self,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        _oe_offset: i32,
        _ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::delete_edge(
            &mut self.edge_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            ts,
        )
    }

    fn revert_delete_vertex_properties(
        &mut self,
        _label_name: &str,
        _prop_names: &[String],
    ) -> UndoLogResult<()> {
        Ok(())
    }

    fn revert_delete_edge_properties(
        &mut self,
        _src_label: &str,
        _dst_label: &str,
        _edge_label: &str,
        _prop_names: &[String],
    ) -> UndoLogResult<()> {
        Ok(())
    }

    fn revert_delete_vertex_label(&mut self, label_name: &str) -> UndoLogResult<()> {
        let label = self.schema_ops.vertex_label_counter;
        TransactionOps::create_vertex_type_undo(
            &mut self.schema_ops,
            label_name,
            label,
        )
    }

    fn revert_delete_edge_label(
        &mut self,
        _src_label: &str,
        _dst_label: &str,
        edge_label: &str,
    ) -> UndoLogResult<()> {
        let label = self.edge_ops.edge_label_counter;
        TransactionOps::create_edge_type_undo(
            &mut self.edge_ops,
            edge_label,
            label,
        )
    }

    fn revert_rename_vertex_properties(
        &mut self,
        label: &str,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_rename_vertex_properties(
            &mut self.schema_ops,
            label,
            current_names,
            original_names,
        )
    }

    fn revert_rename_edge_properties(
        &mut self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        current_names: &[String],
        original_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_rename_edge_properties(
            &mut self.edge_ops,
            src_label,
            dst_label,
            edge_label,
            &self.schema_ops,
            current_names,
            original_names,
        )
    }
}
