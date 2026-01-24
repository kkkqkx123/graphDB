use super::{StorageEngine, TransactionId};
use crate::core::{Edge, StorageError, Value, Vertex, EdgeDirection};
use crate::core::vertex_edge_path::Tag;
use crate::common::memory::MemoryPool;
use crate::common::id::IdGenerator;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type VertexKey = Vec<u8>;
type EdgeKey = (Vec<u8>, Vec<u8>, String);

#[derive(Clone)]
pub struct MemoryStorage {
    vertices: Arc<Mutex<HashMap<VertexKey, Vertex>>>,
    edges: Arc<Mutex<HashMap<EdgeKey, Edge>>>,
    vertex_tags: Arc<Mutex<HashMap<String, Vec<VertexKey>>>>,
    edge_types: Arc<Mutex<HashMap<String, Vec<EdgeKey>>>>,
    active_transactions: Arc<Mutex<HashMap<TransactionId, TransactionState>>>,
    next_tx_id: Arc<Mutex<TransactionId>>,
    _vertex_props: Arc<Mutex<HashMap<(String, String, Vec<u8>), Vec<VertexKey>>>>,
    _memory_pool: Arc<MemoryPool>,
    id_generator: Arc<Mutex<IdGenerator>>,
}

impl std::fmt::Debug for MemoryStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryStorage")
            .field("vertices", &self.vertices)
            .field("edges", &self.edges)
            .field("vertex_tags", &self.vertex_tags)
            .field("edge_types", &self.edge_types)
            .field("active_transactions", &self.active_transactions)
            .field("next_tx_id", &self.next_tx_id)
            .field("id_generator", &self.id_generator)
            .finish()
    }
}

#[derive(Debug)]
struct TransactionState {
    vertices_to_insert: HashMap<VertexKey, Vertex>,
    vertices_to_update: HashMap<VertexKey, Vertex>,
    vertices_to_delete: Vec<VertexKey>,
    edges_to_insert: HashMap<EdgeKey, Edge>,
    edges_to_delete: Vec<EdgeKey>,
}

impl MemoryStorage {
    pub fn new() -> Result<Self, StorageError> {
        let memory_pool = Arc::new(MemoryPool::new(100 * 1024 * 1024).map_err(|e| StorageError::DbError(e))?); // 100MB
        let id_generator = Arc::new(Mutex::new(IdGenerator::new()));

        Ok(Self {
            vertices: Arc::new(Mutex::new(HashMap::new())),
            edges: Arc::new(Mutex::new(HashMap::new())),
            vertex_tags: Arc::new(Mutex::new(HashMap::new())),
            edge_types: Arc::new(Mutex::new(HashMap::new())),
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
            next_tx_id: Arc::new(Mutex::new(1)),
            _vertex_props: Arc::new(Mutex::new(HashMap::new())),
            _memory_pool: memory_pool,
            id_generator,
        })
    }

    fn serialize_value(value: &Value) -> Vec<u8> {
        match value {
            Value::String(s) => s.as_bytes().to_vec(),
            Value::Int(i) => i.to_be_bytes().to_vec(),
            Value::Float(f) => f.to_be_bytes().to_vec(),
            Value::Bool(b) => vec![*b as u8],
            Value::Null(_) => vec![0],
            Value::List(arr) => arr.iter().flat_map(|v| Self::serialize_value(v)).collect(),
            Value::Map(map) => map.iter().flat_map(|(k, v)| {
                [k.as_bytes().to_vec(), Self::serialize_value(v)].concat()
            }).collect(),
            _ => vec![],
        }
    }

    fn serialize_vertex_key(id: &Value) -> VertexKey {
        Self::serialize_value(id)
    }

    fn serialize_edge_key(src: &Value, dst: &Value, edge_type: &str) -> EdgeKey {
        (
            Self::serialize_value(src),
            Self::serialize_value(dst),
            edge_type.to_string(),
        )
    }
}

impl StorageEngine for MemoryStorage {
    fn insert_node(&mut self, vertex: Vertex) -> Result<Value, StorageError> {
        let id = vertex.vid.clone();
        let key = Self::serialize_vertex_key(&id);
        let tag = vertex.tags.first().map(|t| t.name.clone()).unwrap_or_default();

        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        vertices.insert(key.clone(), vertex);

        let mut vertex_tags = self.vertex_tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        vertex_tags.entry(tag).or_insert_with(Vec::new).push(key.clone());

        Ok(*id)
    }

