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
use crate::storage::{EdgeDeletionContext, EdgeIdentifier, EdgeKey, VertexIdentifier};
use crate::transaction::insert_transaction::{AddEdgeInsertParam, InsertTarget, InsertTransactionResult};
use crate::transaction::undo_log::{PropertyValue, UndoLogError, UndoLogResult, UndoTarget};
use crate::transaction::wal::types::{
    ColumnId, LabelId as TxnLabelId, Timestamp, VertexId as TxnVertexId,
};
use crate::transaction::wal::writer::WalWriter;
use crate::storage::persistence::{DirtyPageId, DirtyPageTracker, FlushManager, TableType};
use crate::storage::page::{PageLockId, PageLockManager, PageManager, LockMode, LockResult};

use super::cache::CacheManager;
use super::config::PropertyGraphConfig;
use super::edge::{CreateEdgeTypeParams, EdgeOperationParams, EdgeTraversalParams, EdgeOps};
use super::query::QueryOps;
use super::schema::SchemaOps;
use super::transaction::{
    AddEdgeParams, DeleteEdgeParams, DeleteEdgeTypeParams, 
    RevertDeleteEdgeParams, TransactionOps, UpdateEdgePropertyUndoParams,
};
use super::wal_manager::WalManager;

const DATA_FORMAT_VERSION: u32 = 1;

pub struct PropertyGraph {
    schema_ops: SchemaOps,
    edge_ops: EdgeOps,
    cache_manager: CacheManager,
    wal_manager: WalManager,
    page_manager: Arc<PageManager>,
    dirty_tracker: Arc<DirtyPageTracker>,
    flush_manager: Option<FlushManager>,
    lock_manager: PageLockManager,
    memory_tracker: SharedMemoryTracker,
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

        let page_manager = Arc::new(PageManager::with_config(
            crate::storage::page::PageManagerConfig {
                base_path: config.work_dir.clone(),
                max_pages: (config.cache_memory / 4096) as u64,
            },
        ));

        let dirty_tracker = Arc::new(DirtyPageTracker::with_config(
            crate::storage::persistence::DirtyTrackerConfig {
                flush_threshold: config.flush_config.flush_threshold,
                flush_interval: config.flush_config.flush_interval,
            },
        ));

        let flush_manager = if config.enable_background_flush {
            Some(
                FlushManager::new(config.flush_config.clone())
                    .with_page_manager(page_manager.clone()),
            )
        } else {
            None
        };

        let lock_manager = PageLockManager::new();

