//! 统一的索引存储实现
//!
//! 提供内存索引和持久化存储的统一接口：
//! - 细粒度锁，避免读操作阻塞
//! - 高并发读写性能
//! - 支持前缀查询和范围查询
//! - 支持索引元数据和数据的持久化
//!
//! 索引只存储 ID 引用，实际数据从 StorageEngine 获取

use crate::core::{Edge, StorageError, Value, Vertex};
use crate::index::{Index, IndexBinaryEncoder, IndexField, IndexQueryStats, QueryType};
use crate::storage::StorageEngine;
use bincode;
use dashmap::DashMap;
use redb::{Database, ReadableTable, TableDefinition, TypeName};
use std::cmp::Ordering as CmpOrdering;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ByteKey(pub Vec<u8>);

impl redb::Key for ByteKey {
    fn compare(data1: &[u8], data2: &[u8]) -> CmpOrdering {
        data1.cmp(data2)
    }
}

impl redb::Value for ByteKey {
    type SelfType<'a> = ByteKey where Self: 'a;
    type AsBytes<'a> = Vec<u8> where Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> ByteKey where Self: 'a {
        ByteKey(data.to_vec())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Vec<u8> where Self: 'b {
        value.0.clone()
    }

    fn type_name() -> TypeName {
        TypeName::new("graphdb::ByteKey")
    }
}

const INDEX_META_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("index_metadata");
const INDEX_DATA_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("index_data");
const INDEX_COUNTER_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("index_counter");

pub trait IndexPersistence {
    fn save_index(&self, index: &Index) -> Result<(), StorageError>;
    fn load_index(&self, index_id: i32) -> Result<Option<Index>, StorageError>;
    fn delete_index(&self, index_id: i32) -> Result<(), StorageError>;
    fn list_indexes(&self) -> Result<Vec<Index>, StorageError>;
    fn save_index_data(&self, index_id: i32, data: &[u8]) -> Result<(), StorageError>;
    fn load_index_data(&self, index_id: i32) -> Result<Option<Vec<u8>>, StorageError>;
    fn delete_index_data(&self, index_id: i32) -> Result<(), StorageError>;
    fn get_next_index_id(&self) -> Result<i32, StorageError>;
    fn increment_index_id(&self) -> Result<i32, StorageError>;
}

pub struct RedbIndexPersistence {
    db: Database,
    db_path: String,
}

impl std::fmt::Debug for RedbIndexPersistence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbIndexPersistence")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl Clone for RedbIndexPersistence {
    fn clone(&self) -> Self {
        Self::new(&self.db_path).expect("Failed to clone RedbIndexPersistence")
    }
}

impl RedbIndexPersistence {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_string_lossy().to_string();

        let db = Database::create(path.as_ref())
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(Self { db, db_path })
    }

    fn value_to_bytes<T: serde::Serialize + bincode::Encode>(&self, value: &T) -> Result<Vec<u8>, StorageError> {
        bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn value_from_bytes<'a, T: serde::Deserialize<'a> + bincode::Decode<()>>(&self, bytes: &'a [u8]) -> Result<T, StorageError> {
        let (value, _): (T, usize) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(value)
    }

    fn make_meta_key(index_id: i32) -> Vec<u8> {
        format!("index_meta:{}", index_id).as_bytes().to_vec()
    }

    fn make_data_key(index_id: i32) -> Vec<u8> {
        format!("index_data:{}", index_id).as_bytes().to_vec()
    }

    fn make_counter_key() -> Vec<u8> {
        "next_index_id".as_bytes().to_vec()
    }
}

