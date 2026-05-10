//! Property Graph Storage
//!
//! Main entry point for property graph storage combining vertex and edge tables.
//! This module acts as a facade that delegates to specialized sub-modules.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::{StorageError, StorageResult, Value};
use crate::storage::cache::RecordCacheStats;
use crate::storage::edge::{
    EdgeId, EdgeRecord, EdgeStrategy, EdgeTable, PropertyDef as EdgePropertyDef,
};
use crate::storage::memory::{MemoryTracker, SharedMemoryTracker};
use crate::storage::vertex::vertex_table::VertexIterator;
use crate::storage::vertex::{
    LabelId, PropertyDef as VertexPropertyDef, VertexRecord, VertexTable,
};
use crate::transaction::insert_transaction::{InsertTarget, InsertTransactionResult};
use crate::transaction::undo_log::{PropertyValue, UndoLogError, UndoLogResult, UndoTarget};
use crate::transaction::wal::types::{
    ColumnId, LabelId as TxnLabelId, Timestamp, VertexId as TxnVertexId,
};
use crate::transaction::wal::writer::WalWriter;
use crate::transaction::wal::{DirtyPageId, DirtyPageTracker};

use super::cache::CacheManager;
use super::config::PropertyGraphConfig;
use super::edge::EdgeOps;
use super::query::QueryOps;
use super::schema::SchemaOps;
use super::transaction::TransactionOps;
use super::wal_manager::WalManager;

const DATA_FORMAT_VERSION: u32 = 1;

