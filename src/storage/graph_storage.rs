//! Graph Storage Implementation
//!
//! Main storage implementation combining PropertyGraph with schema management.

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use crate::core::types::{
    EdgeTypeInfo, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo, PropertyDef, SpaceInfo,
    TagInfo, UpdateInfo, UserAlterInfo, UserInfo,
};
use crate::core::{
    Edge, EdgeDirection, NullType, RoleType, StorageError, StorageResult, Value, Vertex,
};
use crate::storage::api::{StorageClient, StorageStats};
use crate::storage::metadata::{
    InMemoryIndexMetadataManager, InMemorySchemaManager, IndexMetadataManager, Schema,
    SchemaManager,
};
use crate::storage::property_graph::{PropertyGraph, PropertyGraphConfig};
use crate::storage::shared_state::{StorageInner, StorageSharedState};
use crate::storage::entity::UserStorage;
use crate::storage::index::secondary::{InMemoryIndexDataManager, IndexDataManager};
use crate::transaction::context::TransactionContext;
use crate::transaction::version_manager::VersionManager;

#[derive(Clone)]
pub struct GraphStorage {
    graph: Arc<RwLock<PropertyGraph>>,
    schema_manager: Arc<InMemorySchemaManager>,
    index_metadata_manager: Arc<InMemoryIndexMetadataManager>,
    index_data_manager: Arc<InMemoryIndexDataManager>,
    version_manager: Arc<VersionManager>,
    user_storage: Arc<UserStorage>,
    state: Arc<StorageInner>,
    current_txn_context: Arc<Mutex<Option<Arc<TransactionContext>>>>,
    work_dir: Option<PathBuf>,
    db_path: String,
}

impl std::fmt::Debug for GraphStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStorage")
            .field("work_dir", &self.work_dir)
            .finish()
    }
}

impl GraphStorage {
    pub fn new() -> StorageResult<Self> {
        let config = PropertyGraphConfig::default();
        let graph = Arc::new(RwLock::new(PropertyGraph::with_config(config)));
        let schema_manager = Arc::new(InMemorySchemaManager::new());
        let index_metadata_manager = Arc::new(InMemoryIndexMetadataManager::new());
        let index_data_manager = Arc::new(InMemoryIndexDataManager::new());
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        let state = Arc::new(StorageInner::new(graph.clone(), version_manager.clone()));

        Ok(Self {
            graph,
            schema_manager,
            index_metadata_manager,
            index_data_manager,
            version_manager,
            user_storage,
            state,
            current_txn_context: Arc::new(Mutex::new(None)),
            work_dir: None,
            db_path: String::new(),
        })
    }

    pub fn new_with_path(path: PathBuf) -> StorageResult<Self> {
        let config = PropertyGraphConfig {
            work_dir: path.clone(),
            ..Default::default()
        };
        let graph = Arc::new(RwLock::new(PropertyGraph::with_config(config)));
        let schema_manager = Arc::new(InMemorySchemaManager::new());
        let index_metadata_manager = Arc::new(InMemoryIndexMetadataManager::new());
        let index_data_manager = Arc::new(InMemoryIndexDataManager::new());
        let version_manager = Arc::new(VersionManager::new());
        let user_storage = Arc::new(UserStorage::new());

        let state = Arc::new(StorageInner::new(graph.clone(), version_manager.clone()));

        Ok(Self {
            graph,
            schema_manager,
            index_metadata_manager,
            index_data_manager,
            version_manager,
            user_storage,
            state,
            current_txn_context: Arc::new(Mutex::new(None)),
            work_dir: Some(path.clone()),
            db_path: path.to_string_lossy().to_string(),
        })
    }

    pub fn get_db(&self) -> Arc<RwLock<PropertyGraph>> {
        self.graph.clone()
    }

    pub fn state(&self) -> &StorageInner {
        &self.state
    }

    pub fn get_schema_manager(&self) -> Arc<InMemorySchemaManager> {
        self.schema_manager.clone()
    }

    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.current_txn_context.lock().clone()
    }

    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContext>>) {
        *self.current_txn_context.lock() = context;
    }

    pub fn get_shared_state(&self) -> StorageSharedState {
        StorageSharedState::new(
            self.graph.clone(),
            self.version_manager.clone(),
            self.schema_manager.clone(),
            self.index_metadata_manager.clone(),
        )
    }
}

