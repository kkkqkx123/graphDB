//! Property Graph Storage
//!
//! Main entry point for property graph storage combining vertex and edge tables.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;

use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::core::value::null::NullType;
use crate::transaction::insert_transaction::{InsertTarget, InsertTransactionError, InsertTransactionResult};
use crate::transaction::undo_log::{PropertyValue, UndoLogError, UndoLogResult, UndoTarget};
use crate::transaction::wal::types::{ColumnId, EdgeId as TxnEdgeId, LabelId as TxnLabelId, Timestamp, VertexId as TxnVertexId};
use crate::transaction::wal::writer::WalWriter;

use super::cache::{
    BlockCache, BlockId, CacheConfig, CachedEdge, CachedVertex, EdgeCacheKey, RecordCache,
    RecordCacheConfig, RecordCacheStats, SharedBlockCache, SharedRecordCache, TableType,
    VertexCacheKey,
};
use super::edge::{EdgeDirection, EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, EdgeTable, PropertyDef as EdgePropertyDef};
use super::memory::{MemoryConfig, MemoryTracker, SharedMemoryTracker};
use super::vertex::{LabelId, PropertyDef as VertexPropertyDef, Timestamp as StorageTimestamp, VertexId, VertexRecord, VertexSchema, VertexTable};
use super::vertex::vertex_table::VertexIterator;

const DATA_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct PropertyGraphConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
    pub work_dir: PathBuf,
    pub enable_cache: bool,
    pub cache_memory: usize,
    pub memory_config: MemoryConfig,
}

impl Default for PropertyGraphConfig {
    fn default() -> Self {
        Self {
            initial_vertex_capacity: 4096,
            initial_edge_capacity: 4096,
            work_dir: PathBuf::from("./data"),
            enable_cache: true,
            cache_memory: 256 * 1024 * 1024,
            memory_config: MemoryConfig::default(),
        }
    }
}

impl PropertyGraphConfig {
    pub fn with_cache(mut self, enable: bool, cache_memory: usize) -> Self {
        self.enable_cache = enable;
        self.cache_memory = cache_memory;
        self
    }

    pub fn with_memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = config;
        self
    }
}

pub struct PropertyGraph {
    vertex_tables: HashMap<LabelId, VertexTable>,
    edge_tables: HashMap<(LabelId, LabelId, LabelId), EdgeTable>,
    vertex_label_names: HashMap<String, LabelId>,
    edge_label_names: HashMap<String, LabelId>,
    vertex_label_counter: LabelId,
    edge_label_counter: LabelId,
    config: PropertyGraphConfig,
    is_open: bool,
    wal_writer: Option<Arc<RwLock<Box<dyn WalWriter>>>>,
    wal_enabled: bool,
    cache: Option<SharedBlockCache>,
    record_cache: Option<SharedRecordCache>,
    memory_tracker: Option<SharedMemoryTracker>,
}

impl std::fmt::Debug for PropertyGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyGraph")
            .field("vertex_tables", &self.vertex_tables)
            .field("edge_tables", &self.edge_tables)
            .field("vertex_label_names", &self.vertex_label_names)
            .field("edge_label_names", &self.edge_label_names)
            .field("vertex_label_counter", &self.vertex_label_counter)
            .field("edge_label_counter", &self.edge_label_counter)
            .field("config", &self.config)
            .field("is_open", &self.is_open)
            .field("wal_writer", &self.wal_writer.as_ref().map(|_| "WalWriter"))
            .field("wal_enabled", &self.wal_enabled)
            .field("cache", &self.cache.as_ref().map(|c: &Arc<BlockCache>| c.stats()))
            .field("memory_tracker", &self.memory_tracker.as_ref().map(|t: &Arc<MemoryTracker>| t.stats()))
            .finish()
    }
}

impl PropertyGraph {
    pub fn new() -> Self {
        Self::with_config(PropertyGraphConfig::default())
    }

    pub fn with_config(config: PropertyGraphConfig) -> Self {
        let cache = if config.enable_cache {
            Some(Arc::new(BlockCache::with_memory(config.cache_memory)))
        } else {
            None
        };

        let memory_tracker = Arc::new(MemoryTracker::new(config.memory_config.clone()));

        let record_cache = if config.enable_cache {
            let record_cache_config = RecordCacheConfig {
                max_memory: config.cache_memory / 2,
                shard_count: 8,
            };
            Some(Arc::new(
                RecordCache::with_config(record_cache_config).with_memory_tracker(memory_tracker.clone()),
            ))
        } else {
            None
        };

        Self {
            vertex_tables: HashMap::new(),
            edge_tables: HashMap::new(),
            vertex_label_names: HashMap::new(),
            edge_label_names: HashMap::new(),
            vertex_label_counter: 0,
            edge_label_counter: 0,
            config,
            is_open: true,
            wal_writer: None,
            wal_enabled: false,
            cache,
            record_cache,
            memory_tracker: Some(memory_tracker),
        }
    }

