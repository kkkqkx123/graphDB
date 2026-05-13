//! Schema Adapter Operations
//!
//! Provides high-level schema management operations for spaces, tags, and edge types.
//! This module acts as an adapter between the StorageClient API and the underlying schema system.

use crate::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, TagInfo};
use crate::core::{StorageError, StorageResult};

use super::context::GraphStorageContext;

pub struct SchemaAdapterOps<'a> {
    ctx: &'a GraphStorageContext,
}

impl<'a> SchemaAdapterOps<'a> {
    pub fn new(ctx: &'a GraphStorageContext) -> Self {
        Self { ctx }
    }

    pub fn create_space(&self, space: &mut SpaceInfo) -> StorageResult<bool> {
        self.ctx.schema_manager.create_space(space)
    }

    pub fn drop_space(&self, space: &str) -> StorageResult<bool> {
        let tags = self.ctx.schema_manager.list_tags(space)?;
        let edge_types = self.ctx.schema_manager.list_edge_types(space)?;

        let mut graph = self.ctx.graph.write();
        for tag in tags {
            let _ = graph.drop_vertex_type(&tag.tag_name);
        }
        for et in edge_types {
            let _ = graph.drop_edge_type(&et.edge_type_name);
        }

        self.ctx.schema_manager.drop_space(space)
    }

    pub fn get_space(&self, space: &str) -> StorageResult<Option<SpaceInfo>> {
        self.ctx.schema_manager.get_space(space)
    }

    pub fn get_space_by_id(&self, space_id: u64) -> StorageResult<Option<SpaceInfo>> {
        self.ctx.schema_manager.get_space_by_id(space_id)
    }

    pub fn list_spaces(&self) -> StorageResult<Vec<SpaceInfo>> {
        self.ctx.schema_manager.list_spaces()
    }

    pub fn get_space_id(&self, space: &str) -> StorageResult<u64> {
        self.ctx.schema_manager.get_space_id(space)
    }

    pub fn space_exists(&self, space: &str) -> bool {
        self.ctx.schema_manager.get_space(space).ok().flatten().is_some()
    }

    pub fn clear_space(&self, space: &str) -> StorageResult<bool> {
        let tags = self.ctx.schema_manager.list_tags(space)?;
        let edge_types = self.ctx.schema_manager.list_edge_types(space)?;

        {
            let mut graph = self.ctx.graph.write();
            for tag in tags {
                let _ = graph.drop_vertex_type(&tag.tag_name);
            }
            for et in edge_types {
                let _ = graph.drop_edge_type(&et.edge_type_name);
            }
        }

        self.ctx.schema_manager.clear_space(space)
    }

    pub fn alter_space_comment(&self, space_id: u64, comment: String) -> StorageResult<bool> {
        self.ctx.schema_manager.alter_space_comment(space_id, comment)
    }

    pub fn create_tag(&self, space: &str, tag: &TagInfo) -> StorageResult<u32> {
        let tag_id = self.ctx.schema_manager.create_tag(space, tag)?;

        let properties: Vec<crate::storage::vertex::PropertyDef> =
            tag.properties.iter().map(|p| p.into()).collect();

        let primary_key = tag.properties.first().map(|p| p.name.as_str()).unwrap_or("id");

        let mut graph = self.ctx.graph.write();
        graph.create_vertex_type_with_id(&tag.tag_name, tag_id, properties, primary_key)?;

        Ok(tag_id)
    }

    pub fn drop_tag(&self, space: &str, tag_name: &str) -> StorageResult<bool> {
        {
            let mut graph = self.ctx.graph.write();
            let _ = graph.drop_vertex_type(tag_name);
        }

        self.ctx.schema_manager.drop_tag(space, tag_name)
    }

    pub fn get_tag(&self, space: &str, tag_name: &str) -> StorageResult<Option<TagInfo>> {
        self.ctx.schema_manager.get_tag(space, tag_name)
    }

    pub fn list_tags(&self, space: &str) -> StorageResult<Vec<TagInfo>> {
        self.ctx.schema_manager.list_tags(space)
    }

    pub fn alter_tag(
        &self,
        space: &str,
        tag_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> StorageResult<bool> {
        self.ctx
            .schema_manager
            .alter_tag(space, tag_name, additions, deletions)
    }

    pub fn create_edge_type(&self, space: &str, edge_type: &EdgeTypeInfo) -> StorageResult<u32> {
        let edge_type_id = self.ctx.schema_manager.create_edge_type(space, edge_type)?;

        let mut graph = self.ctx.graph.write();

        let src_label_id = graph
            .get_vertex_label_id(&edge_type.src_tag_name)
            .ok_or_else(|| {
                StorageError::not_found(format!("Source tag {} not found", edge_type.src_tag_name))
            })?;
        let dst_label_id = graph
            .get_vertex_label_id(&edge_type.dst_tag_name)
            .ok_or_else(|| {
                StorageError::not_found(format!(
                    "Destination tag {} not found",
                    edge_type.dst_tag_name
                ))
            })?;

        let properties: Vec<crate::storage::edge::PropertyDef> =
            edge_type.properties.iter().map(|p| p.into()).collect();

        use crate::storage::edge::EdgeStrategy;
        use crate::storage::engine::edge::CreateEdgeTypeParams;
        graph.create_edge_type_with_id(
            CreateEdgeTypeParams {
                name: &edge_type.edge_type_name,
                src_label: src_label_id,
                dst_label: dst_label_id,
                properties,
                oe_strategy: EdgeStrategy::Multiple,
                ie_strategy: EdgeStrategy::Multiple,
            },
            edge_type_id,
        )?;

        Ok(edge_type_id)
    }

    pub fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> StorageResult<bool> {
        {
            let mut graph = self.ctx.graph.write();
            let _ = graph.drop_edge_type(edge_type_name);
        }

        self.ctx.schema_manager.drop_edge_type(space, edge_type_name)
    }

    pub fn get_edge_type(
        &self,
        space: &str,
        edge_type_name: &str,
    ) -> StorageResult<Option<EdgeTypeInfo>> {
        self.ctx.schema_manager.get_edge_type(space, edge_type_name)
    }

    pub fn list_edge_types(&self, space: &str) -> StorageResult<Vec<EdgeTypeInfo>> {
        self.ctx.schema_manager.list_edge_types(space)
    }

    pub fn alter_edge_type(
        &self,
        space: &str,
        edge_type_name: &str,
        additions: Vec<PropertyDef>,
        deletions: Vec<String>,
    ) -> StorageResult<bool> {
        self.ctx
            .schema_manager
            .alter_edge_type(space, edge_type_name, additions, deletions)
    }
}