impl IndexPersistence for RedbIndexPersistence {
    fn save_index(&self, index: &Index) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INDEX_META_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let index_bytes = self.value_to_bytes(index)?;
            let key = Self::make_meta_key(index.id);
            table.insert(ByteKey(key), ByteKey(index_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    fn load_index(&self, index_id: i32) -> Result<Option<Index>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEX_META_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let key = Self::make_meta_key(index_id);
        match table.get(ByteKey(key)).map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let index_bytes = value.value().0;
                let index: Index = self.value_from_bytes(&index_bytes)?;
                Ok(Some(index))
            }
            None => Ok(None),
        }
    }

    fn delete_index(&self, index_id: i32) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut meta_table = write_txn
                .open_table(INDEX_META_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let mut data_table = write_txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let meta_key = Self::make_meta_key(index_id);
            meta_table
                .remove(ByteKey(meta_key))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let data_key = Self::make_data_key(index_id);
            data_table
                .remove(ByteKey(data_key))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    fn list_indexes(&self) -> Result<Vec<Index>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEX_META_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut indexes = Vec::new();
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (_, value) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let index_bytes = value.value().0;
            let index: Index = self.value_from_bytes(&index_bytes)?;
            indexes.push(index);
        }
        Ok(indexes)
    }

    fn save_index_data(&self, index_id: i32, data: &[u8]) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let key = Self::make_data_key(index_id);
            table.insert(ByteKey(key), ByteKey(data.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    fn load_index_data(&self, index_id: i32) -> Result<Option<Vec<u8>>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEX_DATA_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let key = Self::make_data_key(index_id);
        match table.get(ByteKey(key)).map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => Ok(Some(value.value().0)),
            None => Ok(None),
        }
    }

    fn delete_index_data(&self, index_id: i32) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INDEX_DATA_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let key = Self::make_data_key(index_id);
            table.remove(ByteKey(key))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    fn get_next_index_id(&self) -> Result<i32, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEX_COUNTER_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let key = Self::make_counter_key();
        match table.get(ByteKey(key)).map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let id: i32 = self.value_from_bytes(&value.value().0)?;
                Ok(id)
            }
            None => Ok(1),
        }
    }

    fn increment_index_id(&self) -> Result<i32, StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let next_id = {
            let mut table = write_txn
                .open_table(INDEX_COUNTER_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let key = Self::make_counter_key();
            let current_id: i32 = match table.get(ByteKey(key.clone()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => self.value_from_bytes(&value.value().0)?,
                None => 1,
            };

            let new_id = current_id + 1;
            let new_id_bytes = self.value_to_bytes(&new_id)?;
            table.insert(ByteKey(key), ByteKey(new_id_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            new_id
        };
        Ok(next_id)
    }
}

pub type StorageRef = Arc<Mutex<dyn StorageEngine + Send + Sync>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexEntryType {
    Vertex(i64),
    Edge(i64),
}

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub entry_type: IndexEntryType,
    pub created_at: i64,
    pub access_count: Arc<AtomicU64>,
    pub last_accessed: Arc<AtomicU64>,
}

impl IndexEntry {
    pub fn new_vertex(vertex_id: i64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            entry_type: IndexEntryType::Vertex(vertex_id),
            created_at: now as i64,
            access_count: Arc::new(AtomicU64::new(0)),
            last_accessed: Arc::new(AtomicU64::new(now)),
        }
    }

    pub fn new_edge(edge_id: i64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            entry_type: IndexEntryType::Edge(edge_id),
            created_at: now as i64,
            access_count: Arc::new(AtomicU64::new(0)),
            last_accessed: Arc::new(AtomicU64::new(now)),
        }
    }

    pub fn touch(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_accessed.store(now, Ordering::Relaxed);
        self.access_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn is_vertex(&self) -> bool {
        matches!(self.entry_type, IndexEntryType::Vertex(_))
    }

    pub fn is_edge(&self) -> bool {
        matches!(self.entry_type, IndexEntryType::Edge(_))
    }

    pub fn get_id(&self) -> i64 {
        match self.entry_type {
            IndexEntryType::Vertex(id) => id,
            IndexEntryType::Edge(id) => id,
        }
    }
}

pub struct ConcurrentIndexStorage {
    space_id: i32,
    index_id: i32,
    index_name: String,
    storage: StorageRef,
    field_indexes: DashMap<String, DashMap<Vec<u8>, Vec<IndexEntry>>>,
    query_stats: IndexQueryStats,
    entry_count: Arc<AtomicU64>,
    last_updated: Arc<AtomicU64>,
}

