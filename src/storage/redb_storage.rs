use super::{StorageEngine, TransactionId};
use crate::common::fs::FileLock;
use crate::core::{Edge, StorageError, Value, Vertex, EdgeDirection};
use bincode;
use lru::LruCache;
use redb::{Database, ReadableTable, TableDefinition, TypeName};
use std::collections::HashMap;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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

    fn type_name() -> TypeName {
        TypeName::new("graphdb::ByteKey")
    }
}

const NODES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("nodes");
const EDGES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("edges");
const INDEXES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("indexes");

pub struct RedbStorage {
    db: Database,
    db_path: String,
    lock_file_path: String,
    vertex_cache: Arc<Mutex<LruCache<Vec<u8>, Vertex>>>,
    edge_cache: Arc<Mutex<LruCache<Vec<u8>, Edge>>>,
    active_transactions: Arc<Mutex<HashMap<TransactionId, ()>>>,
}

impl std::fmt::Debug for RedbStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbStorage")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl Clone for RedbStorage {
    fn clone(&self) -> Self {
        Self::new(&self.db_path).expect("Failed to clone RedbStorage")
    }
}

impl RedbStorage {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db_path = path.as_ref().to_string_lossy().to_string();

        let db = Database::create(path.as_ref())
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let lock_file_path = format!("{}.lock", db_path);

