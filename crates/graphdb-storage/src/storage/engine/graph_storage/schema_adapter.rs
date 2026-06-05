use crate::core::error::storage::StorageErrorKind;
use crate::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, TagInfo};
use crate::core::{StorageError, StorageResult};
use crate::storage::engine::params::CreateEdgeTypeParams;
use crate::storage::types::StoragePropertyDef;
use crate::transaction::wal::{
    AddEdgePropRedo, AddVertexPropRedo, AlterSpaceCommentRedo, ClearSpaceRedo, CreateEdgeTypeRedo,
    CreateSpaceRedo, CreateVertexTypeRedo, DeleteEdgePropRedo, DeleteEdgeTypeRedo,
    DeleteVertexPropRedo, DeleteVertexTypeRedo, DropSpaceRedo, WalOpType,
};

use super::context::GraphStorageContext;
use super::ops::{
    edge_type_storage_name, endpoint_label_id, tag_label_id, vertex_type_storage_name,
};

fn schema_properties(properties: &[PropertyDef]) -> Vec<(String, String)> {
    properties
        .iter()
        .map(|prop| (prop.name.clone(), prop.data_type.to_string()))
        .collect()
}

fn append_schema_redo<T: serde::Serialize>(
    ctx: &GraphStorageContext,
    op_type: WalOpType,
    redo: &T,
) -> StorageResult<()> {
    let timestamp = ctx.get_write_timestamp();
    ctx.append_wal_redo(op_type, timestamp, redo)
}

pub(crate) fn create_space(
    ctx: &GraphStorageContext,
    space: &mut SpaceInfo,
) -> StorageResult<bool> {
    if ctx.schema_manager().get_space(&space.space_name)?.is_some() {
        return Ok(false);
    }

    if space.space_id == 0 {
        space.space_id = ctx.schema_manager().peek_next_space_id();
    }

    append_schema_redo(
        ctx,
        WalOpType::CreateSpace,
        &CreateSpaceRedo {
            space: space.clone(),
        },
    )?;

    ctx.schema_manager().create_space(space)
}

pub(crate) fn drop_space(ctx: &GraphStorageContext, space: &str) -> StorageResult<bool> {
    let Some(space_info) = ctx.schema_manager().get_space(space)? else {
        return Ok(false);
    };
    let space_id = space_info.space_id;
    let tags = ctx.schema_manager().list_tags(space)?;
    let edge_types = ctx.schema_manager().list_edge_types(space)?;

    append_schema_redo(
        ctx,
        WalOpType::DropSpace,
        &DropSpaceRedo {
            space_name: space_info.space_name.clone(),
        },
    )?;

    for tag in tags {
        let _ = ctx.drop_vertex_type(&vertex_type_storage_name(space_id, &tag.tag_name));
    }
    for et in edge_types {
        let _ = ctx.drop_edge_type(&edge_type_storage_name(space_id, &et.edge_type_name));
    }

    ctx.schema_manager().drop_space(space)
}