impl ConcurrentIndexStorage {
    pub fn new(space_id: i32, index_id: i32, index_name: String, storage: StorageRef) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            space_id,
            index_id,
            index_name,
            storage,
            field_indexes: DashMap::new(),
            query_stats: IndexQueryStats::new(),
            entry_count: Arc::new(AtomicU64::new(0)),
            last_updated: Arc::new(AtomicU64::new(now)),
        }
    }

    pub fn insert_vertex(&self, field_name: &str, field_value: &Value, vertex: &Vertex) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.last_updated.store(now, Ordering::Relaxed);

        let key = IndexBinaryEncoder::encode_index_key(field_value);
        let entry = IndexEntry::new_vertex(vertex.id);

        let field_index = self.field_indexes
            .entry(field_name.to_string())
            .or_insert_with(DashMap::new);
        field_index
            .entry(key)
            .or_insert_with(Vec::new)
            .push(entry);

        self.entry_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn insert_edge(&self, field_name: &str, field_value: &Value, edge: &Edge) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.last_updated.store(now, Ordering::Relaxed);

        let key = IndexBinaryEncoder::encode_index_key(field_value);
        let entry = IndexEntry::new_edge(edge.id);

        let field_index = self.field_indexes
            .entry(field_name.to_string())
            .or_insert_with(DashMap::new);
        field_index
            .entry(key)
            .or_insert_with(Vec::new)
            .push(entry);

        self.entry_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn exact_lookup(&self, field_name: &str, field_value: &Value) -> Result<(Vec<Vertex>, Vec<Edge>, Duration), String> {
        let start = Instant::now();

        let key = IndexBinaryEncoder::encode_index_key(field_value);
        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut found = false;

        if let Some(field_index) = self.field_indexes.get(field_name) {
            if let Some(entries) = field_index.get(&key) {
                let storage = self.storage.lock().map_err(|e| e.to_string())?;
                for entry in entries.iter() {
                    entry.touch();
                    found = true;
                    match entry.entry_type {
                        IndexEntryType::Vertex(id) => {
                            if let Ok(Some(vertex)) = storage.get_node(&Value::Int(id)) {
                                vertices.push(vertex);
                            }
                        }
                        IndexEntryType::Edge(id) => {
                            if let Ok(Some(edge)) = storage.get_edge(&Value::Int(id), &Value::Int(0), "") {
                                edges.push(edge);
                            }
                        }
                    }
                }
            }
        }

        let duration = start.elapsed();
        self.query_stats.record_query(found, duration, QueryType::Exact);

        Ok((vertices, edges, duration))
    }

    pub fn prefix_lookup(&self, field_name: &str, prefix: &[Value]) -> Result<(Vec<Vertex>, Vec<Edge>, Duration), String> {
        let start = Instant::now();

        let prefix_bytes = IndexBinaryEncoder::encode_prefix(prefix, prefix.len());
        let (start_key, end_key) = IndexBinaryEncoder::encode_prefix_range(&prefix_bytes);

        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut found = false;

        let storage = self.storage.lock().map_err(|e| e.to_string())?;

        if let Some(field_index) = self.field_indexes.get(field_name) {
            for item in field_index.iter() {
                let key: &Vec<u8> = item.key();
                let key_bytes: &[u8] = key.as_slice();
                if key_bytes >= start_key.as_slice() && key_bytes < end_key.as_slice() {
                    let entries = item.value();
                    for entry in entries.iter() {
                        entry.touch();
                        found = true;
                        match entry.entry_type {
                            IndexEntryType::Vertex(id) => {
                                if let Ok(Some(vertex)) = storage.get_node(&Value::Int(id)) {
                                    vertices.push(vertex);
                                }
                            }
                            IndexEntryType::Edge(id) => {
                                if let Ok(Some(edge)) = storage.get_edge(&Value::Int(id), &Value::Int(0), "") {
                                    edges.push(edge);
                                }
                            }
                        }
                    }
                }
            }
        }

        let duration = start.elapsed();
        self.query_stats.record_query(found, duration, QueryType::Prefix);

        Ok((vertices, edges, duration))
    }

    pub fn range_lookup(
        &self,
        field_name: &str,
        start_value: &Value,
        end_value: &Value,
    ) -> Result<(Vec<Vertex>, Vec<Edge>, Duration), String> {
        let start = Instant::now();

        let start_key = IndexBinaryEncoder::encode_value(start_value);
        let mut end_key = IndexBinaryEncoder::encode_value(end_value);
        end_key.push(0xFFu8);

        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut found = false;

        let storage = self.storage.lock().map_err(|e| e.to_string())?;

        if let Some(field_index) = self.field_indexes.get(field_name) {
            for item in field_index.iter() {
                let key: &Vec<u8> = item.key();
                let key_bytes: &[u8] = key.as_slice();
                if key_bytes >= start_key.as_slice() && key_bytes < end_key.as_slice() {
                    let entries = item.value();
                    for entry in entries.iter() {
                        entry.touch();
                        found = true;
                        match entry.entry_type {
                            IndexEntryType::Vertex(id) => {
                                if let Ok(Some(vertex)) = storage.get_node(&Value::Int(id)) {
                                    vertices.push(vertex);
                                }
                            }
                            IndexEntryType::Edge(id) => {
                                if let Ok(Some(edge)) = storage.get_edge(&Value::Int(id), &Value::Int(0), "") {
                                    edges.push(edge);
                                }
                            }
                        }
                    }
                }
            }
        }

        let duration = start.elapsed();
        self.query_stats.record_query(found, duration, QueryType::Range);

        Ok((vertices, edges, duration))
    }

    pub fn delete_vertex(&self, _vertex: &Vertex) {
        self.entry_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn delete_edge(&self, _edge: &Edge) {
        self.entry_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn clear(&self) {
        self.field_indexes.clear();
        self.entry_count.store(0, Ordering::Relaxed);
        self.query_stats.reset();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_updated.store(now, Ordering::Relaxed);
    }

    pub fn get_query_stats(&self) -> &IndexQueryStats {
        &self.query_stats
    }

    pub fn get_entry_count(&self) -> usize {
        self.entry_count.load(Ordering::Relaxed) as usize
    }

    pub fn get_memory_usage(&self) -> usize {
        self.field_indexes.len() * std::mem::size_of::<String>()
    }

    pub fn get_last_updated(&self) -> u64 {
        self.last_updated.load(Ordering::Relaxed)
    }
}

pub struct ConcurrentIndexManager {
    space_id: i32,
    storages: DashMap<i32, ConcurrentIndexStorage>,
    index_metadata: DashMap<i32, IndexMetadata>,
    global_stats: Arc<IndexQueryStats>,
    storage: StorageRef,
}

#[derive(Debug, Clone)]
pub struct IndexMetadata {
    pub id: i32,
    pub name: String,
    pub space_id: i32,
    pub schema_name: String,
    pub fields: Vec<IndexField>,
    pub is_unique: bool,
    pub status: IndexStorageStatus,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexStorageStatus {
    Active,
    Building,
    Dropped,
    Failed(String),
}

impl Default for ConcurrentIndexManager {
    fn default() -> Self {
        Self::new(0, Arc::new(Mutex::new(crate::storage::MemoryStorage::new().expect("Failed to create MemoryStorage in ConcurrentIndexManager default implementation"))))
    }
}

impl ConcurrentIndexManager {
    pub fn new(space_id: i32, storage: StorageRef) -> Self {
        Self {
            space_id,
            storages: DashMap::new(),
            index_metadata: DashMap::new(),
            global_stats: Arc::new(IndexQueryStats::new()),
            storage,
        }
    }

    pub fn create_index(&self, name: &str, schema_name: &str, fields: Vec<IndexField>, is_unique: bool) -> Result<i32, String> {
        let index_id = self.storages.len() as i32 + 1;

        let storage = ConcurrentIndexStorage::new(self.space_id, index_id, name.to_string(), Arc::clone(&self.storage));
        self.storages.insert(index_id, storage);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let metadata = IndexMetadata {
            id: index_id,
            name: name.to_string(),
            space_id: self.space_id,
            schema_name: schema_name.to_string(),
            fields,
            is_unique,
            status: IndexStorageStatus::Active,
            created_at: now,
        };
        self.index_metadata.insert(index_id, metadata);

        Ok(index_id)
    }

    pub fn drop_index(&self, index_id: i32) -> Result<(), String> {
        self.storages.remove(&index_id);
        self.index_metadata.remove(&index_id);
        Ok(())
    }

    pub fn insert_vertex(&self, index_id: i32, field_name: &str, field_value: &Value, vertex: &Vertex) -> Result<(), String> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.insert_vertex(field_name, field_value, vertex);
            Ok(())
        } else {
            Err(format!("索引 {} 不存在", index_id))
        }
    }

    pub fn insert_edge(&self, index_id: i32, field_name: &str, field_value: &Value, edge: &Edge) -> Result<(), String> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.insert_edge(field_name, field_value, edge);
            Ok(())
        } else {
            Err(format!("索引 {} 不存在", index_id))
        }
    }

    pub fn exact_lookup(
        &self,
        index_id: i32,
        field_name: &str,
        field_value: &Value,
    ) -> Result<(Vec<Vertex>, Vec<Edge>), String> {
        if let Some(storage) = self.storages.get(&index_id) {
            let (vertices, edges, _) = storage.exact_lookup(field_name, field_value)?;
            Ok((vertices, edges))
        } else {
            Err(format!("索引 {} 不存在", index_id))
        }
    }

    pub fn prefix_lookup(
        &self,
        index_id: i32,
        field_name: &str,
        prefix: &[Value],
    ) -> Result<(Vec<Vertex>, Vec<Edge>), String> {
        if let Some(storage) = self.storages.get(&index_id) {
            let (vertices, edges, _) = storage.prefix_lookup(field_name, prefix)?;
            Ok((vertices, edges))
        } else {
            Err(format!("索引 {} 不存在", index_id))
        }
    }

    pub fn range_lookup(
        &self,
        index_id: i32,
        field_name: &str,
        start_value: &Value,
        end_value: &Value,
    ) -> Result<(Vec<Vertex>, Vec<Edge>), String> {
        if let Some(storage) = self.storages.get(&index_id) {
            let (vertices, edges, _) = storage.range_lookup(field_name, start_value, end_value)?;
            Ok((vertices, edges))
        } else {
            Err(format!("索引 {} 不存在", index_id))
        }
    }

    pub fn delete_vertex(&self, index_id: i32, vertex: &Vertex) -> Result<(), String> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.delete_vertex(vertex);
            Ok(())
        } else {
            Err(format!("索引 {} 不存在", index_id))
        }
    }

    pub fn delete_edge(&self, index_id: i32, edge: &Edge) -> Result<(), String> {
        if let Some(storage) = self.storages.get(&index_id) {
            storage.delete_edge(edge);
            Ok(())
        } else {
            Err(format!("索引 {} 不存在", index_id))
        }
    }

    pub fn get_global_stats(&self) -> &IndexQueryStats {
        &self.global_stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Tag;
    use crate::storage::MemoryStorage;

    fn create_test_vertex(id: i64, name: &str, age: i64) -> Vertex {
        Vertex {
            vid: Box::new(Value::Int(id)),
            id,
            tags: vec![Tag {
                name: "person".to_string(),
                properties: vec![
                    ("name".to_string(), Value::String(name.to_string())),
                    ("age".to_string(), Value::Int(age)),
                ]
                .into_iter()
                .collect(),
            }],
            properties: vec![
                ("name".to_string(), Value::String(name.to_string())),
                ("age".to_string(), Value::Int(age)),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn create_test_edge(id: i64, edge_type: &str, weight: f64) -> Edge {
        Edge {
            src: Box::new(Value::Int(1)),
            dst: Box::new(Value::Int(2)),
            edge_type: edge_type.to_string(),
            props: vec![
                ("weight".to_string(), Value::Float(weight)),
            ]
            .into_iter()
            .collect(),
            ranking: 0,
            id,
        }
    }

    #[test]
    fn test_concurrent_index_storage_insert() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new().expect("Failed to create MemoryStorage in test")));
        let index_storage = ConcurrentIndexStorage::new(1, 1, "test".to_string(), storage.clone());

        let vertex = create_test_vertex(1, "Alice", 30);
        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex.clone()).expect("Failed to insert node in test");
        index_storage.insert_vertex("name", &Value::String("Alice".to_string()), &vertex);

        assert_eq!(index_storage.get_entry_count(), 1);

        let (vertices, _, _) = index_storage.exact_lookup("name", &Value::String("Alice".to_string())).expect("Failed to perform exact lookup in test");
        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].id, 1);
    }

    #[test]
    fn test_concurrent_index_storage_prefix_lookup() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new().expect("Failed to create MemoryStorage in test")));
        let index_storage = ConcurrentIndexStorage::new(1, 1, "test".to_string(), storage.clone());

        let vertex1 = create_test_vertex(1, "Alice", 30);
        let vertex2 = create_test_vertex(2, "Bob", 25);
        let vertex3 = create_test_vertex(3, "Alex", 35);

        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex1.clone()).expect("Failed to insert node in test");
        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex2.clone()).expect("Failed to insert node in test");
        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex3.clone()).expect("Failed to insert node in test");

        index_storage.insert_vertex("name", &Value::String("Alice".to_string()), &vertex1);
        index_storage.insert_vertex("name", &Value::String("Bob".to_string()), &vertex2);
        index_storage.insert_vertex("name", &Value::String("Alex".to_string()), &vertex3);

        let prefix = vec![Value::String("A".to_string())];
        let (vertices, _, _) = index_storage.prefix_lookup("name", &prefix).expect("Failed to perform prefix lookup in test");

        assert_eq!(vertices.len(), 2);
    }

    #[test]
    fn test_concurrent_index_storage_range_lookup() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new().expect("Failed to create MemoryStorage in test")));
        let index_storage = ConcurrentIndexStorage::new(1, 1, "test".to_string(), storage.clone());

        let vertex1 = create_test_vertex(1, "Alice", 20);
        let vertex2 = create_test_vertex(2, "Bob", 30);
        let vertex3 = create_test_vertex(3, "Charlie", 40);

        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex1.clone()).expect("Failed to insert node in test");
        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex2.clone()).expect("Failed to insert node in test");
        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex3.clone()).expect("Failed to insert node in test");

        index_storage.insert_vertex("age", &Value::Int(20), &vertex1);
        index_storage.insert_vertex("age", &Value::Int(30), &vertex2);
        index_storage.insert_vertex("age", &Value::Int(40), &vertex3);

        let (vertices, _, _) = index_storage.range_lookup("age", &Value::Int(25), &Value::Int(35)).expect("Failed to perform range lookup in test");

        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices[0].id, 2);
    }

    #[test]
    fn test_concurrent_index_manager() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new().expect("Failed to create MemoryStorage in test")));
        let manager = ConcurrentIndexManager::new(1, storage.clone());

        let fields = vec![
            IndexField {
                name: "name".to_string(),
                value_type: Value::String("".to_string()),
                is_nullable: false,
            },
        ];

        let index_id = manager.create_index("person_name", "person", fields, false).expect("Failed to create index in test");

        let vertex = create_test_vertex(1, "Alice", 30);
        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex.clone()).expect("Failed to insert node in test");
        manager.insert_vertex(index_id, "name", &Value::String("Alice".to_string()), &vertex).expect("Failed to insert vertex in test");

        let (vertices, _) = manager.exact_lookup(index_id, "name", &Value::String("Alice".to_string())).expect("Failed to perform exact lookup in test");
        assert_eq!(vertices.len(), 1);
    }

    #[test]
    fn test_concurrent_edge_index() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new().expect("Failed to create MemoryStorage in test")));
        let index_storage = ConcurrentIndexStorage::new(1, 1, "test".to_string(), storage.clone());

        let edge = create_test_edge(1, "friend", 0.9);
        index_storage.insert_edge("weight", &Value::Float(0.9), &edge);

        assert_eq!(index_storage.get_entry_count(), 1);
    }

    #[test]
    fn test_query_stats() {
        let storage = Arc::new(Mutex::new(MemoryStorage::new().expect("Failed to create MemoryStorage in test")));
        let index_storage = ConcurrentIndexStorage::new(1, 1, "test".to_string(), storage.clone());

        let vertex = create_test_vertex(1, "Alice", 30);
        storage.lock().expect("Failed to acquire lock in test").insert_node(vertex.clone()).expect("Failed to insert node in test");
        index_storage.insert_vertex("name", &Value::String("Alice".to_string()), &vertex);

        let _ = index_storage.exact_lookup("name", &Value::String("Alice".to_string())).expect("Failed to perform exact lookup in test");
        let _ = index_storage.exact_lookup("name", &Value::String("NonExistent".to_string())).expect("Failed to perform exact lookup in test");

        let stats = index_storage.get_query_stats();
        assert_eq!(stats.get_total_queries(), 2);
        assert_eq!(stats.get_exact_hits(), 1);
        let total_hits = stats.get_total_hits();
        let total_queries = stats.get_total_queries();
        assert_eq!(total_queries - total_hits, 1);
        assert!((stats.get_hit_rate() - 0.5).abs() < 0.01);
    }
}
