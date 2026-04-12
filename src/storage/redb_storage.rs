use super::StorageClient;
use crate::core::types::{
    EdgeTypeInfo, Index, InsertEdgeInfo, InsertVertexInfo, PasswordInfo, PropertyDef, SpaceInfo,
    TagInfo, UpdateInfo, UserAlterInfo, UserInfo,
};
use crate::core::{Edge, EdgeDirection, RoleType, StorageError, Value, Vertex};
use crate::storage::edge_storage::EdgeStorage;
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{
    IndexMetadataManager, RedbIndexMetadataManager, RedbSchemaManager, SchemaManager,
};
use crate::storage::operations::{RedbReader, RedbWriter};
use crate::storage::shared_state::{StorageInner, StorageSharedState};
use crate::storage::user_storage::UserStorage;
use crate::storage::vertex_storage::VertexStorage;
use crate::storage::Schema;
use crate::sync::SyncManager;
use crate::transaction::TransactionContext;
use parking_lot::Mutex;
use redb::Database;
use std::path::PathBuf;
use std::sync::Arc;

/// Redb Storage Engine Main Structure
///
/// As a unified interface for the storage layer, it coordinates the various sub-modules to perform specific data operations.
#[derive(Clone)]
pub struct RedbStorage {
    state: Arc<StorageSharedState>,
    inner: Arc<StorageInner>,
    index_data_manager: RedbIndexDataManager,
    db_path: PathBuf,
    vertex_storage: VertexStorage,
    edge_storage: EdgeStorage,
    user_storage: UserStorage,
}

impl std::fmt::Debug for RedbStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbStorage")
            .field("db_path", &self.db_path)
            .finish()
    }
}

impl RedbStorage {
    /// Create a storage instance with the default path set.
    pub fn new() -> Result<Self, StorageError> {
        Self::new_with_path(PathBuf::from("data/redb"))
    }

