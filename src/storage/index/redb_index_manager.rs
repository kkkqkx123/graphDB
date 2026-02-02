use crate::core::{Edge, Value, Vertex};
use crate::core::StorageResult;
use crate::index::{Index, IndexStatus, IndexInfo, IndexOptimization};
use crate::storage::serializer::{vertex_from_bytes, edge_from_bytes, value_to_bytes};
use redb::{Database, ReadableTable, TableDefinition};
use serde_json;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ByteKey(pub Vec<u8>);

impl redb::Key for ByteKey {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
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

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("graphdb::ByteKey")
    }
}

const INDEXES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("indexes");

pub struct RedbIndexManager {
    db: Database,
    indexes: Arc<std::sync::Mutex<HashMap<String, Index>>>,
}

impl std::fmt::Debug for RedbIndexManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbIndexManager").finish()
    }
}

impl RedbIndexManager {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path.as_ref())
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(Self {
            db,
            indexes: Arc::new(std::sync::Mutex::new(HashMap::new())),
        })
    }

    fn update_node_edge_index(
        &self,
        node_id: &Value,
        edge_key: &[u8],
        add: bool,
    ) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let node_id_bytes = value_to_bytes(node_id)?;
            let index_key = format!("node_edge_index:{:?}", node_id);
            let index_key_bytes = index_key.as_bytes();

            let mut edge_list: Vec<Vec<u8>> = match table
                .get(ByteKey(index_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let list_bytes = value.value();
                    let result: Vec<Vec<u8>> =
                        serde_json::from_slice(&list_bytes.0)
                            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
                    result
                }
                None => Vec::new(),
            };

            if add {
                if !edge_list.contains(&edge_key.to_vec()) {
                    edge_list.push(edge_key.to_vec());
                }
            } else {
                edge_list.retain(|key| key != edge_key);
            }

            let list_bytes =
                serde_json::to_vec(&edge_list)
                    .map_err(|e| StorageError::SerializeError(e.to_string()))?;

            table
                .insert(ByteKey(index_key_bytes.to_vec()), ByteKey(list_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn get_node_edge_keys(&self, node_id: &Value) -> Result<Vec<Vec<u8>>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let node_id_bytes = value_to_bytes(node_id)?;
        let index_key = format!("node_edge_index:{:?}", node_id);
        let index_key_bytes = index_key.as_bytes();

        match table
            .get(ByteKey(index_key_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let list_bytes = value.value();
                    let edge_key_list: Vec<Vec<u8>> =
                        serde_json::from_slice(&list_bytes.0)
                            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
                    Ok(edge_key_list)
                }
                None => Ok(Vec::new()),
            }
    }

    fn update_edge_type_index(
        &self,
        edge_type: &str,
        edge_key: &[u8],
        add: bool,
    ) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let index_key = format!("edge_type_index:{}", edge_type);
            let index_key_bytes = index_key.as_bytes();

            let mut edge_list: Vec<Vec<u8>> = match table
                .get(ByteKey(index_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let list_bytes = value.value();
                    let result: Vec<Vec<u8>> =
                        serde_json::from_slice(&list_bytes.0)
                            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
                    result
                }
                None => Vec::new(),
            };

            if add {
                if !edge_list.contains(&edge_key.to_vec()) {
                    edge_list.push(edge_key.to_vec());
                }
            } else {
                edge_list.retain(|key| key != edge_key);
            }

            let list_bytes =
                serde_json::to_vec(&edge_list)
                    .map_err(|e| StorageError::SerializeError(e.to_string()))?;

            table
                .insert(ByteKey(index_key_bytes.to_vec()), ByteKey(list_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn get_edge_keys_by_type(&self, edge_type: &str) -> Result<Vec<Vec<u8>>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let index_key = format!("edge_type_index:{}", edge_type);
        let index_key_bytes = index_key.as_bytes();

        match table
            .get(ByteKey(index_key_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            Some(value) => {
                let list_bytes = value.value();
                let edge_key_list: Vec<Vec<u8>> =
                    serde_json::from_slice(&list_bytes.0)
                        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
                Ok(edge_key_list)
            }
            None => Ok(Vec::new()),
        }
    }

    fn update_prop_index(
        &self,
        tag: &str,
        prop: &str,
        value: &Value,
        vertex_id: &Value,
        add: bool,
    ) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let index_key = format!("prop_index:{}:{}:{:?}", tag, prop, value);
            let index_key_bytes = index_key.as_bytes();
            let vertex_id_bytes = value_to_bytes(vertex_id)?;

            let mut vertex_list: Vec<Vec<u8>> = match table
                .get(ByteKey(index_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let list_bytes = value.value();
                    let result: Vec<Vec<u8>> =
                        serde_json::from_slice(&list_bytes.0)
                            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
                    result
                }
                None => Vec::new(),
            };

            if add {
                if !vertex_list.contains(&vertex_id_bytes) {
                    vertex_list.push(vertex_id_bytes);
                }
            } else {
                vertex_list.retain(|id| id != &vertex_id_bytes);
            }

            let list_bytes =
                serde_json::to_vec(&vertex_list)
                    .map_err(|e| StorageError::SerializeError(e.to_string()))?;

            table
                .insert(ByteKey(index_key_bytes.to_vec()), ByteKey(list_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }
}

impl crate::storage::index::IndexManager for RedbIndexManager {
    fn get_index(&self, _name: &str) -> Option<Index> {
        let indexes = self.indexes.lock().unwrap();
        indexes.get(_name).cloned()
    }

    fn list_indexes(&self) -> Vec<String> {
        let indexes = self.indexes.lock().unwrap();
        indexes.keys().cloned().collect()
    }

    fn has_index(&self, _name: &str) -> bool {
        let indexes = self.indexes.lock().unwrap();
        indexes.contains_key(_name)
    }

    fn create_index(&self, _space_id: i32, _index: Index) -> StorageResult<i32> {
        let mut indexes = self.indexes.lock().unwrap();
        let id = indexes.len() as i32;
        indexes.insert(_index.name.clone(), _index);
        Ok(id)
    }

    fn drop_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> {
        let mut indexes = self.indexes.lock().unwrap();
        indexes.retain(|_, v| v.id != _index_id);
        Ok(())
    }

    fn get_index_status(&self, _space_id: i32, _index_id: i32) -> Option<IndexStatus> {
        Some(IndexStatus::Active)
    }

    fn list_indexes_by_space(&self, _space_id: i32) -> StorageResult<Vec<Index>> {
        let indexes = self.indexes.lock().unwrap();
        Ok(indexes.values().cloned().collect())
    }

    fn lookup_vertex_by_index(
        &self,
        _space_id: i32,
        _index_name: &str,
        _values: &[Value],
    ) -> StorageResult<Vec<Vertex>> {
        Ok(Vec::new())
    }

    fn lookup_edge_by_index(
        &self,
        _space_id: i32,
        _index_name: &str,
        _values: &[Value],
    ) -> StorageResult<Vec<Edge>> {
        Ok(Vec::new())
    }

    fn range_lookup_vertex(
        &self,
        _space_id: i32,
        _index_name: &str,
        _start: &Value,
        _end: &Value,
    ) -> StorageResult<Vec<Vertex>> {
        Ok(Vec::new())
    }

    fn range_lookup_edge(
        &self,
        _space_id: i32,
        _index_name: &str,
        _start: &Value,
        _end: &Value,
    ) -> StorageResult<Vec<Edge>> {
        Ok(Vec::new())
    }

    fn insert_vertex_to_index(&self, _space_id: i32, _vertex: &Vertex) -> StorageResult<()> {
        Ok(())
    }

    fn delete_vertex_from_index(&self, _space_id: i32, _vertex: &Vertex) -> StorageResult<()> {
        Ok(())
    }

    fn update_vertex_in_index(
        &self,
        _space_id: i32,
        _old_vertex: &Vertex,
        _new_vertex: &Vertex,
    ) -> StorageResult<()> {
        Ok(())
    }

    fn insert_edge_to_index(&self, _space_id: i32, _edge: &Edge) -> StorageResult<()> {
        Ok(())
    }

    fn delete_edge_from_index(&self, _space_id: i32, _edge: &Edge) -> StorageResult<()> {
        Ok(())
    }

    fn update_edge_in_index(
        &self,
        _space_id: i32,
        _old_edge: &Edge,
        _new_edge: &Edge,
    ) -> StorageResult<()> {
        Ok(())
    }

    fn load_from_disk(&self) -> StorageResult<()> {
        Ok(())
    }

    fn save_to_disk(&self) -> StorageResult<()> {
        Ok(())
    }

    fn rebuild_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> {
        Ok(())
    }

    fn rebuild_all_indexes(&self, _space_id: i32) -> StorageResult<()> {
        Ok(())
    }

    fn get_index_stats(&self, _space_id: i32, _index_id: i32) -> StorageResult<IndexInfo> {
        Ok(IndexInfo::default())
    }

    fn get_all_index_stats(&self, _space_id: i32) -> StorageResult<Vec<IndexInfo>> {
        Ok(Vec::new())
    }

    fn analyze_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<IndexOptimization> {
        Ok(IndexOptimization::default())
    }

    fn analyze_all_indexes(&self, _space_id: i32) -> StorageResult<Vec<IndexOptimization>> {
        Ok(Vec::new())
    }

    fn check_index_consistency(&self, _space_id: i32, _index_id: i32) -> StorageResult<bool> {
        Ok(true)
    }

    fn repair_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> {
        Ok(())
    }

    fn cleanup_index(&self, _space_id: i32, _index_id: i32) -> StorageResult<()> {
        Ok(())
    }

    fn batch_insert_vertices(&self, _space_id: i32, _vertices: &[Vertex]) -> StorageResult<()> {
        Ok(())
    }

    fn batch_delete_vertices(&self, _space_id: i32, _vertices: &[Vertex]) -> StorageResult<()> {
        Ok(())
    }

    fn batch_insert_edges(&self, _space_id: i32, _edges: &[Edge]) -> StorageResult<()> {
        Ok(())
    }

    fn batch_delete_edges(&self, _space_id: i32, _edges: &[Edge]) -> StorageResult<()> {
        Ok(())
    }
}

use crate::core::StorageError;
