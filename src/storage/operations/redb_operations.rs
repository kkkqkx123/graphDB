use crate::core::{Edge, EdgeDirection, Value, Vertex, StorageError};
use crate::storage::operations::{VertexReader, EdgeReader, VertexWriter, EdgeWriter, ScanResult};
use crate::storage::redb_types::{ByteKey, NODES_TABLE, EDGES_TABLE};
use crate::storage::serializer::{vertex_to_bytes, vertex_from_bytes, edge_to_bytes, edge_from_bytes, value_to_bytes};
use crate::utils::id_gen::generate_id;
use redb::{Database, ReadableTable};
use lru::LruCache;
use std::sync::Arc;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct RedbReader {
    db: Arc<Database>,
    vertex_cache: Arc<Mutex<LruCache<Vec<u8>, Vertex>>>,
    edge_cache: Arc<Mutex<LruCache<Vec<u8>, Edge>>>,
}

impl std::fmt::Debug for RedbReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbReader").finish()
    }
}

impl RedbReader {
    pub fn new(db: Arc<Database>) -> Result<Self, StorageError> {
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
}

impl VertexReader for RedbReader {
    fn get_vertex(&self, _space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let id_bytes = value_to_bytes(id)?;

        {
            let mut cache = self.vertex_cache.lock();
            if let Some(vertex) = cache.get(&id_bytes) {
                return Ok(Some(vertex.clone()));
            }
        }

        match self.get_node_from_bytes(&id_bytes)? {
            Some(vertex) => {
                {
                    let mut cache = self.vertex_cache.lock();
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
            let mut cache = self.edge_cache.lock();
            if let Some(edge) = cache.get(&edge_key_bytes) {
                return Ok(Some(edge.clone()));
            }
        }

        match self.get_edge_from_bytes(&edge_key_bytes)? {
            Some(edge) => {
                {
                    let mut cache = self.edge_cache.lock();
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

        Ok(ScanResult::new(edges))
    }

    fn scan_edges_by_type(
        &self,
        _space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError> {
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

        let filtered_edges: Vec<Edge> = edges.into_iter()
            .filter(|e| e.edge_type == edge_type)
            .collect();

        Ok(ScanResult::new(filtered_edges))
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
    db: Arc<Database>,
}

impl std::fmt::Debug for RedbWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbWriter").finish()
    }
}

impl RedbWriter {
    pub fn new(db: Arc<Database>) -> Result<Self, StorageError> {
        Ok(Self {
            db,
        })
    }
}

impl VertexWriter for RedbWriter {
    fn insert_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        // 如果顶点已有有效ID，使用它；否则生成新ID
        let id = match vertex.vid() {
            Value::Int(0) | Value::Null(_) => Value::Int(generate_id() as i64),
            _ => vertex.vid().clone(),
        };
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
                .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

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

            if table.get(ByteKey(id_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Err(StorageError::NodeNotFound(id.clone()));
            }

            table.remove(ByteKey(id_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

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
                let id = Value::Int(generate_id() as i64);
                let vertex_with_id = Vertex::new(id.clone(), vertex.tags);
                let vertex_bytes = vertex_to_bytes(&vertex_with_id)?;
                let id_bytes = value_to_bytes(&id)?;

                table.insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                ids.push(id);
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(ids)
    }

    fn delete_tags(
        &mut self,
        _space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let id_bytes = value_to_bytes(vertex_id)?;

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let mut deleted_count = 0;
        
        {
            let mut table = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;

            // 获取现有顶点
            let vertex = match table.get(ByteKey(id_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))? {
                Some(value) => {
                    let vertex_bytes = value.value();
                    vertex_from_bytes(&vertex_bytes.0)?
                }
                None => return Err(StorageError::NodeNotFound(vertex_id.clone())),
            };

            // 过滤掉要删除的标签
            let original_tag_count = vertex.tags.len();
            let remaining_tags: Vec<_> = vertex.tags
                .into_iter()
                .filter(|tag| !tag_names.contains(&tag.name))
                .collect();
            
            deleted_count = original_tag_count - remaining_tags.len();

            // 如果没有标签了，可以选择删除整个顶点或保留空标签列表
            // 这里选择保留空标签列表的顶点
            let updated_vertex = Vertex::new(vertex_id.clone(), remaining_tags);
            let vertex_bytes = vertex_to_bytes(&updated_vertex)?;

            table.insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(deleted_count)
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
                .insert(ByteKey(edge_key_bytes), ByteKey(edge_bytes))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

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

            if table.get(ByteKey(edge_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?
                .is_none() {
                return Err(StorageError::EdgeNotFound(Value::String(format!("{:?}_{:?}_{}", src, dst, edge_type))));
            }

            table.remove(ByteKey(edge_key_bytes.to_vec()))
                .map_err(|e| StorageError::DbError(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

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

            for edge in edges {
                let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
                let edge_key_bytes = edge_key.as_bytes().to_vec();
                let edge_bytes = edge_to_bytes(&edge)?;

                table.insert(ByteKey(edge_key_bytes), ByteKey(edge_bytes))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }
        write_txn.commit()
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }
}
