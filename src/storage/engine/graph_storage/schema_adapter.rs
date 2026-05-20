//! Schema Adapter Operations
//!
//! Provides high-level schema management operations for spaces, tags, and edge types.
//! This module acts as an adapter between the StorageClient API and the underlying schema system.

use crate::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, TagInfo};
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::EdgeStrategy;
use crate::storage::engine::edge_params::CreateEdgeTypeParams;
use crate::storage::storage_types::StoragePropertyDef;

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

        for tag in tags {
            let _ = self.ctx.graph.drop_vertex_type(&tag.tag_name);
        }
        for et in edge_types {
            let _ = self.ctx.graph.drop_edge_type(&et.edge_type_name);
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
        self.ctx
            .schema_manager
            .get_space(space)
            .ok()
            .flatten()
            .is_some()
    }

    pub fn clear_space(&self, space: &str) -> StorageResult<bool> {
        let tags = self.ctx.schema_manager.list_tags(space)?;
        let edge_types = self.ctx.schema_manager.list_edge_types(space)?;

        for tag in tags {
            let _ = self.ctx.graph.drop_vertex_type(&tag.tag_name);
        }
        for et in edge_types {
            let _ = self.ctx.graph.drop_edge_type(&et.edge_type_name);
        }

        self.ctx.schema_manager.clear_space(space)
    }

    pub fn alter_space_comment(&self, space_id: u64, comment: String) -> StorageResult<bool> {
        self.ctx
            .schema_manager
            .alter_space_comment(space_id, comment)
    }

    pub fn create_tag(&self, space: &str, tag: &TagInfo) -> StorageResult<u32> {
        let tag_id = self.ctx.schema_manager.create_tag(space, tag)?;

        let properties: Vec<StoragePropertyDef> =
            tag.properties.iter().map(StoragePropertyDef::from_core).collect();

        let primary_key = tag
            .properties
            .first()
            .map(|p| p.name.as_str())
            .unwrap_or("id");

        self.ctx.graph.create_vertex_type_with_id(
            &tag.tag_name,
            tag_id,
            properties,
            primary_key,
        )?;

        Ok(tag_id)
    }

    pub fn drop_tag(&self, space: &str, tag_name: &str) -> StorageResult<bool> {
        let _ = self.ctx.graph.drop_vertex_type(tag_name);

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
            .alter_tag(space, tag_name, additions.clone(), deletions)?;

        if let Some(label_id) = self.ctx.graph.get_vertex_label_id(tag_name) {
            for prop in additions {
                let storage_prop = crate::storage::storage_types::StoragePropertyDef::from_core(&prop);
                self.ctx.graph.add_vertex_property(label_id, storage_prop)?;
            }
        }

        Ok(true)
    }

    pub fn create_edge_type(&self, space: &str, edge_type: &EdgeTypeInfo) -> StorageResult<u32> {
        let edge_type_id = self.ctx.schema_manager.create_edge_type(space, edge_type)?;

        let src_label_id = self
            .ctx
            .graph
            .get_vertex_label_id(&edge_type.src_tag_name)
            .ok_or_else(|| {
                StorageError::not_found(format!("Source tag {} not found", edge_type.src_tag_name))
            })?;
        let dst_label_id = self
            .ctx
            .graph
            .get_vertex_label_id(&edge_type.dst_tag_name)
            .ok_or_else(|| {
                StorageError::not_found(format!(
                    "Destination tag {} not found",
                    edge_type.dst_tag_name
                ))
            })?;

        let properties: Vec<StoragePropertyDef> =
            edge_type.properties.iter().map(StoragePropertyDef::from_core).collect();
        self.ctx.graph.create_edge_type_with_id(
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
        let _ = self.ctx.graph.drop_edge_type(edge_type_name);

        self.ctx
            .schema_manager
            .drop_edge_type(space, edge_type_name)
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
            .alter_edge_type(space, edge_type_name, additions.clone(), deletions)?;

        if let Some(edge_label_id) = self.ctx.graph.get_edge_label_id(edge_type_name) {
            for prop in additions {
                let storage_prop = StoragePropertyDef::from_core(&prop);
                self.ctx.graph.add_edge_property(edge_label_id, storage_prop)?;
            }
        }

        Ok(true)
    }
}
