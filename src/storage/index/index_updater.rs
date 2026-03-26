//! index updater
//!
//! Provide index linkage update function during DML operation.
//! Includes vertex index updates, edge index updates, index deletions, etc.
//! All operations identify a space by its space_id, enabling multi-space data segregation.

use crate::core::types::Index;
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, StorageError, Value};
use crate::storage::index::IndexDataManager;
use crate::storage::metadata::IndexMetadataManager;

/// index updater
///
/// Responsible for automatically maintaining index consistency during DML operations
/// All operations identify a space by its space_id, enabling multi-space data segregation.
pub struct IndexUpdater<'a, I: IndexDataManager, M: IndexMetadataManager> {
    index_data_manager: &'a I,
    index_metadata_manager: &'a M,
    space_name: String,
    /// Spatial ID for multi-spatial data isolation
    space_id: u64,
}

impl<'a, I: IndexDataManager, M: IndexMetadataManager> IndexUpdater<'a, I, M> {
    /// Creating a new index updater
    pub fn new(
        index_data_manager: &'a I,
        index_metadata_manager: &'a M,
        space_name: String,
        space_id: u64,
    ) -> Self {
        Self {
            index_data_manager,
            index_metadata_manager,
            space_name,
            space_id,
        }
    }

    /// Get space name
    pub fn space_name(&self) -> &str {
        &self.space_name
    }

    /// Get Space ID
    pub fn space_id(&self) -> u64 {
        self.space_id
    }

    /// Update all indexes of a vertex
    ///
    /// Called on vertex insertion or update, automatically updates all related indexes
    ///
    /// # Arguments
    /// * `vertex_id` - vertex ID
    /// * `tags` - all tags of the vertex
    pub fn update_vertex_indexes(
        &self,
        vertex_id: &Value,
        tags: &[Tag],
    ) -> Result<(), StorageError> {
        // Get the index of all tags in the space
        let indexes = self
            .index_metadata_manager
            .list_tag_indexes(self.space_id)?;

        for index in indexes {
            // Check if the index is associated with a label of the current vertex
            for tag in tags {
                if index.schema_name == tag.name {
                    self.update_vertex_index_for_tag(vertex_id, tag, &index)?;
                }
            }
        }

        Ok(())
    }

    /// Updates the index of the specified tag
    fn update_vertex_index_for_tag(
        &self,
        vertex_id: &Value,
        tag: &Tag,
        index: &Index,
    ) -> Result<(), StorageError> {
        // Collecting index field values
        let mut index_props: Vec<(String, Value)> = Vec::new();

        for field in &index.fields {
            if let Some(value) = tag.properties.get(&field.name) {
                index_props.push((field.name.clone(), value.clone()));
            }
        }

        // If all index fields have values, update the index
        if !index_props.is_empty() {
            self.index_data_manager.update_vertex_indexes(
                self.space_id,
                vertex_id,
                &index.name,
                &index_props,
            )?;
        }

        Ok(())
    }

    /// Delete all indexes of a vertex
    ///
    /// Called on vertex deletion, automatically removes all related indexes
    ///
    /// # Arguments
    /// * `vertex_id` - 顶点ID
    pub fn delete_vertex_indexes(&self, vertex_id: &Value) -> Result<(), StorageError> {
        self.index_data_manager
            .delete_vertex_indexes(self.space_id, vertex_id)
    }

    /// Deletes the index of the specified tag
    ///
    /// Called when removing a label from a vertex
    ///
    /// # Arguments
    /// * `vertex_id` - 顶点ID
    /// * `tag_name` – Name of the tag
    pub fn delete_tag_indexes(
        &self,
        vertex_id: &Value,
        tag_name: &str,
    ) -> Result<(), StorageError> {
        self.index_data_manager
            .delete_tag_indexes(self.space_id, vertex_id, tag_name)
    }

    /// Update all indexes of the side
    ///
    /// Called on edge insertion or update, automatically updates all related indexes
    ///
    /// # Arguments
    /// * :: `edge` -- edge object
    pub fn update_edge_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        // Get all edge indices of this space
        let indexes = self
            .index_metadata_manager
            .list_edge_indexes(self.space_id)?;

        for index in indexes {
            // Check if the index is associated with the current edge type
            if index.schema_name == edge.edge_type {
                self.update_edge_index(edge, &index)?;
            }
        }

