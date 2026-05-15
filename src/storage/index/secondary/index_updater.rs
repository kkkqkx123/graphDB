//! index updater
//!
//! Provide index linkage update function during DML operation.
//! Includes vertex index updates, edge index updates, index deletions, etc.
//! All operations identify a space by its space_id, enabling multi-space data segregation.
//! Supports undo logging for transaction rollback.
//! Supports MVCC timestamp parameters for snapshot isolation.

use crate::core::types::Index;
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, StorageError, Value};
use super::index_data_manager::{IndexDataManager, Timestamp, MAX_TIMESTAMP};
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
        self.update_vertex_indexes_mvcc(vertex_id, tags, MAX_TIMESTAMP)
    }

    /// Update all indexes of a vertex with MVCC timestamp
    ///
    /// Called on vertex insertion or update, automatically updates all related indexes
    ///
    /// # Arguments
    /// * `vertex_id` - vertex ID
    /// * `tags` - all tags of the vertex
    /// * `write_ts` - MVCC write timestamp
    pub fn update_vertex_indexes_mvcc(
        &self,
        vertex_id: &Value,
        tags: &[Tag],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let indexes = self
            .index_metadata_manager
            .list_tag_indexes(self.space_id)?;

        for index in indexes {
            for tag in tags {
                if index.schema_name == tag.name {
                    self.update_vertex_index_for_tag_mvcc(vertex_id, tag, &index, write_ts)?;
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
        self.update_vertex_index_for_tag_mvcc(vertex_id, tag, index, MAX_TIMESTAMP)
    }

    /// Updates the index of the specified tag with MVCC timestamp
    fn update_vertex_index_for_tag_mvcc(
        &self,
        vertex_id: &Value,
        tag: &Tag,
        index: &Index,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let mut index_props: Vec<(String, Value)> = Vec::new();

        for field in &index.fields {
            if let Some(value) = tag.properties.get(&field.name) {
                index_props.push((field.name.clone(), value.clone()));
            }
        }

        if !index_props.is_empty() {
            self.index_data_manager.update_vertex_indexes_mvcc(
                self.space_id,
                vertex_id,
                &index.name,
                &index_props,
                write_ts,
            )?;
        }

        Ok(())
    }

    /// Delete all indexes of a vertex
    ///
    /// Called on vertex deletion, automatically removes all related indexes
    ///
    /// # Arguments
    /// * `vertex_id` - vertex ID
    pub fn delete_vertex_indexes(&self, vertex_id: &Value) -> Result<(), StorageError> {
        self.delete_vertex_indexes_mvcc(vertex_id, MAX_TIMESTAMP)
    }

    /// Delete all indexes of a vertex with MVCC timestamp
    ///
    /// Called on vertex deletion, automatically removes all related indexes
    ///
    /// # Arguments
    /// * `vertex_id` - vertex ID
    /// * `write_ts` - MVCC write timestamp
    pub fn delete_vertex_indexes_mvcc(
        &self,
        vertex_id: &Value,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.index_data_manager
            .delete_vertex_indexes_mvcc(self.space_id, vertex_id, write_ts)
    }

    /// Deletes the index of the specified tag
    ///
    /// Called when removing a label from a vertex
    ///
    /// # Arguments
    /// * `vertex_id` - vertex ID
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
        self.update_edge_indexes_mvcc(edge, MAX_TIMESTAMP)
    }

    /// Update all indexes of the edge with MVCC timestamp
    ///
    /// Called on edge insertion or update, automatically updates all related indexes
    ///
    /// # Arguments
    /// * `edge` - edge object
    /// * `write_ts` - MVCC write timestamp
    pub fn update_edge_indexes_mvcc(
        &self,
        edge: &Edge,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let indexes = self
            .index_metadata_manager
            .list_edge_indexes(self.space_id)?;

        for index in indexes {
            if index.schema_name == edge.edge_type {
                self.update_edge_index_mvcc(edge, &index, write_ts)?;
            }
        }

        Ok(())
    }

    /// Updates the index of the specified edge
    fn update_edge_index(&self, edge: &Edge, index: &Index) -> Result<(), StorageError> {
        self.update_edge_index_mvcc(edge, index, MAX_TIMESTAMP)
    }

    /// Updates the index of the specified edge with MVCC timestamp
    fn update_edge_index_mvcc(
        &self,
        edge: &Edge,
        index: &Index,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let mut index_props: Vec<(String, Value)> = Vec::new();

        for field in &index.fields {
            if let Some(value) = edge.props.get(&field.name) {
                index_props.push((field.name.clone(), value.clone()));
            }
        }

        if !index_props.is_empty() {
            let src_value = Value::from(edge.src);
            let dst_value = Value::from(edge.dst);
            self.index_data_manager.update_edge_indexes_mvcc(
                self.space_id,
                &src_value,
                &dst_value,
                &index.name,
                &index_props,
                write_ts,
            )?;
        }

        Ok(())
    }

    /// Delete all indexes of an edge
    ///
    /// Called on edge deletion to automatically delete all related indexes
    ///
    /// # Arguments
    /// * `edge` - edge object
    pub fn delete_edge_indexes(&self, edge: &Edge) -> Result<(), StorageError> {
        self.delete_edge_indexes_mvcc(edge, MAX_TIMESTAMP)
    }

    /// Delete all indexes of an edge with MVCC timestamp
    ///
    /// Called on edge deletion to automatically delete all related indexes
    ///
    /// # Arguments
    /// * `edge` - edge object
    /// * `write_ts` - MVCC write timestamp
    pub fn delete_edge_indexes_mvcc(
        &self,
        edge: &Edge,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        let indexes = self
            .index_metadata_manager
            .list_edge_indexes(self.space_id)?;

        let index_names: Vec<String> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == edge.edge_type)
            .map(|idx| idx.name)
            .collect();

        let src_value = Value::from(edge.src);
        let dst_value = Value::from(edge.dst);
        self.index_data_manager.delete_edge_indexes_mvcc(
            self.space_id,
            &src_value,
            &dst_value,
            &index_names,
            write_ts,
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
    /// * `edges` - edge list
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
    /// * `tag_name` - tag name
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
    /// used for index rebuild operation
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

    // ========================================================================
    // Native ID Methods (CSR-compatible)
    // ========================================================================

    /// Update vertex indexes with native VertexId
    ///
    /// This is a CSR-aware method that uses native u64 vertex IDs.
    pub fn update_vertex_indexes_native(
        &self,
        vertex_id: u64,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_vertex_indexes_native_mvcc(vertex_id, index_name, props, MAX_TIMESTAMP)
    }

    /// Update vertex indexes with native VertexId and MVCC timestamp
    pub fn update_vertex_indexes_native_mvcc(
        &self,
        vertex_id: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.index_data_manager.update_vertex_indexes_native_mvcc(
            self.space_id,
            vertex_id,
            index_name,
            props,
            write_ts,
        )
    }

    /// Delete vertex indexes with native VertexId
    pub fn delete_vertex_indexes_native(&self, vertex_id: u64) -> Result<(), StorageError> {
        self.delete_vertex_indexes_native_mvcc(vertex_id, MAX_TIMESTAMP)
    }

    /// Delete vertex indexes with native VertexId and MVCC timestamp
    pub fn delete_vertex_indexes_native_mvcc(
        &self,
        vertex_id: u64,
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.index_data_manager.delete_vertex_indexes_native_mvcc(
            self.space_id,
            vertex_id,
            write_ts,
        )
    }

    /// Update edge indexes with native VertexId
    pub fn update_edge_indexes_native(
        &self,
        src: u64,
        dst: u64,
        index_name: &str,
        props: &[(String, Value)],
    ) -> Result<(), StorageError> {
        self.update_edge_indexes_native_mvcc(src, dst, index_name, props, MAX_TIMESTAMP)
    }

    /// Update edge indexes with native VertexId and MVCC timestamp
    pub fn update_edge_indexes_native_mvcc(
        &self,
        src: u64,
        dst: u64,
        index_name: &str,
        props: &[(String, Value)],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.index_data_manager.update_edge_indexes_native_mvcc(
            self.space_id,
            src,
            dst,
            index_name,
            props,
            write_ts,
        )
    }

    /// Delete edge indexes with native VertexId
    pub fn delete_edge_indexes_native(
        &self,
        src: u64,
        dst: u64,
        index_names: &[String],
    ) -> Result<(), StorageError> {
        self.delete_edge_indexes_native_mvcc(src, dst, index_names, MAX_TIMESTAMP)
    }

    /// Delete edge indexes with native VertexId and MVCC timestamp
    pub fn delete_edge_indexes_native_mvcc(
        &self,
        src: u64,
        dst: u64,
        index_names: &[String],
        write_ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.index_data_manager.delete_edge_indexes_native_mvcc(
            self.space_id,
            src,
            dst,
            index_names,
            write_ts,
        )
    }
}

/// Index update context
///
/// Index update management for use in batch DML operations
/// Supports undo logging for transaction rollback.
pub struct IndexUpdateContext<'a, I: IndexDataManager, M: IndexMetadataManager> {
    updater: IndexUpdater<'a, I, M>,
    pending_vertex_updates: Vec<(Value, Vec<Tag>)>,
    pending_edge_updates: Vec<Edge>,
    pending_vertex_deletes: Vec<(Value, Vec<Tag>)>,
    pending_edge_deletes: Vec<Edge>,
    undo_log: IndexUndoLog,
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
            undo_log: IndexUndoLog::new(),
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
    ///
    /// # Arguments
    /// * `vertex_id` - vertex ID
    /// * `tags` - vertex tags (needed for undo logging)
    pub fn add_vertex_delete(&mut self, vertex_id: Value, tags: Vec<Tag>) {
        self.pending_vertex_deletes.push((vertex_id, tags));
    }

    /// Add edges; remove edges.
    pub fn add_edge_delete(&mut self, edge: Edge) {
        self.pending_edge_deletes.push(edge);
    }

    /// Submit all index updates.
    ///
    /// Called when a transaction is committed, to apply all index updates in batches.
    /// Records undo entries for transaction rollback support.
    pub fn commit(&mut self) -> Result<(), StorageError> {
        // Record undo entries before applying index updates
        self.record_undo_entries()?;

        // First, handle the deletion operations, and then proceed with the update operations.
        // This can prevent the problem of having to recreate the index after it has been deleted.

        // Delete the vertex index.
        if !self.pending_vertex_deletes.is_empty() {
            let vertex_ids: Vec<Value> = self.pending_vertex_deletes.iter().map(|(vid, _)| vid.clone()).collect();
            self.updater
                .batch_delete_vertex_indexes(&vertex_ids)?;
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

    /// Record undo entries for all pending index updates
    fn record_undo_entries(&mut self) -> Result<(), StorageError> {
        let space_id = self.updater.space_id();

        // Record undo entries for vertex deletes (need to re-insert indexes)
        for (vertex_id, tags) in &self.pending_vertex_deletes {
            let indexes = self
                .updater
                .index_metadata_manager
                .list_tag_indexes(space_id)?;

            for index in &indexes {
                for tag in tags {
                    if index.schema_name == tag.name {
                        for field in &index.fields {
                            if let Some(prop_value) = tag.properties.get(&field.name) {
                                self.undo_log.add(IndexUndoEntry::DeleteVertexIndex {
                                    space_id,
                                    index_name: index.name.clone(),
                                    vertex_id: vertex_id.clone(),
                                    prop_name: field.name.clone(),
                                    prop_value: prop_value.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Record undo entries for edge deletes (need to re-insert indexes)
        for edge in &self.pending_edge_deletes {
            let indexes = self
                .updater
                .index_metadata_manager
                .list_edge_indexes(space_id)?;

            for index in &indexes {
                if index.schema_name == edge.edge_type {
                    for field in &index.fields {
                        if let Some(prop_value) = edge.props.get(&field.name) {
                            self.undo_log.add(IndexUndoEntry::DeleteEdgeIndex {
                                space_id,
                                index_name: index.name.clone(),
                                src: Value::from(edge.src.clone()),
                                dst: Value::from(edge.dst.clone()),
                                prop_name: field.name.clone(),
                                prop_value: prop_value.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Record undo entries for vertex inserts (need to delete indexes on rollback)
        for (vertex_id, tags) in &self.pending_vertex_updates {
            let indexes = self
                .updater
                .index_metadata_manager
                .list_tag_indexes(space_id)?;

            for index in &indexes {
                for tag in tags {
                    if index.schema_name == tag.name {
                        for field in &index.fields {
                            if let Some(prop_value) = tag.properties.get(&field.name) {
                                self.undo_log.add(IndexUndoEntry::InsertVertexIndex {
                                    space_id,
                                    index_name: index.name.clone(),
                                    vertex_id: vertex_id.clone(),
                                    prop_name: field.name.clone(),
                                    prop_value: prop_value.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Record undo entries for edge inserts (need to delete indexes on rollback)
        for edge in &self.pending_edge_updates {
            let indexes = self
                .updater
                .index_metadata_manager
                .list_edge_indexes(space_id)?;

            for index in &indexes {
                if index.schema_name == edge.edge_type {
                    for field in &index.fields {
                        if let Some(prop_value) = edge.props.get(&field.name) {
                            self.undo_log.add(IndexUndoEntry::InsertEdgeIndex {
                                space_id,
                                index_name: index.name.clone(),
                                src: Value::from(edge.src.clone()),
                                dst: Value::from(edge.dst.clone()),
                                prop_name: field.name.clone(),
                                prop_value: prop_value.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Roll back all pending index updates.
    ///
    /// Called during transaction rollback to clear all pending updates
    /// and execute undo operations to revert index changes.
    pub fn rollback(&mut self) -> Result<(), StorageError> {
        // Execute undo operations to revert index changes
        if !self.undo_log.is_empty() {
            self.undo_log.execute_undo(self.updater.index_data_manager)?;
        }

        // Clear all pending updates
        self.pending_vertex_updates.clear();
        self.pending_edge_updates.clear();
        self.pending_vertex_deletes.clear();
        self.pending_edge_deletes.clear();

        Ok(())
    }

    /// Get the undo log for external management
    pub fn undo_log(&self) -> &IndexUndoLog {
        &self.undo_log
    }

    /// Take the undo log for external management
    pub fn take_undo_log(&mut self) -> IndexUndoLog {
        IndexUndoLog {
            entries: std::mem::take(&mut self.undo_log.entries),
        }
    }
}

/// Index undo entry for transaction rollback
#[derive(Debug, Clone)]
pub enum IndexUndoEntry {
    /// Undo vertex index insertion (delete the index entry)
    InsertVertexIndex {
        space_id: u64,
        index_name: String,
        vertex_id: Value,
        prop_name: String,
        prop_value: Value,
    },
    /// Undo vertex index deletion (re-insert the index entry)
    DeleteVertexIndex {
        space_id: u64,
        index_name: String,
        vertex_id: Value,
        prop_name: String,
        prop_value: Value,
    },
    /// Undo edge index insertion (delete the index entry)
    InsertEdgeIndex {
        space_id: u64,
        index_name: String,
        src: Value,
        dst: Value,
        prop_name: String,
        prop_value: Value,
    },
    /// Undo edge index deletion (re-insert the index entry)
    DeleteEdgeIndex {
        space_id: u64,
        index_name: String,
        src: Value,
        dst: Value,
        prop_name: String,
        prop_value: Value,
    },
}

impl IndexUndoEntry {
    pub fn insert_vertex_index(
        space_id: u64,
        index_name: String,
        vertex_id: Value,
        prop_name: String,
        prop_value: Value,
    ) -> Self {
        Self::InsertVertexIndex {
            space_id,
            index_name,
            vertex_id,
            prop_name,
            prop_value,
        }
    }

    pub fn delete_vertex_index(
        space_id: u64,
        index_name: String,
        vertex_id: Value,
        prop_name: String,
        prop_value: Value,
    ) -> Self {
        Self::DeleteVertexIndex {
            space_id,
            index_name,
            vertex_id,
            prop_name,
            prop_value,
        }
    }

    pub fn insert_edge_index(
        space_id: u64,
        index_name: String,
        src: Value,
        dst: Value,
        prop_name: String,
        prop_value: Value,
    ) -> Self {
        Self::InsertEdgeIndex {
            space_id,
            index_name,
            src,
            dst,
            prop_name,
            prop_value,
        }
    }

    pub fn delete_edge_index(
        space_id: u64,
        index_name: String,
        src: Value,
        dst: Value,
        prop_name: String,
        prop_value: Value,
    ) -> Self {
        Self::DeleteEdgeIndex {
            space_id,
            index_name,
            src,
            dst,
            prop_name,
            prop_value,
        }
    }
}

/// Index undo log manager
#[derive(Debug, Clone, Default)]
pub struct IndexUndoLog {
    entries: Vec<IndexUndoEntry>,
}

impl IndexUndoLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: IndexUndoEntry) {
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entries(&self) -> &[IndexUndoEntry] {
        &self.entries
    }

    pub fn take_entries(&mut self) -> Vec<IndexUndoEntry> {
        std::mem::take(&mut self.entries)
    }

    /// Execute undo operations in reverse order
    pub fn execute_undo<I: IndexDataManager>(&mut self, manager: &I) -> Result<(), StorageError> {
        while let Some(entry) = self.entries.pop() {
            match entry {
                IndexUndoEntry::InsertVertexIndex {
                    space_id,
                    index_name,
                    vertex_id,
                    prop_name: _,
                    prop_value,
                } => {
                    manager.delete_vertex_index_single(
                        space_id,
                        &vertex_id,
                        &index_name,
                        &prop_value,
                        MAX_TIMESTAMP,
                    )?;
                }
                IndexUndoEntry::DeleteVertexIndex {
                    space_id,
                    index_name,
                    vertex_id,
                    prop_name,
                    prop_value,
                } => {
                    manager.update_vertex_indexes(
                        space_id,
                        &vertex_id,
                        &index_name,
                        &[(prop_name, prop_value)],
                    )?;
                }
                IndexUndoEntry::InsertEdgeIndex {
                    space_id,
                    index_name,
                    src,
                    dst,
                    prop_name: _,
                    prop_value,
                } => {
                    manager.delete_edge_index_single(
                        space_id,
                        &src,
                        &dst,
                        &index_name,
                        &prop_value,
                        MAX_TIMESTAMP,
                    )?;
                }
                IndexUndoEntry::DeleteEdgeIndex {
                    space_id,
                    index_name,
                    src,
                    dst,
                    prop_name,
                    prop_value,
                } => {
                    manager.update_edge_indexes(
                        space_id,
                        &src,
                        &dst,
                        &index_name,
                        &[(prop_name, prop_value)],
                    )?;
                }
            }
        }
        Ok(())
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
            partial_condition: None,
        });

        assert_eq!(index.name, "idx_person_name");
        assert_eq!(index.schema_name, "person");
        assert_eq!(index.index_type, IndexType::TagIndex);
    }
}