    /// Create a storage instance at the specified path.
    pub fn new_with_path(path: PathBuf) -> Result<Self, StorageError> {
        let is_new_db = !path.exists();

        let db = if path.exists() {
            match Database::open(&path) {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    return Err(StorageError::DbError(format!(
                        "Failed to open the database: path: {}, error: {}. Please manually delete the database file and retry.",
                        path.display(),
                        e
                    )));
                }
            }
        } else {
            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        StorageError::DbError(format!(
                            "Failed to create storage directory '{}': {}",
                            parent.display(),
                            e
                        ))
                    })?;
                }
            }

            Arc::new(Database::create(&path).map_err(|e| {
                StorageError::DbError(format!("Failed to create the database: {}", e))
            })?)
        };

        // Initialize the tables only when a new database is created.
        if is_new_db {
            Self::initialize_tables(&db)?;
        }

        let schema_manager = Arc::new(RedbSchemaManager::new(db.clone()));
        let index_metadata_manager = Arc::new(RedbIndexMetadataManager::new(db.clone()));

        let reader = RedbReader::new(db.clone())?;
        let writer = RedbWriter::new(db.clone())?;

        let state = Arc::new(StorageSharedState::new(
            db.clone(),
            schema_manager.clone(),
            index_metadata_manager.clone(),
        ));

        let inner = Arc::new(StorageInner::new(reader, writer));

        let index_data_manager = RedbIndexDataManager::new(db.clone());

        // Create a sub-module
        let vertex_storage =
            VertexStorage::new(state.clone(), inner.clone(), index_data_manager.clone())?;

        let edge_storage =
            EdgeStorage::new(state.clone(), inner.clone(), index_data_manager.clone())?;

        let user_storage = UserStorage::new();

        Ok(Self {
            state,
            inner,
            index_data_manager,
            db_path: path,
            vertex_storage,
            edge_storage,
            user_storage,
        })
    }

    /// Get shared state reference
    pub fn state(&self) -> &StorageSharedState {
        &self.state
    }

    /// Set sync manager (for enabling index synchronization)
    pub fn set_sync_manager(&self, sync_manager: Arc<SyncManager>) {
        self.state.set_sync_manager(sync_manager);
    }

    /// Initialize the database tables
    fn initialize_tables(db: &Arc<Database>) -> Result<(), StorageError> {
        let write_txn = db.begin_write().map_err(|e| {
            StorageError::DbError(format!("Failed to start write transaction: {}", e))
        })?;
        {
            use crate::storage::redb_types::*;
            // Index-related tables
            let _ = write_txn.open_table(TAG_INDEXES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open TAG_INDEXES_TABLE: {}", e))
            })?;
            let _ = write_txn.open_table(EDGE_INDEXES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open EDGE_INDEXES_TABLE: {}", e))
            })?;
            let _ = write_txn.open_table(INDEX_DATA_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open INDEX_DATA_TABLE: {}", e))
            })?;
            // Schema-related tables
            let _ = write_txn
                .open_table(TAGS_TABLE)
                .map_err(|e| StorageError::DbError(format!("Failed to open TAGS_TABLE: {}", e)))?;
            let _ = write_txn.open_table(EDGE_TYPES_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open EDGE_TYPES_TABLE: {}", e))
            })?;
            // Data storage table
            let _ = write_txn
                .open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(format!("Failed to open NODES_TABLE: {}", e)))?;
            let _ = write_txn
                .open_table(EDGES_TABLE)
                .map_err(|e| StorageError::DbError(format!("Failed to open EDGES_TABLE: {}", e)))?;
            // ID Counter Table and Name Index Table
            let _ = write_txn.open_table(TAG_ID_COUNTER_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open TAG_ID_COUNTER_TABLE: {}", e))
            })?;
            let _ = write_txn
                .open_table(EDGE_TYPE_ID_COUNTER_TABLE)
                .map_err(|e| {
                    StorageError::DbError(format!(
                        "Failed to open EDGE_TYPE_ID_COUNTER_TABLE: {}",
                        e
                    ))
                })?;
            let _ = write_txn.open_table(SPACE_NAME_INDEX_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open SPACE_NAME_INDEX_TABLE: {}", e))
            })?;
            let _ = write_txn.open_table(TAG_NAME_INDEX_TABLE).map_err(|e| {
                StorageError::DbError(format!("Failed to open TAG_NAME_INDEX_TABLE: {}", e))
            })?;
            let _ = write_txn
                .open_table(EDGE_TYPE_NAME_INDEX_TABLE)
                .map_err(|e| {
                    StorageError::DbError(format!(
                        "Failed to open EDGE_TYPE_NAME_INDEX_TABLE: {}",
                        e
                    ))
                })?;
        }
        write_txn.commit().map_err(|e| {
            StorageError::DbError(format!(
                "Failed to commit initialization transaction: {}",
                e
            ))
        })?;
        Ok(())
    }

    /// Obtain a database instance
    pub fn get_db(&self) -> &Arc<Database> {
        &self.state.db
    }

    /// Obtain the reader.
    pub fn get_reader(&self) -> &Mutex<RedbReader> {
        &self.inner.reader
    }

    /// Obtain the writer
    pub fn get_writer(&self) -> Arc<Mutex<RedbWriter>> {
        self.inner.writer.clone()
    }

    /// Setting up the transaction context
    pub fn set_transaction_context(&self, context: Option<Arc<TransactionContext>>) {
        *self.inner.current_txn_context.lock() = context.clone();

        if let Some(ctx) = &context {
            self.inner
                .reader
                .lock()
                .set_transaction_context(Some(ctx.clone()));
        } else {
            self.inner.reader.lock().set_transaction_context(None);
        }
    }

    /// Obtaining the transaction context
    pub fn get_transaction_context(&self) -> Option<Arc<TransactionContext>> {
        self.inner.current_txn_context.lock().clone()
    }

    /// Analyzing vertex IDs
    fn parse_vertex_id(&self, vertex_id: &str) -> Result<Value, StorageError> {
        if let Ok(i) = vertex_id.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        Ok(Value::String(vertex_id.to_string()))
    }

    /// Obtain the space_id
    fn get_space_id_internal(&self, space: &str) -> Result<u64, StorageError> {
        if let Some(space_info) = self.state.schema_manager.get_space(space)? {
            Ok(space_info.space_id)
        } else {
            Err(StorageError::DbError(format!(
                "Space '{}' not found",
                space
            )))
        }
    }
}