    fn get_node(&self, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let key = Self::serialize_vertex_key(id);
        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(vertices.get(&key).cloned())
    }

    fn update_node(&mut self, vertex: Vertex) -> Result<(), StorageError> {
        let id = vertex.vid.clone();
        let key = Self::serialize_vertex_key(&id);

        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        vertices.insert(key, vertex);

        Ok(())
    }

    fn delete_node(&mut self, id: &Value) -> Result<(), StorageError> {
        let key = Self::serialize_vertex_key(id);

        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(vertex) = vertices.remove(&key) {
            let mut vertex_tags = self.vertex_tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let tag = vertex.tags.first().map(|t| t.name.clone()).unwrap_or_default();
            if let Some(tag_vertices) = vertex_tags.get_mut(&tag) {
                tag_vertices.retain(|k| k != &key);
            }
        }

        Ok(())
    }

    fn scan_all_vertices(&self) -> Result<Vec<Vertex>, StorageError> {
        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(vertices.values().cloned().collect())
    }

    fn scan_vertices_by_tag(&self, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        let vertex_tags = self.vertex_tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        let keys = vertex_tags.get(tag).cloned().unwrap_or_default();
        let result: Vec<Vertex> = keys
            .iter()
            .filter_map(|key| vertices.get(key).cloned())
            .collect();

        Ok(result)
    }

    fn scan_vertices_by_prop(&self, tag: &str, prop: &str, value: &Value) -> Result<Vec<Vertex>, StorageError> {
        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let result: Vec<Vertex> = vertices
            .values()
            .filter(|v| {
                v.tags.iter().any(|t| t.name == tag)
                    && v.properties.get(prop).map_or(false, |p| p == value)
            })
            .cloned()
            .collect();

        Ok(result)
    }

    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError> {
        let edge_type = edge.edge_type.clone();
        let _edge_id = {
            let generator = self.id_generator.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            generator.generate_edge_id()
        };

        let key = Self::serialize_edge_key(&edge.src, &edge.dst, &edge_type);

        let mut edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edges.insert(key.clone(), edge);

        let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_types.entry(edge_type).or_insert_with(Vec::new).push(key);

        Ok(())
    }

    fn get_edge(&self, src: &Value, dst: &Value, edge_type: &str) -> Result<Option<Edge>, StorageError> {
        let key = Self::serialize_edge_key(src, dst, edge_type);
        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(edges.get(&key).cloned())
    }

    fn get_node_edges(&self, node_id: &Value, direction: EdgeDirection) -> Result<Vec<Edge>, StorageError> {
        let node_key = Self::serialize_vertex_key(node_id);
        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        let result: Vec<Edge> = edges
            .values()
            .filter(|e| match direction {
                EdgeDirection::Out => Self::serialize_vertex_key(&e.src) == node_key,
                EdgeDirection::In => Self::serialize_vertex_key(&e.dst) == node_key,
                EdgeDirection::Both => {
                    Self::serialize_vertex_key(&e.src) == node_key
                        || Self::serialize_vertex_key(&e.dst) == node_key
                }
            })
            .cloned()
            .collect();

        Ok(result)
    }