        Ok(())
    }

    /// Updates the index of the specified edge
    fn update_edge_index(&self, edge: &Edge, index: &Index) -> Result<(), StorageError> {
        // Collecting index field values
        let mut index_props: Vec<(String, Value)> = Vec::new();

        for field in &index.fields {
            if let Some(value) = edge.props.get(&field.name) {
                index_props.push((field.name.clone(), value.clone()));
            }
        }

        // If all index fields have values, update the index
        if !index_props.is_empty() {
            self.index_data_manager.update_edge_indexes(
                self.space_id,
                &edge.src,
                &edge.dst,
                &index.name,
                &index_props,
            )?;
        }

        Ok(())
    }

    /// Delete all indexes of an edge
    ///
    /// Called on edge deletion to automatically delete all related indexes
    ///
    /// # Arguments
    /// * `edge` - 边对象
    pub fn delete_edge_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        let indexes = self
            .index_metadata_manager
            .list_edge_indexes(self.space_id)?;

        let index_names: Vec<String> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == edge.edge_type)
            .map(|idx| idx.name)
            .collect();

        self.index_data_manager.delete_edge_indexes(
            self.space_id,
            &edge.src,
            &edge.dst,
            &index_names,
        )
    }

    /// Batch update vertex index
    ///
    /// For efficient index updates when inserting vertices in batches
    ///
    /// # Arguments
    /// * `vertices` - list of vertices
    pub fn batch_update_vertex_indexes(
        &self,
        vertices: &[(Value, Vec<Tag>)],
    ) -> Result<(), StorageError> {
        for (vertex_id, tags) in vertices {
            self.update_vertex_indexes(vertex_id, tags)?;
        }
        Ok(())
    }

    /// Batch update side indexes
    ///
    /// For efficient index updates when inserting edges in batches
    ///
    /// # Arguments
    /// * :: `edges` - list of edges
    pub fn batch_update_edge_indexes(&self, edges: &[Edge]) -> Result<(), StorageError> {
        for edge in edges {
            self.update_edge_indexes(edge)?;
        }
        Ok(())
    }

    /// Batch Delete Vertex Index
    ///
    /// Efficient Index Deletion for Batch Deletion of Vertices
    ///
    /// # Arguments
    /// * `vertex_ids` - list of vertex IDs
    pub fn batch_delete_vertex_indexes(&self, vertex_ids: &[Value]) -> Result<(), StorageError> {
        for vertex_id in vertex_ids {
            self.delete_vertex_indexes(vertex_id)?;
        }
        Ok(())
    }

    /// Batch Delete Side Indexes
    ///
    /// Efficient index deletion for batch deletion of edges
    ///
    /// # Arguments
    /// * `edges` - 边列表
    pub fn batch_delete_edge_indexes(&self, edges: &[Edge]) -> Result<(), StorageError> {
        for edge in edges {
            self.delete_edge_indexes(edge)?;
        }
        Ok(())
    }

    /// Rebuild all indexes for the specified tags.
    ///
    /// Used for index reconstruction operations
    ///
    /// # Arguments
    /// * `tag_name` - 标签名称
    /// * `vertices` – All the vertices associated with this tag.
    pub fn rebuild_tag_indexes(
        &self,
        tag_name: &str,
        vertices: &[(Value, Tag)],
    ) -> Result<(), StorageError> {
        // Retrieve all indexes for that tag.
        let indexes: Vec<Index> = self
            .index_metadata_manager
            .list_tag_indexes(self.space_id)?
            .into_iter()
            .filter(|idx| idx.schema_name == tag_name)
            .collect();

        // Rebuild the index for each vertex.
        for (vertex_id, tag) in vertices {
            for index in &indexes {
                self.update_vertex_index_for_tag(vertex_id, tag, index)?;
            }
        }

        Ok(())
    }

    /// Rebuild all indexes for the specified edge type.
    ///
    /// 用于索引重建操作
    ///
    /// # Arguments
    /// * `edge_type` – Type of the edge
    /// * `edges` – All the edges of this type.
    pub fn rebuild_edge_indexes(
        &self,
        edge_type: &str,
        edges: &[Edge],
    ) -> Result<(), StorageError> {
        // Obtain all indexes for this edge type.
        let indexes: Vec<Index> = self
            .index_metadata_manager
            .list_edge_indexes(self.space_id)?
            .into_iter()
            .filter(|idx| idx.schema_name == edge_type)
            .collect();

        // Rebuild the index for each edge.
        for edge in edges {
            for index in &indexes {
                self.update_edge_index(edge, index)?;
            }
        }

        Ok(())
    }
}