        let vertex_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for vertex cache");
        let edge_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for edge cache");
        let vertex_cache = Arc::new(Mutex::new(LruCache::new(vertex_cache_size)));
        let edge_cache = Arc::new(Mutex::new(LruCache::new(edge_cache_size)));
        let active_transactions = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            db,
            db_path,
            lock_file_path,
            vertex_cache,
            edge_cache,
            active_transactions,
        })
    }

    fn acquire_exclusive_lock(&self) -> Result<FileLock, StorageError> {
        FileLock::acquire_exclusive(&self.lock_file_path)
            .map_err(|e| StorageError::DbError(format!("获取文件锁失败: {}", e)))
    }

    fn value_to_bytes(&self, value: &Value) -> Result<Vec<u8>, StorageError> {
        bincode::encode_to_vec(value, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn value_from_bytes(&self, bytes: &[u8]) -> Result<Value, StorageError> {
        let (value, _): (Value, usize) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(value)
    }

    fn vertex_to_bytes(&self, vertex: &Vertex) -> Result<Vec<u8>, StorageError> {
        bincode::encode_to_vec(vertex, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn vertex_from_bytes(&self, bytes: &[u8]) -> Result<Vertex, StorageError> {
        let (vertex, _): (Vertex, usize) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(vertex)
    }

    fn edge_to_bytes(&self, edge: &Edge) -> Result<Vec<u8>, StorageError> {
        bincode::encode_to_vec(edge, bincode::config::standard())
            .map_err(|e| StorageError::SerializationError(e.to_string()))
    }

    fn edge_from_bytes(&self, bytes: &[u8]) -> Result<Edge, StorageError> {
        let (edge, _): (Edge, usize) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(edge)
    }

    fn generate_id(&self) -> Value {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;
        Value::Int(id as i64)
    }

    fn get_node_from_bytes(&self, id_bytes: &[u8]) -> Result<Option<Vertex>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(NODES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table.get(ByteKey(id_bytes.to_vec())).map_err(|e| StorageError::DbError(e.to_string()))? {
            Some(value) => {
                let vertex_bytes = value.value();
                let vertex: Vertex = self.vertex_from_bytes(&vertex_bytes.0)?;
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }

    fn get_edge_from_bytes(&self, edge_key_bytes: &[u8]) -> Result<Option<Edge>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(EDGES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        match table
            .get(ByteKey(edge_key_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            Some(value) => {
                let edge_bytes = value.value();
                let edge: Edge = self.edge_from_bytes(&edge_bytes.0)?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
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

            let _node_id_bytes = self.value_to_bytes(node_id)?;
            let index_key = format!("node_edge_index:{:?}", node_id);
            let index_key_bytes = index_key.as_bytes();

            let mut edge_list: Vec<Vec<u8>> = match table
                .get(ByteKey(index_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let list_bytes = value.value();
                    let (result, _): (Vec<Vec<u8>>, usize) =
                        bincode::decode_from_slice(&list_bytes.0, bincode::config::standard())
                            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
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
                bincode::encode_to_vec(&edge_list, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

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

        let _node_id_bytes = self.value_to_bytes(node_id)?;
        let index_key = format!("node_edge_index:{:?}", node_id);
        let index_key_bytes = index_key.as_bytes();

        match table
            .get(ByteKey(index_key_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let list_bytes = value.value();
                    let (edge_key_list, _): (Vec<Vec<u8>>, usize) =
                        bincode::decode_from_slice(&list_bytes.0, bincode::config::standard())
                            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
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
                    let (result, _): (Vec<Vec<u8>>, usize) =
                        bincode::decode_from_slice(&list_bytes.0, bincode::config::standard())
                            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
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
                bincode::encode_to_vec(&edge_list, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

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

        let edge_type_bytes = edge_type.as_bytes();
        let index_key = format!("edge_type_index:{}", edge_type);
        let index_key_bytes = index_key.as_bytes();

        match table
            .get(ByteKey(index_key_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            Some(value) => {
                let list_bytes = value.value();
                let (edge_key_list, _): (Vec<Vec<u8>>, usize) =
                    bincode::decode_from_slice(&list_bytes.0, bincode::config::standard())
                        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
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
            let vertex_id_bytes = self.value_to_bytes(vertex_id)?;

            let mut vertex_list: Vec<Vec<u8>> = match table
                .get(ByteKey(index_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
            {
                Some(value) => {
                    let list_bytes = value.value();
                    let (result, _): (Vec<Vec<u8>>, usize) =
                        bincode::decode_from_slice(&list_bytes.0, bincode::config::standard())
                            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
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
                bincode::encode_to_vec(&vertex_list, bincode::config::standard())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            table
                .insert(ByteKey(index_key_bytes.to_vec()), ByteKey(list_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }

    fn get_vertices_by_prop(
        &self,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let index_key = format!("prop_index:{}:{}:{:?}", tag, prop, value);
        let index_key_bytes = index_key.as_bytes();

        match table
            .get(ByteKey(index_key_bytes.to_vec()))
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            Some(value) => {
                let list_bytes = value.value();
                let (vertex_id_list, _): (Vec<Vec<u8>>, usize) =
                    bincode::decode_from_slice(&list_bytes.0, bincode::config::standard())
                        .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                let mut vertices = Vec::new();
                for vertex_id_bytes in vertex_id_list {
                    if let Some(vertex) = self.get_node_from_bytes(&vertex_id_bytes)? {
                        vertices.push(vertex);
                    }
                }
                Ok(vertices)
            }
            None => Ok(Vec::new()),
        }
    }
}

impl StorageEngine for RedbStorage {
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError> {
        let id = self.generate_id();
        let vertex_with_id = Vertex::new(id.clone(), vertex.tags);

        let vertex_bytes = self.vertex_to_bytes(&vertex_with_id)?;
        let id_bytes = self.value_to_bytes(&id)?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(id)
    }

    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let id_bytes = self.value_to_bytes(id)?;

        {
            let mut cache = self.vertex_cache.lock().expect("Failed to lock vertex cache");
            if let Some(vertex) = cache.get(&id_bytes) {
                return Ok(Some(vertex.clone()));
            }
        }

        match self.get_node_from_bytes(&id_bytes)? {
            Some(vertex) => {
                {
                    let mut cache = self.vertex_cache.lock().expect("Failed to lock vertex cache");
                    cache.put(id_bytes.clone(), vertex.clone());
                }
                Ok(Some(vertex))
            }
            None => Ok(None),
        }
    }

    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError> {
        if matches!(*vertex.vid, Value::Null(_)) {
            return Err(StorageError::NodeNotFound(Value::Null(Default::default())));
        }

        let vertex_bytes = self.vertex_to_bytes(&vertex)?;
        let id_bytes = self.value_to_bytes(&vertex.vid)?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .insert(ByteKey(id_bytes.clone()), ByteKey(vertex_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        {
            let mut cache = self.vertex_cache.lock().expect("Failed to lock vertex cache");
            cache.put(id_bytes, vertex);
        }

        Ok(())
    }

    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError> {
        let edges_to_delete = self.get_node_edges(id, EdgeDirection::Both)?;
        for edge in edges_to_delete {
            self.delete_edge(&edge.src, &edge.dst, &edge.edge_type)?;
        }

        let id_bytes = self.value_to_bytes(id)?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .remove(ByteKey(id_bytes.clone()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let mut index_table = write_txn
                .open_table(INDEXES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            let index_key = format!("node_edge_index:{:?}", id);
            let index_key_bytes = index_key.as_bytes().to_vec();
            index_table
                .remove(ByteKey(index_key_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        {
            let mut cache = self.vertex_cache.lock().expect("Failed to lock vertex cache");
            cache.pop(&id_bytes);
        }

        Ok(())
    }

    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(NODES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut vertices = Vec::new();
        for result in table.iter()
             .map_err(|e| StorageError::DbError(e.to_string()))?
         {
             let (_, vertex_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
             let vertex: Vertex = self.vertex_from_bytes(&vertex_bytes.value().0)?;
             vertices.push(vertex);
         }

        Ok(vertices)
    }

    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        let all_vertices = self.scan_all_vertices()?;
        let filtered_vertices = all_vertices
            .into_iter()
            .filter(|vertex| vertex.tags.iter().any(|vertex_tag| vertex_tag.name == tag))
            .collect();

        Ok(filtered_vertices)
    }

    fn scan_vertices_by_prop(&self, tag: &str, prop: &str, value: &Value) -> Result<Vec<Vertex>, StorageError> {
        self.get_vertices_by_prop(tag, prop, value)
    }

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        let edge_bytes = self.edge_to_bytes(&edge)?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .insert(ByteKey(edge_key_bytes.clone()), ByteKey(edge_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        self.update_node_edge_index(&edge.src, &edge_key_bytes, true)?;
        self.update_node_edge_index(&edge.dst, &edge_key_bytes, true)?;
        self.update_edge_type_index(&edge.edge_type, &edge_key_bytes, true)?;

        for (prop_name, prop_value) in &edge.props {
            self.update_prop_index(&edge.edge_type, prop_name, prop_value, &edge.src, true)?;
        }

        Ok(())
    }

    fn get_edge(
        &self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", src, dst, edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        {
            let mut cache = self.edge_cache.lock().expect("Failed to lock edge cache");
            if let Some(edge) = cache.get(&edge_key_bytes) {
                return Ok(Some(edge.clone()));
            }
        }

        match self.get_edge_from_bytes(&edge_key_bytes)? {
            Some(edge) => {
                {
                    let mut cache = self.edge_cache.lock().expect("Failed to lock edge cache");
                    cache.put(edge_key_bytes.clone(), edge.clone());
                }
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    fn get_node_edges(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.get_node_edges_filtered(node_id, direction, None)
    }

    fn get_node_edges_filtered(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, StorageError> {
        let edge_keys = self.get_node_edge_keys(node_id)?;
        let mut edges = Vec::new();

        for edge_key_bytes in edge_keys {
            if let Some(edge) = self.get_edge_from_bytes(&edge_key_bytes)? {
                let matches_direction = match direction {
                    EdgeDirection::Out => *edge.src == *node_id,
                    EdgeDirection::In => *edge.dst == *node_id,
                    EdgeDirection::Both => *edge.src == *node_id || *edge.dst == *node_id,
                };

                if matches_direction {
                    if let Some(ref f) = filter {
                        if !f(&edge) {
                            continue;
                        }
                    }
                    edges.push(edge);
                }
            }
        }

        Ok(edges)
    }

    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", src, dst, edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            table
                .remove(ByteKey(edge_key_bytes.clone()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        self.update_node_edge_index(src, &edge_key_bytes, false)?;
        self.update_node_edge_index(dst, &edge_key_bytes, false)?;
        self.update_edge_type_index(edge_type, &edge_key_bytes, false)?;

        {
            let mut cache = self.edge_cache.lock().expect("Failed to lock edge cache");
            cache.pop(&edge_key_bytes);
        }

        Ok(())
    }

    fn scan_edges_by_type(&self, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        let edge_keys = self.get_edge_keys_by_type(edge_type)?;
        let mut edges = Vec::new();

        for edge_key_bytes in edge_keys {
            if let Some(edge) = self.get_edge_from_bytes(&edge_key_bytes)? {
                edges.push(edge);
            }
        }

        Ok(edges)
    }

    fn scan_all_edges(&self) -> Result<Vec<Edge>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(EDGES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let mut edges = Vec::new();
        for result in table.iter()
            .map_err(|e| StorageError::DbError(e.to_string()))?
        {
            let (_, edge_bytes) = result.map_err(|e| StorageError::DbError(e.to_string()))?;
            let edge: Edge = self.edge_from_bytes(&edge_bytes.value().0)?;
            edges.push(edge);
        }

        Ok(edges)
    }

    fn batch_insert_nodes(&mut self, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        let mut ids = Vec::new();

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            for vertex in vertices {
                let id = self.generate_id();
                let vertex_with_id = Vertex::new(id.clone(), vertex.tags);

                let vertex_bytes = self.vertex_to_bytes(&vertex_with_id)?;
                let id_bytes = self.value_to_bytes(&id)?;

                table
                    .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;

                ids.push(id);
            }
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(ids)
    }

    fn batch_insert_edges(&mut self, edges: Vec<Edge>) -> Result<(), StorageError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            for edge in &edges {
                let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
                let edge_key_bytes = edge_key.as_bytes().to_vec();
                let edge_bytes = self.edge_to_bytes(edge)?;

                table
                    .insert(ByteKey(edge_key_bytes), ByteKey(edge_bytes))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        for edge in edges {
            let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
            let edge_key_bytes = edge_key.as_bytes().to_vec();

            self.update_node_edge_index(&edge.src, &edge_key_bytes, true)?;
            self.update_node_edge_index(&edge.dst, &edge_key_bytes, true)?;
            self.update_edge_type_index(&edge.edge_type, &edge_key_bytes, true)?;

            for (prop_name, prop_value) in &edge.props {
                self.update_prop_index(&edge.edge_type, prop_name, prop_value, &edge.src, true)?;
            }
        }

        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        let _lock = self.acquire_exclusive_lock()?;

        let tx_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos() as u64;

        let mut active_transactions = self.active_transactions.lock().expect("Failed to lock active transactions");
        active_transactions.insert(tx_id, ());

        Ok(tx_id)
    }

    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.lock().expect("Failed to lock active transactions");
        if !active_transactions.contains_key(&tx_id) {
            return Err(StorageError::TransactionNotFound(tx_id));
        }

        active_transactions.remove(&tx_id);

        Ok(())
    }

    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.lock().expect("Failed to lock active transactions");
        if !active_transactions.contains_key(&tx_id) {
            return Err(StorageError::TransactionNotFound(tx_id));
        }

        active_transactions.remove(&tx_id);

        Ok(())
    }
}