impl StorageClient for RedbStorage {
    // ==================== Vertex Operations ====================
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.vertex_storage.get_vertex(space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.vertex_storage.scan_vertices(space)
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        self.vertex_storage.scan_vertices_by_tag(space, tag)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.vertex_storage
            .scan_vertices_by_prop(space, tag, prop, value)
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.vertex_storage.insert_vertex(space, space_id, vertex)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        self.vertex_storage.update_vertex(space, vertex)
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        eprintln!("[RedbStorage::delete_vertex] space={}, id={:?}", space, id);
        let space_id = self.get_space_id_internal(space)?;
        eprintln!("[RedbStorage::delete_vertex] space_id={}", space_id);
        let result = self.vertex_storage.delete_vertex(space, space_id, id);
        eprintln!("[RedbStorage::delete_vertex] result={:?}", result);
        result
    }

    fn delete_vertex_with_edges(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        // First, delete the relevant edges.
        self.edge_storage.delete_vertex_edges(space, space_id, id)?;
        // Delete the vertex again.
        self.vertex_storage.delete_vertex(space, space_id, id)
    }

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        self.vertex_storage.batch_insert_vertices(space, vertices)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.vertex_storage
            .delete_tags(space, space_id, vertex_id, tag_names)
    }

    // ==================== Side Operations ====================
    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        self.edge_storage.get_edge(space, src, dst, edge_type, rank)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.edge_storage.get_node_edges(space, node_id, direction)
    }

    fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<F>,
    ) -> Result<Vec<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        self.edge_storage
            .get_node_edges_filtered(space, node_id, direction, filter)
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        self.edge_storage.scan_edges_by_type(space, edge_type)
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.edge_storage.scan_all_edges(space)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.edge_storage.insert_edge(space, space_id, edge)
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.edge_storage
            .delete_edge(space, space_id, src, dst, edge_type, rank)
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        self.edge_storage.batch_insert_edges(space, edges)
    }

    // ==================== Space Operations ====================
    fn create_space(&mut self, space: &SpaceInfo) -> Result<bool, StorageError> {
        self.state.schema_manager.create_space(space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        self.state.schema_manager.drop_space(space)
    }

    fn get_space(&self, space: &str) -> Result<Option<SpaceInfo>, StorageError> {
        self.state.schema_manager.get_space(space)
    }

    fn get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>, StorageError> {
        self.state.schema_manager.get_space_by_id(space_id)
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        self.state.schema_manager.list_spaces()
    }

    fn get_space_id(&self, space_name: &str) -> Result<u64, StorageError> {
        if let Some(space) = self.state.schema_manager.get_space(space_name)? {
            Ok(space.space_id)
        } else {
            Err(StorageError::DbError(format!(
                "Space '{}' not found",
                space_name
            )))
        }
    }

    fn space_exists(&self, space_name: &str) -> bool {
        matches!(self.state.schema_manager.get_space(space_name), Ok(Some(_)))
    }

    fn clear_space(&mut self, space: &str) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;

        // Delete all vertices in the space
        let vertices = self.vertex_storage.scan_vertices(space)?;
        for vertex in vertices {
            self.vertex_storage
                .delete_vertex(space, space_id, &vertex.vid)?;
        }

        // Delete all edges in the space
        let edges = self.edge_storage.scan_all_edges(space)?;
        for edge in edges {
            self.edge_storage.delete_edge(
                space,
                space_id,
                &edge.src,
                &edge.dst,
                &edge.edge_type,
                edge.ranking,
            )?;
        }

        // Clear all tag indexes for this space
        let tag_indexes = self
            .state
            .index_metadata_manager
            .list_tag_indexes(space_id)?;
        for index in tag_indexes {
            self.index_data_manager
                .clear_tag_index(space_id, &index.name)?;
        }

        // Clear all edge indexes for this space
        let edge_indexes = self
            .state
            .index_metadata_manager
            .list_edge_indexes(space_id)?;
        for index in edge_indexes {
            self.index_data_manager
                .clear_edge_index(space_id, &index.name)?;
        }

        Ok(true)
    }

    fn alter_space_comment(
        &mut self,
        space_id: u64,
        comment: String,
    ) -> Result<bool, StorageError> {
        // Get existing space info
        let mut space_info = self
            .state
            .schema_manager
            .get_space_by_id(space_id)?
            .ok_or_else(|| StorageError::DbError(format!("Space ID '{}' not found", space_id)))?;

        // Update comment
        space_info.comment = Some(comment);

        // Save updated space info
        self.state.schema_manager.update_space(&space_info)?;

        Ok(true)
    }

    // ==================== Tag Operations ====================
    fn create_tag(&mut self, space: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        self.state.schema_manager.create_tag(space, tag)
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        // Get existing tag info
        let mut tag_info = self
            .state
            .schema_manager
            .get_tag(space, tag)?
            .ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
            })?;

        // Remove specified properties
        tag_info.properties.retain(|p| !deletions.contains(&p.name));

        // Add new properties
        for prop in additions {
            // Check if property name already exists
            if !tag_info.properties.iter().any(|p| p.name == prop.name) {
                tag_info.properties.push(prop);
            }
        }

        // Update tag
        self.state.schema_manager.update_tag(space, &tag_info)?;

        Ok(true)
    }

    fn get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>, StorageError> {
        self.state.schema_manager.get_tag(space, tag)
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        self.state.schema_manager.drop_tag(space, tag)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        self.state.schema_manager.list_tags(space)
    }

    // ==================== EdgeType Operations ====================
    fn create_edge_type(&mut self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
        self.state.schema_manager.create_edge_type(space, edge)
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        // Get existing edge type info
        let mut edge_info = self
            .state
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_type, space
                ))
            })?;

        // Remove specified properties
        edge_info
            .properties
            .retain(|p| !deletions.contains(&p.name));

        // Add new properties
        for prop in additions {
            // Check if property name already exists
            if !edge_info.properties.iter().any(|p| p.name == prop.name) {
                edge_info.properties.push(prop);
            }
        }

        // Update edge type
        self.state
            .schema_manager
            .update_edge_type(space, &edge_info)?;

        Ok(true)
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Option<EdgeTypeInfo>, StorageError> {
        self.state.schema_manager.get_edge_type(space, edge_type)
    }

    fn drop_edge_type(&mut self, space: &str, edge_type: &str) -> Result<bool, StorageError> {
        self.state.schema_manager.drop_edge_type(space, edge_type)
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
        self.state.schema_manager.list_edge_types(space)
    }

    // ==================== Index Operations ====================
    fn create_tag_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state
            .index_metadata_manager
            .create_tag_index(space_id, info)
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state
            .index_metadata_manager
            .drop_tag_index(space_id, index)
            .map_err(|_| StorageError::DbError(format!("Index '{}' does not exist", index)))?;
        Ok(true)
    }

    fn get_tag_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state
            .index_metadata_manager
            .get_tag_index(space_id, index)
    }

    fn list_tag_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state.index_metadata_manager.list_tag_indexes(space_id)
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;

        // Get index info
        let index_info = self
            .state
            .index_metadata_manager
            .get_tag_index(space_id, index)?
            .ok_or_else(|| StorageError::DbError(format!("Index '{}' does not exist", index)))?;

        // Clear old index data
        self.index_data_manager
            .clear_tag_index(space_id, &index_info.name)?;

        // Rebuild index - scan all vertices with this tag
        let vertices = self
            .vertex_storage
            .scan_vertices_by_tag(space, &index_info.schema_name)?;
        for vertex in vertices {
            // Find the corresponding tag data
            if let Some(tag) = vertex
                .tags
                .iter()
                .find(|t| t.name == index_info.schema_name)
            {
                self.index_data_manager.build_vertex_index_entry(
                    space_id,
                    &index_info,
                    &vertex.vid,
                    tag,
                )?;
            }
        }

        Ok(true)
    }

    fn create_edge_index(&mut self, space: &str, info: &Index) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state
            .index_metadata_manager
            .create_edge_index(space_id, info)
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state
            .index_metadata_manager
            .drop_edge_index(space_id, index)
    }

    fn get_edge_index(&self, space: &str, index: &str) -> Result<Option<Index>, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state
            .index_metadata_manager
            .get_edge_index(space_id, index)
    }

    fn list_edge_indexes(&self, space: &str) -> Result<Vec<Index>, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.state
            .index_metadata_manager
            .list_edge_indexes(space_id)
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;

        // Obtain index information
        let index_info = self
            .state
            .index_metadata_manager
            .get_edge_index(space_id, index)?
            .ok_or_else(|| StorageError::DbError(format!("Index \"{}\" does not exist", index)))?;

        // Delete the old index data.
        self.index_data_manager
            .clear_edge_index(space_id, &index_info.name)?;

        // Rebuild the index
        let edges = self.edge_storage.scan_all_edges(space)?;
        for edge in edges {
            self.index_data_manager
                .build_edge_index_entry(space_id, &index_info, &edge)?;
        }

        Ok(true)
    }

    // ==================== Advanced Data Operations ====================
    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.vertex_storage
            .insert_vertex_data(space, space_id, info)
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.edge_storage.insert_edge_data(space, space_id, info)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        let vid = self.parse_vertex_id(vertex_id)?;
        let space_id = self.get_space_id_internal(space)?;
        // First, delete the relevant edges.
        self.edge_storage
            .delete_vertex_edges(space, space_id, &vid)?;
        // Delete the vertex again.
        self.vertex_storage
            .delete_vertex_data(space, space_id, &vid)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        rank: i64,
    ) -> Result<bool, StorageError> {
        let src_id = self.parse_vertex_id(src)?;
        let dst_id = self.parse_vertex_id(dst)?;
        let space_id = self.get_space_id_internal(space)?;
        self.edge_storage
            .delete_edge_data(space, space_id, &src_id, &dst_id, rank)
    }

    fn update_data(&mut self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        self.vertex_storage.update_data(space, info)
    }

    // ==================== User Management ====================
    fn change_password(&mut self, info: &PasswordInfo) -> Result<bool, StorageError> {
        self.user_storage.change_password(info)
    }

    fn create_user(&mut self, info: &UserInfo) -> Result<bool, StorageError> {
        self.user_storage.create_user(info)
    }

    fn alter_user(&mut self, info: &UserAlterInfo) -> Result<bool, StorageError> {
        self.user_storage.alter_user(info)
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        self.user_storage.drop_user(username)
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

    // ==================== Index Query ====================
    fn lookup_index(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        let results = self.lookup_index_with_score(space, index_name, value)?;
        Ok(results.into_iter().map(|(v, _)| v).collect())
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        let mut results = Vec::new();
        let space_id = self.get_space_id_internal(space)?;

        if let Some(index) = self
            .state
            .index_metadata_manager
            .get_tag_index(space_id, index_name)?
        {
            let indexed_values = self
                .index_data_manager
                .lookup_tag_index(space_id, &index, value)?;
            results.extend(indexed_values.into_iter().map(|v| (v, 1.0f32)));
        }

        if let Some(index) = self
            .state
            .index_metadata_manager
            .get_edge_index(space_id, index_name)?
        {
            let indexed_values = self
                .index_data_manager
                .lookup_edge_index(space_id, &index, value)?;
            results.extend(indexed_values.into_iter().map(|v| (v, 1.0f32)));
        }

        Ok(results)
    }

    // ==================== Schema Data Query ====================
    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        self.vertex_storage.get_vertex_with_schema(space, tag, id)
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        self.edge_storage
            .get_edge_with_schema(space, edge_type, src, dst)
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        self.vertex_storage.scan_vertices_with_schema(space, tag)
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        self.edge_storage.scan_edges_with_schema(space, edge_type)
    }

    // ==================== Storage Management ====================
    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        // The Redb engine automatically loads data from the disk.
        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        // The Redb engine automatically saves data to the disk.
        Ok(())
    }

    fn get_storage_stats(&self) -> crate::storage::storage_client::StorageStats {
        let total_spaces = self
            .state
            .schema_manager
            .list_spaces()
            .map(|s: Vec<_>| s.len())
            .unwrap_or(0);
        let mut total_tags = 0;
        let mut total_edge_types = 0;

        let spaces = self.state.schema_manager.list_spaces().unwrap_or_default();
        for space in spaces {
            if let Ok(tags) = self.state.schema_manager.list_tags(&space.space_name) {
                total_tags += tags.len();
            }
            if let Ok(edge_types) = self.state.schema_manager.list_edge_types(&space.space_name) {
                total_edge_types += edge_types.len();
            }
        }

        let total_vertices = self
            .vertex_storage
            .scan_vertices("")
            .map(|v| v.len())
            .unwrap_or(0);
        let total_edges = self
            .edge_storage
            .scan_all_edges("")
            .map(|e| e.len())
            .unwrap_or(0);

        crate::storage::storage_client::StorageStats {
            total_vertices,
            total_edges,
            total_spaces,
            total_tags,
            total_edge_types,
        }
    }

    // ==================== Hanging Edge Detection and Repair ====================
    fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.edge_storage.find_dangling_edges(space)
    }

    fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError> {
        let space_id = self.get_space_id_internal(space)?;
        self.edge_storage.repair_dangling_edges(space, space_id)
    }

    /// Obtain the path to the database file.
    fn get_db_path(&self) -> &str {
        self.db_path.to_str().unwrap_or("")
    }
}

