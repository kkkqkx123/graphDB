use crate::core::types::{EdgeTypeInfo, InsertEdgeInfo};
use crate::core::{Edge, EdgeDirection, StorageError, Value};
use crate::storage::index::{IndexDataManager, RedbIndexDataManager};
use crate::storage::metadata::{IndexMetadataManager, Schema, SchemaManager};
use crate::storage::operations::{EdgeReader, EdgeWriter, VertexReader};
use crate::storage::shared_state::{StorageInner, StorageSharedState};
use std::sync::Arc;

/// Side Storage Manager
///
/// Responsible for edge additions, deletions, deletions and hanging edge detection and repair.
#[derive(Clone)]
pub struct EdgeStorage {
    state: Arc<StorageSharedState>,
    inner: Arc<StorageInner>,
    index_data_manager: RedbIndexDataManager,
}

impl std::fmt::Debug for EdgeStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EdgeStorage").finish()
    }
}

impl EdgeStorage {
    /// Creating a new edge store instance
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

    /// Get Single Edge
    pub fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<Option<Edge>, StorageError> {
        self.inner
            .reader
            .lock()
            .get_edge(space, src, dst, edge_type, rank)
    }

    /// Get the edges of the node
    pub fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<Vec<Edge>, StorageError> {
        self.inner
            .reader
            .lock()
            .get_node_edges(space, node_id, direction)
            .map(|r| r.into_vec())
    }

    /// Get the edges of a node (with filtering)
    pub fn get_node_edges_filtered<F>(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
        filter: Option<F>,
    ) -> Result<Vec<Edge>, StorageError>
    where
        F: Fn(&Edge) -> bool,
    {
        self.inner
            .reader
            .lock()
            .get_node_edges_filtered(space, node_id, direction, filter)
            .map(|r| r.into_vec())
    }

    /// Scanning Edges by Type
    pub fn scan_edges_by_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<Edge>, StorageError> {
        self.inner
            .reader
            .lock()
            .scan_edges_by_type(space, edge_type)
            .map(|r| r.into_vec())
    }

