//! 基于 redb 的索引持久化实现
//!
//! 提供索引元数据和索引数据的持久化存储功能

use crate::core::StorageError;
use crate::index::Index;
use bincode::{Decode, Encode};
use redb::{Database, ReadableTable, TableDefinition, TypeName};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
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

#[derive(Debug)]
pub struct RedbIndexPersistence {
    db: Database,
    db_path: String,
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
            .map_err(|e| StorageError::SerializeError(e.to_string()))
    }

    fn value_from_bytes<'a, T: serde::Deserialize<'a> + bincode::Decode<()>>(&self, bytes: &'a [u8]) -> Result<T, StorageError> {
        let (value, _): (T, usize) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
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
