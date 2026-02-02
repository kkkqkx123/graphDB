use super::{StorageClient, TransactionId, VertexReader, VertexWriter, EdgeReader, EdgeWriter, ScanResult, MemorySchemaManager};
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
use crate::storage::engine::{Engine, RedbEngine};
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RedbStorage<E: Engine> {
    engine: Arc<Mutex<E>>,
    id_generator: Arc<Mutex<IdGenerator>>,
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_type_infos: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
    tag_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    edge_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    users: Arc<Mutex<HashMap<String, String>>>,
    pub schema_manager: Arc<MemorySchemaManager>,
    db_path: PathBuf,
}

impl<E: Engine> std::fmt::Debug for RedbStorage<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbStorage")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl RedbStorage<RedbEngine> {
    pub fn new() -> Result<Self, StorageError> {
        Self::new_with_path(PathBuf::from("data/redb"))
    }

    pub fn new_with_path(path: PathBuf) -> Result<Self, StorageError> {
        let id_generator = Arc::new(Mutex::new(IdGenerator::new()));
        let schema_manager = Arc::new(MemorySchemaManager::new());

        let engine = Arc::new(Mutex::new(RedbEngine::new(&path)?));

        Ok(Self {
            engine,
            id_generator,
            spaces: Arc::new(Mutex::new(HashMap::new())),
            tags: Arc::new(Mutex::new(HashMap::new())),
            edge_type_infos: Arc::new(Mutex::new(HashMap::new())),
            tag_indexes: Arc::new(Mutex::new(HashMap::new())),
            edge_indexes: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(HashMap::new())),
            schema_manager,
            db_path: path,
        })
    }
}

impl<E: Engine> RedbStorage<E> {
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

    fn encode_vertex_key(space: &str, id: &Value) -> Vec<u8> {
        format!("{}:v:", space).into_bytes().into_iter()
            .chain(Self::serialize_value(id))
            .collect()
    }

    fn encode_edge_key(space: &str, src: &Value, dst: &Value, edge_type: &str) -> Vec<u8> {
        format!("{}:e:{}:", space, edge_type).into_bytes().into_iter()
            .chain(Self::serialize_value(src))
            .chain(vec![b':'])
            .chain(Self::serialize_value(dst))
            .collect()
    }