    /// Scan all edges
    pub fn scan_all_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        self.inner
            .reader
            .lock()
            .scan_all_edges(space)
            .map(|r| r.into_vec())
    }

    /// insertion side
    pub fn insert_edge(&self, space: &str, space_id: u64, edge: Edge) -> Result<(), StorageError> {
        // Get current transaction ID
        let txn_id = self.get_current_txn_id();

        {
            let mut writer = self.inner.writer.lock();
            writer.insert_edge(space, edge.clone())?;
        }

        // Update Index
        let indexes = self
            .state
            .index_metadata_manager
            .list_edge_indexes(space_id)?;

        for index in indexes {
            if index.schema_name == edge.edge_type {
                let mut index_props = Vec::new();
                for field in &index.fields {
                    if let Some(value) = edge.props.get(&field.name) {
                        index_props.push((field.name.clone(), value.clone()));
                    }
                }

                if !index_props.is_empty() {
                    self.index_data_manager.update_edge_indexes(
                        space_id,
                        &edge.src,
                        &edge.dst,
                        &index.name,
                        &index_props,
                    )?;
                }
            }
        }

        // Sync to fulltext/vector index (if enabled)
        if let Some(sync_manager) = self.state.get_sync_manager() {
            sync_manager
                .on_edge_insert(txn_id, space_id, &edge)
                .map_err(|e| StorageError::DbError(format!("Failed to sync edge insert: {}", e)))?;
        }

        Ok(())
    }

    /// Remove Edge
    pub fn delete_edge(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        edge_type: &str,
        rank: i64,
    ) -> Result<(), StorageError> {
        // Get old edge to sync deletion
        let old_edge = self
            .inner
            .reader
            .lock()
            .get_edge(space, src, dst, edge_type, rank)?;

        {
            let mut writer = self.inner.writer.lock();
            writer.delete_edge(space, src, dst, edge_type, rank)?;
        }

        // Delete Index
        let indexes = self
            .state
            .index_metadata_manager
            .list_edge_indexes(space_id)?;
        let index_names: Vec<String> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == edge_type)
            .map(|idx| idx.name)
            .collect();
        self.index_data_manager
            .delete_edge_indexes(space_id, src, dst, &index_names)?;

        // Sync to fulltext/vector index (if enabled)
        if let Some(sync_manager) = self.state.get_sync_manager() {
            if let Some(edge) = old_edge {
                sync_manager
                    .on_edge_delete(
                        0, // txn_id
                        space_id,
                        &edge.src,
                        &edge.dst,
                        &edge.edge_type,
                    )
                    .map_err(|e| {
                        StorageError::DbError(format!("Failed to sync edge delete: {}", e))
                    })?;
            }
        }

        Ok(())
    }

    /// Batch insertion of edges
    pub fn batch_insert_edges(&self, space: &str, edges: Vec<Edge>) -> Result<(), StorageError> {
        let mut writer = self.inner.writer.lock();
        for edge in edges {
            writer.insert_edge(space, edge)?;
        }
        Ok(())
    }

    /// Delete all edges associated with a vertex
    pub fn delete_vertex_edges(
        &self,
        space: &str,
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<(), StorageError> {
        let edges = self.inner.reader.lock().scan_all_edges(space)?;

        for edge in edges {
            if *edge.src == *vertex_id || *edge.dst == *vertex_id {
                {
                    let mut writer = self.inner.writer.lock();
                    writer.delete_edge(
                        space,
                        &edge.src,
                        &edge.dst,
                        &edge.edge_type,
                        edge.ranking,
                    )?;
                }
                let indexes = self
                    .state
                    .index_metadata_manager
                    .list_edge_indexes(space_id)?;
                let index_names: Vec<String> = indexes
                    .into_iter()
                    .filter(|idx| idx.schema_name == edge.edge_type)
                    .map(|idx| idx.name)
                    .collect();
                self.index_data_manager.delete_edge_indexes(
                    space_id,
                    &edge.src,
                    &edge.dst,
                    &index_names,
                )?;
            }
        }
        Ok(())
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

    /// Insertion side data (advanced interface)
    pub fn insert_edge_data(
        &self,
        space: &str,
        space_id: u64,
        info: &InsertEdgeInfo,
    ) -> Result<bool, StorageError> {
        let edge_name = info.edge_name.clone();
        let src_vertex_id = info.src_vertex_id.clone();
        let dst_vertex_id = info.dst_vertex_id.clone();
        let rank = info.rank;
        let props = info.props.clone();

        let _edge_type_info = self
            .state
            .schema_manager
            .get_edge_type(space, &edge_name)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_name, space
                ))
            })?;

        // Constructing edge attribute mappings
        let mut properties = std::collections::HashMap::new();
        for (prop_name, prop_value) in &props {
            properties.insert(prop_name.clone(), prop_value.clone());
        }

        // Creating Edges
        let edge = crate::core::Edge {
            src: Box::new(src_vertex_id.clone()),
            dst: Box::new(dst_vertex_id.clone()),
            edge_type: edge_name.clone(),
            ranking: rank,
            id: 0,
            props: properties,
        };

        // insertion side
        {
            let mut writer = self.inner.writer.lock();
            writer.insert_edge(space, edge)?;
        }

        // Updating the side index
        self.index_data_manager.update_edge_indexes(
            space_id,
            &src_vertex_id,
            &dst_vertex_id,
            &edge_name,
            &props,
        )?;

        Ok(true)
    }

    /// Deletion of side data (advanced interface)
    pub fn delete_edge_data(
        &self,
        space: &str,
        space_id: u64,
        src: &Value,
        dst: &Value,
        rank: i64,
    ) -> Result<bool, StorageError> {
        // Scan to find matching edges
        let edges = self.inner.reader.lock().scan_all_edges(space)?;
        let mut deleted = false;

        for edge in edges {
            if *edge.src == *src && *edge.dst == *dst && edge.ranking == rank {
                {
                    let mut writer = self.inner.writer.lock();
                    writer.delete_edge(
                        space,
                        &edge.src,
                        &edge.dst,
                        &edge.edge_type,
                        edge.ranking,
                    )?;
                }
                let indexes = self
                    .state
                    .index_metadata_manager
                    .list_edge_indexes(space_id)?;
                let index_names: Vec<String> = indexes
                    .into_iter()
                    .filter(|idx| idx.schema_name == edge.edge_type)
                    .map(|idx| idx.name)
                    .collect();
                self.index_data_manager.delete_edge_indexes(
                    space_id,
                    &edge.src,
                    &edge.dst,
                    &index_names,
                )?;
                deleted = true;
                break;
            }
        }

        Ok(deleted)
    }

    /// Find Hanging Edge
    pub fn find_dangling_edges(&self, space: &str) -> Result<Vec<Edge>, StorageError> {
        let mut dangling_edges = Vec::new();
        let edges = self.inner.reader.lock().scan_all_edges(space)?;

        for edge in edges {
            let src_exists = self
                .inner
                .reader
                .lock()
                .get_vertex(space, &edge.src)?
                .is_some();
            let dst_exists = self
                .inner
                .reader
                .lock()
                .get_vertex(space, &edge.dst)?
                .is_some();

            if !src_exists || !dst_exists {
                dangling_edges.push(edge);
            }
        }

        Ok(dangling_edges)
    }

    /// Repair of overhanging edges
    pub fn repair_dangling_edges(&self, space: &str, space_id: u64) -> Result<usize, StorageError> {
        let dangling_edges = self.find_dangling_edges(space)?;
        let count = dangling_edges.len();

        for edge in dangling_edges {
            {
                let mut writer = self.inner.writer.lock();
                writer.delete_edge(space, &edge.src, &edge.dst, &edge.edge_type, edge.ranking)?;
            }
            let indexes = self
                .state
                .index_metadata_manager
                .list_edge_indexes(space_id)?;
            let index_names: Vec<String> = indexes
                .into_iter()
                .filter(|idx| idx.schema_name == edge.edge_type)
                .map(|idx| idx.name)
                .collect();
            self.index_data_manager.delete_edge_indexes(
                space_id,
                &edge.src,
                &edge.dst,
                &index_names,
            )?;
        }

        Ok(count)
    }

    /// Build edge schema
    pub fn build_edge_schema(&self, edge_type_info: &EdgeTypeInfo) -> Result<Schema, StorageError> {
        let mut schema = Schema::new(edge_type_info.edge_type_name.clone(), 1);
        for prop in &edge_type_info.properties {
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

    /// Get the edge with schema
    pub fn get_edge_with_schema(
        &self,
        space: &str,
        edge_type: &str,
        src: &Value,
        dst: &Value,
    ) -> Result<Option<(Schema, Vec<u8>)>, StorageError> {
        use oxicoide::encode_to_vec;

        if let Some(edge) = self
            .inner
            .reader
            .lock()
            .get_edge(space, src, dst, edge_type, 0)?
        {
            let edge_type_info = self
                .state
                .schema_manager
                .get_edge_type(space, edge_type)?
                .ok_or_else(|| {
                    StorageError::DbError(format!(
                        "Edge type '{}' not found in space '{}'",
                        edge_type, space
                    ))
                })?;
            let schema = self.build_edge_schema(&edge_type_info)?;
            let edge_data = encode_to_vec(&edge)?;
            return Ok(Some((schema, edge_data)));
        }
        Ok(None)
    }

    /// Scanning edges with schema
    pub fn scan_edges_with_schema(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<Vec<(Schema, Vec<u8>)>, StorageError> {
        use oxicoide::encode_to_vec;

        let mut results = Vec::new();
        let edge_type_info = self
            .state
            .schema_manager
            .get_edge_type(space, edge_type)?
            .ok_or_else(|| {
                StorageError::DbError(format!(
                    "Edge type '{}' not found in space '{}'",
                    edge_type, space
                ))
            })?;
        let schema = self.build_edge_schema(&edge_type_info)?;

        let edges = self
            .inner
            .reader
            .lock()
            .scan_edges_by_type(space, edge_type)?;
        for edge in edges {
            let edge_data = encode_to_vec(&edge)?;
            results.push((schema.clone(), edge_data));
        }

        Ok(results)
    }
}