        Self {
            schema_ops: SchemaOps::new(),
            edge_ops: EdgeOps::new(),
            cache_manager,
            wal_manager: WalManager::new(),
            page_manager,
            dirty_tracker,
            flush_manager,
            lock_manager,
            memory_tracker,
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

    pub fn memory_tracker(&self) -> &SharedMemoryTracker {
        &self.memory_tracker
    }

    pub fn dirty_tracker(&self) -> &Arc<DirtyPageTracker> {
        &self.dirty_tracker
    }

    pub fn page_manager(&self) -> &Arc<PageManager> {
        &self.page_manager
    }

    pub fn lock_manager(&self) -> &PageLockManager {
        &self.lock_manager
    }

    pub fn acquire_page_lock(
        &self,
        page_id: PageLockId,
        txn_id: u64,
        mode: LockMode,
    ) -> LockResult {
        self.lock_manager.acquire_lock(page_id, txn_id, mode)
    }

    pub fn release_page_lock(&self, page_id: PageLockId, txn_id: u64) -> bool {
        self.lock_manager.release_lock(page_id, txn_id)
    }

    pub fn release_all_page_locks(&self, txn_id: u64) -> usize {
        self.lock_manager.release_all_locks(txn_id)
    }

    pub fn should_flush(&self) -> bool {
        self.dirty_tracker.should_flush()
    }

    pub fn get_dirty_page_count(&self) -> usize {
        self.dirty_tracker.get_dirty_page_count()
    }

    pub fn mark_page_dirty(&self, table_type: TableType, label_id: u16, block_number: u64) {
        let page_id = DirtyPageId::new(table_type, label_id, block_number);
        self.dirty_tracker.mark_dirty(page_id);
        if let Some(ref flush_manager) = self.flush_manager {
            flush_manager.mark_dirty(page_id);
        }
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

    pub fn open<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let config = PropertyGraphConfig {
            work_dir: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        let mut graph = Self::with_config(config);
        graph.load_data()?;
        Ok(graph)
    }

    pub fn start_background_flush(&self) -> StorageResult<()> {
        if let Some(ref flush_manager) = self.flush_manager {
            flush_manager.start_background_flush()?;
        }
        Ok(())
    }

    pub fn stop_background_flush(&self) {
        if let Some(ref flush_manager) = self.flush_manager {
            flush_manager.stop_background_flush();
        }
    }

    pub fn flush_manager(&self) -> Option<&FlushManager> {
        self.flush_manager.as_ref()
    }

    pub fn close(&mut self) {
        self.stop_background_flush();
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
            return Err(StorageError::storage_not_open());
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
            return Err(StorageError::storage_not_open());
        }
        let params = CreateEdgeTypeParams {
            name,
            src_label,
            dst_label,
            properties,
            oe_strategy,
            ie_strategy,
        };
        self.edge_ops.create_edge_type(params, &self.schema_ops.vertex_tables)
    }

    pub fn drop_vertex_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }
        let label_id = self
            .schema_ops
            .vertex_label_names
            .get(name)
            .copied()
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?;
        self.schema_ops.drop_vertex_type(name)?;
        self.edge_ops.drop_edges_for_vertex_label(label_id);
        Ok(())
    }

    pub fn drop_edge_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
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
            return Err(StorageError::storage_not_open());
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
            return Err(StorageError::storage_not_open());
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
            return Err(StorageError::storage_not_open());
        }
        self.schema_ops
            .update_vertex_property(label, external_id, property_name, value, ts)
    }

    pub fn insert_edge(&mut self, params: InsertEdgeParams) -> StorageResult<EdgeId> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }
        let op_params = EdgeOperationParams {
            edge_label: params.edge_label,
            src_label: params.src_label,
            src_id: params.src_id,
            dst_label: params.dst_label,
            dst_id: params.dst_id,
        };
        self.edge_ops.insert_edge(op_params, params.properties, params.ts, &self.schema_ops.vertex_tables)
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
        let params = EdgeOperationParams {
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
        };
        self.edge_ops.get_edge(params, ts, &self.schema_ops.vertex_tables)
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
            return Err(StorageError::storage_not_open());
        }
        let params = EdgeOperationParams {
            edge_label,
            src_label,
            src_id,
            dst_label,
            dst_id,
        };
        self.edge_ops.delete_edge(params, ts, &self.schema_ops.vertex_tables)
    }

    pub fn update_edge_property(&mut self, params: PropertyGraphUpdateEdgePropertyParams) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }
        let op_params = EdgeOperationParams {
            edge_label: params.edge_label,
            src_label: params.src_label,
            src_id: params.src_id,
            dst_label: params.dst_label,
            dst_id: params.dst_id,
        };
        self.edge_ops.update_edge_property(op_params, params.prop_name, params.value, params.ts, &self.schema_ops.vertex_tables)
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
        let params = EdgeTraversalParams {
            edge_label,
            src_label,
            dst_label,
        };
        self.edge_ops.out_edges(params, src_id, ts, &self.schema_ops.vertex_tables)
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
        let params = EdgeTraversalParams {
            edge_label,
            src_label,
            dst_label,
        };
        self.edge_ops.in_edges(params, dst_id, ts, &self.schema_ops.vertex_tables)
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
            .map_err(|e| StorageError::io_error(format!("Failed to create version file: {}", e)))?;
        writeln!(file, "{}", DATA_FORMAT_VERSION)
            .map_err(|e| StorageError::io_error(format!("Failed to write version file: {}", e)))?;

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

        self.dirty_tracker.clear();

        Ok(())
    }

    pub fn flush_incremental(&self) -> StorageResult<Vec<DirtyPageId>> {
        let dirty_pages = self.dirty_tracker.flush_and_reset();

        if dirty_pages.is_empty() {
            return Ok(dirty_pages);
        }

        use std::fs;
        let data_dir = self.config.work_dir.join("data");
        fs::create_dir_all(&data_dir)?;

        let mut flushed_labels = std::collections::HashSet::new();

        for page_id in &dirty_pages {
            match page_id.table_type {
                crate::storage::persistence::TableType::Vertex => {
                    if flushed_labels.insert(("vertex", page_id.label_id)) {
                        if let Some(table) = self.schema_ops.vertex_tables.get(&page_id.label_id) {
                            let vertex_dir = data_dir.join("vertices");
                            let table_dir = vertex_dir.join(format!("label_{}", page_id.label_id));
                            table.flush(&table_dir)?;
                        }
                    }
                }
                crate::storage::persistence::TableType::Edge => {
                    if flushed_labels.insert(("edge", page_id.label_id)) {
                        for ((src, dst, label), table) in &self.edge_ops.edge_tables {
                            if *label == page_id.label_id {
                                let edge_dir = data_dir.join("edges");
                                let table_dir = edge_dir.join(format!("{}_{}_{}", src, dst, label));
                                table.flush(&table_dir)?;
                            }
                        }
                    }
                }
                crate::storage::persistence::TableType::Schema => {}
                crate::storage::persistence::TableType::Property => {}
            }
        }

        self.wal_manager.sync()?;

        Ok(dirty_pages)
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
                .map_err(|e| StorageError::io_error(format!("Failed to open version file: {}", e)))?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| StorageError::io_error(format!("Failed to read version file: {}", e)))?;
            let version: u32 = content.trim().parse().map_err(|e| {
                StorageError::deserialize_error(format!("Invalid version format: {}", e))
            })?;
            if version > DATA_FORMAT_VERSION {
                return Err(StorageError::deserialize_error(format!(
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
        param: AddEdgeInsertParam,
    ) -> InsertTransactionResult<EdgeId> {
        let params = AddEdgeParams {
            src_label: param.src_label,
            src_vid: param.src_vid,
            dst_label: param.dst_label,
            dst_vid: param.dst_vid,
            edge_label: param.edge_label,
        };
        let result = TransactionOps::add_edge(
            &mut self.edge_ops,
            &self.schema_ops,
            params,
            param.properties,
            param.ts,
        )?;
        
        self.dirty_tracker.mark_dirty(DirtyPageId::edge(param.edge_label, 0));
        self.dirty_tracker.mark_modified_since_checkpoint(DirtyPageId::edge(param.edge_label, 0));
        
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
        TransactionOps::delete_vertex_type(&mut self.schema_ops, &mut self.edge_ops, label)?;
        self.dirty_tracker.mark_dirty(DirtyPageId::vertex(label, 0));
        Ok(())
    }

    fn delete_edge_type(&mut self, edge_key: EdgeKey) -> UndoLogResult<()> {
        let params = DeleteEdgeTypeParams {
            src_label: edge_key.src_label,
            dst_label: edge_key.dst_label,
            edge_label: edge_key.edge_label,
        };
        TransactionOps::delete_edge_type(
            &mut self.edge_ops,
            params,
        )?;
        self.dirty_tracker.mark_dirty(DirtyPageId::edge(edge_key.edge_label, 0));
        Ok(())
    }

    fn delete_vertex(&mut self, vertex: VertexIdentifier, ts: Timestamp) -> UndoLogResult<()> {
        TransactionOps::delete_vertex(&mut self.schema_ops, vertex.label, vertex.vid, ts)?;
        self.dirty_tracker.mark_dirty(DirtyPageId::vertex(vertex.label, 0));
        Ok(())
    }

    fn delete_edge(&mut self, edge_ctx: EdgeDeletionContext) -> UndoLogResult<()> {
        let params = DeleteEdgeParams {
            src_label: edge_ctx.edge_id.src_label,
            src_vid: edge_ctx.edge_id.src_vid,
            dst_label: edge_ctx.edge_id.dst_label,
            dst_vid: edge_ctx.edge_id.dst_vid,
            edge_label: edge_ctx.edge_id.edge_label,
        };
        TransactionOps::delete_edge(
            &mut self.edge_ops,
            params,
            edge_ctx.oe_offset,
            edge_ctx.ie_offset,
            edge_ctx.timestamp,
        )?;
        self.dirty_tracker.mark_dirty(DirtyPageId::edge(edge_ctx.edge_id.edge_label, 0));
        Ok(())
    }

    fn undo_update_vertex_property(
        &mut self,
        vertex: VertexIdentifier,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        TransactionOps::update_vertex_property_undo(
            &mut self.schema_ops,
            vertex.label,
            vertex.vid,
            col_id,
            value,
            ts,
        )?;
        self.dirty_tracker.mark_dirty(DirtyPageId::vertex(vertex.label, 0));
        Ok(())
    }

    fn undo_update_edge_property(
        &mut self,
        edge_id: EdgeIdentifier,
        oe_offset: i32,
        ie_offset: i32,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let params = UpdateEdgePropertyUndoParams {
            src_label: edge_id.src_label,
            src_vid: edge_id.src_vid,
            dst_label: edge_id.dst_label,
            dst_vid: edge_id.dst_vid,
            edge_label: edge_id.edge_label,
        };
        TransactionOps::update_edge_property_undo(
            &mut self.edge_ops,
            params,
            oe_offset,
            ie_offset,
            col_id,
            value,
            ts,
        )?;
        self.dirty_tracker.mark_dirty(DirtyPageId::edge(edge_id.edge_label, 0));
        Ok(())
    }

    fn revert_delete_vertex(&mut self, vertex: VertexIdentifier, ts: Timestamp) -> UndoLogResult<()> {
        let params = RevertDeleteEdgeParams {
            src_label: vertex.label,
            src_vid: vertex.vid,
            dst_label: vertex.label,
            dst_vid: vertex.vid,
            edge_label: vertex.label,
        };
        TransactionOps::revert_delete_edge(
            &mut self.edge_ops,
            params,
            0,
            0,
            ts,
        )?;
        self.dirty_tracker.mark_dirty(DirtyPageId::vertex(vertex.label, 0));
        Ok(())
    }

    fn revert_delete_edge(&mut self, edge_ctx: EdgeDeletionContext) -> UndoLogResult<()> {
        let params = RevertDeleteEdgeParams {
            src_label: edge_ctx.edge_id.src_label,
            src_vid: edge_ctx.edge_id.src_vid,
            dst_label: edge_ctx.edge_id.dst_label,
            dst_vid: edge_ctx.edge_id.dst_vid,
            edge_label: edge_ctx.edge_id.edge_label,
        };
        TransactionOps::revert_delete_edge(
            &mut self.edge_ops,
            params,
            edge_ctx.oe_offset,
            edge_ctx.ie_offset,
            edge_ctx.timestamp,
        )?;
        self.dirty_tracker.mark_dirty(DirtyPageId::edge(edge_ctx.edge_id.edge_label, 0));
        Ok(())
    }

    fn revert_delete_vertex_properties(
        &mut self,
        label_name: &str,
        prop_names: &[String],
    ) -> UndoLogResult<()> {
        TransactionOps::revert_delete_vertex_properties(&mut self.schema_ops, label_name, prop_names)?;
        if let Some(label) = self.schema_ops.vertex_label_names.get(label_name) {
            self.dirty_tracker.mark_dirty(DirtyPageId::vertex(*label, 0));
        }
        Ok(())
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
        )?;
        if let Some(label) = self.edge_ops.edge_label_names.get(edge_label) {
            self.dirty_tracker.mark_dirty(DirtyPageId::edge(*label, 0));
        }
        Ok(())
    }

    fn revert_delete_vertex_label(&mut self, label_name: &str) -> UndoLogResult<()> {
        let label = self.schema_ops.vertex_label_counter;
        TransactionOps::create_vertex_type_undo(&mut self.schema_ops, label_name, label)?;
        self.dirty_tracker.mark_dirty(DirtyPageId::vertex(label, 0));
        Ok(())
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

        let params = CreateEdgeTypeParams {
            name: edge_label,
            src_label: src_label_id,
            dst_label: dst_label_id,
            properties: Vec::new(),
            oe_strategy: EdgeStrategy::None,
            ie_strategy: EdgeStrategy::None,
        };
        self.edge_ops
            .create_edge_type(
                params,
                self.schema_ops.vertex_tables(),
            )
            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;

        if let Some(label) = self.edge_ops.edge_label_names.get(edge_label) {
            self.dirty_tracker.mark_dirty(DirtyPageId::edge(*label, 0));
        }

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
        )?;
        if let Some(label_id) = self.schema_ops.vertex_label_names.get(label) {
            self.dirty_tracker.mark_dirty(DirtyPageId::vertex(*label_id, 0));
        }
        Ok(())
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
        )?;
        if let Some(label) = self.edge_ops.edge_label_names.get(edge_label) {
            self.dirty_tracker.mark_dirty(DirtyPageId::edge(*label, 0));
        }
        Ok(())
    }
}
