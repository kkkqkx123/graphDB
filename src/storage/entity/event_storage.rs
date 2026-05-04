//! Storage layer synchronous wrapper
//!
//! Package the Storage Client to automatically synchronize to the index during storage operations

use crate::core::{Edge, StorageError, Value, Vertex};
use crate::storage::api::StorageClient;
use crate::storage::metadata::inmemory_schema_manager::InMemorySchemaManager;
use crate::sync::coordinator::ChangeType;
use std::fmt::Debug;
use std::sync::Arc;

/// Storage layer synchronous wrapper
#[derive(Clone, Debug)]
pub struct SyncStorage<S: StorageClient + Debug> {
    inner: S,
    sync_manager: Option<Arc<crate::sync::SyncManager>>,
    enabled: bool,
}

impl<S: StorageClient> SyncStorage<S> {
    /// Detect changed properties between two vertices (static helper method)
    fn detect_changed_properties(old_vertex: &Vertex, new_vertex: &Vertex) -> Vec<(String, Value)> {
        let mut changed_props = Vec::new();

        // Compare properties in each tag
        for new_tag in &new_vertex.tags {
            if let Some(old_tag) = old_vertex.tags.iter().find(|t| t.name == new_tag.name) {
                // Compare properties within the tag
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

                // Check deleted properties
                for (prop_name, old_value) in &old_tag.properties {
                    if !new_tag.properties.contains_key(prop_name) {
                        changed_props.push((prop_name.clone(), old_value.clone()));
                    }
                }
            } else {
                // New tag, all properties are changed
                for (prop_name, value) in &new_tag.properties {
                    changed_props.push((prop_name.clone(), value.clone()));
                }
            }
        }

        changed_props
    }

    /// Get the current transaction ID from storage context
    fn get_current_txn_id(&self) -> crate::transaction::types::TransactionId {
        // Try to get transaction context from GraphStorage
        if let Some(graph_storage) = self
            .inner
            .as_any()
            .downcast_ref::<crate::storage::GraphStorage>()
        {
            if let Some(ctx) = graph_storage.get_transaction_context() {
                return ctx.id;
            }
        }
        // Default to 0 for non-transactional operations
        0
    }

    /// Create a new synchronous storage without a SyncManager
    pub fn new(storage: S) -> Self {
        Self {
            inner: storage,
            sync_manager: None,
            enabled: false,
        }
    }

    /// Create a new synchronous storage with a SyncManager
    pub fn with_sync_manager(storage: S, sync_manager: Arc<crate::sync::SyncManager>) -> Self {
        Self {
            inner: storage,
            sync_manager: Some(sync_manager),
            enabled: true,
        }
    }

