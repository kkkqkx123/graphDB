use crate::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, TagInfo};
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::EdgeStrategy;
use crate::storage::engine::edge_params::CreateEdgeTypeParams;
use crate::storage::storage_types::StoragePropertyDef;

use super::context::GraphStorageContext;

pub(crate) fn create_space(ctx: &GraphStorageContext, space: &mut SpaceInfo) -> StorageResult<bool> {
    ctx.schema_manager.create_space(space)
}

pub(crate) fn drop_space(ctx: &GraphStorageContext, space: &str) -> StorageResult<bool> {
    let tags = ctx.schema_manager.list_tags(space)?;
    let edge_types = ctx.schema_manager.list_edge_types(space)?;

    for tag in tags {
        let _ = ctx.graph.drop_vertex_type(&tag.tag_name);
    }
    for et in edge_types {
        let _ = ctx.graph.drop_edge_type(&et.edge_type_name);
    }

    ctx.schema_manager.drop_space(space)
}

pub(crate) fn get_space(ctx: &GraphStorageContext, space: &str) -> StorageResult<Option<SpaceInfo>> {
    ctx.schema_manager.get_space(space)
}

pub(crate) fn get_space_by_id(ctx: &GraphStorageContext, space_id: u64) -> StorageResult<Option<SpaceInfo>> {
    ctx.schema_manager.get_space_by_id(space_id)
}

pub(crate) fn list_spaces(ctx: &GraphStorageContext) -> StorageResult<Vec<SpaceInfo>> {
    ctx.schema_manager.list_spaces()
}

pub(crate) fn get_space_id(ctx: &GraphStorageContext, space: &str) -> StorageResult<u64> {
    ctx.schema_manager.get_space_id(space)
}

pub(crate) fn space_exists(ctx: &GraphStorageContext, space: &str) -> bool {
    ctx.schema_manager
        .get_space(space)
        .ok()
        .flatten()
        .is_some()
}

pub(crate) fn clear_space(ctx: &GraphStorageContext, space: &str) -> StorageResult<bool> {
    let tags = ctx.schema_manager.list_tags(space)?;
    let edge_types = ctx.schema_manager.list_edge_types(space)?;

    for tag in tags {
        let _ = ctx.graph.drop_vertex_type(&tag.tag_name);
    }
    for et in edge_types {
        let _ = ctx.graph.drop_edge_type(&et.edge_type_name);
    }

    ctx.schema_manager.clear_space(space)
}

pub(crate) fn alter_space_comment(ctx: &GraphStorageContext, space_id: u64, comment: String) -> StorageResult<bool> {
    ctx.schema_manager.alter_space_comment(space_id, comment)
}

pub(crate) fn create_tag(ctx: &GraphStorageContext, space: &str, tag: &TagInfo) -> StorageResult<u32> {
    let tag_id = ctx.schema_manager.create_tag(space, tag)?;

    let properties: Vec<StoragePropertyDef> =
        tag.properties.iter().map(StoragePropertyDef::from_core).collect();

    let primary_key = tag
        .properties
        .first()
        .map(|p| p.name.as_str())
        .unwrap_or("id");

    ctx.graph.create_vertex_type_with_id(
        &tag.tag_name,
        tag_id,
        properties,
        primary_key,
    )?;

    Ok(tag_id)
}

pub(crate) fn drop_tag(ctx: &GraphStorageContext, space: &str, tag_name: &str) -> StorageResult<bool> {
    let _ = ctx.graph.drop_vertex_type(tag_name);

    ctx.schema_manager.drop_tag(space, tag_name)
}

pub(crate) fn get_tag(ctx: &GraphStorageContext, space: &str, tag_name: &str) -> StorageResult<Option<TagInfo>> {
    ctx.schema_manager.get_tag(space, tag_name)
}

pub(crate) fn list_tags(ctx: &GraphStorageContext, space: &str) -> StorageResult<Vec<TagInfo>> {
    ctx.schema_manager.list_tags(space)
}

pub(crate) fn alter_tag(
    ctx: &GraphStorageContext,
    space: &str,
    tag_name: &str,
    additions: Vec<PropertyDef>,
    deletions: Vec<String>,
) -> StorageResult<bool> {
    let result = ctx.schema_manager.alter_tag(space, tag_name, additions.clone(), deletions)?;

    if !result {
        return Ok(false);
    }

    if let Some(label_id) = ctx.graph.get_vertex_label_id(tag_name) {
        for prop in additions {
            let storage_prop = StoragePropertyDef::from_core(&prop);
            ctx.graph.add_vertex_property(label_id, storage_prop)?;
        }
    }

    Ok(true)
}

pub(crate) fn create_edge_type(ctx: &GraphStorageContext, space: &str, edge_type: &EdgeTypeInfo) -> StorageResult<u32> {
    let edge_type_id = ctx.schema_manager.create_edge_type(space, edge_type)?;

    let src_label_id = if edge_type.src_tag_name.is_empty() {
        0
    } else {
        ctx.graph.get_vertex_label_id(&edge_type.src_tag_name)
            .ok_or_else(|| StorageError::not_found(format!("Source tag {} not found", edge_type.src_tag_name)))?
    };
    let dst_label_id = if edge_type.dst_tag_name.is_empty() {
        0
    } else {
        ctx.graph.get_vertex_label_id(&edge_type.dst_tag_name)
            .ok_or_else(|| StorageError::not_found(format!("Destination tag {} not found", edge_type.dst_tag_name)))?
    };

    let properties: Vec<StoragePropertyDef> =
        edge_type.properties.iter().map(StoragePropertyDef::from_core).collect();
    ctx.graph.create_edge_type_with_id(
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

pub(crate) fn drop_edge_type(ctx: &GraphStorageContext, space: &str, edge_type_name: &str) -> StorageResult<bool> {
    let _ = ctx.graph.drop_edge_type(edge_type_name);

    ctx.schema_manager.drop_edge_type(space, edge_type_name)
}

pub(crate) fn get_edge_type(ctx: &GraphStorageContext, space: &str, edge_type_name: &str) -> StorageResult<Option<EdgeTypeInfo>> {
    ctx.schema_manager.get_edge_type(space, edge_type_name)
}

pub(crate) fn list_edge_types(ctx: &GraphStorageContext, space: &str) -> StorageResult<Vec<EdgeTypeInfo>> {
    ctx.schema_manager.list_edge_types(space)
}

pub(crate) fn alter_edge_type(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type_name: &str,
    additions: Vec<PropertyDef>,
    deletions: Vec<String>,
) -> StorageResult<bool> {
    let result = ctx
        .schema_manager
        .alter_edge_type(space, edge_type_name, additions.clone(), deletions)?;

    if !result {
        return Ok(false);
    }

    if let Some(edge_label_id) = ctx.graph.get_edge_label_id(edge_type_name) {
        for prop in additions {
            let storage_prop = StoragePropertyDef::from_core(&prop);
            ctx.graph.add_edge_property(edge_label_id, storage_prop)?;
        }
    }

    Ok(true)
}