pub(crate) fn get_space(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<Option<SpaceInfo>> {
    ctx.schema_manager().get_space(space)
}

pub(crate) fn get_space_by_id(
    ctx: &GraphStorageContext,
    space_id: u64,
) -> StorageResult<Option<SpaceInfo>> {
    ctx.schema_manager().get_space_by_id(space_id)
}

pub(crate) fn list_spaces(ctx: &GraphStorageContext) -> StorageResult<Vec<SpaceInfo>> {
    ctx.schema_manager().list_spaces()
}

pub(crate) fn get_space_id(ctx: &GraphStorageContext, space: &str) -> StorageResult<u64> {
    ctx.schema_manager().get_space_id(space)
}

pub(crate) fn space_exists(ctx: &GraphStorageContext, space: &str) -> bool {
    ctx.schema_manager()
        .get_space(space)
        .ok()
        .flatten()
        .is_some()
}

pub(crate) fn clear_space(ctx: &GraphStorageContext, space: &str) -> StorageResult<bool> {
    let Some(space_info) = ctx.schema_manager().get_space(space)? else {
        return Ok(false);
    };
    let space_id = space_info.space_id;
    let tags = ctx.schema_manager().list_tags(space)?;
    let edge_types = ctx.schema_manager().list_edge_types(space)?;

    append_schema_redo(
        ctx,
        WalOpType::ClearSpace,
        &ClearSpaceRedo {
            space_name: space_info.space_name.clone(),
        },
    )?;

    for tag in tags {
        let _ = ctx.drop_vertex_type(&vertex_type_storage_name(space_id, &tag.tag_name));
    }
    for et in edge_types {
        let _ = ctx.drop_edge_type(&edge_type_storage_name(space_id, &et.edge_type_name));
    }

    ctx.schema_manager().clear_space(space)
}

pub(crate) fn alter_space_comment(
    ctx: &GraphStorageContext,
    space_id: u64,
    comment: String,
) -> StorageResult<bool> {
    if ctx.schema_manager().get_space_by_id(space_id)?.is_none() {
        return Ok(false);
    }

    append_schema_redo(
        ctx,
        WalOpType::AlterSpaceComment,
        &AlterSpaceCommentRedo {
            space_id,
            comment: comment.clone(),
        },
    )?;

    ctx.schema_manager().alter_space_comment(space_id, comment)
}

pub(crate) fn create_tag(
    ctx: &GraphStorageContext,
    space: &str,
    tag: &TagInfo,
) -> StorageResult<u32> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    if ctx
        .schema_manager()
        .get_tag(space, &tag.tag_name)?
        .is_some()
    {
        return Err(StorageError::label_already_exists(tag.tag_name.clone()));
    }
    let tag_id = ctx.schema_manager().peek_next_tag_id();

    append_schema_redo(
        ctx,
        WalOpType::CreateVertexType,
        &CreateVertexTypeRedo {
            space_name: space.to_string(),
            label_id: Some(tag_id),
            label_name: tag.tag_name.clone(),
            schema: schema_properties(&tag.properties),
        },
    )?;

    let tag_id = ctx.schema_manager().create_tag(space, tag)?;

    let properties: Vec<StoragePropertyDef> = tag
        .properties
        .iter()
        .map(StoragePropertyDef::from_core)
        .collect();

    let primary_key = tag
        .properties
        .first()
        .map(|p| p.name.as_str())
        .unwrap_or("id");

    ctx.create_vertex_type_with_id(
        &vertex_type_storage_name(space_id, &tag.tag_name),
        tag_id,
        properties,
        primary_key,
    )?;

    Ok(tag_id)
}

pub(crate) fn drop_tag(
    ctx: &GraphStorageContext,
    space: &str,
    tag_name: &str,
) -> StorageResult<bool> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    if ctx.schema_manager().get_tag(space, tag_name)?.is_none() {
        return Ok(false);
    }

    append_schema_redo(
        ctx,
        WalOpType::DeleteVertexType,
        &DeleteVertexTypeRedo {
            space_name: Some(space.to_string()),
            label_name: tag_name.to_string(),
        },
    )?;

    let _ = ctx.drop_vertex_type(&vertex_type_storage_name(space_id, tag_name));

    ctx.schema_manager().drop_tag(space, tag_name)
}

pub(crate) fn get_tag(
    ctx: &GraphStorageContext,
    space: &str,
    tag_name: &str,
) -> StorageResult<Option<TagInfo>> {
    ctx.schema_manager().get_tag(space, tag_name)
}

pub(crate) fn list_tags(ctx: &GraphStorageContext, space: &str) -> StorageResult<Vec<TagInfo>> {
    ctx.schema_manager().list_tags(space)
}

