//! Index Manager Operations
//!
//! Provides high-level index management and lookup operations.
//! This module acts as an adapter between the StorageClient API and the underlying index system.

use crate::core::types::Index;
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::index::secondary::IndexDataManager;
use crate::storage::metadata::index_manager::IndexMetadataManager;

use super::context::GraphStorageContext;

pub struct IndexManagerOps<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> IndexManagerOps<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn create_tag_index(&self, space: &str, index: &Index) -> StorageResult<bool> {
        let space_id = self
            .ctx
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?
            .space_id;
        self.ctx.index_metadata_manager.create_tag_index(space_id, index)?;
        Ok(true)
    }

    pub fn drop_tag_index(&self, space: &str, index_name: &str) -> StorageResult<bool> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        self.ctx.index_metadata_manager.drop_tag_index(space_id, index_name)
    }

    pub fn get_tag_index(&self, space: &str, index_name: &str) -> StorageResult<Option<Index>> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        self.ctx.index_metadata_manager.get_tag_index(space_id, index_name)
    }

    pub fn list_tag_indexes(&self, space: &str) -> StorageResult<Vec<Index>> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        self.ctx.index_metadata_manager.list_tag_indexes(space_id)
    }

    pub fn create_edge_index(&self, space: &str, index: &Index) -> StorageResult<bool> {
        let space_id = self
            .ctx
            .schema_manager
            .get_space(space)?
            .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?
            .space_id;
        self.ctx.index_metadata_manager.create_edge_index(space_id, index)?;
        Ok(true)
    }

    pub fn drop_edge_index(&self, space: &str, index_name: &str) -> StorageResult<bool> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        self.ctx.index_metadata_manager.drop_edge_index(space_id, index_name)
    }

    pub fn get_edge_index(&self, space: &str, index_name: &str) -> StorageResult<Option<Index>> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        self.ctx.index_metadata_manager.get_edge_index(space_id, index_name)
    }

    pub fn list_edge_indexes(&self, space: &str) -> StorageResult<Vec<Index>> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        self.ctx.index_metadata_manager.list_edge_indexes(space_id)
    }

    pub fn rebuild_tag_index(
        &self,
        space: &str,
        index_name: &str,
        vertices: &[crate::core::Vertex],
    ) -> StorageResult<bool> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        let index = self
            .ctx
            .index_metadata_manager
            .get_tag_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

        let ts = self.ctx.get_write_timestamp();
        let graph = self.ctx.graph.read();
        for vertex in vertices {
            let props: Vec<(String, Value)> = vertex
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            graph.update_vertex_indexes_mvcc(space_id, &vertex.vid, &index.name, &props, ts)?;
        }

        Ok(true)
    }

    pub fn rebuild_edge_index(
        &self,
        space: &str,
        index_name: &str,
        edges: &[crate::core::Edge],
    ) -> StorageResult<bool> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;
        let index = self
            .ctx
            .index_metadata_manager
            .get_edge_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

        let ts = self.ctx.get_write_timestamp();
        let graph = self.ctx.graph.read();
        for edge in edges {
            let props: Vec<(String, Value)> =
                edge.props.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            graph.update_edge_indexes_mvcc(space_id, &edge.src, &edge.dst, &index.name, &props, ts)?;
        }

        Ok(true)
    }

    pub fn lookup_index(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> StorageResult<Vec<Value>> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;

        let index = self
            .ctx
            .index_metadata_manager
            .get_tag_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

        let graph = self.ctx.graph.read();
        let results = graph.index_data_manager().lookup_tag_index(space_id, &index, value)?;
        Ok(results)
    }

    pub fn lookup_index_with_score(
        &self,
        space: &str,
        index_name: &str,
        value: &Value,
    ) -> StorageResult<Vec<(Value, f32)>> {
        let space_id = self.ctx.schema_manager.get_space_id(space)?;

        let index = self
            .ctx
            .index_metadata_manager
            .get_tag_index(space_id, index_name)?
            .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

        let graph = self.ctx.graph.read();
        let results = graph.index_data_manager().lookup_tag_index(space_id, &index, value)?;
        Ok(results.into_iter().map(|v| (v, 1.0)).collect())
    }
}
