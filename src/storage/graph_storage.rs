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
use crate::core::{Edge, EdgeDirection, NullType, RoleType, StorageError, StorageResult, Value, Vertex};
use crate::storage::api::{StorageClient, StorageStats};
use crate::storage::metadata::{InMemorySchemaManager, InMemoryIndexMetadataManager, Schema, SchemaManager, IndexMetadataManager};
use crate::storage::property_graph::{PropertyGraph, PropertyGraphConfig};
use crate::storage::shared_state::{StorageInner, StorageSharedState};
use crate::transaction::version_manager::VersionManager;
use crate::transaction::context::TransactionContext;

#[derive(Clone)]
pub struct GraphStorage {
    graph: Arc<RwLock<PropertyGraph>>,
    schema_manager: Arc<InMemorySchemaManager>,
    index_metadata_manager: Arc<InMemoryIndexMetadataManager>,
    version_manager: Arc<VersionManager>,
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
        let version_manager = Arc::new(VersionManager::new());

        let state = Arc::new(StorageInner::new(graph.clone(), version_manager.clone()));

        Ok(Self {
            graph,
            schema_manager,
            index_metadata_manager,
            version_manager,
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
        let version_manager = Arc::new(VersionManager::new());

        let state = Arc::new(StorageInner::new(graph.clone(), version_manager.clone()));

        Ok(Self {
            graph,
            schema_manager,
            index_metadata_manager,
            version_manager,
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

    fn get_vertex(&self, _space: &str, _id: &Value) -> Result<Option<Vertex>, StorageError> {
        Ok(None)
    }

    fn scan_vertices(&self, _space: &str) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_tag(&self, _space: &str, _tag: &str) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_vertices_by_prop(
        &self,
        _space: &str,
        _tag: &str,
        _prop: &str,
        _value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        Ok(Vec::new())
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
        self.schema_manager.get_space(space).ok().flatten().is_some()
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
        self.schema_manager.alter_tag(space, tag_name, additions, deletions)
    }

    fn create_edge_type(&mut self, space: &str, edge_type: &EdgeTypeInfo) -> Result<bool, StorageError> {
        self.schema_manager.create_edge_type(space, edge_type)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        self.schema_manager.drop_edge_type(space, edge_type_name)
    }

    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
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
        self.schema_manager.alter_edge_type(space, edge_type_name, additions, deletions)
    }

    fn create_tag_index(&mut self, space: &str, index: &Index) -> Result<bool, StorageError> {
        self.schema_manager.create_tag_index(space, index)
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
        self.schema_manager.create_edge_index(space, index)
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

    fn get_schema_manager(&self) -> Option<Arc<dyn crate::storage::metadata::SchemaManager + Send + Sync>> {
        Some(self.schema_manager.clone())
    }

    fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        None
    }

    fn create_user(&mut self, _info: &UserInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn drop_user(&mut self, _username: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn alter_user(&mut self, _info: &UserAlterInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn grant_role(
        &mut self,
        _username: &str,
        _space_id: u64,
        _role: RoleType,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn revoke_role(&mut self, _username: &str, _space_id: u64) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn delete_vertex_with_edges(&mut self, _space: &str, _id: &Value) -> Result<(), StorageError> {
        Ok(())
    }

    fn delete_tags(
        &mut self,
        _space: &str,
        _vertex_id: &Value,
        _tag_names: &[String],
    ) -> Result<usize, StorageError> {
        Ok(0)
    }

    fn rebuild_tag_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn rebuild_edge_index(&mut self, _space: &str, _index: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn insert_vertex_data(
        &mut self,
        _space: &str,
        _info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn insert_edge_data(
        &mut self,
        _space: &str,
        _info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn delete_vertex_data(&mut self, _space: &str, _vertex_id: &str) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn delete_edge_data(
        &mut self,
        _space: &str,
        _src: &str,
        _dst: &str,
        _rank: i64,
    ) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn update_data(&mut self, _space: &str, _info: &UpdateInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn change_password(&mut self, _info: &PasswordInfo) -> Result<bool, StorageError> {
        Ok(true)
    }

    fn lookup_index(
        &self,
        _space: &str,
        _index: &str,
        _value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        Ok(Vec::new())
    }

    fn lookup_index_with_score(
        &self,
        _space: &str,
        _index: &str,
        _value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        Ok(Vec::new())
    }

    fn get_vertex_with_schema(
        &self,
        _space: &str,
        _tag: &str,
        _id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        Ok(None)
    }

    fn get_edge_with_schema(
        &self,
        _space: &str,
        _edge_type: &str,
        _src: &Value,
        _dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        Ok(None)
    }

    fn scan_vertices_with_schema(
        &self,
        _space: &str,
        _tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        Ok(Vec::new())
    }

    fn scan_edges_with_schema(
        &self,
        _space: &str,
        _edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        Ok(Vec::new())
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        let work_dir = match &self.work_dir {
            Some(path) => path,
            None => return Err(StorageError::StorageError("Work directory not set".to_string())),
        };

        let schema_path = work_dir.join("schema").join("schema.json");
        if schema_path.exists() {
            Arc::get_mut(&mut self.schema_manager)
                .ok_or_else(|| StorageError::StorageError("Cannot mutate schema manager".to_string()))?
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
            None => return Err(StorageError::StorageError("Work directory not set".to_string())),
        };

        std::fs::create_dir_all(work_dir)
            .map_err(|e| StorageError::IOError(e.to_string()))?;

        let schema_dir = work_dir.join("schema");
        std::fs::create_dir_all(&schema_dir)
            .map_err(|e| StorageError::IOError(e.to_string()))?;
        
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