impl Default for GraphStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create GraphStorage")
    }
}

impl StorageClient for GraphStorage {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        let _space_info = self.get_space(space)?
            .ok_or_else(|| StorageError::NotFound(format!("Space {} not found", space)))?;
        
        let tags = self.list_tags(space)?;
        if tags.is_empty() {
            return Ok(None);
        }

        let ts = self.version_manager.read_timestamp();
        let graph = self.graph.read();

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                let id_str = match id {
                    Value::String(s) => s.clone(),
                    _ => id.to_string().unwrap_or_else(|e| format!("{:?}", e)),
                };
                
                if let Some(_record) = graph.get_vertex(label_id, &id_str, ts) {
                    let vertex = Vertex {
                        vid: Box::new(id.clone()),
                        id: 0,
                        tags: Vec::new(),
                        properties: std::collections::HashMap::new(),
                    };
                    return Ok(Some(vertex));
                }
            }
        }

        Ok(None)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        let tags = self.list_tags(space)?;
        let ts = self.version_manager.read_timestamp();
        let graph = self.graph.read();
        let mut vertices = Vec::new();

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                if let Some(iterator) = graph.scan_vertices(label_id, ts) {
                    for record in iterator {
                        let vid_value = Value::String(record.vid.to_string());
                        let vertex = Vertex {
                            vid: Box::new(vid_value),
                            id: 0,
                            tags: Vec::new(),
                            properties: std::collections::HashMap::new(),
                        };
                        vertices.push(vertex);
                    }
                }
            }
        }

        Ok(vertices)
    }

    fn scan_vertices_by_tag(&self, _space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        let ts = self.version_manager.read_timestamp();
        let graph = self.graph.read();
        let mut vertices = Vec::new();

        if let Some(label_id) = graph.get_vertex_label_id(tag) {
            if let Some(iterator) = graph.scan_vertices(label_id, ts) {
                for record in iterator {
                    let vid_value = Value::String(record.vid.to_string());
                    let vertex = Vertex {
                        vid: Box::new(vid_value),
                        id: 0,
                        tags: Vec::new(),
                        properties: std::collections::HashMap::new(),
                    };
                    vertices.push(vertex);
                }
            }
        }

        Ok(vertices)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        let vertices = self.scan_vertices_by_tag(space, tag)?;
        let filtered = vertices.into_iter()
            .filter(|v| {
                v.properties.get(prop) == Some(value) ||
                v.tags.iter().any(|t| t.properties.get(prop) == Some(value))
            })
            .collect();
        Ok(filtered)
    }

    fn get_edge(
        &self,
        _space: &str,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
        _rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        Ok(None)
    }

    fn get_node_edges(
        &self,
        _space: &str,
        _node_id: &Value,
        _direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn get_node_edges_filtered<F>(
        &self,
        _space: &str,
        _node_id: &Value,
        _direction: EdgeDirection,
        _filter: Option<F>,
    ) -> Result<Vec<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        Ok(Vec::new())
    }

    fn scan_edges_by_type(
        &self,
        _space: &str,
        _edge_type: &str,
    ) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_all_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn insert_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<Value, StorageError> {
        Ok(Value::Null(NullType::NaN))
    }

    fn update_vertex(&mut self, _space: &str, _vertex: Vertex) -> Result<(), StorageError> {
        Ok(())
    }

    fn delete_vertex(&mut self, _space: &str, _id: &Value) -> Result<(), StorageError> {
        Ok(())
    }

    fn batch_insert_vertices(
        &mut self,
        _space: &str,
        _vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        Ok(Vec::new())
    }

    fn insert_edge(&mut self, _space: &str, _edge: Edge) -> Result<(), StorageError> {
        Ok(())
    }

    fn delete_edge(
        &mut self,
        _space: &str,
        _src: &Value,
        _dst: &Value,
        _edge_type: &str,
        _rank: i64,
    ) -> Result<(), StorageError> {
        Ok(())
    }

    fn batch_insert_edges(&mut self, _space: &str, _edges: Vec<Edge>) -> Result<(), StorageError> {
        Ok(())
    }

    fn create_space(&mut self, space: &mut SpaceInfo) -> Result<bool, StorageError> {
        self.schema_manager.create_space(space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        self.schema_manager.drop_space(space)
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        self.schema_manager.get_space(space)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        self.schema_manager.get_space_by_id(space_id)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        self.schema_manager.list_spaces()
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        self.schema_manager.get_space_id(space)
    }

    fn space_exists(&self, space: &str) -> bool {
        self.schema_manager
            .get_space(space)
            .ok()
            .flatten()
            .is_some()
    }

    fn clear_space(&mut self, _space: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn alter_space_comment(
        &mut self,
        _space_id: u64,
        _comment: String,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn create_tag(&mut self, space: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        self.schema_manager.create_tag(space, tag)
    }

    fn drop_tag(&mut self, space: &str, tag_name: &str) -> Result<bool, StorageError> {
        self.schema_manager.drop_tag(space, tag_name)
    }

    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        self.schema_manager.get_tag(space, tag_name)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        self.schema_manager.list_tags(space)
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.schema_manager
            .alter_tag(space, tag_name, additions, deletions)
    }

    fn create_edge_type(
        &mut self,
        space: &str,
        edge_type: &EdgeTypeInfo,
    ) -> Result<bool, StorageError> {
        self.schema_manager.create_edge_type(space, edge_type)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        self.schema_manager.drop_edge_type(space, edge_type_name)
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge_type_name: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        self.schema_manager.get_edge_type(space, edge_type_name)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        self.schema_manager.list_edge_types(space)
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.schema_manager
            .alter_edge_type(space, edge_type_name, additions, deletions)
    }

    fn create_tag_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        let space_id = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::NotFound(format!("Space {} not found", space)))?
            .space_id;
        self.index_metadata_manager
            .create_tag_index(space_id, index)?;
        Ok(true)
    }

    fn drop_tag_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        self.schema_manager.drop_tag_index(space, index_name)
    }

    fn get_tag_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        self.schema_manager.get_tag_index(space, index_name)
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        self.schema_manager.list_tag_indexes(space)
    }

    fn create_edge_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        let space_id = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::NotFound(format!("Space {} not found", space)))?
            .space_id;
        self.index_metadata_manager
            .create_edge_index(space_id, index)?;
        Ok(true)
    }

    fn drop_edge_index(&mut self, space: &str, index_name: &str) -> Result<bool, StorageError> {
        self.schema_manager.drop_edge_index(space, index_name)
    }

    fn get_edge_index(&self, space: &str, index_name: &str) -> Result<Option<Index>, StorageError> {
        self.schema_manager.get_edge_index(space, index_name)
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        self.schema_manager.list_edge_indexes(space)
    }

    fn get_schema_manager(
        &self,
    ) -> Option<Arc<dyn crate::storage::metadata::SchemaManager + Send + Sync>> {
        Some(self.schema_manager.clone())
    }

    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        None
    }

    fn create_user(&mut self, info: &UserInfo) -> Result<bool, StorageError> {
        self.user_storage.create_user(info)
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        self.user_storage.drop_user(username)
    }

    fn alter_user(&mut self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        self.user_storage.alter_user(info)
    }

    fn grant_role(
        &mut self,
        username: &str,
        space_id: u64,
        role: RoleType,
    ) -> Result<bool, StorageError> {
        self.user_storage.grant_role(username, space_id, role)
    }

    fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError> {
        self.user_storage.revoke_role(username, space_id)
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let tags = self.list_tags(space)?;
        let ts = self.version_manager.read_timestamp();
        let mut graph = self.graph.write();

        let id_str = match id {
            Value::String(s) => s.clone(),
            _ => id.to_string().unwrap_or_default(),
        };

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                let _ = graph.delete_vertex(label_id, &id_str, ts);
            }
        }

        Ok(())
    }

    fn delete_tags(
        &mut self,
        _space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let ts = self.version_manager.read_timestamp();
        let mut graph = self.graph.write();
        let mut deleted_count = 0;

        let id_str = match vertex_id {
            Value::String(s) => s.clone(),
            _ => vertex_id.to_string().unwrap_or_default(),
        };

        for tag_name in tag_names {
            if let Some(label_id) = graph.get_vertex_label_id(tag_name) {
                if graph.delete_vertex(label_id, &id_str, ts).is_ok() {
                    deleted_count += 1;
                }
            }
        }

        Ok(deleted_count)
    }

    fn rebuild_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn rebuild_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        let _tag = self.get_tag(space, &info.tag_name)?
            .ok_or_else(|| StorageError::NotFound(format!("Tag {} not found", info.tag_name)))?;

        let space_id = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::NotFound(format!("Space {} not found", space)))?
            .space_id;
        if info.space_id != space_id {
            return Err(StorageError::DbError("Space ID mismatch".to_string()));
        }

        let ts = self.version_manager.read_timestamp();
        let mut graph = self.graph.write();

        if let Some(label_id) = graph.get_vertex_label_id(&info.tag_name) {
            let id_str = match &info.vertex_id {
                Value::String(s) => s.clone(),
                _ => info.vertex_id.to_string().unwrap_or_default(),
            };

            let result = graph.insert_vertex(label_id, &id_str, &info.props, ts);
            match result {
                Ok(_) => Ok(true),
                Err(StorageError::VertexAlreadyExists(_)) => Ok(false),
                Err(e) => Err(e),
            }
        } else {
            Err(StorageError::NotFound(format!("Tag {} not found", info.tag_name)))
        }
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        let _edge_type = self.get_edge_type(space, &info.edge_name)?
            .ok_or_else(|| StorageError::NotFound(format!("Edge type {} not found", info.edge_name)))?;

        let space_id = self
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::NotFound(format!("Space {} not found", space)))?
            .space_id;
        if info.space_id != space_id {
            return Err(StorageError::DbError("Space ID mismatch".to_string()));
        }

        let ts = self.version_manager.read_timestamp();
        let mut graph = self.graph.write();

        if let Some(edge_label_id) = graph.get_edge_label_id(&info.edge_name) {
            let src_id = match &info.src_vertex_id {
                Value::String(s) => s.clone(),
                _ => info.src_vertex_id.to_string().unwrap_or_default(),
            };
            let dst_id = match &info.dst_vertex_id {
                Value::String(s) => s.clone(),
                _ => info.dst_vertex_id.to_string().unwrap_or_default(),
            };

            let src_label_id = graph.get_vertex_label_id("vertex")
                .ok_or_else(|| StorageError::NotFound("Default vertex label not found".to_string()))?;
            let dst_label_id = src_label_id;

            let result = graph.insert_edge(
                edge_label_id,
                src_label_id,
                &src_id,
                dst_label_id,
                &dst_id,
                &info.props,
                ts,
            );
            match result {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
        } else {
            Err(StorageError::NotFound(format!("Edge type {} not found", info.edge_name)))
        }
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        let tags = self.list_tags(space)?;
        let ts = self.version_manager.read_timestamp();
        let mut graph = self.graph.write();
        let mut deleted = false;

        for tag in &tags {
            if let Some(label_id) = graph.get_vertex_label_id(&tag.tag_name) {
                if graph.delete_vertex(label_id, vertex_id, ts).is_ok() {
                    deleted = true;
                }
            }
        }

        Ok(deleted)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        _rank: i64,
    ) -> Result<bool, StorageError> {
        let edge_types = self.list_edge_types(space)?;
        let ts = self.version_manager.read_timestamp();
        let mut graph = self.graph.write();

        for edge_type in &edge_types {
            if let Some(edge_label_id) = graph.get_edge_label_id(&edge_type.edge_type_name) {
                let src_label_id = graph.get_vertex_label_id("vertex");
                let dst_label_id = src_label_id;

                if let (Some(src_label_id), Some(dst_label_id)) = (src_label_id, dst_label_id) {
                    let result = graph.delete_edge(
                        edge_label_id,
                        src_label_id,
                        src,
                        dst_label_id,
                        dst,
                        ts,
                    );
                    if result.is_ok_and(|r| r) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn update_data(
        &mut self,
        _space: &str,
        _space_id: u64,
        info: &UpdateInfo,
    ) -> Result<bool, StorageError> {
        let ts = self.version_manager.read_timestamp();
        let mut graph = self.graph.write();

        let id_str = match &info.update_target.id {
            Value::String(s) => s.clone(),
            _ => info.update_target.id.to_string().unwrap_or_default(),
        };

        if let Some(label_id) = graph.get_vertex_label_id(&info.update_target.label) {
            let result = graph.update_vertex_property(
                label_id,
                &id_str,
                &info.update_target.prop,
                &info.value,
                ts,
            );
            match result {
                Ok(_) => Ok(true),
                Err(StorageError::VertexNotFound) => Ok(false),
                Err(e) => Err(e),
            }
        } else {
            Ok(false)
        }
    }

    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        self.user_storage.change_password(info)
    }

    fn lookup_index(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        let space_id = self.get_space_id(space)?;
        let tag_index = self.index_metadata_manager.get_tag_index(space_id, index)?;
        if let Some(idx) = tag_index {
            return self.index_data_manager.lookup_tag_index(space_id, &idx, value);
        }

        let edge_index = self.index_metadata_manager.get_edge_index(space_id, index)?;
        if let Some(idx) = edge_index {
            return self.index_data_manager.lookup_edge_index(space_id, &idx, value);
        }

        Ok(Vec::new())
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        let results = self.lookup_index(space, index, value)?;
        Ok(results.into_iter().map(|v| (v, 1.0)).collect())
    }

    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let tag_info = self.get_tag(space, tag)?
            .ok_or_else(|| StorageError::NotFound(format!("Tag {} not found", tag)))?;

        let schema = self.schema_manager.get_schema(space, tag)?
            .ok_or_else(|| StorageError::NotFound(format!("Tag {} not found", tag)))?;

        let id_str = match id {
            Value::String(s) => s.clone(),
            _ => id.to_string().unwrap_or_default(),
        };

        let ts = self.version_manager.read_timestamp();
        let graph = self.graph.read();

        if let Some(label_id) = graph.get_vertex_label_id(tag) {
            if let Some(record) = graph.get_vertex(label_id, &id_str, ts) {
                let bytes = serialize_vertex_properties(&record.properties, &tag_info);
                return Ok(Some((schema, bytes)));
            }
        }

        Ok(None)
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        let _edge_type_info = self.get_edge_type(space, edge_type)?
            .ok_or_else(|| StorageError::NotFound(format!("Edge type {} not found", edge_type)))?;

        let schema = self.schema_manager.get_schema(space, edge_type)?
            .ok_or_else(|| StorageError::NotFound(format!("Edge type {} not found", edge_type)))?;

        let src_id = match src {
            Value::String(s) => s.clone(),
            _ => src.to_string().unwrap_or_default(),
        };
        let dst_id = match dst {
            Value::String(s) => s.clone(),
            _ => dst.to_string().unwrap_or_default(),
        };

        let ts = self.version_manager.read_timestamp();
        let graph = self.graph.read();

        if let Some(edge_label_id) = graph.get_edge_label_id(edge_type) {
            let src_label_id = graph.get_vertex_label_id("vertex");
            let dst_label_id = src_label_id;

            if let (Some(src_label_id), Some(dst_label_id)) = (src_label_id, dst_label_id) {
                if let Some(record) = graph.get_edge(
                    edge_label_id,
                    src_label_id,
                    &src_id,
                    dst_label_id,
                    &dst_id,
                    ts,
                ) {
                    let bytes = serialize_edge_properties(&record.properties, &_edge_type_info);
                    return Ok(Some((schema, bytes)));
                }
            }
        }

        Ok(None)
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let tag_info = self.get_tag(space, tag)?
            .ok_or_else(|| StorageError::NotFound(format!("Tag {} not found", tag)))?;

        let schema = self.schema_manager.get_schema(space, tag)?
            .ok_or_else(|| StorageError::NotFound(format!("Tag {} not found", tag)))?;

        let ts = self.version_manager.read_timestamp();
        let graph = self.graph.read();
        let mut results = Vec::new();

        if let Some(label_id) = graph.get_vertex_label_id(tag) {
            if let Some(iterator) = graph.scan_vertices(label_id, ts) {
                for record in iterator {
                    let bytes = serialize_vertex_properties(&record.properties, &tag_info);
                    results.push((schema.clone(), bytes));
                }
            }
        }

        Ok(results)
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        let edge_type_info = self.get_edge_type(space, edge_type)?
            .ok_or_else(|| StorageError::NotFound(format!("Edge type {} not found", edge_type)))?;

        let schema = self.schema_manager.get_schema(space, edge_type)?
            .ok_or_else(|| StorageError::NotFound(format!("Edge type {} not found", edge_type)))?;

        let ts = self.version_manager.read_timestamp();
        let graph = self.graph.read();
        let mut results = Vec::new();

        if let Some(edge_label_id) = graph.get_edge_label_id(edge_type) {
            let src_label_id = graph.get_vertex_label_id("vertex");
            let dst_label_id = src_label_id;

            if let (Some(src_label_id), Some(dst_label_id)) = (src_label_id, dst_label_id) {
                if let Some(src_iterator) = graph.scan_vertices(src_label_id, ts) {
                    for src_record in src_iterator {
                        let src_id_str = src_record.vid.to_string();
                        if let Some(edges) = graph.out_edges(
                            edge_label_id,
                            src_label_id,
                            dst_label_id,
                            &src_id_str,
                            ts,
                        ) {
                            for record in edges {
                                let bytes = serialize_edge_properties(&record.properties, &edge_type_info);
                                results.push((schema.clone(), bytes));
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        let work_dir = match &self.work_dir {
            Some(path) => path,
            None => {
                return Err(StorageError::StorageError(
                    "Work directory not set".to_string(),
                ))
            }
        };

        let schema_path = work_dir.join("schema").join("schema.json");
        if schema_path.exists() {
            Arc::get_mut(&mut self.schema_manager)
                .ok_or_else(|| {
                    StorageError::StorageError("Cannot mutate schema manager".to_string())
                })?
                .load_schema(&schema_path)?;
        }

        let data_dir = work_dir.join("data");
        if data_dir.exists() {
            self.graph.write().load()?;
        }

        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        let work_dir = match &self.work_dir {
            Some(path) => path,
            None => {
                return Err(StorageError::StorageError(
                    "Work directory not set".to_string(),
                ))
            }
        };

        std::fs::create_dir_all(work_dir).map_err(|e| StorageError::IOError(e.to_string()))?;

        let schema_dir = work_dir.join("schema");
        std::fs::create_dir_all(&schema_dir).map_err(|e| StorageError::IOError(e.to_string()))?;

        let schema_path = schema_dir.join("schema.json");
        self.schema_manager.save_schema(&schema_path)?;

        self.graph.read().flush()?;

        Ok(())
    }

    fn get_storage_stats(&self) -> StorageStats {
        StorageStats {
            total_vertices: 0,
            total_edges: 0,
            total_spaces: 0,
            total_tags: 0,
            total_edge_types: 0,
        }
    }

    fn find_dangling_edges(&self, _space: &str) -> Result<Vec<Edge>, StorageError> {
        Ok(Vec::new())
    }

    fn repair_dangling_edges(&mut self, _space: &str) -> Result<usize, StorageError> {
        Ok(0)
    }

    fn get_db_path(&self) -> &str {
        &self.db_path
    }
}

fn serialize_vertex_properties(properties: &[(String, Value)], tag_info: &TagInfo) -> Vec<u8> {
    let mut bytes = Vec::new();
    for prop_def in &tag_info.properties {
        if let Some((_, value)) = properties.iter().find(|(name, _)| name == &prop_def.name) {
            bytes.extend_from_slice(&value_to_bytes(value));
        } else {
            bytes.push(0);
        }
    }
    bytes
}

fn serialize_edge_properties(properties: &[(String, Value)], edge_info: &EdgeTypeInfo) -> Vec<u8> {
    let mut bytes = Vec::new();
    for prop_def in &edge_info.properties {
        if let Some((_, value)) = properties.iter().find(|(name, _)| name == &prop_def.name) {
            bytes.extend_from_slice(&value_to_bytes(value));
        } else {
            bytes.push(0);
        }
    }
    bytes
}

fn value_to_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::Null(_) | Value::Empty => vec![0],
        Value::Bool(v) => {
            let mut buf = vec![1];
            buf.extend_from_slice(&[*v as u8]);
            buf
        }
        Value::Int(v) => {
            let mut buf = vec![2];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::BigInt(v) => {
            let mut buf = vec![3];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::Float(v) => {
            let mut buf = vec![4];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::Double(v) => {
            let mut buf = vec![5];
            buf.extend_from_slice(&v.to_le_bytes());
            buf
        }
        Value::String(v) => {
            let mut buf = vec![6];
            let bytes = v.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytes);
            buf
        }
        Value::Blob(v) => {
            let mut buf = vec![7];
            buf.extend_from_slice(&(v.len() as u32).to_le_bytes());
            buf.extend_from_slice(v);
            buf
        }
        _ => vec![0],
    }
}