    fn serialize_vertex(vertex: &Vertex) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(vertex).map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn deserialize_vertex(data: &[u8]) -> Result<Vertex, StorageError> {
        serde_json::from_slice(data).map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn serialize_edge(edge: &Edge) -> Result<Vec<u8>, StorageError> {
        serde_json::to_vec(edge).map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn deserialize_edge(data: &[u8]) -> Result<Edge, StorageError> {
        serde_json::from_slice(data).map_err(|e| StorageError::DbError(e.to_string()))
    }
}

impl<E: Engine> VertexReader for RedbStorage<E> {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let key = Self::encode_vertex_key(space, id);
        let engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(data) = engine.get(&key)? {
            Self::deserialize_vertex(&data).map(Some)
        } else {
            Ok(None)
        }
    }

    fn scan_vertices(&self, space: &str) -> Result<ScanResult<Vertex>, StorageError> {
        let prefix = format!("{}:v:", space).into_bytes();
        let engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let iter = engine.scan(&prefix)?;

        let mut vertices = Vec::new();
        let mut iter = iter;
        while iter.next() {
            if let (Some(_key), Some(value)) = (iter.key(), iter.value()) {
                if let Ok(vertex) = Self::deserialize_vertex(value) {
                    vertices.push(vertex);
                }
            }
        }

        Ok(ScanResult::new(vertices))
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<ScanResult<Vertex>, StorageError> {
        let all_vertices = VertexReader::scan_vertices(self, space)?;
        let filtered_vertices = all_vertices
            .into_vec()
            .into_iter()
            .filter(|v| v.tags.iter().any(|t| t.name == tag))
            .collect();

        Ok(ScanResult::new(filtered_vertices))
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        let all_vertices = VertexReader::scan_vertices(self, space)?;
        let filtered_vertices = all_vertices
            .into_vec()
            .into_iter()
            .filter(|v| {
                v.tags.iter().any(|t| t.name == tag)
                    && v.properties.get(prop).map_or(false, |p| p == value)
            })
            .collect();

        Ok(ScanResult::new(filtered_vertices))
    }
}

impl<E: Engine> EdgeReader for RedbStorage<E> {
    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        let key = Self::encode_edge_key(space, src, dst, edge_type);
        let engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(data) = engine.get(&key)? {
            Self::deserialize_edge(&data).map(Some)
        } else {
            Ok(None)
        }
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError> {
        EdgeReader::get_node_edges_filtered(self, space, node_id, direction, None)
    }

    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        _filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync>>,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let prefix = format!("{}:e:", space).into_bytes();
        let engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let iter = engine.scan(&prefix)?;

        let src_encoded = Self::serialize_value(node_id);
        let mut edges = Vec::new();
        let mut iter = iter;
        while iter.next() {
            if let (Some(_key), Some(value)) = (iter.key(), iter.value()) {
                if let Ok(edge) = Self::deserialize_edge(value) {
                    let matches = match direction {
                        EdgeDirection::Out => Self::serialize_value(&edge.src) == src_encoded,
                        EdgeDirection::In => Self::serialize_value(&edge.dst) == src_encoded,
                        EdgeDirection::Both => {
                            Self::serialize_value(&edge.src) == src_encoded
                                || Self::serialize_value(&edge.dst) == src_encoded
                        }
                    };
                    if matches {
                        edges.push(edge);
                    }
                }
            }
        }

        Ok(ScanResult::new(edges))
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<ScanResult<Edge>, StorageError> {
        let prefix = format!("{}:e:{}:", space, edge_type).into_bytes();
        let engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let iter = engine.scan(&prefix)?;

        let mut edges = Vec::new();
        let mut iter = iter;
        while iter.next() {
            if let (Some(_key), Some(value)) = (iter.key(), iter.value()) {
                if let Ok(edge) = Self::deserialize_edge(value) {
                    edges.push(edge);
                }
            }
        }

        Ok(ScanResult::new(edges))
    }

    fn scan_all_edges(&self, space: &str) -> Result<ScanResult<Edge>, StorageError> {
        let prefix = format!("{}:e:", space).into_bytes();
        let engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let iter = engine.scan(&prefix)?;

        let mut edges = Vec::new();
        let mut iter = iter;
        while iter.next() {
            if let (Some(_key), Some(value)) = (iter.key(), iter.value()) {
                if let Ok(edge) = Self::deserialize_edge(value) {
                    edges.push(edge);
                }
            }
        }

        Ok(ScanResult::new(edges))
    }
}

impl<E: Engine> VertexWriter for RedbStorage<E> {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let id = vertex.vid.clone();
        let key = Self::encode_vertex_key(space, &id);
        let data = Self::serialize_vertex(&vertex)?;

        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.put(&key, &data)?;

        Ok(*id)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let key = Self::encode_vertex_key(space, &vertex.vid);
        let data = Self::serialize_vertex(&vertex)?;

        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.put(&key, &data)?;

        Ok(())
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let key = Self::encode_vertex_key(space, id);

        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.delete(&key)?;

        Ok(())
    }

    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        let mut ids = Vec::new();
        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        for vertex in vertices {
            let id = vertex.vid.clone();
            let key = Self::encode_vertex_key(space, &id);
            let data = Self::serialize_vertex(&vertex)?;
            engine.put(&key, &data)?;
            ids.push(*id);
        }

        Ok(ids)
    }
}

impl<E: Engine> EdgeWriter for RedbStorage<E> {
    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let key = Self::encode_edge_key(space, &edge.src, &edge.dst, &edge.edge_type);
        let data = Self::serialize_edge(&edge)?;

        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.put(&key, &data)?;

        Ok(())
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        let key = Self::encode_edge_key(space, src, dst, edge_type);

        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.delete(&key)?;

        Ok(())
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;

        for edge in edges {
            let key = Self::encode_edge_key(space, &edge.src, &edge.dst, &edge.edge_type);
            let data = Self::serialize_edge(&edge)?;
            engine.put(&key, &data)?;
        }

        Ok(())
    }
}