    /// Enable/disable synchronization
    pub fn enable_sync(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if synchronization is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get sync manager reference
    pub fn get_sync_manager(&self) -> Option<Arc<crate::sync::SyncManager>> {
        self.sync_manager.clone()
    }

    /// Get reference to the inner storage client
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get mutable reference to the inner storage client
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S: StorageClient + 'static> StorageClient for SyncStorage<S> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_schema_manager(&self) -> Option<Arc<dyn crate::storage::metadata::SchemaManager + Send + Sync>> {
        self.inner.get_schema_manager()
    }

    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.inner.get_vertex(space, id)
    }

    fn scan_vertices(&self, space: &str) -> Result<Vec<Vertex>, StorageError> {
        self.inner.scan_vertices(space)
    }

    fn scan_vertices_by_tag(&self, space: &str, tag: &str) -> Result<Vec<Vertex>, StorageError> {
        self.inner.scan_vertices_by_tag(space, tag)
    }

    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Vertex>, StorageError> {
        self.inner.scan_vertices_by_prop(space, tag, prop, value)
    }

    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        self.inner.get_edge(space, src, dst, edge_type, rank)
    }

    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: crate::core::EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.inner.get_node_edges(space, node_id, direction)
    }

    fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: crate::core::EdgeDirection,
        filter: Option<F>,
    ) -> Result<Vec<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        self.inner
            .get_node_edges_filtered(space, node_id, direction, filter)
    }

    fn scan_edges_by_type(&self, space: &str, edge_type: &str) -> Result<Vec<Edge>, StorageError> {
        self.inner.scan_edges_by_type(space, edge_type)
    }

    fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.inner.scan_all_edges(space)
    }

    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let result = self.inner.insert_vertex(space, vertex.clone())?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;

                // Get the current transaction ID from storage context
                let txn_id = self.get_current_txn_id();

                // Process each tag separately
                for tag in &vertex.tags {
                    let tag_name = &tag.name;
                    let props: Vec<(String, crate::core::Value)> = tag
                        .properties
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();

                    if !props.is_empty() {
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                tag_name,
                                &vertex.vid,
                                &props,
                                ChangeType::Insert,
                            )
                            .map_err(|e| {
                                StorageError::DbError(format!(
                                    "Failed to sync vertex insert: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(result)
    }

    fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
        let old_vertex = self
            .inner
            .get_vertex(space, &vertex.vid)?
            .ok_or_else(|| StorageError::NodeNotFound(*vertex.vid.clone()))?;

        self.inner.update_vertex(space, vertex.clone())?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                if let Some(first_tag) = vertex.tags.first() {
                    let tag_name = &first_tag.name;

                    // Detecting changed properties
                    let changed_props = Self::detect_changed_properties(&old_vertex, &vertex);

                    if !changed_props.is_empty() {
                        sync_manager
                            .on_vertex_change_with_txn(
                                txn_id,
                                space_id,
                                tag_name,
                                &vertex.vid,
                                &changed_props,
                                ChangeType::Update,
                            )
                            .map_err(|e| {
                                StorageError::DbError(format!(
                                    "Failed to sync vertex update: {}",
                                    e
                                ))
                            })?;
                    }
                }
            }
        }

        Ok(())
    }

    fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let vertex = self
            .inner
            .get_vertex(space, id)?
            .ok_or_else(|| StorageError::NodeNotFound(id.clone()))?;

        self.inner.delete_vertex(space, id)?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                // Call SyncManager for each tag
                for tag in &vertex.tags {
                    let tag_name = &tag.name;

                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            tag_name,
                            id,
                            &[], // Delete without attributes
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

    fn delete_vertex_with_edges(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
        let vertex = self
            .inner
            .get_vertex(space, id)?
            .ok_or_else(|| StorageError::NodeNotFound(id.clone()))?;

        self.inner.delete_vertex_with_edges(space, id)?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                // Call SyncManager for each tag
                for tag in &vertex.tags {
                    let tag_name = &tag.name;

                    sync_manager
                        .on_vertex_change_with_txn(
                            txn_id,
                            space_id,
                            tag_name,
                            id,
                            &[],
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

    fn batch_insert_vertices(
        &mut self,
        space: &str,
        vertices: Vec<Vertex>,
    ) -> Result<Vec<Value>, StorageError> {
        let results = self.inner.batch_insert_vertices(space, vertices.clone())?;

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                let space_id = self.inner.get_space_id(space)?;
                let txn_id = self.get_current_txn_id();

                for vertex in &vertices {
                    for tag in &vertex.tags {
                        let tag_name = &tag.name;
                        let props: Vec<(String, crate::core::Value)> = tag
                            .properties
                            .iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();

                        if !props.is_empty() {
                            sync_manager
                                .on_vertex_change_with_txn(
                                    txn_id,
                                    space_id,
                                    tag_name,
                                    &vertex.vid,
                                    &props,
                                    ChangeType::Insert,
                                )
                                .map_err(|e| {
                                    StorageError::DbError(format!(
                                        "Failed to sync vertex insert: {}",
                                        e
                                    ))
                                })?;
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn delete_tags(
        &mut self,
        space: &str,
        vertex_id: &Value,
        tag_names: &[String],
    ) -> Result<usize, StorageError> {
        self.inner.delete_tags(space, vertex_id, tag_names)
    }

    fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
        let result = self.inner.insert_edge(space, edge.clone());

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                if let Ok(space_id) = self.inner.get_space_id(space) {
                    let txn_id = self.get_current_txn_id();

                    sync_manager
                        .on_edge_insert(txn_id, space_id, &edge)
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync edge insert: {}", e))
                        })?;
                }
            }
        }

        result
    }

    fn delete_edge(
        &mut self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        let result = self.inner.delete_edge(space, src, dst, edge_type, rank);

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                if let Ok(space_id) = self.inner.get_space_id(space) {
                    let txn_id = self.get_current_txn_id();

                    sync_manager
                        .on_edge_delete(txn_id, space_id, src, dst, edge_type)
                        .map_err(|e| {
                            StorageError::DbError(format!("Failed to sync edge delete: {}", e))
                        })?;
                }
            }
        }

        result
    }

    fn batch_insert_edges(&mut self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let result = self.inner.batch_insert_edges(space, edges.clone());

        if self.enabled {
            if let Some(sync_manager) = self.get_sync_manager() {
                if let Ok(space_id) = self.inner.get_space_id(space) {
                    let txn_id = self.get_current_txn_id();

                    for edge in &edges {
                        sync_manager
                            .on_edge_insert(txn_id, space_id, edge)
                            .map_err(|e| {
                                StorageError::DbError(format!("Failed to sync edge insert: {}", e))
                            })?;
                    }
                }
            }
        }

        result
    }

    fn create_space(
        &mut self,
        space: &mut crate::core::types::SpaceInfo,
    ) -> Result<bool, StorageError> {
        self.inner.create_space(space)
    }

    fn drop_space(&mut self, space: &str) -> Result<bool, StorageError> {
        self.inner.drop_space(space)
    }

    fn get_space(
        &self,
        space: &str,
    ) -> Result<Option<crate::core::types::SpaceInfo>, StorageError> {
        self.inner.get_space(space)
    }

    fn get_space_by_id(
        &self,
        space_id: u64,
    ) -> Result<Option<crate::core::types::SpaceInfo>, StorageError> {
        self.inner.get_space_by_id(space_id)
    }

    fn list_spaces(&self) -> Result<Vec<crate::core::types::SpaceInfo>, StorageError> {
        self.inner.list_spaces()
    }

    fn get_space_id(&self, space: &str) -> Result<u64, StorageError> {
        self.inner.get_space_id(space)
    }

    fn space_exists(&self, space: &str) -> bool {
        self.inner.space_exists(space)
    }

    fn clear_space(&mut self, space: &str) -> Result<bool, StorageError> {
        self.inner.clear_space(space)
    }

    fn alter_space_comment(
        &mut self,
        space_id: u64,
        comment: String,
    ) -> Result<bool, StorageError> {
        self.inner.alter_space_comment(space_id, comment)
    }

    fn create_tag(
        &mut self,
        space: &str,
        tag: &crate::core::types::TagInfo,
    ) -> Result<bool, StorageError> {
        self.inner.create_tag(space, tag)
    }

    fn alter_tag(
        &mut self,
        space: &str,
        tag: &str,
        additions: Vec<crate::core::types::PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.inner.alter_tag(space, tag, additions, deletions)
    }

    fn drop_tag(&mut self, space: &str, tag: &str) -> Result<bool, StorageError> {
        self.inner.drop_tag(space, tag)
    }

    fn get_tag(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Option<crate::core::types::TagInfo>, StorageError> {
        self.inner.get_tag(space, tag)
    }

    fn list_tags(&self, space: &str) -> Result<Vec<crate::core::types::TagInfo>, StorageError> {
        self.inner.list_tags(space)
    }

    fn create_edge_type(
        &mut self,
        space: &str,
        edge: &crate::core::types::EdgeTypeInfo,
    ) -> Result<bool, StorageError> {
        self.inner.create_edge_type(space, edge)
    }

    fn drop_edge_type(&mut self, space: &str, edge: &str) -> Result<bool, StorageError> {
        self.inner.drop_edge_type(space, edge)
    }

    fn get_edge_type(
        &self,
        space: &str,
        edge: &str,
    ) -> Result<Option<crate::core::types::EdgeTypeInfo>, StorageError> {
        self.inner.get_edge_type(space, edge)
    }

    fn list_edge_types(
        &self,
        space: &str,
    ) -> Result<Vec<crate::core::types::EdgeTypeInfo>, StorageError> {
        self.inner.list_edge_types(space)
    }

    fn alter_edge_type(
        &mut self,
        space: &str,
        edge_type: &str,
        additions: Vec<crate::core::types::PropertyDef>,
        deletions: Vec<String>,
    ) -> Result<bool, StorageError> {
        self.inner
            .alter_edge_type(space, edge_type, additions, deletions)
    }

    fn create_tag_index(
        &mut self,
        space: &str,
        info: &crate::core::types::Index,
    ) -> Result<bool, StorageError> {
        self.inner.create_tag_index(space, info)
    }

    fn drop_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.drop_tag_index(space, index)
    }

    fn get_tag_index(
        &self,
        space: &str,
        index: &str,
    ) -> Result<Option<crate::core::types::Index>, StorageError> {
        self.inner.get_tag_index(space, index)
    }

    fn list_tag_indexes(
        &self,
        space: &str,
    ) -> Result<Vec<crate::core::types::Index>, StorageError> {
        self.inner.list_tag_indexes(space)
    }

    fn rebuild_tag_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.rebuild_tag_index(space, index)
    }

    fn create_edge_index(
        &mut self,
        space: &str,
        info: &crate::core::types::Index,
    ) -> Result<bool, StorageError> {
        self.inner.create_edge_index(space, info)
    }

    fn drop_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.drop_edge_index(space, index)
    }

    fn get_edge_index(
        &self,
        space: &str,
        index: &str,
    ) -> Result<Option<crate::core::types::Index>, StorageError> {
        self.inner.get_edge_index(space, index)
    }

    fn list_edge_indexes(
        &self,
        space: &str,
    ) -> Result<Vec<crate::core::types::Index>, StorageError> {
        self.inner.list_edge_indexes(space)
    }

    fn rebuild_edge_index(&mut self, space: &str, index: &str) -> Result<bool, StorageError> {
        self.inner.rebuild_edge_index(space, index)
    }

    fn insert_vertex_data(
        &mut self,
        space: &str,
        info: &crate::core::types::InsertVertexInfo,
    ) -> Result<bool, StorageError> {
        self.inner.insert_vertex_data(space, info)
    }

    fn insert_edge_data(
        &mut self,
        space: &str,
        info: &crate::core::types::InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        self.inner.insert_edge_data(space, info)
    }

    fn delete_vertex_data(&mut self, space: &str, vertex_id: &str) -> Result<bool, StorageError> {
        self.inner.delete_vertex_data(space, vertex_id)
    }

    fn delete_edge_data(
        &mut self,
        space: &str,
        src: &str,
        dst: &str,
        rank: i64,
    ) -> Result<bool, StorageError> {
        self.inner.delete_edge_data(space, src, dst, rank)
    }

    fn update_data(
        &mut self,
        space: &str,
        info: &crate::core::types::UpdateInfo,
    ) -> Result<bool, StorageError> {
        self.inner.update_data(space, info)
    }

    fn change_password(
        &mut self,
        info: &crate::core::types::PasswordInfo,
    ) -> Result<bool, StorageError> {
        self.inner.change_password(info)
    }

    fn create_user(&mut self, info: &crate::core::types::UserInfo) -> Result<bool, StorageError> {
        self.inner.create_user(info)
    }

    fn alter_user(
        &mut self,
        info: &crate::core::types::UserAlterInfo,
    ) -> Result<bool, StorageError> {
        self.inner.alter_user(info)
    }

    fn drop_user(&mut self, username: &str) -> Result<bool, StorageError> {
        self.inner.drop_user(username)
    }

    fn grant_role(
        &mut self,
        username: &str,
        space_id: u64,
        role: crate::core::RoleType,
    ) -> Result<bool, StorageError> {
        self.inner.grant_role(username, space_id, role)
    }

    fn revoke_role(&mut self, username: &str, space_id: u64) -> Result<bool, StorageError> {
        self.inner.revoke_role(username, space_id)
    }

    fn lookup_index(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<Value>, StorageError> {
        self.inner.lookup_index(space, index, value)
    }

    fn lookup_index_with_score(
        &self,
        space: &str,
        index: &str,
        value: &Value,
    ) -> Result<Vec<(Value, f32)>, StorageError> {
        self.inner.lookup_index_with_score(space, index, value)
    }

    fn get_vertex_with_schema(
        &self,
        space: &str,
        tag: &str,
        id: &Value,
    ) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.get_vertex_with_schema(space, tag, id)
    }

    fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.get_edge_with_schema(space, edge_type, src, dst)
    }

    fn scan_vertices_with_schema(
        &self,
        space: &str,
        tag: &str,
    ) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.scan_vertices_with_schema(space, tag)
    }

    fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, StorageError> {
        self.inner.scan_edges_with_schema(space, edge_type)
    }

    fn load_from_disk(&mut self) -> Result<(), StorageError> {
        self.inner.load_from_disk()
    }

    fn save_to_disk(&self) -> Result<(), StorageError> {
        self.inner.save_to_disk()
    }

    fn get_storage_stats(&self) -> crate::storage::StorageStats {
        self.inner.get_storage_stats()
    }

    fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.inner.find_dangling_edges(space)
    }

    fn repair_dangling_edges(&mut self, space: &str) -> Result<usize, StorageError> {
        self.inner.repair_dangling_edges(space)
    }

    fn get_db_path(&self) -> &str {
        self.inner.get_db_path()
    }

    fn get_sync_manager(&self) -> Option<std::sync::Arc<crate::sync::SyncManager>> {
        self.sync_manager.clone()
    }
}