    pub fn with_wal(mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) -> Self {
        self.wal_writer = Some(wal_writer);
        self.wal_enabled = true;
        self
    }

    pub fn set_wal_writer(&mut self, wal_writer: Arc<RwLock<Box<dyn WalWriter>>>) {
        self.wal_writer = Some(wal_writer);
        self.wal_enabled = true;
    }

    pub fn wal_enabled(&self) -> bool {
        self.wal_enabled
    }

    pub fn cache(&self) -> Option<&SharedBlockCache> {
        self.cache.as_ref()
    }

    pub fn record_cache(&self) -> Option<&SharedRecordCache> {
        self.record_cache.as_ref()
    }

    pub fn memory_tracker(&self) -> Option<&SharedMemoryTracker> {
        self.memory_tracker.as_ref()
    }

    pub fn cache_stats(&self) -> Option<super::cache::CacheStats> {
        self.cache.as_ref().map(|c: &Arc<BlockCache>| c.stats())
    }

    pub fn record_cache_stats(&self) -> Option<RecordCacheStats> {
        self.record_cache.as_ref().map(|c: &Arc<RecordCache>| c.stats())
    }

    pub fn memory_stats(&self) -> Option<super::memory::MemoryStats> {
        self.memory_tracker.as_ref().map(|t: &Arc<MemoryTracker>| t.stats())
    }