pub(crate) fn alter_tag(
    ctx: &GraphStorageContext,
    space: &str,
    tag_name: &str,
    additions: Vec<PropertyDef>,
    deletions: Vec<String>,
) -> StorageResult<bool> {
    let _ = ctx.schema_manager().get_space_id(space)?;
    let tag = ctx
        .schema_manager()
        .get_tag(space, tag_name)?
        .ok_or_else(|| StorageError::label_not_found(tag_name.to_string()))?;

    if !deletions.is_empty() {
        append_schema_redo(
            ctx,
            WalOpType::DeleteVertexProp,
            &DeleteVertexPropRedo {
                label: tag.tag_id,
                prop_names: deletions.clone(),
            },
        )?;
    }

    if !additions.is_empty() {
        append_schema_redo(
            ctx,
            WalOpType::AddVertexProp,
            &AddVertexPropRedo {
                label: tag.tag_id,
                properties: schema_properties(&additions),
            },
        )?;
    }

    let result =
        ctx.schema_manager()
            .alter_tag(space, tag_name, additions.clone(), deletions.clone())?;

    if !result {
        return Ok(false);
    }

    if let Some(label_id) = tag_label_id(ctx, space, tag_name)? {
        for deletion in &deletions {
            ctx.delete_vertex_property(label_id, deletion)?;
        }
        for prop in additions {
            let storage_prop = StoragePropertyDef::from_core(&prop);
            ctx.add_vertex_property(label_id, storage_prop)?;
        }
    }

    Ok(true)
}

pub(crate) fn create_edge_type(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type: &EdgeTypeInfo,
) -> StorageResult<u32> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    if ctx
        .schema_manager()
        .get_edge_type(space, &edge_type.edge_type_name)?
        .is_some()
    {
        return Err(StorageError::label_already_exists(
            edge_type.edge_type_name.clone(),
        ));
    }
    let src_label_id =
        endpoint_label_id(ctx, space, &edge_type.src_tag_name)?.ok_or_else(|| {
            StorageError::not_found(format!("Source tag {} not found", edge_type.src_tag_name))
        })?;
    let dst_label_id =
        endpoint_label_id(ctx, space, &edge_type.dst_tag_name)?.ok_or_else(|| {
            StorageError::not_found(format!(
                "Destination tag {} not found",
                edge_type.dst_tag_name
            ))
        })?;
    let edge_type_id = ctx.schema_manager().peek_next_edge_type_id();

    append_schema_redo(
        ctx,
        WalOpType::CreateEdgeType,
        &CreateEdgeTypeRedo {
            space_name: space.to_string(),
            label_id: Some(edge_type_id),
            src_label: edge_type.src_tag_name.clone(),
            dst_label: edge_type.dst_tag_name.clone(),
            edge_label: edge_type.edge_type_name.clone(),
            schema: schema_properties(&edge_type.properties),
        },
    )?;

    let edge_type_id = ctx.schema_manager().create_edge_type(space, edge_type)?;

    let properties: Vec<StoragePropertyDef> = edge_type
        .properties
        .iter()
        .map(StoragePropertyDef::from_core)
        .collect();
    ctx.create_edge_type_with_id(
        CreateEdgeTypeParams {
            name: &edge_type_storage_name(space_id, &edge_type.edge_type_name),
            src_label: src_label_id,
            dst_label: dst_label_id,
            properties,
            oe_strategy: edge_type.oe_strategy,
            ie_strategy: edge_type.ie_strategy,
        },
        edge_type_id,
    )?;

    Ok(edge_type_id)
}

pub(crate) fn drop_edge_type(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type_name: &str,
) -> StorageResult<bool> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    let Some(edge_type) = ctx.schema_manager().get_edge_type(space, edge_type_name)? else {
        return Ok(false);
    };

    append_schema_redo(
        ctx,
        WalOpType::DeleteEdgeType,
        &DeleteEdgeTypeRedo {
            space_name: Some(space.to_string()),
            src_label: edge_type.src_tag_name.clone(),
            dst_label: edge_type.dst_tag_name.clone(),
            edge_label: edge_type.edge_type_name.clone(),
        },
    )?;

    let _ = ctx.drop_edge_type(&edge_type_storage_name(space_id, edge_type_name));

    ctx.schema_manager().drop_edge_type(space, edge_type_name)
}

pub(crate) fn get_edge_type(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type_name: &str,
) -> StorageResult<Option<EdgeTypeInfo>> {
    ctx.schema_manager().get_edge_type(space, edge_type_name)
}