    fn get_node_edges_filtered(
        &self,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, StorageError> {
        let edges = self.get_node_edges(node_id, direction)?;
        if let Some(filter_fn) = filter {
            Ok(edges.into_iter().filter(|e| filter_fn(e)).collect())
        } else {
            Ok(edges)
        }
    }

    fn delete_edge(&mut self, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        let key = Self::serialize_edge_key(src, dst, edge_type);

        let mut edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(edge) = edges.remove(&key) {
            let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            if let Some(type_edges) = edge_types.get_mut(&edge.edge_type) {
                type_edges.retain(|k| k != &key);
            }
        }

        Ok(())
    }

    fn scan_edges_by_type(&self, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        let edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        let keys = edge_types.get(edge_type).cloned().unwrap_or_default();
        let result: Vec<Edge> = keys
            .iter()
            .filter_map(|key| edges.get(key).cloned())
            .collect();

        Ok(result)
    }

    fn scan_all_edges(&self) -> Result<Vec<Edge>, StorageError> {
        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(edges.values().cloned().collect())
    }

    fn batch_insert_nodes(&mut self, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        let mut ids = Vec::new();
        for vertex in vertices {
            ids.push(self.insert_node(vertex)?);
        }
        Ok(ids)
    }

    fn batch_insert_edges(&mut self, edges: Vec<Edge>) -> Result<(), StorageError> {
        for edge in edges {
            self.insert_edge(edge)?;
        }
        Ok(())
    }

    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError> {
        let mut next_tx_id = self.next_tx_id.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tx_id = *next_tx_id;
        *next_tx_id += 1;

        let mut active_transactions = self.active_transactions.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        active_transactions.insert(tx_id, TransactionState {
            vertices_to_insert: HashMap::new(),
            vertices_to_update: HashMap::new(),
            vertices_to_delete: Vec::new(),
            edges_to_insert: HashMap::new(),
            edges_to_delete: Vec::new(),
        });

        Ok(tx_id)
    }

    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(tx_state) = active_transactions.remove(&tx_id) {
            let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
            let mut edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

            for (key, vertex) in tx_state.vertices_to_insert {
                vertices.insert(key, vertex);
            }

            for (key, vertex) in tx_state.vertices_to_update {
                vertices.insert(key, vertex);
            }

            for key in tx_state.vertices_to_delete {
                vertices.remove(&key);
            }

            for (key, edge) in tx_state.edges_to_insert {
                edges.insert(key, edge);
            }

            for key in tx_state.edges_to_delete {
                edges.remove(&key);
            }
        }

        Ok(())
    }

    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        active_transactions.remove(&tx_id);
        Ok(())
    }

    fn get_input(&self, _input_var: &str) -> Result<Option<Vec<Value>>, StorageError> {
        Ok(None)
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create MemoryStorage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_creation() {
        let storage = MemoryStorage::new();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_insert_and_get_node() {
        let mut storage = MemoryStorage::new().expect("MemoryStorage::new should succeed");
        let vertex = Vertex::new_with_properties(
            Value::String("user1".to_string()),
            vec![Tag::new("user".to_string(), HashMap::new())],
            HashMap::new(),
        );

        let id = storage.insert_node(vertex.clone()).expect("insert_node should succeed");
        assert_eq!(id, Value::String("user1".to_string()));

        let retrieved = storage.get_node(&id).expect("get_node should succeed");
        assert_eq!(retrieved, Some(vertex));
    }

    #[test]
    fn test_insert_and_get_edge() {
        let mut storage = MemoryStorage::new().expect("MemoryStorage::new should succeed");
        let edge = Edge::new(
            Value::String("user1".to_string()),
            Value::String("user2".to_string()),
            "follows".to_string(),
            0,
            HashMap::new(),
        );

        storage.insert_edge(edge.clone()).expect("insert_edge should succeed");

        let retrieved = storage.get_edge(&edge.src, &edge.dst, &edge.edge_type).expect("get_edge should succeed");
        assert_eq!(retrieved, Some(edge));
    }

    #[test]
    fn test_scan_vertices_by_tag() {
        let mut storage = MemoryStorage::new().expect("MemoryStorage::new should succeed");

        storage.insert_node(Vertex::new_with_properties(
            Value::String("user1".to_string()),
            vec![Tag::new("user".to_string(), HashMap::new())],
            HashMap::new(),
        )).expect("insert_node should succeed");

        storage.insert_node(Vertex::new_with_properties(
            Value::String("post1".to_string()),
            vec![Tag::new("post".to_string(), HashMap::new())],
            HashMap::new(),
        )).expect("insert_node should succeed");

        let users = storage.scan_vertices_by_tag("user").expect("scan_vertices_by_tag should succeed");
        assert_eq!(users.len(), 1);
        assert_eq!(*users[0].vid, Value::String("user1".to_string()));
    }

    #[test]
    fn test_transaction() {
        let mut storage = MemoryStorage::new().expect("MemoryStorage::new should succeed");

        let tx_id = storage.begin_transaction().expect("begin_transaction should succeed");

        let vertex = Vertex::new_with_properties(
            Value::String("user1".to_string()),
            vec![Tag::new("user".to_string(), HashMap::new())],
            HashMap::new(),
        );

        storage.insert_node(vertex).expect("insert_node should succeed");

        storage.commit_transaction(tx_id).expect("commit_transaction should succeed");

        let retrieved = storage.get_node(&Value::String("user1".to_string())).expect("get_node should succeed");
        assert!(retrieved.is_some());
    }
}