pub struct PropertyGraph {
    schema_ops: SchemaOps,
    edge_ops: EdgeOps,
    cache_manager: CacheManager,
    wal_manager: WalManager,
    dirty_tracker: Arc<DirtyPageTracker>,
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

impl Default for PropertyGraph {
    fn default() -> Self {
        Self::new()
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

        let dirty_tracker = Arc::new(DirtyPageTracker::default());

        Self {
            schema_ops: SchemaOps::new(),
            edge_ops: EdgeOps::new(),
            cache_manager,
            wal_manager: WalManager::new(),
            dirty_tracker,
            config,
            is_open: true,
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

    pub fn dirty_tracker(&self) -> &Arc<DirtyPageTracker> {
        &self.dirty_tracker
    }

    pub fn should_flush(&self) -> bool {
        self.dirty_tracker.should_flush()
    }

    pub fn get_dirty_page_count(&self) -> usize {
        self.dirty_tracker.get_dirty_page_count()
    }

    pub fn record_cache(&self) -> Option<&crate::storage::cache::SharedRecordCache> {
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
        self.schema_ops.create_vertex_type(name, properties, primary_key)
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
        self.edge_ops.create_edge_type(
            name,
            src_label,
            dst_label,
            properties,
            oe_strategy,
            ie_strategy,
            &self.schema_ops.vertex_tables,
        )
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
        self.schema_ops.insert_vertex(label, external_id, properties, ts)
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
        self.schema_ops.delete_vertex(label, external_id, ts)
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
        self.schema_ops
            .update_vertex_property(label, external_id, property_name, value, ts)
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
        self.edge_ops.insert_edge(
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
            properties,
            ts,
            &self.schema_ops.vertex_tables,
        )
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
        self.edge_ops.delete_edge(
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
            ts,
            &self.schema_ops.vertex_tables,
        )
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
        self.edge_ops.update_edge_property(
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
            prop_name,
            value,
            ts,
            &self.schema_ops.vertex_tables,
        )
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

    // ==================== Persistence Operations ====================

    pub fn flush(&self) -> StorageResult<()> {
        use std::fs;
        use std::io::Write;

        let data_dir = self.config.work_dir.join("data");
        fs::create_dir_all(&data_dir)?;

        let version_file = data_dir.join("version");
        let mut file = fs::File::create(&version_file)
            .map_err(|e| StorageError::IOError(format!("Failed to create version file: {}", e)))?;
        writeln!(file, "{}", DATA_FORMAT_VERSION)
            .map_err(|e| StorageError::IOError(format!("Failed to write version file: {}", e)))?;

        let vertex_dir = data_dir.join("vertices");
        fs::create_dir_all(&vertex_dir)?;

        for (label_id, table) in &self.schema_ops.vertex_tables {
            let table_dir = vertex_dir.join(format!("label_{}", label_id));
            table.flush(&table_dir)?;
        }

        let edge_dir = data_dir.join("edges");
        fs::create_dir_all(&edge_dir)?;

        for ((src_label, dst_label, edge_label), table) in &self.edge_ops.edge_tables {
            let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
            table.flush(&table_dir)?;
        }

        self.wal_manager.sync()?;

        Ok(())
    }

    pub fn flush_tables_to_dir(&self, data_dir: &Path) -> StorageResult<()> {
        use std::fs;

        let vertex_dir = data_dir.join("vertices");
        fs::create_dir_all(&vertex_dir)?;

        for (label_id, table) in &self.schema_ops.vertex_tables {
            let table_dir = vertex_dir.join(format!("label_{}", label_id));
            table.flush(&table_dir)?;
        }

        let edge_dir = data_dir.join("edges");
        fs::create_dir_all(&edge_dir)?;

        for ((src_label, dst_label, edge_label), table) in &self.edge_ops.edge_tables {
            let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
            table.flush(&table_dir)?;
        }

        self.wal_manager.sync()?;

        Ok(())
    }

    pub fn load(&mut self) -> StorageResult<()> {
        self.load_data()
    }

    fn load_data(&mut self) -> StorageResult<()> {
        use std::fs;
        use std::io::Read;

        let data_dir = self.config.work_dir.join("data");

        let version_file = data_dir.join("version");
        if version_file.exists() {
            let mut file = fs::File::open(&version_file)
                .map_err(|e| StorageError::IOError(format!("Failed to open version file: {}", e)))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| StorageError::IOError(format!("Failed to read version file: {}", e)))?;
            let version: u32 = content.trim().parse().map_err(|e| {
                StorageError::DeserializeError(format!("Invalid version format: {}", e))
            })?;
            if version > DATA_FORMAT_VERSION {
                return Err(StorageError::DeserializeError(format!(
                    "Data format version {} is newer than supported version {}",
                    version, DATA_FORMAT_VERSION
                )));
            }
        }

        let vertex_dir = data_dir.join("vertices");
        if vertex_dir.exists() {
            for entry in fs::read_dir(&vertex_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name() {
                        if let Some(name_str) = dir_name.to_str() {
                            if let Some(label_str) = name_str.strip_prefix("label_") {
                                if let Ok(label_id) = label_str.parse::<LabelId>() {
                                    if let Some(table) = self.schema_ops.vertex_tables.get_mut(&label_id) {
                                        table.load(&path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let edge_dir = data_dir.join("edges");
        if edge_dir.exists() {
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
                                    let key = (src_label, dst_label, edge_label);
                                    if let Some(table) = self.edge_ops.edge_tables.get_mut(&key) {
                                        table.load(&path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn restore_from_checkpoint(&mut self, checkpoint_dir: &Path) -> StorageResult<()> {
        use std::fs;

        let data_dir = checkpoint_dir.join("data");

        let vertex_dir = data_dir.join("vertices");
        if vertex_dir.exists() {
            for entry in fs::read_dir(&vertex_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(dir_name) = path.file_name() {
                        if let Some(name_str) = dir_name.to_str() {
                            if let Some(label_str) = name_str.strip_prefix("label_") {
                                if let Ok(label_id) = label_str.parse::<LabelId>() {
                                    if let Some(table) = self.schema_ops.vertex_tables.get_mut(&label_id) {
                                        table.load(&path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let edge_dir = data_dir.join("edges");
        if edge_dir.exists() {
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
                                    let key = (src_label, dst_label, edge_label);
                                    if let Some(table) = self.edge_ops.edge_tables.get_mut(&key) {
                                        table.load(&path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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
        let result = TransactionOps::add_vertex(&mut self.schema_ops, label, oid, properties, ts)?;
        
        self.dirty_tracker.mark_dirty(DirtyPageId::vertex(label, 0));
        self.dirty_tracker.mark_modified_since_checkpoint(DirtyPageId::vertex(label, 0));
        
        Ok(result)
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
        let result = TransactionOps::add_edge(
            &mut self.edge_ops,
            &self.schema_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            properties,
            ts,
        )?;
        
        self.dirty_tracker.mark_dirty(DirtyPageId::edge(edge_label, 0));
        self.dirty_tracker.mark_modified_since_checkpoint(DirtyPageId::edge(edge_label, 0));
        
        Ok(result)
    }

    fn get_vertex_id(&self, label: TxnLabelId, oid: &[u8], ts: Timestamp) -> Option<TxnVertexId> {
        let oid_str = String::from_utf8_lossy(oid).to_string();
        self.get_vertex(label, &oid_str, ts)
            .map(|v| v.internal_id as TxnVertexId)
    }

    fn get_vertex_oid(&self, label: TxnLabelId, vid: TxnVertexId, ts: Timestamp) -> Option<Vec<u8>> {
        TransactionOps::get_vertex_oid(&self.schema_ops, label, vid, ts)
    }

    fn get_vertex_property_types(&self, label: TxnLabelId) -> Vec<String> {
        TransactionOps::get_vertex_property_types(&self.schema_ops, label)
    }

    fn get_edge_property_types(
        &self,
        src_label: TxnLabelId,
        dst_label: TxnLabelId,
        edge_label: TxnLabelId,
    ) -> Vec<String> {
        TransactionOps::get_edge_property_types(&self.edge_ops, src_label, dst_label, edge_label)
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
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::delete_edge(
            &mut self.edge_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            oe_offset,
            ie_offset,
            ts,
        )
    }

    fn undo_update_vertex_property(
        &mut self,
        label: TxnLabelId,
        vid: TxnVertexId,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::update_vertex_property_undo(&mut self.schema_ops, label, vid, col_id, value, ts)
    }

    fn undo_update_edge_property(
        &mut self,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        oe_offset: i32,
        ie_offset: i32,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::update_edge_property_undo(
            &mut self.edge_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            oe_offset,
            ie_offset,
            col_id,
            value,
            ts,
        )
    }

    fn revert_delete_vertex(
        &mut self,
        label: TxnLabelId,
        vid: TxnVertexId,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::revert_delete_edge(
            &mut self.edge_ops,
            label,
            vid,
            label,
            vid,
            label,
            0,
            0,
            ts,
        )
    }

    fn revert_delete_edge(
        &mut self,
        src_label: TxnLabelId,
        src_vid: TxnVertexId,
        dst_label: TxnLabelId,
        dst_vid: TxnVertexId,
        edge_label: TxnLabelId,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::revert_delete_edge(
            &mut self.edge_ops,
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            oe_offset,
            ie_offset,
            ts,
        )
    }

    fn revert_delete_vertex_properties(
        &mut self,
        label_name: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_delete_vertex_properties(&mut self.schema_ops, label_name, prop_names)
    }

    fn revert_delete_edge_properties(
        &mut self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_delete_edge_properties(
            &mut self.edge_ops,
            src_label,
            dst_label,
            edge_label,
            &self.schema_ops,
            prop_names,
        )
    }

    fn revert_delete_vertex_label(&mut self, label_name: &str) -> UndoLogResult<()> {
        let label = self.schema_ops.vertex_label_counter;
        TransactionOps::create_vertex_type_undo(&mut self.schema_ops, label_name, label)
    }

    fn revert_delete_edge_label(
        &mut self,
        src_label: &str,
        dst_label: &str,
        edge_label: &str,
    ) -> UndoLogResult<()> {
        let src_label_id = self
            .schema_ops
            .vertex_label_names
            .get(src_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;
        let dst_label_id = self
            .schema_ops
            .vertex_label_names
            .get(dst_label)
            .copied()
            .ok_or(UndoLogError::LabelNotFound(0))?;

        self.edge_ops
            .create_edge_type(
                edge_label,
                src_label_id,
                dst_label_id,
                Vec::new(),
                EdgeStrategy::None,
                EdgeStrategy::None,
                self.schema_ops.vertex_tables(),
            )
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;

        Ok(())
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