    pub fn clear_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.clear();
        }
        if let Some(ref record_cache) = self.record_cache {
            record_cache.clear();
        }
    }

    fn write_wal(&self, _op_type: u8, _data: &[u8]) -> StorageResult<()> {
        if !self.wal_enabled {
            return Ok(());
        }

        if let Some(ref wal_writer) = self.wal_writer {
            let mut writer = wal_writer.write();
            writer.append(_data)
                .map_err(|e| StorageError::WalError(format!("Failed to write WAL: {:?}", e)))?;
        }
        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let config = PropertyGraphConfig {
            work_dir: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        Ok(Self::with_config(config))
    }

    pub fn close(&mut self) {
        self.is_open = false;
        for table in self.vertex_tables.values_mut() {
            table.close();
        }
        for table in self.edge_tables.values_mut() {
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

        if self.vertex_label_names.contains_key(name) {
            return Err(StorageError::LabelAlreadyExists(name.to_string()));
        }

        let label_id = self.vertex_label_counter;
        self.vertex_label_counter += 1;

        let primary_key_index = properties
            .iter()
            .position(|p| p.name == primary_key)
            .ok_or_else(|| StorageError::PropertyNotFound(primary_key.to_string()))?;

        let schema = VertexSchema {
            label_id,
            label_name: name.to_string(),
            properties,
            primary_key_index,
        };

        let table = VertexTable::new(label_id, name.to_string(), schema);
        self.vertex_tables.insert(label_id, table);
        self.vertex_label_names.insert(name.to_string(), label_id);

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

        if !self.vertex_tables.contains_key(&src_label) {
            return Err(StorageError::LabelNotFound(format!("source label {}", src_label)));
        }
        if !self.vertex_tables.contains_key(&dst_label) {
            return Err(StorageError::LabelNotFound(format!("destination label {}", dst_label)));
        }

        if self.edge_label_names.contains_key(name) {
            return Err(StorageError::LabelAlreadyExists(name.to_string()));
        }

        let label_id = self.edge_label_counter;
        self.edge_label_counter += 1;

        let schema = EdgeSchema {
            label_id,
            label_name: name.to_string(),
            src_label,
            dst_label,
            properties,
            oe_strategy,
            ie_strategy,
        };

        let table = EdgeTable::new(schema);
        let key = (src_label, dst_label, label_id);
        self.edge_tables.insert(key, table);
        self.edge_label_names.insert(name.to_string(), label_id);

        Ok(label_id)
    }

    pub fn drop_vertex_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let label_id = self.vertex_label_names
            .remove(name)
            .ok_or_else(|| StorageError::LabelNotFound(name.to_string()))?;

        self.vertex_tables.remove(&label_id);

        let keys_to_remove: Vec<_> = self.edge_tables
            .keys()
            .filter(|(src, dst, _)| *src == label_id || *dst == label_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.edge_tables.remove(&key);
        }

        Ok(())
    }

    pub fn drop_edge_type(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let label_id = self.edge_label_names
            .remove(name)
            .ok_or_else(|| StorageError::LabelNotFound(name.to_string()))?;

        let keys_to_remove: Vec<_> = self.edge_tables
            .keys()
            .filter(|(_, _, e)| *e == label_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.edge_tables.remove(&key);
        }

        Ok(())
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

        let table = self.vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("vertex label {}", label)))?;

        table.insert(external_id, properties, ts)
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

        let table = self.vertex_tables.get(&label)?;
        let internal_id = table.get_internal_id(external_id, ts)?;

        if let Some(ref record_cache) = self.record_cache {
            let cache_key = VertexCacheKey::new(label, internal_id);
            if let Some(cached) = record_cache.get_vertex(&cache_key) {
                return Some(VertexRecord {
                    internal_id: cached.internal_id,
                    vid: cached.internal_id as u64,
                    properties: cached.properties,
                });
            }
        }

        let record = table.get_by_internal_id(internal_id, ts)?;

        if let Some(ref record_cache) = self.record_cache {
            let cache_key = VertexCacheKey::new(label, internal_id);
            let cached = CachedVertex {
                internal_id: record.internal_id,
                external_id: external_id.to_string(),
                properties: record.properties.clone(),
            };
            record_cache.insert_vertex(cache_key, cached);
        }

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

        if let Some(ref record_cache) = self.record_cache {
            let cache_key = VertexCacheKey::new(label, internal_id);
            if let Some(cached) = record_cache.get_vertex(&cache_key) {
                return Some(VertexRecord {
                    internal_id: cached.internal_id,
                    vid: cached.internal_id as u64,
                    properties: cached.properties,
                });
            }
        }

        let table = self.vertex_tables.get(&label)?;
        let record = table.get_by_internal_id(internal_id, ts)?;

        if let Some(ref record_cache) = self.record_cache {
            let cache_key = VertexCacheKey::new(label, internal_id);
            let cached = CachedVertex {
                internal_id: record.internal_id,
                external_id: String::new(),
                properties: record.properties.clone(),
            };
            record_cache.insert_vertex(cache_key, cached);
        }

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

        let table = self.vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("vertex label {}", label)))?;

        if let Some(internal_id) = table.get_internal_id(external_id, ts) {
            if let Some(ref record_cache) = self.record_cache {
                let cache_key = VertexCacheKey::new(label, internal_id);
                record_cache.remove_vertex(&cache_key);
            }
        }

        table.delete(external_id, ts)
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

        let table = self.vertex_tables
            .get_mut(&label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("vertex label {}", label)))?;

        let internal_id = table.get_internal_id(external_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        if let Some(ref record_cache) = self.record_cache {
            let cache_key = VertexCacheKey::new(label, internal_id);
            record_cache.remove_vertex(&cache_key);
        }

        table.update_property(internal_id, property_name, value, ts)
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

        let src_table = self.vertex_tables.get(&src_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("source vertex label {}", src_label)))?;
        let dst_table = self.vertex_tables.get(&dst_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("destination vertex label {}", dst_label)))?;

        let src_internal = src_table.get_internal_id(src_id, ts)
            .ok_or(StorageError::VertexNotFound)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::LabelNotFound(format!("edge label {}", edge_label)))?;

        edge_table.insert_edge(src_internal as VertexId, dst_internal as VertexId, properties, ts)
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

        let src_table = self.vertex_tables.get(&src_label)?;
        let dst_table = self.vertex_tables.get(&dst_label)?;

        let src_internal = src_table.get_internal_id(src_id, ts)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        let record = edge_table.get_edge(src_internal as VertexId, dst_internal as VertexId, ts)?;

        if let Some(ref record_cache) = self.record_cache {
            let cache_key = EdgeCacheKey::new(edge_label, src_internal as u64, dst_internal as u64, record.edge_id);
            let cached = CachedEdge {
                edge_id: record.edge_id,
                src_vid: src_internal as u64,
                dst_vid: dst_internal as u64,
                properties: record.properties.clone(),
            };
            record_cache.insert_edge(cache_key, cached);
        }

        Some(record)
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

        let src_table = self.vertex_tables.get(&src_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("source vertex label {}", src_label)))?;
        let dst_table = self.vertex_tables.get(&dst_label)
            .ok_or_else(|| StorageError::LabelNotFound(format!("destination vertex label {}", dst_label)))?;

        let src_internal = src_table.get_internal_id(src_id, ts)
            .ok_or(StorageError::VertexNotFound)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)
            .ok_or(StorageError::VertexNotFound)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables
            .get_mut(&key)
            .ok_or_else(|| StorageError::LabelNotFound(format!("edge label {}", edge_label)))?;

        if let Some(ref record_cache) = self.record_cache {
            if let Some(nbr) = edge_table.get_edge_nbr(src_internal as VertexId, dst_internal as VertexId, ts) {
                let cache_key = EdgeCacheKey::new(edge_label, src_internal as u64, dst_internal as u64, nbr.edge_id);
                record_cache.remove_edge(&cache_key);
            }
        }

        edge_table.delete_edge(src_internal as VertexId, dst_internal as VertexId, ts)
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

        let src_table = self.vertex_tables.get(&src_label)?;
        let src_internal = src_table.get_internal_id(src_id, ts)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        Some(edge_table.out_edges(src_internal as VertexId, ts))
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

        let dst_table = self.vertex_tables.get(&dst_label)?;
        let dst_internal = dst_table.get_internal_id(dst_id, ts)?;

        let key = (src_label, dst_label, edge_label);
        let edge_table = self.edge_tables.get(&key)?;

        Some(edge_table.in_edges(dst_internal as VertexId, ts))
    }

    pub fn scan_vertices(&self, label: LabelId, ts: Timestamp) -> Option<VertexIterator> {
        if !self.is_open {
            return None;
        }
        self.vertex_tables.get(&label).map(|t| t.scan(ts))
    }

    pub fn vertex_count(&self, label: LabelId, ts: Timestamp) -> usize {
        self.vertex_tables
            .get(&label)
            .map(|t| t.vertex_count(ts))
            .unwrap_or(0)
    }

    pub fn edge_count(&self, edge_label: LabelId) -> u64 {
        self.edge_tables
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

    pub fn get_vertex_label_id(&self, name: &str) -> Option<LabelId> {
        self.vertex_label_names.get(name).copied()
    }

    pub fn get_edge_label_id(&self, name: &str) -> Option<LabelId> {
        self.edge_label_names.get(name).copied()
    }

    pub fn vertex_label_names(&self) -> Vec<&str> {
        self.vertex_label_names.keys().map(|s| s.as_str()).collect()
    }

    pub fn edge_label_names(&self) -> Vec<&str> {
        self.edge_label_names.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_vertex_table(&self, label: LabelId) -> Option<&VertexTable> {
        self.vertex_tables.get(&label)
    }

    pub fn get_edge_table(&self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) -> Option<&EdgeTable> {
        self.edge_tables.get(&(src_label, dst_label, edge_label))
    }

    pub fn get_edge_table_by_label(&self, edge_label: LabelId) -> Option<&EdgeTable> {
        self.edge_tables.values().find(|t| t.label() == edge_label)
    }

    pub fn edge_tables(&self) -> impl Iterator<Item = (&(LabelId, LabelId, LabelId), &EdgeTable)> {
        self.edge_tables.iter()
    }

    pub fn vertex_tables(&self) -> impl Iterator<Item = (&LabelId, &VertexTable)> {
        self.vertex_tables.iter()
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn compact(&mut self) {
        for table in self.vertex_tables.values_mut() {
            table.compact();
        }
        for table in self.edge_tables.values_mut() {
            table.compact();
        }
    }

    pub fn clear(&mut self) {
        self.vertex_tables.clear();
        self.edge_tables.clear();
        self.vertex_label_names.clear();
        self.edge_label_names.clear();
        self.vertex_label_counter = 0;
        self.edge_label_counter = 0;
    }

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

        for (label_id, table) in &self.vertex_tables {
            let table_dir = vertex_dir.join(format!("label_{}", label_id));
            table.flush(&table_dir)?;
        }

        let edge_dir = data_dir.join("edges");
        fs::create_dir_all(&edge_dir)?;

        for ((src_label, dst_label, edge_label), table) in &self.edge_tables {
            let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
            table.flush(&table_dir)?;
        }

        if let Some(ref wal_writer) = self.wal_writer {
            let mut writer = wal_writer.write();
            writer.sync()
                .map_err(|e| StorageError::WalError(format!("Failed to sync WAL: {:?}", e)))?;
        }

        Ok(())
    }

    pub fn load(&mut self) -> StorageResult<()> {
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
            let version: u32 = content.trim().parse()
                .map_err(|e| StorageError::DeserializeError(format!("Invalid version format: {}", e)))?;
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
                                    if let Some(table) = self.vertex_tables.get_mut(&label_id) {
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
                                    if let Some(table) = self.edge_tables.get_mut(&key) {
                                        table.load(&path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.is_open = true;
        Ok(())
    }

    pub fn checkpoint(&mut self) -> StorageResult<()> {
        self.create_checkpoint()
    }

    fn temp_checkpoint_dir(work_dir: &Path) -> PathBuf {
        work_dir.join("temp_checkpoint")
    }

    fn checkpoint_dir(work_dir: &Path) -> PathBuf {
        work_dir.join("checkpoint")
    }

    pub fn create_checkpoint(&mut self) -> StorageResult<()> {
        use std::fs;

        let temp_dir = Self::temp_checkpoint_dir(&self.config.work_dir);
        let checkpoint_dir = Self::checkpoint_dir(&self.config.work_dir);

        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)
                .map_err(|e| StorageError::IOError(format!("Failed to remove temp checkpoint dir: {}", e)))?;
        }
        fs::create_dir_all(&temp_dir)
            .map_err(|e| StorageError::IOError(format!("Failed to create temp checkpoint dir: {}", e)))?;

        let data_dir = temp_dir.join("data");
        fs::create_dir_all(&data_dir)?;

        let vertex_dir = data_dir.join("vertices");
        fs::create_dir_all(&vertex_dir)?;

        for (label_id, table) in &self.vertex_tables {
            let table_dir = vertex_dir.join(format!("label_{}", label_id));
            table.flush(&table_dir)?;
        }

        let edge_dir = data_dir.join("edges");
        fs::create_dir_all(&edge_dir)?;

        for ((src_label, dst_label, edge_label), table) in &self.edge_tables {
            let table_dir = edge_dir.join(format!("{}_{}_{}", src_label, dst_label, edge_label));
            table.flush(&table_dir)?;
        }

        if let Some(ref wal_writer) = self.wal_writer {
            let mut writer = wal_writer.write();
            writer.sync()
                .map_err(|e| StorageError::WalError(format!("Failed to sync WAL: {:?}", e)))?;
        }

        if checkpoint_dir.exists() {
            fs::remove_dir_all(&checkpoint_dir)
                .map_err(|e| StorageError::IOError(format!("Failed to remove old checkpoint dir: {}", e)))?;
        }

        fs::rename(&temp_dir, &checkpoint_dir)
            .map_err(|e| StorageError::IOError(format!("Failed to move checkpoint: {}", e)))?;

        Ok(())
    }

    pub fn restore_checkpoint(&mut self) -> StorageResult<()> {
        use std::fs;

        let checkpoint_dir = Self::checkpoint_dir(&self.config.work_dir);
        
        if !checkpoint_dir.exists() {
            return Err(StorageError::NotFound(format!("Checkpoint directory not found: {:?}", checkpoint_dir)));
        }

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
                                    if let Some(table) = self.vertex_tables.get_mut(&label_id) {
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
                                    if let Some(table) = self.edge_tables.get_mut(&key) {
                                        table.load(&path)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.is_open = true;
        Ok(())
    }

    pub fn has_checkpoint(&self) -> bool {
        Self::checkpoint_dir(&self.config.work_dir).exists()
    }
}

impl Default for PropertyGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn value_to_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::Null(_) | Value::Empty => vec![0],
        Value::Bool(v) => {
            let mut buf = vec![1];
            buf.extend_from_slice(&[*v as u8]);
            buf
        }
        Value::Int(v) => {
            let mut buf = vec![2];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::BigInt(v) => {
            let mut buf = vec![3];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::Float(v) => {
            let mut buf = vec![4];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::Double(v) => {
            let mut buf = vec![5];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::String(v) => {
            let mut buf = vec![6];
            let bytes = v.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
            buf
        }
        Value::Blob(v) => {
            let mut buf = vec![7];
            buf.extend_from_slice(&(v.len() as u32).to_le_bytes());
            buf.extend_from_slice(v);
            buf
        }
        _ => vec![0],
    }
}

fn bytes_to_value(data: &[u8]) -> Option<Value> {
    if data.is_empty() {
        return None;
    }
    match data[0] {
        0 => Some(Value::Null(NullType::Null)),
        1 => data.get(1).map(|&v| Value::Bool(v != 0)),
        2 => data.get(1..9)
            .map(|b| i32::from_le_bytes(b.try_into().unwrap_or([0; 4])))
            .map(Value::Int),
        3 => data.get(1..9)
            .map(|b| i64::from_le_bytes(b.try_into().unwrap_or([0; 8])))
            .map(Value::BigInt),
        4 => data.get(1..9)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap_or([0; 4])))
            .map(Value::Float),
        5 => data.get(1..9)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap_or([0; 8])))
            .map(Value::Double),
        6 => {
            if data.len() < 5 {
                return None;
            }
            let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            data.get(5..5 + len)
                .and_then(|b| std::str::from_utf8(b).ok())
                .map(|s| Value::String(s.to_string()))
        }
        7 => {
            if data.len() < 5 {
                return None;
            }
            let len = u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as usize;
            data.get(5..5 + len).map(|b| Value::Blob(b.to_vec()))
        }
        _ => None,
    }
}

fn property_value_to_value(pv: PropertyValue) -> Value {
    match pv {
        PropertyValue::Int(v) => Value::BigInt(v),
        PropertyValue::Float(v) => Value::Double(v),
        PropertyValue::String(v) => Value::String(v),
        PropertyValue::Bytes(v) => Value::Blob(v),
        PropertyValue::Bool(v) => Value::Bool(v),
        PropertyValue::Null => Value::Null(NullType::Null),
    }
}

fn value_to_property_value(value: &Value) -> PropertyValue {
    match value {
        Value::BigInt(v) => PropertyValue::Int(*v),
        Value::Double(v) => PropertyValue::Float(*v),
        Value::String(v) => PropertyValue::String(v.clone()),
        Value::Blob(v) => PropertyValue::Bytes(v.clone()),
        Value::Bool(v) => PropertyValue::Bool(*v),
        Value::Null(_) | Value::Empty => PropertyValue::Null,
        _ => PropertyValue::Null,
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
        let external_id = std::str::from_utf8(oid)
            .map_err(|e| InsertTransactionError::SerializationError(e.to_string()))?;

        let props: Vec<(String, Value)> = properties
            .iter()
            .filter_map(|(k, v)| {
                bytes_to_value(v).map(|val| (k.clone(), val))
            })
            .collect();

        let internal_id = self.insert_vertex(label as LabelId, external_id, &props, ts)
            .map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(internal_id as TxnVertexId)
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
        let src_label_id = src_label as LabelId;
        let dst_label_id = dst_label as LabelId;
        let src_table = self.vertex_tables.get(&src_label_id)
            .ok_or_else(|| InsertTransactionError::LabelNotFound(src_label))?;
        let dst_table = self.vertex_tables.get(&dst_label_id)
            .ok_or_else(|| InsertTransactionError::LabelNotFound(dst_label))?;

        let src_external = src_table.get_external_id(src_vid as u32)
            .ok_or(InsertTransactionError::VertexNotFound(src_vid))?;
        let dst_external = dst_table.get_external_id(dst_vid as u32)
            .ok_or(InsertTransactionError::VertexNotFound(dst_vid))?;

        let props: Vec<(String, Value)> = properties
            .iter()
            .filter_map(|(k, v)| {
                bytes_to_value(v).map(|val| (k.clone(), val))
            })
            .collect();

        let edge_id = self.insert_edge(
            edge_label as LabelId,
            src_label as LabelId,
            &src_external,
            dst_label as LabelId,
            &dst_external,
            &props,
            ts,
        ).map_err(|e| InsertTransactionError::SchemaError(e.to_string()))?;

        Ok(edge_id)
    }

    fn get_vertex_id(
        &self,
        label: TxnLabelId,
        oid: &[u8],
        ts: Timestamp,
    ) -> Option<TxnVertexId> {
        let external_id = std::str::from_utf8(oid).ok()?;
        let label_id = label as LabelId;
        self.vertex_tables.get(&label_id)?
            .get_internal_id(external_id, ts)
            .map(|id| id as TxnVertexId)
    }

    fn get_vertex_oid(
        &self,
        label: TxnLabelId,
        vid: TxnVertexId,
        ts: Timestamp,
    ) -> Option<Vec<u8>> {
        let label_id = label as LabelId;
        self.vertex_tables.get(&label_id)?
            .get_external_id(vid as u32)
            .map(|s| s.into_bytes())
    }

    fn get_vertex_property_types(&self, label: TxnLabelId) -> Vec<String> {
        let label_id = label as LabelId;
        self.vertex_tables
            .get(&label_id)
            .map(|t| t.schema().properties.iter().map(|p| p.name.clone()).collect())
            .unwrap_or_default()
    }

    fn get_edge_property_types(&self, _src_label: TxnLabelId, _dst_label: TxnLabelId, edge_label: TxnLabelId) -> Vec<String> {
        let edge_label_id = edge_label as LabelId;
        self.edge_tables
            .values()
            .find(|t| t.label() == edge_label_id)
            .map(|t| t.schema().properties.iter().map(|p| p.name.clone()).collect())
            .unwrap_or_default()
    }

    fn vertex_label_num(&self) -> usize {
        self.vertex_tables.len()
    }

    fn lid_num(&self, label: TxnLabelId) -> usize {
        let label_id = label as LabelId;
        self.vertex_tables
            .get(&label_id)
            .map(|t| t.total_count())
            .unwrap_or(0)
    }
}

impl UndoTarget for PropertyGraph {
    fn delete_vertex_type(&mut self, label: TxnLabelId) -> UndoLogResult<()> {
        let label_id = label as LabelId;
        let label_name = self.vertex_tables
            .get(&label_id)
            .map(|t| t.label_name().to_string());

        if let Some(name) = label_name {
            self.vertex_label_names.remove(&name);
        }

        self.vertex_tables.remove(&label_id);

        let keys_to_remove: Vec<_> = self.edge_tables
            .keys()
            .filter(|(src, dst, _)| *src == label_id || *dst == label_id)
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.edge_tables.remove(&key);
        }

        Ok(())
    }

    fn delete_edge_type(&mut self, src_label: TxnLabelId, dst_label: TxnLabelId, edge_label: TxnLabelId) -> UndoLogResult<()> {
        let key = (src_label as LabelId, dst_label as LabelId, edge_label as LabelId);
        self.edge_tables.remove(&key);
        Ok(())
    }

    fn delete_vertex(&mut self, label: TxnLabelId, vid: TxnVertexId, ts: Timestamp) -> UndoLogResult<()> {
        let label_id = label as LabelId;
        if let Some(table) = self.vertex_tables.get_mut(&label_id) {
            table.delete_by_internal_id(vid as u32, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
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
        let key = (src_label as LabelId, dst_label as LabelId, edge_label as LabelId);
        if let Some(table) = self.edge_tables.get_mut(&key) {
            table.delete_edge(src_vid as VertexId, dst_vid as VertexId, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    fn undo_update_vertex_property(
        &mut self,
        label: TxnLabelId,
        vid: TxnVertexId,
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let label_id = label as LabelId;
        if let Some(table) = self.vertex_tables.get_mut(&label_id) {
            let schema = table.schema();
            let prop_name = schema.properties.get(col_id as usize)
                .map(|p| p.name.clone())
                .ok_or_else(|| UndoLogError::PropertyNotFound(format!("column {}", col_id)))?;

            let val = property_value_to_value(value);
            table.update_property(vid as u32, &prop_name, &val, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
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
        col_id: ColumnId,
        value: PropertyValue,
        ts: Timestamp,
    ) -> UndoLogResult<()> {
        let key = (src_label as LabelId, dst_label as LabelId, edge_label as LabelId);
        if let Some(table) = self.edge_tables.get_mut(&key) {
            let schema = table.schema();
            let prop_name = schema.properties.get(col_id as usize)
                .map(|p| p.name.clone())
                .ok_or_else(|| UndoLogError::PropertyNotFound(format!("column {}", col_id)))?;

            let val = property_value_to_value(value);
            table.update_edge_property(src_vid as VertexId, dst_vid as VertexId, &prop_name, &val, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    fn revert_delete_vertex(&mut self, label: TxnLabelId, vid: TxnVertexId, ts: Timestamp) -> UndoLogResult<()> {
        let label_id = label as LabelId;
        if let Some(table) = self.vertex_tables.get_mut(&label_id) {
            table.revert_delete(vid as u32, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
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
        let key = (src_label as LabelId, dst_label as LabelId, edge_label as LabelId);
        if let Some(table) = self.edge_tables.get_mut(&key) {
            table.revert_delete_edge(src_vid as VertexId, dst_vid as VertexId, ts)
                .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
        }
        Ok(())
    }

    fn revert_delete_vertex_properties(&mut self, label_name: &str, prop_names: &[String]) -> UndoLogResult<()> {
        if let Some(label_id) = self.vertex_label_names.get(label_name).copied() {
            if let Some(table) = self.vertex_tables.get_mut(&label_id) {
                for prop_name in prop_names {
                    let prop_def = VertexPropertyDef::new(prop_name.clone(), DataType::String);
                    table.add_property(prop_def)
                        .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
                }
            }
        }
        Ok(())
    }

    fn revert_delete_edge_properties(&mut self, _src_label: &str, _dst_label: &str, edge_label: &str, prop_names: &[String]) -> UndoLogResult<()> {
        if let Some(label_id) = self.edge_label_names.get(edge_label).copied() {
            for table in self.edge_tables.values_mut() {
                if table.label() == label_id {
                    for prop_name in prop_names {
                        table.add_property(prop_name.clone(), DataType::String, true)
                            .map_err(|e| UndoLogError::UndoFailed(e.to_string()))?;
                    }
                }
            }
        }
        Ok(())
    }

    fn revert_delete_vertex_label(&mut self, label_name: &str) -> UndoLogResult<()> {
        let label_id = self.vertex_label_counter;
        self.vertex_label_counter += 1;

        let schema = VertexSchema {
            label_id,
            label_name: label_name.to_string(),
            properties: vec![VertexPropertyDef::new("id".to_string(), DataType::String)],
            primary_key_index: 0,
        };

        let table = VertexTable::new(label_id, label_name.to_string(), schema);
        self.vertex_tables.insert(label_id, table);
        self.vertex_label_names.insert(label_name.to_string(), label_id);

        Ok(())
    }

    fn revert_delete_edge_label(&mut self, src_label: &str, dst_label: &str, edge_label: &str) -> UndoLogResult<()> {
        let src_label_id = self.vertex_label_names.get(src_label).copied()
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;
        let dst_label_id = self.vertex_label_names.get(dst_label).copied()
            .ok_or_else(|| UndoLogError::LabelNotFound(0))?;

        let label_id = self.edge_label_counter;
        self.edge_label_counter += 1;

        let schema = EdgeSchema {
            label_id,
            label_name: edge_label.to_string(),
            src_label: src_label_id,
            dst_label: dst_label_id,
            properties: vec![],
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        };

        let table = EdgeTable::new(schema);
        let key = (src_label_id, dst_label_id, label_id);
        self.edge_tables.insert(key, table);
        self.edge_label_names.insert(edge_label.to_string(), label_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vertex_type() {
        let mut graph = PropertyGraph::new();

        let label = graph.create_vertex_type(
            "person",
            vec![
                VertexPropertyDef::new("name".to_string(), DataType::String),
                VertexPropertyDef::new("age".to_string(), DataType::Int).nullable(true),
            ],
            "name",
        ).unwrap();

        assert_eq!(label, 0);
        assert_eq!(graph.get_vertex_label_id("person"), Some(0));
    }

    #[test]
    fn test_insert_and_get_vertex() {
        let mut graph = PropertyGraph::new();

        graph.create_vertex_type(
            "person",
            vec![VertexPropertyDef::new("name".to_string(), DataType::String)],
            "name",
        ).unwrap();

        graph.insert_vertex(
            0,
            "v1",
            &[("name".to_string(), Value::String("Alice".to_string()))],
            100,
        ).unwrap();

        let vertex = graph.get_vertex(0, "v1", 100).unwrap();
        assert_eq!(vertex.properties.len(), 1);
    }

    #[test]
    fn test_create_and_insert_edge() {
        let mut graph = PropertyGraph::new();

        graph.create_vertex_type(
            "person",
            vec![VertexPropertyDef::new("name".to_string(), DataType::String)],
            "name",
        ).unwrap();

        graph.insert_vertex(0, "v1", &[("name".to_string(), Value::String("Alice".to_string()))], 100).unwrap();
        graph.insert_vertex(0, "v2", &[("name".to_string(), Value::String("Bob".to_string()))], 100).unwrap();

        graph.create_edge_type(
            "knows",
            0,
            0,
            vec![EdgePropertyDef::new("since".to_string(), DataType::Int)],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        ).unwrap();

        let edge_id = graph.insert_edge(
            0,
            0,
            "v1",
            0,
            "v2",
            &[("since".to_string(), Value::Int(2020))],
            100,
        ).unwrap();

        let edge = graph.get_edge(0, 0, "v1", 0, "v2", 100).unwrap();
        assert_eq!(edge.edge_id, edge_id);
    }

    #[test]
    fn test_out_in_edges() {
        let mut graph = PropertyGraph::new();

        graph.create_vertex_type(
            "person",
            vec![VertexPropertyDef::new("name".to_string(), DataType::String)],
            "name",
        ).unwrap();

        graph.insert_vertex(0, "v1", &[("name".to_string(), Value::String("Alice".to_string()))], 100).unwrap();
        graph.insert_vertex(0, "v2", &[("name".to_string(), Value::String("Bob".to_string()))], 100).unwrap();
        graph.insert_vertex(0, "v3", &[("name".to_string(), Value::String("Charlie".to_string()))], 100).unwrap();

        graph.create_edge_type(
            "knows",
            0,
            0,
            vec![],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        ).unwrap();

        graph.insert_edge(0, 0, "v1", 0, "v2", &[], 100).unwrap();
        graph.insert_edge(0, 0, "v1", 0, "v3", &[], 100).unwrap();

        let out_edges = graph.out_edges(0, 0, 0, "v1", 100).unwrap();
        assert_eq!(out_edges.len(), 2);

        let in_edges = graph.in_edges(0, 0, 0, "v2", 100).unwrap();
        assert_eq!(in_edges.len(), 1);
    }
}