/// Index update context
///
/// Index update management for use in batch DML operations
pub struct IndexUpdateContext<'a, I: IndexDataManager, M: IndexMetadataManager> {
    updater: IndexUpdater<'a, I, M>,
    pending_vertex_updates: Vec<(Value, Vec<Tag>)>,
    pending_edge_updates: Vec<Edge>,
    pending_vertex_deletes: Vec<Value>,
    pending_edge_deletes: Vec<Edge>,
}

impl<'a, I: IndexDataManager, M: IndexMetadataManager> IndexUpdateContext<'a, I, M> {
    /// Create a new index to update the context.
    pub fn new(
        index_data_manager: &'a I,
        index_metadata_manager: &'a M,
        space_name: String,
        space_id: u64,
    ) -> Self {
        Self {
            updater: IndexUpdater::new(
                index_data_manager,
                index_metadata_manager,
                space_name,
                space_id,
            ),
            pending_vertex_updates: Vec::new(),
            pending_edge_updates: Vec::new(),
            pending_vertex_deletes: Vec::new(),
            pending_edge_deletes: Vec::new(),
        }
    }

    /// Add vertex updates
    pub fn add_vertex_update(&mut self, vertex_id: Value, tags: Vec<Tag>) {
        self.pending_vertex_updates.push((vertex_id, tags));
    }

    /// Add edges and update accordingly.
    pub fn add_edge_update(&mut self, edge: Edge) {
        self.pending_edge_updates.push(edge);
    }

    /// Add a vertex and delete it.
    pub fn add_vertex_delete(&mut self, vertex_id: Value) {
        self.pending_vertex_deletes.push(vertex_id);
    }

    /// Add edges; remove edges.
    pub fn add_edge_delete(&mut self, edge: Edge) {
        self.pending_edge_deletes.push(edge);
    }

    /// Submit all index updates.
    ///
    /// Called when a transaction is committed, to apply all index updates in batches.
    pub fn commit(&mut self) -> Result<(), StorageError> {
        // First, handle the deletion operations, and then proceed with the update operations.
        // This can prevent the problem of having to recreate the index after it has been deleted.

        // Delete the vertex index.
        if !self.pending_vertex_deletes.is_empty() {
            self.updater
                .batch_delete_vertex_indexes(&self.pending_vertex_deletes)?;
            self.pending_vertex_deletes.clear();
        }

        // Delete the edge index.
        if !self.pending_edge_deletes.is_empty() {
            self.updater
                .batch_delete_edge_indexes(&self.pending_edge_deletes)?;
            self.pending_edge_deletes.clear();
        }

        // Update the vertex index
        if !self.pending_vertex_updates.is_empty() {
            self.updater
                .batch_update_vertex_indexes(&self.pending_vertex_updates)?;
            self.pending_vertex_updates.clear();
        }

        // Update the edge index.
        if !self.pending_edge_updates.is_empty() {
            self.updater
                .batch_update_edge_indexes(&self.pending_edge_updates)?;
            self.pending_edge_updates.clear();
        }

        Ok(())
    }

    /// Roll back all pending index updates.
    ///
    /// Called during transaction rollback to clear all pending updates.
    pub fn rollback(&mut self) {
        self.pending_vertex_updates.clear();
        self.pending_edge_updates.clear();
        self.pending_vertex_deletes.clear();
        self.pending_edge_deletes.clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::types::{Index, IndexConfig, IndexField, IndexType};
    use crate::core::Value;

    #[test]
    fn test_index_field_creation() {
        let field = IndexField::new("name".to_string(), Value::String("test".to_string()), false);

        assert_eq!(field.name, "name");
        assert!(!field.is_nullable);
    }

    #[test]
    fn test_index_creation() {
        let index = Index::new(IndexConfig {
            id: 1,
            name: "idx_person_name".to_string(),
            space_id: 1,
            schema_name: "person".to_string(),
            fields: vec![IndexField::new(
                "name".to_string(),
                Value::String("".to_string()),
                false,
            )],
            properties: vec![],
            index_type: IndexType::TagIndex,
            is_unique: false,
        });

        assert_eq!(index.name, "idx_person_name");
        assert_eq!(index.schema_name, "person");
        assert_eq!(index.index_type, IndexType::TagIndex);
    }
}
