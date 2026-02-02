use super::{StorageClient, TransactionId, EdgeReader, EdgeWriter, ScanResult, VertexReader, VertexWriter, MemorySchemaManager};
use crate::core::{Edge, StorageError, Value, Vertex, EdgeDirection};
use crate::core::vertex_edge_path::Tag;
use crate::core::types::{
    SpaceInfo, TagInfo, EdgeTypeInfo,
    PropertyDef, InsertVertexInfo, InsertEdgeInfo, UpdateInfo,
    PasswordInfo,
};
pub use crate::core::types::EdgeTypeInfo as EdgeTypeSchema;
use crate::index::Index;
use crate::storage::{FieldDef, Schema};
use crate::storage::utils::{tag_info_to_schema, edge_type_info_to_schema};
use crate::common::id::IdGenerator;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
    id_generator: Arc<Mutex<IdGenerator>>,
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_type_infos: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
    tag_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    edge_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    users: Arc<Mutex<HashMap<String, String>>>,
    pub schema_manager: Arc<MemorySchemaManager>,
    storage_path: PathBuf,
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
        Self::new_with_path(PathBuf::from("data/memory"))
    }

    pub fn new_with_path(path: PathBuf) -> Result<Self, StorageError> {
        let id_generator = Arc::new(Mutex::new(IdGenerator::new()));
        let schema_manager = Arc::new(MemorySchemaManager::new());

        fs::create_dir_all(&path).map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(Self {
            vertices: Arc::new(Mutex::new(HashMap::new())),
            edges: Arc::new(Mutex::new(HashMap::new())),
            vertex_tags: Arc::new(Mutex::new(HashMap::new())),
            edge_types: Arc::new(Mutex::new(HashMap::new())),
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
            next_tx_id: Arc::new(Mutex::new(TransactionId::new(1))),
            id_generator,
            spaces: Arc::new(Mutex::new(HashMap::new())),
            tags: Arc::new(Mutex::new(HashMap::new())),
            edge_type_infos: Arc::new(Mutex::new(HashMap::new())),
            tag_indexes: Arc::new(Mutex::new(HashMap::new())),
            edge_indexes: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(HashMap::new())),
            schema_manager,
            storage_path: path,
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

    fn tag_info_to_schema(tag_name: &str, tag_info: &TagInfo) -> Schema {
        tag_info_to_schema(tag_name, tag_info)
    }

    fn edge_type_schema_to_schema(edge_type_name: &str, edge_schema: &EdgeTypeInfo) -> Schema {
        edge_type_info_to_schema(edge_type_name, edge_schema)
    }

    fn serialize_vertex(vertex: &Vertex) -> Vec<u8> {
        let mut data = Vec::new();
        if let Some(vid) = Self::value_to_bytes(&vertex.vid) {
            data.extend_from_slice(&vid);
        } else {
            data.extend_from_slice(&[0u8; 8]);
        }
        data
    }

    fn serialize_edge(edge: &Edge) -> Vec<u8> {
        let mut data = Vec::new();
        if let Some(src) = Self::value_to_bytes(&edge.src) {
            data.extend_from_slice(&src);
        } else {
            data.extend_from_slice(&[0u8; 8]);
        }
        if let Some(dst) = Self::value_to_bytes(&edge.dst) {
            data.extend_from_slice(&dst);
        } else {
            data.extend_from_slice(&[0u8; 8]);
        }
        data
    }

    fn value_to_bytes(value: &Value) -> Option<Vec<u8>> {
        match value {
            Value::String(s) => Some(s.as_bytes().to_vec()),
            Value::Int(i) => Some(i.to_le_bytes().to_vec()),
            Value::Float(f) => Some(f.to_le_bytes().to_vec()),
            Value::Bool(b) => Some(vec![*b as u8]),
            _ => None,
        }
    }
}

impl StorageClient for MemoryStorage {
    fn insert_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let id = vertex.vid.clone();
        let key = Self::serialize_vertex_key(&id);
        let tag = vertex.tags.first().map(|t| t.name.clone()).unwrap_or_default();

        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        vertices.insert(key.clone(), vertex);

        let mut vertex_tags = self.vertex_tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        vertex_tags.entry(tag).or_insert_with(Vec::new).push(key.clone());

        Ok(*id)
    }

    fn get_vertex(&self, _space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let key = Self::serialize_vertex_key(id);
        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(vertices.get(&key).cloned())
    }

    fn update_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let id = vertex.vid.clone();
        let key = Self::serialize_vertex_key(&id);

        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        vertices.insert(key, vertex);

        Ok(())
    }

    fn delete_vertex(&mut self, _space: &str, id: &Value) -> Result<(), StorageError> {
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

    fn scan_vertices(&self, _space: &str) -> Result<Vec<Vertex>, StorageError> {
        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(vertices.values().cloned().collect())
    }

    fn scan_vertices_by_tag(&self, _space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        let vertex_tags = self.vertex_tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        let keys = vertex_tags.get(tag).cloned().unwrap_or_default();
        let result: Vec<Vertex> = keys
            .iter()
            .filter_map(|key| vertices.get(key).cloned())
            .collect();

        Ok(result)
    }

    fn scan_vertices_by_prop(&self, _space: &str, tag: &str, prop: &str, value: &Value) -> Result<Vec<Vertex>, StorageError> {
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

    fn insert_edge(&mut self, _space: &str, edge: Edge) -> Result<(), StorageError> {
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

    fn get_edge(&self, _space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<Option<Edge>, StorageError> {
        let key = Self::serialize_edge_key(src, dst, edge_type);
        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(edges.get(&key).cloned())
    }

    fn get_node_edges(&self, _space: &str, node_id: &Value, direction: EdgeDirection) -> Result<Vec<Edge>, StorageError> {
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
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<Vec<Edge>, StorageError> {
        let edges = <Self as StorageClient>::get_node_edges(self, _space, node_id, direction)?;
        if let Some(filter_fn) = filter {
            Ok(edges.into_iter().filter(|e| filter_fn(e)).collect())
        } else {
            Ok(edges)
        }
    }

    fn delete_edge(&mut self, _space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
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

    fn scan_edges_by_type(&self, _space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        let edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        let keys = edge_types.get(edge_type).cloned().unwrap_or_default();
        let result: Vec<Edge> = keys
            .iter()
            .filter_map(|key| edges.get(key).cloned())
            .collect();

        Ok(result)
    }

    fn scan_all_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(edges.values().cloned().collect())
    }

    fn batch_insert_vertices(&mut self, _space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        let mut ids = Vec::new();
        for vertex in vertices {
            ids.push(<Self as StorageClient>::insert_vertex(self, _space, vertex)?);
        }
        Ok(ids)
    }

    fn batch_insert_edges(&mut self, _space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        for edge in edges {
            <Self as StorageClient>::insert_edge(self, _space, edge)?;
        }
        Ok(())
    }

    fn begin_transaction(&mut self, _space: &str) -> Result<TransactionId, StorageError> {
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

    fn commit_transaction(&mut self, _space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
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

    fn rollback_transaction(&mut self, _space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut active_transactions = self.active_transactions.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        active_transactions.remove(&tx_id);
        Ok(())
    }

    // ========== 空间管理 ==========
    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if spaces.contains_key(&space.space_name) {
            return Ok(false);
        }
        spaces.insert(space.space_name.clone(), space.clone());
        
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.insert(space.space_name.clone(), HashMap::new());
        
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_type_infos.insert(space.space_name.clone(), HashMap::new());
        
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tag_indexes.insert(space.space_name.clone(), HashMap::new());
        
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_indexes.insert(space.space_name.clone(), HashMap::new());
        
        Ok(true)
    }

    fn drop_space(&mut self, space_name: &str) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if !spaces.contains_key(space_name) {
            return Ok(false);
        }
        spaces.remove(space_name);
        
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.remove(space_name);
        
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_type_infos.remove(space_name);
        
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tag_indexes.remove(space_name);
        
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_indexes.remove(space_name);
        
        Ok(true)
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(spaces.get(space_name).cloned())
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(spaces.values().cloned().collect())
    }

    // ========== 标签管理 ==========
    fn create_tag(&mut self, _space: &str, info: &TagInfo) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(_space) {
            if space_tags.contains_key(&info.tag_name) {
                return Ok(false);
            }
            space_tags.insert(info.tag_name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", _space)))
        }
    }

    fn alter_tag(&mut self, space_name: &str, tag_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space_name) {
            if let Some(tag_info) = space_tags.get_mut(tag_name) {
                for prop in additions {
                    tag_info.properties.retain(|p| p.name != prop.name);
                    tag_info.properties.push(prop);
                }
                for prop_name in deletions {
                    tag_info.properties.retain(|p| p.name != prop_name);
                }
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn get_tag(&self, space_name: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space_name) {
            Ok(space_tags.get(tag_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn drop_tag(&mut self, space_name: &str, tag_name: &str) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space_name) {
            Ok(space_tags.remove(tag_name).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_tags(&self, space_name: &str) -> Result<Vec<TagInfo>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space_name) {
            Ok(space_tags.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    // ========== 边类型管理 ==========
    fn create_edge_type(&mut self, _space: &str, info: &EdgeTypeSchema) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(_space) {
            if space_edge_types.contains_key(&info.edge_type_name) {
                return Ok(false);
            }
            space_edge_types.insert(info.edge_type_name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", _space)))
        }
    }

    fn alter_edge_type(&mut self, space_name: &str, edge_type_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(space_name) {
            if let Some(edge_type_info) = space_edge_types.get_mut(edge_type_name) {
                for prop in additions {
                    edge_type_info.properties.retain(|p| p.name != prop.name);
                    edge_type_info.properties.push(prop);
                }
                for prop_name in deletions {
                    edge_type_info.properties.retain(|p| p.name != prop_name);
                }
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn get_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<Option<EdgeTypeSchema>, StorageError> {
        let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get(space_name) {
            Ok(space_edge_types.get(edge_type_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn drop_edge_type(&mut self, space_name: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(space_name) {
            Ok(space_edge_types.remove(edge_type_name).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_edge_types(&self, space_name: &str) -> Result<Vec<EdgeTypeSchema>, StorageError> {
        let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get(space_name) {
            Ok(space_edge_types.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    // ========== 索引管理 ==========
    fn create_tag_index(&mut self, _space: &str, info: &Index) -> Result<bool, StorageError> {
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get_mut(_space) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", _space)))
        }
    }

    fn drop_tag_index(&mut self, space_name: &str, index_name: &str) -> Result<bool, StorageError> {
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get_mut(space_name) {
            Ok(space_indexes.remove(index_name).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn get_tag_index(&self, space_name: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get(space_name) {
            Ok(space_indexes.get(index_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_tag_indexes(&self, space_name: &str) -> Result<Vec<Index>, StorageError> {
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get(space_name) {
            Ok(space_indexes.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn rebuild_tag_index(&mut self, _space_name: &str, _index_name: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn create_edge_index(&mut self, _space: &str, info: &Index) -> Result<bool, StorageError> {
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get_mut(_space) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", _space)))
        }
    }

    fn drop_edge_index(&mut self, space_name: &str, index_name: &str) -> Result<bool, StorageError> {
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get_mut(space_name) {
            Ok(space_indexes.remove(index_name).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn get_edge_index(&self, space_name: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get(space_name) {
            Ok(space_indexes.get(index_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_edge_indexes(&self, space_name: &str) -> Result<Vec<Index>, StorageError> {
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get(space_name) {
            Ok(space_indexes.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn rebuild_edge_index(&mut self, _space_name: &str, _index_name: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    // ========== 数据变更 ==========
    fn insert_vertex_data(&mut self, _space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError> {
        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let vertex_key = Self::serialize_value(&info.vertex_id);
        
        if vertices.contains_key(&vertex_key) {
            return Ok(false);
        }
        
        let vertex = Vertex::new_with_properties(
            info.vertex_id.clone(),
            vec![Tag::new(info.tag_name.clone(), HashMap::new())],
            info.properties.iter().cloned().collect(),
        );
        
        vertices.insert(vertex_key, vertex);
        Ok(true)
    }

    fn insert_edge_data(&mut self, _space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        let mut edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let edge_key = (
            Self::serialize_value(&info.src_vertex_id),
            Self::serialize_value(&info.dst_vertex_id),
            info.edge_name.clone(),
        );
        
        if edges.contains_key(&edge_key) {
            return Ok(false);
        }
        
        let edge = Edge::new(
            info.src_vertex_id.clone(),
            info.dst_vertex_id.clone(),
            info.edge_name.clone(),
            info.rank,
            info.properties.iter().cloned().collect(),
        );
        
        edges.insert(edge_key, edge);
        Ok(true)
    }

    fn delete_vertex_data(&mut self, _space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let vertex_key = Self::serialize_value(&Value::String(vertex_id.to_string()));
        Ok(vertices.remove(&vertex_key).is_some())
    }

    fn delete_edge_data(&mut self, _space_name: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError> {
        let mut edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let edge_key = (
            Self::serialize_value(&Value::String(src.to_string())),
            Self::serialize_value(&Value::String(dst.to_string())),
            rank.to_string(),
        );
        
        Ok(edges.remove(&edge_key).is_some())
    }

    fn update_data(&mut self, _space: &str, _info: &UpdateInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    // ========== 用户管理 ==========
    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(hashed_password) = users.get(&info.username) {
            if *hashed_password == info.old_password {
                users.insert(info.username.clone(), info.new_password.clone());
                return Ok(true);
            }
            return Ok(false);
        }
        Ok(false)
    }

    // ========== 二进制数据接口实现 ==========
    fn get_vertex_with_schema(&self, space_name: &str, tag_name: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tag_info = match tags.get(space_name)
            .and_then(|space_tags| space_tags.get(tag_name))
            .cloned() {
            Some(info) => info,
            None => return Ok(None),
        };

        let vertex = <Self as StorageClient>::get_vertex(self, space_name, id)?;
        vertex.as_ref().map(|v| {
            let schema = Self::tag_info_to_schema(&tag_name, &tag_info);
            let binary_data = Self::serialize_vertex(v);
            Ok((schema, binary_data))
        }).transpose()
    }

    fn get_edge_with_schema(&self, space_name: &str, edge_type_name: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let edge_types = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_schema = match edge_types.get(space_name)
            .and_then(|space_edges| space_edges.get(edge_type_name))
            .cloned() {
            Some(info) => info,
            None => return Ok(None),
        };

        let edge = <Self as StorageClient>::get_edge(self, space_name, src, dst, edge_type_name)?;
        edge.as_ref().map(|e| {
            let schema = Self::edge_type_schema_to_schema(&edge_type_name, &edge_schema);
            let binary_data = Self::serialize_edge(e);
            Ok((schema, binary_data))
        }).transpose()
    }

    fn scan_vertices_with_schema(&self, space_name: &str, tag_name: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let vertices = <Self as StorageClient>::scan_vertices_by_tag(self, space_name, tag_name)?;
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tag_info = tags.get(space_name)
            .and_then(|space_tags| space_tags.get(tag_name))
            .cloned()
            .unwrap_or_else(|| TagInfo::new(tag_name.to_string()));

        let schema = Self::tag_info_to_schema(&tag_name, &tag_info);
        let binary_data_list: Vec<Vec<u8>> = vertices.iter().map(|v| Self::serialize_vertex(v)).collect();

        Ok(binary_data_list.into_iter().map(|data| (schema.clone(), data)).collect())
    }

    fn scan_edges_with_schema(&self, space_name: &str, edge_type_name: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let edges = <Self as StorageClient>::scan_edges_by_type(self, space_name, edge_type_name)?;
        let edge_types = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_schema = edge_types.get(space_name)
            .and_then(|space_edges| space_edges.get(edge_type_name))
            .cloned()
            .unwrap_or_else(|| EdgeTypeInfo::new(edge_type_name.to_string()));

        let schema = Self::edge_type_schema_to_schema(&edge_type_name, &edge_schema);
        let binary_data_list: Vec<Vec<u8>> = edges.iter().map(|e| Self::serialize_edge(e)).collect();

        Ok(binary_data_list.into_iter().map(|data| (schema.clone(), data)).collect())
    }

    fn lookup_index(&self, _space: &str, _index: &str, _value: &Value) -> Result<Vec<Value>, StorageError> {
        Ok(Vec::new())
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        let path = &self.storage_path;
        if !path.exists() {
            return Ok(());
        }

        let vertices_file = path.join("vertices.json");
        if vertices_file.exists() {
            let content = fs::read_to_string(&vertices_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let vertices_map: HashMap<VertexKey, Vertex> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            *self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))? = vertices_map;
        }

        let edges_file = path.join("edges.json");
        if edges_file.exists() {
            let content = fs::read_to_string(&edges_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let edges_map: HashMap<EdgeKey, Edge> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            *self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))? = edges_map;
        }

        let vertex_tags_file = path.join("vertex_tags.json");
        if vertex_tags_file.exists() {
            let content = fs::read_to_string(&vertex_tags_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let tags_map: HashMap<String, Vec<VertexKey>> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            *self.vertex_tags.lock().map_err(|e| StorageError::DbError(e.to_string()))? = tags_map;
        }

        let edge_types_file = path.join("edge_types.json");
        if edge_types_file.exists() {
            let content = fs::read_to_string(&edge_types_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let types_map: HashMap<String, Vec<EdgeKey>> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            *self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))? = types_map;
        }

        let spaces_file = path.join("spaces.json");
        if spaces_file.exists() {
            let content = fs::read_to_string(&spaces_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let spaces_map: HashMap<String, SpaceInfo> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            *self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))? = spaces_map;
        }

        let tags_file = path.join("tags.json");
        if tags_file.exists() {
            let content = fs::read_to_string(&tags_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let tags_map: HashMap<String, HashMap<String, TagInfo>> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            *self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))? = tags_map;
        }

        let edge_type_infos_file = path.join("edge_type_infos.json");
        if edge_type_infos_file.exists() {
            let content = fs::read_to_string(&edge_type_infos_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let infos_map: HashMap<String, HashMap<String, EdgeTypeSchema>> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializeError(e.to_string()))?;
            *self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))? = infos_map;
        }

        let tag_indexes_file = path.join("tag_indexes.bin");
        if tag_indexes_file.exists() {
            let content = fs::read(&tag_indexes_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let indexes_map: HashMap<String, HashMap<String, Index>> = bincode::decode_from_slice(&content, bincode::config::standard())
                .map_err(|e| StorageError::SerializeError(e.to_string()))?.0;
            *self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))? = indexes_map;
        }

        let edge_indexes_file = path.join("edge_indexes.bin");
        if edge_indexes_file.exists() {
            let content = fs::read(&edge_indexes_file)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            let indexes_map: HashMap<String, HashMap<String, Index>> = bincode::decode_from_slice(&content, bincode::config::standard())
                .map_err(|e| StorageError::SerializeError(e.to_string()))?.0;
            *self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))? = indexes_map;
        }

        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        let path = &self.storage_path;
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| StorageError::DbError(e.to_string()))?;
        }

        let vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let vertices_content = serde_json::to_string_pretty(&*vertices)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("vertices.json"), vertices_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edges_content = serde_json::to_string_pretty(&*edges)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("edges.json"), edges_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let vertex_tags = self.vertex_tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tags_content = serde_json::to_string_pretty(&*vertex_tags)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("vertex_tags.json"), tags_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let types_content = serde_json::to_string_pretty(&*edge_types)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("edge_types.json"), types_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let spaces_content = serde_json::to_string_pretty(&*spaces)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("spaces.json"), spaces_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tags_content = serde_json::to_string_pretty(&*tags)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("tags.json"), tags_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let infos_content = serde_json::to_string_pretty(&*edge_type_infos)
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("edge_type_infos.json"), infos_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let indexes_content = bincode::encode_to_vec(&*tag_indexes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("tag_indexes.bin"), indexes_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let indexes_content = bincode::encode_to_vec(&*edge_indexes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
        fs::write(path.join("edge_indexes.bin"), indexes_content)
            .map_err(|e| StorageError::DbError(e.to_string()))?;

        Ok(())
    }
}

impl VertexReader for MemoryStorage {
    fn get_vertex(&self, _space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        <Self as StorageClient>::get_vertex(self, _space, id)
    }

    fn scan_vertices(&self, _space: &str) -> Result<ScanResult<Vertex>, StorageError> {
        let vertices = <Self as StorageClient>::scan_vertices(self, _space)?;
        Ok(ScanResult::new(vertices))
    }

    fn scan_vertices_by_tag(&self, _space: &str, tag_name: &str) -> Result<ScanResult<Vertex>, StorageError> {
        let vertices = <Self as StorageClient>::scan_vertices_by_tag(self, _space, tag_name)?;
        Ok(ScanResult::new(vertices))
    }

    fn scan_vertices_by_prop(
        &self,
        _space: &str,
        tag_name: &str,
        prop_name: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        let vertices = <Self as StorageClient>::scan_vertices_by_prop(self, _space, tag_name, prop_name, value)?;
        Ok(ScanResult::new(vertices))
    }
}

impl EdgeReader for MemoryStorage {
    fn get_edge(
        &self,
        _space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        <Self as StorageClient>::get_edge(self, _space, src, dst, edge_type)
    }

    fn get_node_edges(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let edges = <Self as StorageClient>::get_node_edges(self, _space, node_id, direction)?;
        Ok(ScanResult::new(edges))
    }

    fn get_node_edges_filtered(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let edges = <Self as StorageClient>::get_node_edges_filtered(self, _space, node_id, direction, filter)?;
        Ok(ScanResult::new(edges))
    }

    fn scan_edges_by_type(
        &self,
        _space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let edges = <Self as StorageClient>::scan_edges_by_type(self, _space, edge_type)?;
        Ok(ScanResult::new(edges))
    }

    fn scan_all_edges(&self, _space: &str) -> Result<ScanResult<Edge>, StorageError> {
        let edges = <Self as StorageClient>::scan_all_edges(self, _space)?;
        Ok(ScanResult::new(edges))
    }
}

impl VertexWriter for MemoryStorage {
    fn insert_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        <Self as StorageClient>::insert_vertex(self, _space, vertex)
    }

    fn update_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<(), StorageError> {
        <Self as StorageClient>::update_vertex(self, _space, vertex)
    }

    fn delete_vertex(&mut self, _space: &str, id: &Value) -> Result<(), StorageError> {
        <Self as StorageClient>::delete_vertex(self, _space, id)
    }

    fn batch_insert_vertices(&mut self, _space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        <Self as StorageClient>::batch_insert_vertices(self, _space, vertices)
    }
}

impl EdgeWriter for MemoryStorage {
    fn insert_edge(&mut self, _space: &str, edge: Edge) -> Result<(), StorageError> {
        <Self as StorageClient>::insert_edge(self, _space, edge)
    }

    fn delete_edge(
        &mut self,
        _space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        <Self as StorageClient>::delete_edge(self, _space, src, dst, edge_type)
    }

    fn batch_insert_edges(&mut self, _space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        <Self as StorageClient>::batch_insert_edges(self, _space, edges)
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

        let id = <MemoryStorage as crate::storage::operations::writer::VertexWriter>::insert_vertex(&mut storage, "", vertex.clone()).expect("insert_node should succeed");
        assert_eq!(id, Value::String("user1".to_string()));

        let retrieved = <MemoryStorage as crate::storage::operations::reader::VertexReader>::get_vertex(&storage, "", &id).expect("get_node should succeed");
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

        <MemoryStorage as crate::storage::operations::writer::EdgeWriter>::insert_edge(&mut storage, "", edge.clone()).expect("insert_edge should succeed");

        let retrieved = <MemoryStorage as crate::storage::operations::reader::EdgeReader>::get_edge(&storage, "", &edge.src, &edge.dst, &edge.edge_type).expect("get_edge should succeed");
        assert_eq!(retrieved, Some(edge));
    }

    #[test]
    fn test_scan_vertices_by_tag() {
        let mut storage = MemoryStorage::new().expect("MemoryStorage::new should succeed");

        <MemoryStorage as crate::storage::operations::writer::VertexWriter>::insert_vertex(&mut storage, "", Vertex::new_with_properties(
            Value::String("user1".to_string()),
            vec![Tag::new("user".to_string(), HashMap::new())],
            HashMap::new(),
        )).expect("insert_node should succeed");

        <MemoryStorage as crate::storage::operations::writer::VertexWriter>::insert_vertex(&mut storage, Vertex::new_with_properties(
            Value::String("post1".to_string()),
            vec![Tag::new("post".to_string(), HashMap::new())],
            HashMap::new(),
        )).expect("insert_node should succeed");

        let users = <MemoryStorage as crate::storage::operations::reader::VertexReader>::scan_vertices_by_tag(&storage, "", "user").expect("scan_vertices_by_tag should succeed");
        let users_vec = users.into_vec();
        assert_eq!(users_vec.len(), 1);
        assert_eq!(*users_vec[0].vid, Value::String("user1".to_string()));
    }

    #[test]
    fn test_transaction() {
        let mut storage = MemoryStorage::new().expect("MemoryStorage::new should succeed");

        let tx_id = <MemoryStorage as crate::storage::StorageClient>::begin_transaction(&mut storage, "").expect("begin_transaction should succeed");

        let vertex = Vertex::new_with_properties(
            Value::String("user1".to_string()),
            vec![Tag::new("user".to_string(), HashMap::new())],
            HashMap::new(),
        );

        <MemoryStorage as crate::storage::operations::writer::VertexWriter>::insert_vertex(&mut storage, vertex).expect("insert_node should succeed");

        <MemoryStorage as crate::storage::StorageClient>::commit_transaction(&mut storage, "", tx_id).expect("commit_transaction should succeed");

        let retrieved = <MemoryStorage as crate::storage::operations::reader::VertexReader>::get_vertex(&storage, "", &Value::String("user1".to_string())).expect("get_node should succeed");
        assert!(retrieved.is_some());
    }
}