pub(crate) fn list_edge_types(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<Vec<EdgeTypeInfo>> {
    ctx.schema_manager().list_edge_types(space)
}

pub(crate) fn ensure_graph_types_from_schema(ctx: &GraphStorageContext) -> StorageResult<()> {
    for space in ctx.schema_manager().list_spaces()? {
        let space_id = space.space_id;
        for tag in ctx.schema_manager().list_tags(&space.space_name)? {
            let properties: Vec<StoragePropertyDef> = tag
                .properties
                .iter()
                .map(StoragePropertyDef::from_core)
                .collect();
            let primary_key = tag
                .properties
                .first()
                .map(|p| p.name.as_str())
                .unwrap_or("id");
            let result = ctx.create_vertex_type_with_id(
                &vertex_type_storage_name(space_id, &tag.tag_name),
                tag.tag_id,
                properties,
                primary_key,
            );
            if let Err(e) = result {
                if e.kind() != StorageErrorKind::LabelAlreadyExists {
                    return Err(e);
                }
            }
        }

        for edge_type in ctx.schema_manager().list_edge_types(&space.space_name)? {
            let src_label = endpoint_label_id(ctx, &space.space_name, &edge_type.src_tag_name)?
                .ok_or_else(|| {
                    StorageError::not_found(format!(
                        "Source tag {} not found",
                        edge_type.src_tag_name
                    ))
                })?;
            let dst_label = endpoint_label_id(ctx, &space.space_name, &edge_type.dst_tag_name)?
                .ok_or_else(|| {
                    StorageError::not_found(format!(
                        "Destination tag {} not found",
                        edge_type.dst_tag_name
                    ))
                })?;
            let properties: Vec<StoragePropertyDef> = edge_type
                .properties
                .iter()
                .map(StoragePropertyDef::from_core)
                .collect();
            let result = ctx.create_edge_type_with_id(
                CreateEdgeTypeParams {
                    name: &edge_type_storage_name(space_id, &edge_type.edge_type_name),
                    src_label,
                    dst_label,
                    properties,
                    oe_strategy: edge_type.oe_strategy,
                    ie_strategy: edge_type.ie_strategy,
                },
                edge_type.edge_type_id,
            );
            if let Err(e) = result {
                if e.kind() != StorageErrorKind::LabelAlreadyExists {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn alter_edge_type(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type_name: &str,
    additions: Vec<PropertyDef>,
    deletions: Vec<String>,
) -> StorageResult<bool> {
    let _ = ctx.schema_manager().get_space_id(space)?;
    let edge_type = ctx
        .schema_manager()
        .get_edge_type(space, edge_type_name)?
        .ok_or_else(|| StorageError::label_not_found(edge_type_name.to_string()))?;
    let src_label_id =
        endpoint_label_id(ctx, space, &edge_type.src_tag_name)?.ok_or_else(|| {
            StorageError::not_found(format!("Source tag {} not found", edge_type.src_tag_name))
        })?;
    let dst_label_id =
        endpoint_label_id(ctx, space, &edge_type.dst_tag_name)?.ok_or_else(|| {
            StorageError::not_found(format!(
                "Destination tag {} not found",
                edge_type.dst_tag_name
            ))
        })?;

    if !deletions.is_empty() {
        append_schema_redo(
            ctx,
            WalOpType::DeleteEdgeProp,
            &DeleteEdgePropRedo {
                src_label: src_label_id,
                dst_label: dst_label_id,
                edge_label: edge_type.edge_type_id,
                prop_names: deletions.clone(),
            },
        )?;
    }

    if !additions.is_empty() {
        append_schema_redo(
            ctx,
            WalOpType::AddEdgeProp,
            &AddEdgePropRedo {
                src_label: src_label_id,
                dst_label: dst_label_id,
                edge_label: edge_type.edge_type_id,
                properties: schema_properties(&additions),
            },
        )?;
    }

    let result = ctx.schema_manager().alter_edge_type(
        space,
        edge_type_name,
        additions.clone(),
        deletions.clone(),
    )?;

    if !result {
        return Ok(false);
    }

    if let Some(edge_label_id) = super::ops::edge_label_id(ctx, space, edge_type_name)? {
        for deletion in &deletions {
            ctx.delete_edge_property(edge_label_id, deletion)?;
        }
        for prop in additions {
            let storage_prop = StoragePropertyDef::from_core(&prop);
            ctx.add_edge_property(edge_label_id, storage_prop)?;
        }
    }

    Ok(true)
}
