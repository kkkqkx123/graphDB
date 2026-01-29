use super::{StorageEngine, TransactionId, EdgeReader, EdgeWriter, ScanResult, VertexReader, VertexWriter, SchemaManager, MemorySchemaManager};
use crate::core::{Edge, StorageError, Value, Vertex, EdgeDirection};
use crate::core::vertex_edge_path::Tag;
use crate::core::types::{
    SpaceInfo, TagInfo, EdgeTypeSchema, IndexInfo,
    PropertyDef, InsertVertexInfo, InsertEdgeInfo, UpdateInfo,
    PasswordInfo,
};
use crate::expression::storage::{FieldDef, FieldType, RowReaderWrapper, Schema};
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
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_type_infos: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
    tag_indexes: Arc<Mutex<HashMap<String, HashMap<String, IndexInfo>>>>,
    edge_indexes: Arc<Mutex<HashMap<String, HashMap<String, IndexInfo>>>>,
    users: Arc<Mutex<HashMap<String, String>>>,
    pub schema_manager: Arc<MemorySchemaManager>,
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
        let schema_manager = Arc::new(MemorySchemaManager::new());

        Ok(Self {
            vertices: Arc::new(Mutex::new(HashMap::new())),
            edges: Arc::new(Mutex::new(HashMap::new())),
            vertex_tags: Arc::new(Mutex::new(HashMap::new())),
            edge_types: Arc::new(Mutex::new(HashMap::new())),
            active_transactions: Arc::new(Mutex::new(HashMap::new())),
            next_tx_id: Arc::new(Mutex::new(TransactionId::new(1))),
            _vertex_props: Arc::new(Mutex::new(HashMap::new())),
            _memory_pool: memory_pool,
            id_generator,
            spaces: Arc::new(Mutex::new(HashMap::new())),
            tags: Arc::new(Mutex::new(HashMap::new())),
            edge_type_infos: Arc::new(Mutex::new(HashMap::new())),
            tag_indexes: Arc::new(Mutex::new(HashMap::new())),
            edge_indexes: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(HashMap::new())),
            schema_manager,
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
        let fields: Vec<FieldDef> = tag_info.properties.iter().map(|prop| {
            let field_type = Self::data_type_to_field_type(&prop.type_def);
            FieldDef {
                name: prop.name.clone(),
                field_type,
                nullable: prop.is_nullable,
                default_value: None,
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            }
        }).collect();

        Schema {
            name: tag_name.to_string(),
            version: 1,
            fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }

    fn edge_type_schema_to_schema(edge_type_name: &str, edge_schema: &EdgeTypeSchema) -> Schema {
        let fields: Vec<FieldDef> = edge_schema.properties.iter().map(|prop| {
            let field_type = Self::data_type_to_field_type(&prop.type_def);
            FieldDef {
                name: prop.name.clone(),
                field_type,
                nullable: prop.is_nullable,
                default_value: None,
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            }
        }).collect();

        Schema {
            name: edge_type_name.to_string(),
            version: 1,
            fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }

    fn data_type_to_field_type(data_type: &crate::core::DataType) -> FieldType {
        match data_type {
            crate::core::DataType::Bool => FieldType::Bool,
            crate::core::DataType::Int8 => FieldType::Int8,
            crate::core::DataType::Int16 => FieldType::Int16,
            crate::core::DataType::Int32 => FieldType::Int32,
            crate::core::DataType::Int64 => FieldType::Int64,
            crate::core::DataType::Float => FieldType::Float,
            crate::core::DataType::Double => FieldType::Double,
            crate::core::DataType::String => FieldType::String,
            crate::core::DataType::Date => FieldType::Date,
            crate::core::DataType::Time => FieldType::Time,
            crate::core::DataType::DateTime => FieldType::DateTime,
            crate::core::DataType::List => FieldType::List,
            crate::core::DataType::Map => FieldType::Map,
            crate::core::DataType::Set => FieldType::Set,
            crate::core::DataType::Geography => FieldType::Geography,
            crate::core::DataType::Duration => FieldType::Duration,
            _ => FieldType::String,
        }
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
        let edges = <Self as StorageEngine>::get_node_edges(self, node_id, direction)?;
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
            <Self as StorageEngine>::insert_edge(self, edge)?;
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

    // ========== 空间管理 ==========
    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if spaces.contains_key(&space.name) {
            return Ok(false);
        }
        spaces.insert(space.name.clone(), space.clone());
        
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.insert(space.name.clone(), HashMap::new());
        
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_type_infos.insert(space.name.clone(), HashMap::new());
        
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tag_indexes.insert(space.name.clone(), HashMap::new());
        
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_indexes.insert(space.name.clone(), HashMap::new());
        
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
    fn create_tag(&mut self, info: &TagInfo) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(&info.space_name) {
            if space_tags.contains_key(&info.name) {
                return Ok(false);
            }
            space_tags.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", info.space_name)))
        }
    }

    fn alter_tag(&mut self, space_name: &str, tag_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space_name) {
            if let Some(tag_info) = space_tags.get_mut(tag_name) {
                for prop in additions {
                    let new_prop = super::super::core::types::PropertyType {
                        name: prop.name,
                        type_def: prop.data_type,
                        is_nullable: prop.nullable,
                    };
                    tag_info.properties.retain(|p| p.name != new_prop.name);
                    tag_info.properties.push(new_prop);
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
    fn create_edge_type(&mut self, info: &EdgeTypeSchema) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(&info.space_name) {
            if space_edge_types.contains_key(&info.name) {
                return Ok(false);
            }
            space_edge_types.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", info.space_name)))
        }
    }

    fn alter_edge_type(&mut self, space_name: &str, edge_type_name: &str, additions: Vec<PropertyDef>, deletions: Vec<String>) -> Result<bool, StorageError> {
        let mut edge_type_infos = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edge_types) = edge_type_infos.get_mut(space_name) {
            if let Some(edge_type_info) = space_edge_types.get_mut(edge_type_name) {
                for prop in additions {
                    let new_prop = super::super::core::types::PropertyType {
                        name: prop.name,
                        type_def: prop.data_type,
                        is_nullable: prop.nullable,
                    };
                    edge_type_info.properties.retain(|p| p.name != new_prop.name);
                    edge_type_info.properties.push(new_prop);
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
    fn create_tag_index(&mut self, info: &IndexInfo) -> Result<bool, StorageError> {
        let mut tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get_mut(&info.space_name) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", info.space_name)))
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

    fn get_tag_index(&self, space_name: &str, index_name: &str) -> Result<Option<IndexInfo>, StorageError> {
        let tag_indexes = self.tag_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = tag_indexes.get(space_name) {
            Ok(space_indexes.get(index_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_tag_indexes(&self, space_name: &str) -> Result<Vec<IndexInfo>, StorageError> {
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

    fn create_edge_index(&mut self, info: &IndexInfo) -> Result<bool, StorageError> {
        let mut edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get_mut(&info.space_name) {
            if space_indexes.contains_key(&info.name) {
                return Ok(false);
            }
            space_indexes.insert(info.name.clone(), info.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", info.space_name)))
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

    fn get_edge_index(&self, space_name: &str, index_name: &str) -> Result<Option<IndexInfo>, StorageError> {
        let edge_indexes = self.edge_indexes.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_indexes) = edge_indexes.get(space_name) {
            Ok(space_indexes.get(index_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space_name)))
        }
    }

    fn list_edge_indexes(&self, space_name: &str) -> Result<Vec<IndexInfo>, StorageError> {
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
    fn insert_vertex_data(&mut self, info: &InsertVertexInfo) -> Result<bool, StorageError> {
        let mut vertices = self.vertices.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let vertex_key = Self::serialize_value(&Value::String(info.vertex_id.clone()));
        
        if vertices.contains_key(&vertex_key) {
            return Ok(false);
        }
        
        let vertex = Vertex::new_with_properties(
            Value::String(info.vertex_id.clone()),
            vec![Tag::new(info.tag_name.clone(), HashMap::new())],
            info.properties.iter().cloned().collect(),
        );
        
        vertices.insert(vertex_key, vertex);
        Ok(true)
    }

    fn insert_edge_data(&mut self, info: &InsertEdgeInfo) -> Result<bool, StorageError> {
        let mut edges = self.edges.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        
        let edge_key = (
            Self::serialize_value(&Value::String(info.src_vertex_id.clone())),
            Self::serialize_value(&Value::String(info.dst_vertex_id.clone())),
            info.edge_name.clone(),
        );
        
        if edges.contains_key(&edge_key) {
            return Ok(false);
        }
        
        let edge = Edge::new(
            Value::String(info.src_vertex_id.clone()),
            Value::String(info.dst_vertex_id.clone()),
            info.edge_name.clone(),
            info.rank,
            info.properties.iter().cloned().collect(),
        );
        
        edges.insert(edge_key, edge);
        Ok(true)
    }

    fn delete_vertex_data(&mut self, _space_name: &str, vertex_id: &str) -> Result<bool, StorageError> {
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

    fn update_data(&mut self, _info: &UpdateInfo) -> Result<bool, StorageError> {
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

        let vertex = self.get_node(id)?;
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

        let edge = <Self as StorageEngine>::get_edge(self, src, dst, edge_type_name)?;
        edge.as_ref().map(|e| {
            let schema = Self::edge_type_schema_to_schema(&edge_type_name, &edge_schema);
            let binary_data = Self::serialize_edge(e);
            Ok((schema, binary_data))
        }).transpose()
    }

    fn scan_vertices_with_schema(&self, space_name: &str, tag_name: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let vertices = <Self as StorageEngine>::scan_vertices_by_tag(self, tag_name)?;
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let tag_info = tags.get(space_name)
            .and_then(|space_tags| space_tags.get(tag_name))
            .cloned()
            .unwrap_or_else(|| TagInfo::new(space_name.to_string(), tag_name.to_string()));

        let schema = Self::tag_info_to_schema(&tag_name, &tag_info);
        let binary_data_list: Vec<Vec<u8>> = vertices.iter().map(|v| Self::serialize_vertex(v)).collect();

        Ok(binary_data_list.into_iter().map(|data| (schema.clone(), data)).collect())
    }

    fn scan_edges_with_schema(&self, space_name: &str, edge_type_name: &str) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let edges = <Self as StorageEngine>::scan_edges_by_type(self, edge_type_name)?;
        let edge_types = self.edge_type_infos.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        let edge_schema = edge_types.get(space_name)
            .and_then(|space_edges| space_edges.get(edge_type_name))
            .cloned()
            .unwrap_or_else(|| EdgeTypeSchema::new(space_name.to_string(), edge_type_name.to_string()));

        let schema = Self::edge_type_schema_to_schema(&edge_type_name, &edge_schema);
        let binary_data_list: Vec<Vec<u8>> = edges.iter().map(|e| Self::serialize_edge(e)).collect();

        Ok(binary_data_list.into_iter().map(|data| (schema.clone(), data)).collect())
    }
}

impl VertexReader for MemoryStorage {
    fn get_vertex(&self, _space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        <Self as StorageEngine>::get_node(self, id)
    }

    fn scan_vertices(&self, _space: &str) -> Result<ScanResult<Vertex>, StorageError> {
        let vertices = <Self as StorageEngine>::scan_all_vertices(self)?;
        Ok(ScanResult::new(vertices))
    }

    fn scan_vertices_by_tag(&self, _space: &str, tag_name: &str) -> Result<ScanResult<Vertex>, StorageError> {
        let vertices = <Self as StorageEngine>::scan_vertices_by_tag(self, tag_name)?;
        Ok(ScanResult::new(vertices))
    }

    fn scan_vertices_by_prop(
        &self,
        _space: &str,
        tag_name: &str,
        prop_name: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError> {
        let vertices = <Self as StorageEngine>::scan_vertices_by_prop(self, tag_name, prop_name, value)?;
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
        <Self as StorageEngine>::get_edge(self, src, dst, edge_type)
    }

    fn get_node_edges(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let edges = <Self as StorageEngine>::get_node_edges(self, node_id, direction)?;
        Ok(ScanResult::new(edges))
    }

    fn get_node_edges_filtered(
        &self,
        _space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        edge_type: Option<&str>,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let filter = edge_type.map(|et| {
            let et = et.to_string();
            move |e: &Edge| e.edge_type == et
        });
        let edges = <Self as StorageEngine>::get_node_edges_filtered(self, node_id, direction, filter.map(|f| Box::new(f) as _))?;
        Ok(ScanResult::new(edges))
    }

    fn scan_edges_by_type(
        &self,
        _space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError> {
        let edges = <Self as StorageEngine>::scan_edges_by_type(self, edge_type)?;
        Ok(ScanResult::new(edges))
    }

    fn scan_all_edges(&self, _space: &str) -> Result<ScanResult<Edge>, StorageError> {
        let edges = <Self as StorageEngine>::scan_all_edges(self)?;
        Ok(ScanResult::new(edges))
    }
}

impl VertexWriter for MemoryStorage {
    fn insert_vertex(&mut self, vertex: Vertex) -> Result<Value, StorageError> {
        <Self as StorageEngine>::insert_node(self, vertex)
    }

    fn update_vertex(&mut self, vertex: Vertex) -> Result<(), StorageError> {
        <Self as StorageEngine>::update_node(self, vertex)
    }

    fn delete_vertex(&mut self, id: &Value) -> Result<(), StorageError> {
        <Self as StorageEngine>::delete_node(self, id)
    }

    fn batch_insert_vertices(&mut self, vertices: Vec<Vertex>) -> Result<Vec<Value>, StorageError> {
        <Self as StorageEngine>::batch_insert_nodes(self, vertices)
    }
}

impl EdgeWriter for MemoryStorage {
    fn insert_edge(&mut self, edge: Edge) -> Result<(), StorageError> {
        <Self as StorageEngine>::insert_edge(self, edge)
    }

    fn delete_edge(
        &mut self,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        <Self as StorageEngine>::delete_edge(self, src, dst, edge_type)
    }

    fn batch_insert_edges(&mut self, edges: Vec<Edge>) -> Result<(), StorageError> {
        <Self as StorageEngine>::batch_insert_edges(self, edges)
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
