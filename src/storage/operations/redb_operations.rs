use crate::core::{Edge, EdgeDirection, Value, Vertex, StorageError};
use crate::storage::operations::{VertexReader, EdgeReader, VertexWriter, EdgeWriter, ScanResult};
use crate::storage::serializer::{vertex_to_bytes, vertex_from_bytes, edge_to_bytes, edge_from_bytes, value_to_bytes, generate_id};
use redb::{Database, ReadableTable, TableDefinition};
use lru::LruCache;
use serde_json;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

const NODES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("nodes");
const EDGES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("edges");
const INDEXES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("indexes");

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

pub struct RedbReader {
    db: Database,
    vertex_cache: Arc<Mutex<LruCache<Vec<u8>, Vertex>>>,
    edge_cache: Arc<Mutex<LruCache<Vec<u8>, Edge>>>,
}

impl std::fmt::Debug for RedbReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbReader").finish()
    }
}

impl RedbReader {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path.as_ref())
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let vertex_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for vertex cache");
        let edge_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for edge cache");
        let vertex_cache = Arc::new(Mutex::new(LruCache::new(vertex_cache_size)));
        let edge_cache = Arc::new(Mutex::new(LruCache::new(edge_cache_size)));

        Ok(Self {
            db,
            vertex_cache,
            edge_cache,
        })
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
                let vertex: Vertex = vertex_from_bytes(&vertex_bytes.0)?;
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
                let edge: Edge = edge_from_bytes(&edge_bytes.0)?;
                Ok(Some(edge))
            }
            None => Ok(None),
        }
    }

    fn get_node_edge_keys(&self, node_id: &Value) -> Result<Vec<Vec<u8>>, StorageError> {
        let read_txn = self
            .db
            .begin_read()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        let table = read_txn
            .open_table(INDEXES_TABLE)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

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
}

impl VertexReader for RedbReader {
    fn get_vertex(&self, _space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let id_bytes = value_to_bytes(id)?;

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

    fn scan_vertices(&self, _space: &str) -> Result<ScanResult<Vertex>, StorageError> {
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
             let vertex: Vertex = vertex_from_bytes(&vertex_bytes.value().0)?;
             vertices.push(vertex);
         }

        Ok(ScanResult::new(vertices))
    }

    fn scan_vertices_by_tag(
        &self,
        _space: &str,
        tag: &str,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        let all_vertices = self.scan_vertices(_space)?;
        let filtered_vertices = all_vertices
            .into_vec()
            .into_iter()
            .filter(|vertex| vertex.tags.iter().any(|vertex_tag| vertex_tag.name == tag))
            .collect();

        Ok(ScanResult::new(filtered_vertices))
    }

    fn scan_vertices_by_prop(
        &self,
        _space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        let all_vertices = self.scan_vertices(_space)?;
        let filtered_vertices = all_vertices
            .into_vec()
            .into_iter()
            .filter(|vertex| {
                vertex.tags.iter().any(|vertex_tag| vertex_tag.name == tag)
                    && vertex.properties.get(prop).map_or(false, |p| p == value)
            })
            .collect();

        Ok(ScanResult::new(filtered_vertices))
    }
}

impl EdgeReader for RedbReader {
    fn get_edge(
        &self,
        _space: &str,
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
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError> {
        self.get_node_edges_filtered(_space, node_id, direction, None)
    }

    fn get_node_edges_filtered(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<ScanResult<Edge>, StorageError> {
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

        Ok(ScanResult::new(edges))
    }

    fn scan_edges_by_type(
        &self,
        _space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let edge_keys = self.get_edge_keys_by_type(edge_type)?;
        let mut edges = Vec::new();

        for edge_key_bytes in edge_keys {
            if let Some(edge) = self.get_edge_from_bytes(&edge_key_bytes)? {
                edges.push(edge);
            }
        }

        Ok(ScanResult::new(edges))
    }

    fn scan_all_edges(&self, _space: &str) -> Result<ScanResult<Edge>, StorageError> {
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
            let edge: Edge = edge_from_bytes(&edge_bytes.value().0)?;
            edges.push(edge);
        }

        Ok(ScanResult::new(edges))
    }
}

pub struct RedbWriter {
    db: Database,
    vertex_cache: Arc<Mutex<LruCache<Vec<u8>, Vertex>>>,
    edge_cache: Arc<Mutex<LruCache<Vec<u8>, Edge>>>,
}

impl std::fmt::Debug for RedbWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbWriter").finish()
    }
}

impl RedbWriter {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let db = Database::create(path.as_ref())
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let vertex_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for vertex cache");
        let edge_cache_size = std::num::NonZeroUsize::new(1000)
            .expect("Failed to create NonZeroUsize for edge cache");
        let vertex_cache = Arc::new(Mutex::new(LruCache::new(vertex_cache_size)));
        let edge_cache = Arc::new(Mutex::new(LruCache::new(edge_cache_size)));

        Ok(Self {
            db,
            vertex_cache,
            edge_cache,
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

impl VertexWriter for RedbWriter {
    fn insert_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let id = generate_id();
        let vertex_with_id = Vertex::new(id.clone(), vertex.tags);

        let vertex_bytes = vertex_to_bytes(&vertex_with_id)?;
        let id_bytes = value_to_bytes(&id)?;

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

    fn update_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<(), StorageError> {
        if matches!(*vertex.vid, Value::Null(_)) {
            return Err(StorageError::NodeNotFound(Value::Null(Default::default())));
        }

        let vertex_bytes = vertex_to_bytes(&vertex)?;
        let id_bytes = value_to_bytes(&vertex.vid)?;

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

    fn delete_vertex(&mut self, _space: &str, id: &Value) -> Result<(), StorageError> {
        let id_bytes = value_to_bytes(id)?;

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

    fn batch_insert_vertices(&mut self, _space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
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
                let id = generate_id();
                let vertex_with_id = Vertex::new(id.clone(), vertex.tags);

                let vertex_bytes = vertex_to_bytes(&vertex_with_id)?;
                let id_bytes = value_to_bytes(&id)?;

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
}

impl EdgeWriter for RedbWriter {
    fn insert_edge(&mut self, _space: &str, edge: Edge) -> Result<(), StorageError> {
        let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
        let edge_key_bytes = edge_key.as_bytes().to_vec();

        let edge_bytes = edge_to_bytes(&edge)?;

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

    fn delete_edge(
        &mut self,
        _space: &str,
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

    fn batch_insert_edges(&mut self, _space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
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
                let edge_bytes = edge_to_bytes(edge)?;

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
}