impl<E: Engine> StorageClient for RedbStorage<E> {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        <Self as VertexReader>::get_vertex(self, space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        <Self as VertexReader>::scan_vertices(self, space).map(|r| r.into_vec())
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        <Self as VertexReader>::scan_vertices_by_tag(self, space, tag).map(|r| r.into_vec())
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        <Self as VertexReader>::scan_vertices_by_prop(self, space, tag, prop, value).map(|r| r.into_vec())
    }

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError> {
        <Self as EdgeReader>::get_edge(self, space, src, dst, edge_type)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        <Self as EdgeReader>::get_node_edges(self, space, node_id, direction).map(|r| r.into_vec())
    }

    fn get_node_edges_filtered(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<Box<dyn Fn(&Edge) -> bool + Send + Sync + 'static>>,
    ) -> Result<Vec<Edge>, StorageError> {
        <Self as EdgeReader>::get_node_edges_filtered(self, space, node_id, direction, filter).map(|r| r.into_vec())
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        <Self as EdgeReader>::scan_edges_by_type(self, space, edge_type).map(|r| r.into_vec())
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        <Self as EdgeReader>::scan_all_edges(self, space).map(|r| r.into_vec())
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        <Self as VertexWriter>::insert_vertex(self, space, vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        <Self as VertexWriter>::update_vertex(self, space, vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        <Self as VertexWriter>::delete_vertex(self, space, id)
    }

    fn batch_insert_vertices(&mut self, space: &str, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        <Self as VertexWriter>::batch_insert_vertices(self, space, vertices)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        <Self as EdgeWriter>::insert_edge(self, space, edge)
    }

    fn delete_edge(&mut self, space: &str, src: &Value, dst: &Value, edge_type: &str) -> Result<(), StorageError> {
        <Self as EdgeWriter>::delete_edge(self, space, src, dst, edge_type)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        <Self as EdgeWriter>::batch_insert_edges(self, space, edges)
    }

    fn begin_transaction(&mut self, space: &str) -> Result<TransactionId, StorageError> {
        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.begin_transaction()
    }

    fn commit_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.commit_transaction(tx_id)
    }

    fn rollback_transaction(&mut self, space: &str, tx_id: TransactionId) -> Result<(), StorageError> {
        let mut engine = self.engine.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        engine.rollback_transaction(tx_id)
    }

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

    fn create_tag(&mut self, space: &str, info: &TagInfo) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space) {
            if space_tags.contains_key(&info.tag_name) {
                return Ok(false);
            }
            space_tags.insert(info.tag_name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
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

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space) {
            Ok(space_tags.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn create_edge_type(&mut self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(space) {
            if space_edge_types.contains_key(&edge.edge_type_name) {
                return Ok(false);
            }
            space_edge_types.insert(edge.edge_type_name.clone(), edge.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
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

    fn get_edge_type(&self, space_name: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
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

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        let edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get(space) {
            Ok(space_edge_types.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn create_tag_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get_mut(space) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get_mut(space) {
            Ok(space_indexes.remove(index).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get(space) {
            Ok(space_indexes.get(index).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get(space) {
            Ok(space_indexes.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn create_edge_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get_mut(space) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get_mut(space) {
            Ok(space_indexes.remove(index).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get(space) {
            Ok(space_indexes.get(index).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get(space) {
            Ok(space_indexes.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn insert_vertex_data(&mut self, space: &str, info: &InsertVertexInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn insert_edge_data(&mut self, space: &str, info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn delete_edge_data(&mut self, space: &str, src: &str, dst: &str, rank: i64) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn update_data(&mut self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        let mut users = self.users.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        users.insert(info.username.clone(), info.new_password.clone());
        Ok(true)
    }

    fn lookup_index(&self, space: &str, index: &str, value: &Value) -> Result<Vec<Value>, StorageError> {
        Ok(vec![])
    }

    fn get_vertex_with_schema(&self, space: &str, tag: &str, id: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        Ok(None)
    }

    fn get_edge_with_schema(&self, space: &str, edge_type: &str, src: &Value, dst: &Value) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        Ok(None)
    }

    fn scan_vertices_with_schema(&self, space: &str, tag: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        Ok(vec![])
    }

    fn scan_edges_with_schema(&self, space: &str, edge_type: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        Ok(vec![])
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        Ok(())
    }
}

pub type DefaultStorage = RedbStorage<RedbEngine>;