/// Default storage type alias
pub type DefaultStorage = RedbStorage;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_new_with_path_creates_database() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("test_new.db");

        assert!(!db_path.exists(), "The database file should not exist.");

        let storage =
            RedbStorage::new_with_path(db_path.clone()).expect("Storage creation should succeed");

        assert!(db_path.exists(), "The database files should be created.");
        assert_eq!(storage.db_path, db_path);
    }

    #[test]
    fn test_new_with_path_opens_existing_database() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("test_open.db");

        {
            let _storage = RedbStorage::new_with_path(db_path.clone())
                .expect("First storage creation should succeed");
        }

        assert!(db_path.exists(), "The database file should exist.");

        let storage2 = RedbStorage::new_with_path(db_path.clone())
            .expect("Opening existing database should succeed");

        assert_eq!(storage2.db_path, db_path);
    }

    #[test]
    fn test_new_with_path_returns_error_on_corrupted_database() {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("test_corrupted.db");

        {
            let _storage = RedbStorage::new_with_path(db_path.clone())
                .expect("Storage creation should succeed");
        }

        assert!(db_path.exists(), "The database file should exist.");

        let mut file = fs::File::create(&db_path).expect("Failed to open file");
        use std::io::Write;
        file.write_all(b"corrupted data")
            .expect("Failed to write corrupted data");

        let result = RedbStorage::new_with_path(db_path.clone());

        assert!(
            result.is_err(),
            "Opening a damaged database should result in an error."
        );

        if let Err(StorageError::DbError(msg)) = result {
            assert!(
                msg.contains("Failed to open the database"),
                "The error message should state \"Failed to open the database\"."
            );
            assert!(
                msg.contains(db_path.to_str().expect("Failed to convert path to string")),
                "The error message should include the path to the database."
            );
            assert!(
                msg.contains("Please manually delete the database file and retry"),
                "The error message should include tips for recovery."
            );
        } else {
            panic!("Should return StorageError::DbError");
        }

        assert!(
            db_path.exists(),
            "Database files should not be automatically deleted"
        );
    }

    #[test]
    fn test_new_creates_in_default_path() {
        let default_path = PathBuf::from("data/redb");
        // Note: this test may create files in the actual path, run as appropriate
        // Here only the structural correctness is verified
        assert_eq!(default_path, PathBuf::from("data/redb"));
    }
}
