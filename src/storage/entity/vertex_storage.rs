use crate::core::types::{InsertVertexInfo, TagInfo, UpdateInfo, UpdateOp};
use crate::core::{StorageError, Value, Vertex};
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{IndexMetadataManager, Schema, SchemaManager};
use crate::storage::operations::{VertexReader, VertexWriter};
use crate::storage::shared_state::{StorageInner, StorageSharedState};
use crate::sync::coordinator::ChangeType;
use std::sync::Arc;

/// Vertex Storage Manager
///
/// Responsible for vertex additions, deletions and tag management
#[derive(Clone)]
pub struct VertexStorage {
    state: Arc<StorageSharedState>,
    inner: Arc<StorageInner>,
    index_data_manager: RedbIndexDataManager,
}

impl std::fmt::Debug for VertexStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VertexStorage").finish()
    }
}

impl VertexStorage {
    /// Creating a New Vertex Store Instance
    pub fn new(
        state: Arc<StorageSharedState>,
        inner: Arc<StorageInner>,
        index_data_manager: RedbIndexDataManager,
    ) -> Result<Self, StorageError> {
        Ok(Self {
            state,
            inner,
            index_data_manager,
        })
    }

    /// Get space ID from space name
    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        let space_info = self
            .state
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::DbError(format!("Space '{}' not found", space)))?;
        Ok(space_info.space_id)
    }

    /// Get current transaction ID
    fn get_current_txn_id(&self) -> crate::transaction::types::TransactionId {
        // Try to get transaction ID from current transaction context
        if let Some(ctx) = self.inner.current_txn_context.lock().as_ref() {
            ctx.id
        } else {
            0 // Default transaction ID for non-transactional operations
        }
    }

    /// Detect changed properties between two vertices
    fn detect_changed_properties(
        &self,
        old_vertex: &Vertex,
        new_vertex: &Vertex,
    ) -> Vec<(String, Value)> {
        let mut changed_props = Vec::new();

        // Compare properties in tags
        for new_tag in &new_vertex.tags {
            if let Some(old_tag) = old_vertex.tags.iter().find(|t| t.name == new_tag.name) {
                // Compare properties within the same tag
                for (prop_name, new_value) in &new_tag.properties {
                    if let Some(old_value) = old_tag.properties.get(prop_name) {
                        if old_value != new_value {
                            changed_props.push((prop_name.clone(), new_value.clone()));
                        }
                    } else {
                        // New property added
                        changed_props.push((prop_name.clone(), new_value.clone()));
                    }
                }

                // Check for deleted properties
                for (prop_name, old_value) in &old_tag.properties {
                    if !new_tag.properties.contains_key(prop_name) {
                        changed_props.push((prop_name.clone(), old_value.clone()));
                    }
                }
            } else {
                // New tag added, all properties are considered changed
                for (prop_name, value) in &new_tag.properties {
                    changed_props.push((prop_name.clone(), value.clone()));
                }
            }
        }

        changed_props
    }

    /// Get a single vertex
    pub fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.inner.reader.lock().get_vertex(space, id)
    }

    /// Scan all vertices
    pub fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.inner
            .reader
            .lock()
            .scan_vertices(space)
            .map(|r| r.into_vec())
    }

    /// Scanning vertices by label
    pub fn scan_vertices_by_tag(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.inner
            .reader
            .lock()
            .scan_vertices_by_tag(space, tag)
            .map(|r| r.into_vec())
    }

    /// Scanning vertices by attribute
    pub fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.inner
            .reader
            .lock()
            .scan_vertices_by_prop(space, tag, prop, value)
            .map(|r| r.into_vec())
    }

    /// Insert vertex
    pub fn insert_vertex(
        &self,
        space: &str,
        space_id: u64,
        vertex: Vertex,
    ) -> Result<Value, StorageError> {
        // Get current transaction ID if in transaction context
        let txn_id = self.get_current_txn_id();
        let vid = vertex.vid.clone();

        let id = {
            let mut writer = self.inner.writer.lock();
            writer.insert_vertex(space, vertex.clone())?
        };

        // Update Index
        for tag in &vertex.tags {
            let indexes = self
                .state
                .index_metadata_manager
                .list_tag_indexes(space_id)?;

            for index in indexes {
                if index.schema_name == tag.name {
                    let mut index_props = Vec::new();
                    for field in &index.fields {
                        if let Some(value) = tag.properties.get(&field.name) {
                            index_props.push((field.name.clone(), value.clone()));
                        }
                    }

                    if !index_props.is_empty() {
                        self.index_data_manager.update_vertex_indexes(
                            space_id,
                            &id,
                            &index.name,
                            &index_props,
                        )?;
                    }
                }
            }

            // Sync to fulltext/vector index (if enabled)
            if let Some(sync_manager) = self.state.get_sync_manager() {
                let props: Vec<(String, Value)> = tag
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                if !props.is_empty() {
                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            &tag.name,
                            &vid,
                            &props,
                            ChangeType::Insert,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync vertex insert: {}", e))
                        })?;
                }
            }
        }

        Ok(id)
    }

    /// Update Vertex
    pub fn update_vertex(&self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let vid = vertex.vid.clone();

        // Get current transaction ID
        let txn_id = self.get_current_txn_id();

        // Get old vertex to detect changes
        let old_vertex = self.inner.reader.lock().get_vertex(space, &vid)?;

        // Update storage
        {
            let mut writer = self.inner.writer.lock();
            writer.update_vertex(space, vertex.clone())?;
        }

        // Invalidate cache
        self.inner.reader.lock().invalidate_vertex_cache(&vid);

        // Sync to fulltext/vector index (if enabled)
        if let Some(sync_manager) = self.state.get_sync_manager() {
            if let Some(old_v) = old_vertex {
                // Detect changed properties
                let changed_props = self.detect_changed_properties(&old_v, &vertex);
                if !changed_props.is_empty() {
                    // Get space_id and tag_name for sync
                    let space_id = self.get_space_id(space)?;
                    let tag_name = vertex
                        .tags
                        .first()
                        .map(|t| t.name.as_str())
                        .unwrap_or("default");

                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            tag_name,
                            &vid,
                            &changed_props,
                            ChangeType::Update,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync vertex update: {}", e))
                        })?;
                }
            }
        }

        Ok(())
    }

    /// Delete Vertex
    pub fn delete_vertex(
        &self,
        space: &str,
        space_id: u64,
        id: &Value,
    ) -> Result<(), StorageError> {
        // Get current transaction ID
        let txn_id = self.get_current_txn_id();

        // Get old vertex to sync deletion
        let old_vertex = self.inner.reader.lock().get_vertex(space, id)?;

        // Delete from storage
        {
            let mut writer = self.inner.writer.lock();
            writer.delete_vertex(space, id)?;
        }

        // Delete Index
        self.index_data_manager
            .delete_vertex_indexes(space_id, id)?;

        // Clear cache
        self.inner.reader.lock().invalidate_vertex_cache(id);

        // Sync to fulltext/vector index (if enabled)
        if let Some(sync_manager) = self.state.get_sync_manager() {
            if let Some(vertex) = old_vertex {
                // Get tag name for sync
                if let Some(tag) = vertex.tags.first() {
                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            &tag.name,
                            id,
                            &[], // No properties needed for deletion
                            ChangeType::Delete,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync vertex delete: {}", e))
                        })?;
                }
            }
        }

        Ok(())
    }

    /// Batch insertion of vertices
    pub fn batch_insert_vertices(
        &self,
        space: &str,
        space_id: u64,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let txn_id = self.get_current_txn_id();

        let ids = {
            let mut writer = self.inner.writer.lock();
            writer.batch_insert_vertices(space, vertices.clone())?
        };

        // Sync to fulltext/vector index (if enabled)
        if let Some(sync_manager) = self.state.get_sync_manager() {
            for vertex in &vertices {
                let vid = vertex.vid.clone();
                for tag in &vertex.tags {
                    let props: Vec<(String, Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    if !props.is_empty() {
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                &tag.name,
                                &vid,
                                &props,
                                ChangeType::Insert,
                            )
                            .map_err(|e| {
                                StorageError::DbError(format!(
                                    "Failed to sync vertex insert in batch: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(ids)
    }

    /// Deletes the specified label on a vertex
    pub fn delete_tags(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        let deleted_count = {
            let mut writer = self.inner.writer.lock();
            writer.delete_tags(space, vertex_id, tag_names)?
        };

        // Delete Related Indexes
        for tag_name in tag_names {
            self.index_data_manager
                .delete_tag_indexes(space_id, vertex_id, tag_name)?;
        }

        Ok(deleted_count)
    }

    /// Insert vertex data (advanced interface)
    pub fn insert_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        info: &InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        let txn_id = self.get_current_txn_id();

        // Get label information
        let tag_name = info.tag_name.clone();
        let _tag_info = self
            .state
            .schema_manager
            .get_tag(space, &tag_name)?
            .ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag_name, space))
            })?;

        // Constructing vertex attribute mappings
        let mut properties = std::collections::HashMap::new();
        for (prop_name, prop_value) in &info.props {
            properties.insert(prop_name.clone(), prop_value.clone());
        }

        // Creating Tags
        let tag = crate::core::vertex_edge_path::Tag {
            name: tag_name.clone(),
            properties,
        };

        // Getting or creating vertices
        let vertex = match self
            .inner
            .reader
            .lock()
            .get_vertex(space, &info.vertex_id)?
        {
            Some(mut existing_vertex) => {
                existing_vertex.tags.retain(|t| t.name != tag_name);
                existing_vertex.tags.push(tag);
                existing_vertex
            }
            None => crate::core::Vertex {
                vid: Box::new(info.vertex_id.clone()),
                id: 0,
                tags: vec![tag],
                properties: std::collections::HashMap::new(),
            },
        };

        // Insert vertex
        {
            let mut writer = self.inner.writer.lock();
            writer.update_vertex(space, vertex)?;
        }

        // Update Index
        self.index_data_manager.update_vertex_indexes(
            space_id,
            &info.vertex_id,
            &tag_name,
            &info.props,
        )?;

        // Sync to fulltext/vector index (if enabled)
        if let Some(sync_manager) = self.state.get_sync_manager() {
            if !info.props.is_empty() {
                sync_manager
                    .on_vertex_change_with_txn(
                        txn_id,
                        space_id,
                        &tag_name,
                        &info.vertex_id,
                        &info.props,
                        ChangeType::Insert,
                    )
                    .map_err(|e| {
                        StorageError::DbError(format!("Failed to sync vertex data insert: {}", e))
                    })?;
            }
        }

        Ok(true)
    }

    /// Delete vertex data (advanced interface)
    pub fn delete_vertex_data(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<bool, StorageError> {
        let txn_id = self.get_current_txn_id();

        // Get old vertex to sync deletion
        let old_vertex = self.inner.reader.lock().get_vertex(space, vertex_id)?;

        // Delete Vertex Index
        self.index_data_manager
            .delete_vertex_indexes(space_id, vertex_id)?;

        // Delete the vertex itself
        let mut writer = self.inner.writer.lock();
        writer.delete_vertex(space, vertex_id)?;

        // Sync to fulltext/vector index (if enabled)
        if let Some(sync_manager) = self.state.get_sync_manager() {
            if let Some(vertex) = old_vertex {
                if let Some(tag) = vertex.tags.first() {
                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            &tag.name,
                            vertex_id,
                            &[], // No properties needed for deletion
                            ChangeType::Delete,
                        )
                        .map_err(|e| {
                            StorageError::DbError(format!(
                                "Failed to sync vertex data delete: {}",
                                e
                            ))
                        })?;
                }
            }
        }

        Ok(true)
    }

    /// Updating vertex properties
    pub fn update_data(&self, space: &str, info: &UpdateInfo) -> Result<bool, StorageError> {
        self.update_vertex_property(
            space,
            &info.update_target.id,
            &info.update_target.label,
            &info.update_target.prop,
            &info.update_op,
            &info.value,
        )?;
        Ok(true)
    }

    /// Update vertex properties (internal method)
    fn update_vertex_property(
        &self,
        space: &str,
        vertex_id: &Value,
        tag: &str,
        prop: &str,
        op: &UpdateOp,
        value: &Value,
    ) -> Result<(), StorageError> {
        if let Some(mut vertex) = self.inner.reader.lock().get_vertex(space, vertex_id)? {
            for tag_data in &mut vertex.tags {
                if tag_data.name == tag {
                    match op {
                        UpdateOp::Set => {
                            tag_data.properties.insert(prop.to_string(), value.clone());
                        }
                        UpdateOp::Add => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data
                                        .properties
                                        .insert(prop.to_string(), Value::Int(a + b));
                                }
                            }
                        }
                        UpdateOp::Subtract => {
                            if let Some(existing) = tag_data.properties.get(prop) {
                                if let (Value::Int(a), Value::Int(b)) = (existing, value) {
                                    tag_data
                                        .properties
                                        .insert(prop.to_string(), Value::Int(a - b));
                                }
                            }
                        }
                        _ => {}
                    }
                    break;
                }
            }
            let mut writer = self.inner.writer.lock();
            writer.update_vertex(space, vertex)?;
        }
        Ok(())
    }

    /// Build the vertex schema
    pub fn build_vertex_schema(&self, tag_info: &TagInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(tag_info.tag_name.clone(), 1);
        for prop in &tag_info.properties {
            let field_def = crate::storage::api::types::FieldDef {
                name: prop.name.clone(),
                field_type: prop.data_type.clone(),
                nullable: prop.nullable,
                default_value: prop.default.clone(),
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            };
            schema = schema.add_field(field_def);
        }
        Ok(schema)
    }

    /// Get vertices with schema
    pub fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        use oxicode::encode_to_vec;

        if let Some(vertex) = self.inner.reader.lock().get_vertex(space, id)? {
            let tag_info = self
                .state
                .schema_manager
                .get_tag(space, tag)?
                .ok_or_else(|| {
                    StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
                })?;
            let schema = self.build_vertex_schema(&tag_info)?;
            let vertex_data = encode_to_vec(&vertex)?;
            return Ok(Some((schema, vertex_data)));
        }
        Ok(None)
    }

    /// Scanning vertices with schema
    pub fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        use oxicode::encode_to_vec;

        let mut results = Vec::new();
        let tag_info = self
            .state
            .schema_manager
            .get_tag(space, tag)?
            .ok_or_else(|| {
                StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space))
            })?;
        let schema = self.build_vertex_schema(&tag_info)?;

        let vertices = self.inner.reader.lock().scan_vertices(space)?;
        for vertex in vertices {
            if vertex.tags.iter().any(|t| t.name == tag) {
                let vertex_data = encode_to_vec(&vertex)?;
                results.push((schema.clone(), vertex_data));
            }
        }

        Ok(results)
    }
}
