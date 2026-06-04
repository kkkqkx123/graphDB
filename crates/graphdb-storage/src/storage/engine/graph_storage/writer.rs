use crate::core::metadata::index_manager::IndexMetadataManager;
use crate::core::types::{
    EdgeTypeInfo, InsertEdgeInfo, InsertVertexInfo, LabelId, Timestamp, UpdateInfo, UpdateOp,
    UpdateTarget, VertexId,
};
use crate::core::{Edge, EdgeDirection, StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::params::{EdgeOperationParams, InsertEdgeParams};

use super::context::GraphStorageContext;
use super::ops::{edge_label_id, endpoint_label_id, tag_label_id};
use super::reader;

#[derive(Debug)]
struct InsertedVertexTag {
    label_id: LabelId,
    id: String,
    vid: VertexId,
    vertex_id: Value,
    tag_name: String,
}

#[derive(Debug)]
struct InsertedEdgeRecord {
    edge_label_id: LabelId,
    src_label_id: LabelId,
    dst_label_id: LabelId,
    src: VertexId,
    dst: VertexId,
    edge_type: String,
    rank: i64,
}

pub(crate) fn insert_vertex(
    ctx: &GraphStorageContext,
    space: &str,
    vertex: Vertex,
) -> StorageResult<VertexId> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();
    let mut rollback = Vec::new();
    let result =
        insert_vertex_at_timestamp(ctx, space, space_info.space_id, vertex, ts, &mut rollback);

    if result.is_err() {
        rollback_vertex_tags(ctx, space_info.space_id, &rollback, ts);
    }

    result
}

fn insert_vertex_at_timestamp(
    ctx: &GraphStorageContext,
    space: &str,
    space_id: u64,
    vertex: Vertex,
    ts: Timestamp,
    rollback: &mut Vec<InsertedVertexTag>,
) -> StorageResult<VertexId> {
    for tag in &vertex.tags {
        let label_id = tag_label_id(ctx, space, &tag.name)?
            .ok_or_else(|| StorageError::not_found(format!("Tag {} not found", tag.name)))?;
        let props: Vec<(String, Value)> = tag
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        if let Some(vid_int) = vertex.vid.as_int64() {
            ctx.insert_vertex_by_i64(label_id, vid_int, &props, ts)?;
        } else {
            let id_str = vertex.vid.to_string();
            ctx.insert_vertex(label_id, &id_str, &props, ts)?;
        }

        let vid_value = Value::from(vertex.vid);
        rollback.push(InsertedVertexTag {
            label_id,
            id: vertex.vid.to_string(),
            vid: vertex.vid,
            vertex_id: vid_value.clone(),
            tag_name: tag.name.clone(),
        });

        update_vertex_indexes(
            ctx,
            ctx.index_metadata_manager(),
            space_id,
            &vid_value,
            &tag.name,
            &props,
            ts,
        )?;
    }

    Ok(vertex.vid)
}

fn rollback_vertex_tags(
    ctx: &GraphStorageContext,
    space_id: u64,
    inserted: &[InsertedVertexTag],
    ts: Timestamp,
) {
    for item in inserted.iter().rev() {
        let _ = delete_vertex_indexes(
            ctx,
            ctx.index_metadata_manager(),
            space_id,
            &item.vertex_id,
            &item.tag_name,
            ts,
        );
        if let Some(vid_int) = item.vid.as_int64() {
            let _ = ctx.delete_vertex_by_i64(item.label_id, vid_int, ts);
        } else {
            let _ = ctx.delete_vertex(item.label_id, &item.id, ts);
        }
    }
}

pub(crate) fn update_vertex(
    ctx: &GraphStorageContext,
    space: &str,
    vertex: Vertex,
) -> StorageResult<()> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();
    let vid_int = vertex.vid.as_int64();

    for tag in &vertex.tags {
        if let Some(label_id) = tag_label_id(ctx, space, &tag.name)? {
            for (prop_name, value) in &tag.properties {
                if let Some(id_int) = vid_int {
                    ctx.update_vertex_property_by_i64(label_id, id_int, prop_name, value, ts)?;
                } else {
                    let id_str = vertex.vid.to_string();
                    ctx.update_vertex_property(label_id, &id_str, prop_name, value, ts)?;
                }
            }

            let props: Vec<(String, Value)> = tag
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let vid_value = Value::from(vertex.vid);
            update_vertex_indexes(
                ctx,
                ctx.index_metadata_manager(),
                space_info.space_id,
                &vid_value,
                &tag.name,
                &props,
                ts,
            )?;
        }
    }

    Ok(())
}

pub(crate) fn delete_vertex(
    ctx: &GraphStorageContext,
    space: &str,
    id: &VertexId,
) -> StorageResult<()> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let tags = ctx.schema_manager().list_tags(space)?;
    let ts = ctx.get_write_timestamp();
    let id_int = id.as_int64();

    for tag in &tags {
        let label_id = tag.tag_id;
        if let Some(vid_int) = id_int {
            let _ = ctx.delete_vertex_by_i64(label_id, vid_int, ts);
        } else {
            let id_str = id.to_string();
            let _ = ctx.delete_vertex(label_id, &id_str, ts);
        }

        let id_value = Value::from(*id);
        delete_vertex_indexes(
            ctx,
            ctx.index_metadata_manager(),
            space_info.space_id,
            &id_value,
            &tag.tag_name,
            ts,
        )?;
    }

    Ok(())
}

pub(crate) fn delete_vertex_with_edges(
    ctx: &GraphStorageContext,
    space: &str,
    id: &VertexId,
) -> StorageResult<()> {
    let edges = reader::get_node_edges(ctx, space, id, EdgeDirection::Both)?;

    for edge in edges {
        let _ = delete_edge(
            ctx,
            space,
            &edge.src,
            &edge.dst,
            &edge.edge_type,
            edge.ranking,
        );
    }

    delete_vertex(ctx, space, id)
}

pub(crate) fn batch_insert_vertices(
    ctx: &GraphStorageContext,
    space: &str,
    vertices: Vec<Vertex>,
) -> StorageResult<Vec<VertexId>> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    validate_vertex_batch(ctx, space, &vertices)?;

    let ts = ctx.get_write_timestamp();
    let mut ids = Vec::with_capacity(vertices.len());
    let mut rollback = Vec::new();

    for vertex in vertices {
        let id = match insert_vertex_at_timestamp(
            ctx,
            space,
            space_info.space_id,
            vertex,
            ts,
            &mut rollback,
        ) {
            Ok(id) => id,
            Err(e) => {
                rollback_vertex_tags(ctx, space_info.space_id, &rollback, ts);
                return Err(e);
            }
        };
        ids.push(id);
    }
    Ok(ids)
}

fn validate_vertex_batch(
    ctx: &GraphStorageContext,
    space: &str,
    vertices: &[Vertex],
) -> StorageResult<()> {
    for vertex in vertices {
        for tag in &vertex.tags {
            if tag_label_id(ctx, space, &tag.name)?.is_none() {
                return Err(StorageError::not_found(format!(
                    "Tag {} not found",
                    tag.name
                )));
            }
        }
    }
    Ok(())
}

pub(crate) fn delete_tags(
    ctx: &GraphStorageContext,
    space: &str,
    vertex_id: &VertexId,
    tag_names: &[String],
) -> StorageResult<usize> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();
    let mut deleted_count = 0;

    let id_str = vertex_id.to_string();

    for tag_name in tag_names {
        if let Some(label_id) = tag_label_id(ctx, space, tag_name)? {
            if ctx.delete_vertex(label_id, &id_str, ts).is_ok() {
                let vertex_id_value = Value::from(*vertex_id);
                delete_vertex_indexes(
                    ctx,
                    ctx.index_metadata_manager(),
                    space_info.space_id,
                    &vertex_id_value,
                    tag_name,
                    ts,
                )?;
                deleted_count += 1;
            }
        }
    }

    Ok(deleted_count)
}

pub(crate) fn insert_edge(ctx: &GraphStorageContext, space: &str, edge: Edge) -> StorageResult<()> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();
    let mut rollback = Vec::new();
    let result = insert_edge_at_timestamp(ctx, space, space_info.space_id, edge, ts, &mut rollback);

    if result.is_err() {
        rollback_edges(ctx, space_info.space_id, &rollback, ts);
    }

    result
}

fn insert_edge_at_timestamp(
    ctx: &GraphStorageContext,
    space: &str,
    space_id: u64,
    edge: Edge,
    ts: Timestamp,
    rollback: &mut Vec<InsertedEdgeRecord>,
) -> StorageResult<()> {
    let edge_type = resolve_edge_type(ctx, space, &edge.edge_type)?;
    let edge_label_id = edge_type.edge_type_id;
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
    let props: Vec<(String, Value)> = edge
        .props
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    ctx.insert_edge(InsertEdgeParams {
        edge_label: edge_label_id,
        src_label: src_label_id,
        src_id: edge.src,
        dst_label: dst_label_id,
        dst_id: edge.dst,
        rank: edge.ranking,
        properties: &props,
        ts,
    })?;

    rollback.push(InsertedEdgeRecord {
        edge_label_id,
        src_label_id,
        dst_label_id,
        src: edge.src,
        dst: edge.dst,
        edge_type: edge.edge_type.clone(),
        rank: edge.ranking,
    });

    let src_value = Value::from(edge.src);
    let dst_value = Value::from(edge.dst);
    update_edge_indexes(EdgeIndexUpdateParams {
        ctx,
        index_metadata_manager: ctx.index_metadata_manager(),
        space_id,
        src: &src_value,
        dst: &dst_value,
        edge_type: &edge.edge_type,
        props: &props,
        ts,
    })?;

    Ok(())
}

fn resolve_edge_type(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type: &str,
) -> StorageResult<EdgeTypeInfo> {
    ctx.schema_manager()
        .get_edge_type(space, edge_type)?
        .ok_or_else(|| StorageError::not_found(format!("Edge type {} not found", edge_type)))
}

fn rollback_edges(
    ctx: &GraphStorageContext,
    space_id: u64,
    inserted: &[InsertedEdgeRecord],
    ts: Timestamp,
) {
    for item in inserted.iter().rev() {
        let src_value = Value::from(item.src);
        let dst_value = Value::from(item.dst);
        let _ = delete_edge_indexes(
            ctx,
            ctx.index_metadata_manager(),
            space_id,
            &src_value,
            &dst_value,
            &item.edge_type,
            ts,
        );
        let _ = ctx.delete_edge(
            &EdgeOperationParams {
                edge_label: item.edge_label_id,
                src_label: item.src_label_id,
                src_id: item.src,
                dst_label: item.dst_label_id,
                dst_id: item.dst,
                rank: item.rank,
            },
            ts,
        );
    }
}

pub(crate) fn delete_edge(
    ctx: &GraphStorageContext,
    space: &str,
    src: &VertexId,
    dst: &VertexId,
    edge_type: &str,
    rank: i64,
) -> StorageResult<()> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();

    if let Some(edge_label_id) = edge_label_id(ctx, space, edge_type)? {
        let edge_types = ctx.schema_manager().list_edge_types(space)?;
        for et in edge_types {
            if et.edge_type_name == edge_type {
                let src_label_id = match endpoint_label_id(ctx, space, &et.src_tag_name)? {
                    Some(id) => id,
                    None => break,
                };
                let dst_label_id = match endpoint_label_id(ctx, space, &et.dst_tag_name)? {
                    Some(id) => id,
                    None => break,
                };
                ctx.delete_edge(
                    &EdgeOperationParams {
                        edge_label: edge_label_id,
                        src_label: src_label_id,
                        src_id: *src,
                        dst_label: dst_label_id,
                        dst_id: *dst,
                        rank,
                    },
                    ts,
                )?;

                let src_value = Value::from(*src);
                let dst_value = Value::from(*dst);
                delete_edge_indexes(
                    ctx,
                    ctx.index_metadata_manager(),
                    space_info.space_id,
                    &src_value,
                    &dst_value,
                    edge_type,
                    ts,
                )?;
                break;
            }
        }
    }

    Ok(())
}

pub(crate) fn batch_insert_edges(
    ctx: &GraphStorageContext,
    space: &str,
    edges: Vec<Edge>,
) -> StorageResult<()> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    validate_edge_batch(ctx, space, &edges)?;

    let ts = ctx.get_write_timestamp();
    let mut rollback = Vec::new();

    for edge in edges {
        if let Err(e) =
            insert_edge_at_timestamp(ctx, space, space_info.space_id, edge, ts, &mut rollback)
        {
            rollback_edges(ctx, space_info.space_id, &rollback, ts);
            return Err(e);
        }
    }
    Ok(())
}

fn validate_edge_batch(
    ctx: &GraphStorageContext,
    space: &str,
    edges: &[Edge],
) -> StorageResult<()> {
    for edge in edges {
        let edge_type = resolve_edge_type(ctx, space, &edge.edge_type)?;
        if endpoint_label_id(ctx, space, &edge_type.src_tag_name)?.is_none() {
            return Err(StorageError::not_found(format!(
                "Source tag {} not found",
                edge_type.src_tag_name
            )));
        }
        if endpoint_label_id(ctx, space, &edge_type.dst_tag_name)?.is_none() {
            return Err(StorageError::not_found(format!(
                "Destination tag {} not found",
                edge_type.dst_tag_name
            )));
        }
    }
    Ok(())
}

pub(crate) fn insert_vertex_data(
    ctx: &GraphStorageContext,
    space: &str,
    info: &InsertVertexInfo,
) -> StorageResult<bool> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let tag = ctx
        .schema_manager()
        .get_tag(space, &info.tag_name)?
        .ok_or_else(|| StorageError::not_found(format!("Tag {} not found", info.tag_name)))?;

    if info.space_id != space_info.space_id {
        return Err(StorageError::db_error("Space ID mismatch".to_string()));
    }

    let ts = ctx.get_write_timestamp();

    let label_id = tag.tag_id;
    let vid = VertexId::try_from(&info.vertex_id)
        .map_err(|e| StorageError::invalid_input(e.to_string()))?;
    let id_str = vid.to_string();

    let result = ctx.insert_vertex(label_id, &id_str, &info.props, ts);
    match result {
        Ok(_) => {
            update_vertex_indexes(
                ctx,
                ctx.index_metadata_manager(),
                space_info.space_id,
                &info.vertex_id,
                &info.tag_name,
                &info.props,
                ts,
            )?;
            Ok(true)
        }
        Err(ref e)
            if e.kind() == crate::core::error::storage::StorageErrorKind::VertexAlreadyExists =>
        {
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn insert_edge_data(
    ctx: &GraphStorageContext,
    space: &str,
    info: &InsertEdgeInfo,
) -> StorageResult<bool> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let edge_type = ctx
        .schema_manager()
        .get_edge_type(space, &info.edge_name)?
        .ok_or_else(|| {
            StorageError::not_found(format!("Edge type {} not found", info.edge_name))
        })?;

    if info.space_id != space_info.space_id {
        return Err(StorageError::db_error("Space ID mismatch".to_string()));
    }

    let ts = ctx.get_write_timestamp();

    let edge_label_id = edge_type.edge_type_id;
    let src_vid = VertexId::try_from(&info.src_vertex_id)
        .map_err(|e| StorageError::invalid_input(e.to_string()))?;
    let dst_vid = VertexId::try_from(&info.dst_vertex_id)
        .map_err(|e| StorageError::invalid_input(e.to_string()))?;
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
    let result = ctx.insert_edge(InsertEdgeParams {
        edge_label: edge_label_id,
        src_label: src_label_id,
        src_id: src_vid,
        dst_label: dst_label_id,
        dst_id: dst_vid,
        rank: info.rank,
        properties: &info.props,
        ts,
    });

    match result {
        Ok(_) => {
            update_edge_indexes(EdgeIndexUpdateParams {
                ctx,
                index_metadata_manager: ctx.index_metadata_manager(),
                space_id: space_info.space_id,
                src: &info.src_vertex_id,
                dst: &info.dst_vertex_id,
                edge_type: &info.edge_name,
                props: &info.props,
                ts,
            })?;
            Ok(true)
        }
        Err(e) => {
            if e.kind() == crate::core::error::storage::StorageErrorKind::EdgeAlreadyExists {
                return Ok(false);
            }
            Err(e)
        }
    }
}

pub(crate) fn delete_vertex_data(
    ctx: &GraphStorageContext,
    space: &str,
    vertex_id: &str,
) -> StorageResult<bool> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let tags = ctx.schema_manager().list_tags(space)?;
    let ts = ctx.get_write_timestamp();
    let mut deleted = false;

    for tag in tags {
        let label_id = tag.tag_id;
        if ctx.delete_vertex(label_id, vertex_id, ts).is_ok() {
            delete_vertex_indexes(
                ctx,
                ctx.index_metadata_manager(),
                space_info.space_id,
                &Value::String(vertex_id.to_string()),
                &tag.tag_name,
                ts,
            )?;
            deleted = true;
        }
    }

    Ok(deleted)
}

pub(crate) fn delete_edge_data(
    ctx: &GraphStorageContext,
    space: &str,
    src: &str,
    dst: &str,
    rank: i64,
) -> StorageResult<bool> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let edge_types = ctx.schema_manager().list_edge_types(space)?;
    let ts = ctx.get_write_timestamp();
    let mut deleted = false;

    for et in edge_types {
        let edge_label_id = et.edge_type_id;
        let src_label_id = match endpoint_label_id(ctx, space, &et.src_tag_name)? {
            Some(id) => id,
            None => continue,
        };
        let dst_label_id = match endpoint_label_id(ctx, space, &et.dst_tag_name)? {
            Some(id) => id,
            None => continue,
        };
        let src_vid = src
            .parse::<i64>()
            .map(VertexId::from_int64)
            .unwrap_or_else(|_| VertexId::from_string(src));
        let dst_vid = dst
            .parse::<i64>()
            .map(VertexId::from_int64)
            .unwrap_or_else(|_| VertexId::from_string(dst));
        if ctx
            .delete_edge(
                &EdgeOperationParams {
                    edge_label: edge_label_id,
                    src_label: src_label_id,
                    src_id: src_vid,
                    dst_label: dst_label_id,
                    dst_id: dst_vid,
                    rank,
                },
                ts,
            )
            .is_ok()
        {
            delete_edge_indexes(
                ctx,
                ctx.index_metadata_manager(),
                space_info.space_id,
                &Value::String(src.to_string()),
                &Value::String(dst.to_string()),
                &et.edge_type_name,
                ts,
            )?;
            deleted = true;
        }
    }

    Ok(deleted)
}

pub(crate) fn update_data(
    ctx: &GraphStorageContext,
    space: &str,
    space_id: u64,
    info: &UpdateInfo,
) -> StorageResult<bool> {
    let space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    if space_info.space_id != space_id {
        return Err(StorageError::db_error("Space ID mismatch".to_string()));
    }

    let ts = ctx.get_write_timestamp();

    let UpdateTarget {
        space_name,
        label,
        id,
        prop,
    } = &info.update_target;

    if space_name != space {
        return Err(StorageError::db_error(
            "Space name mismatch in update target".to_string(),
        ));
    }

    if let Some(label_id) = tag_label_id(ctx, space, label)? {
        let vid = VertexId::try_from(id).map_err(|e| StorageError::invalid_input(e.to_string()))?;
        let id_str = vid.to_string();
        let value = match &info.update_op {
            UpdateOp::Set => info.value.clone(),
            UpdateOp::Add => {
                if let Some(current) = ctx.get_vertex(label_id, &id_str, ts) {
                    let current_val = current
                        .properties
                        .iter()
                        .find(|(k, _)| k == prop)
                        .map(|(_, v)| v);
                    if let (Some(crate::core::Value::Int(cv)), crate::core::Value::Int(add_val)) =
                        (current_val, &info.value)
                    {
                        crate::core::Value::Int(cv + add_val)
                    } else {
                        info.value.clone()
                    }
                } else {
                    info.value.clone()
                }
            }
            UpdateOp::Subtract => {
                if let Some(current) = ctx.get_vertex(label_id, &id_str, ts) {
                    let current_val = current
                        .properties
                        .iter()
                        .find(|(k, _)| k == prop)
                        .map(|(_, v)| v);
                    if let (Some(crate::core::Value::Int(cv)), crate::core::Value::Int(sub_val)) =
                        (current_val, &info.value)
                    {
                        crate::core::Value::Int(cv - sub_val)
                    } else {
                        info.value.clone()
                    }
                } else {
                    info.value.clone()
                }
            }
            _ => info.value.clone(),
        };

        ctx.update_vertex_property(label_id, &id_str, prop, &value, ts)?;

        let props = vec![(prop.clone(), value)];
        update_vertex_indexes(
            ctx,
            ctx.index_metadata_manager(),
            space_info.space_id,
            id,
            label,
            &props,
            ts,
        )?;
        Ok(true)
    } else {
        Err(StorageError::not_found(format!(
            "Label {} not found",
            label
        )))
    }
}

fn update_vertex_indexes(
    ctx: &GraphStorageContext,
    index_metadata_manager: &crate::core::metadata::IndexManager,
    space_id: u64,
    vertex_id: &Value,
    tag_name: &str,
    props: &[(String, Value)],
    ts: u32,
) -> StorageResult<()> {
    let indexes = index_metadata_manager.list_tag_indexes(space_id)?;
    for index in indexes {
        if index.schema_name == tag_name {
            ctx.update_vertex_indexes_mvcc(space_id, vertex_id, &index.name, props, ts)?;
        }
    }
    Ok(())
}

struct EdgeIndexUpdateParams<'a> {
    ctx: &'a GraphStorageContext,
    index_metadata_manager: &'a crate::core::metadata::IndexManager,
    space_id: u64,
    src: &'a Value,
    dst: &'a Value,
    edge_type: &'a str,
    props: &'a [(String, Value)],
    ts: u32,
}

fn update_edge_indexes(params: EdgeIndexUpdateParams) -> StorageResult<()> {
    let indexes = params
        .index_metadata_manager
        .list_edge_indexes(params.space_id)?;
    for index in indexes {
        if index.schema_name == params.edge_type {
            params.ctx.update_edge_indexes_mvcc(
                params.space_id,
                params.src,
                params.dst,
                &index.name,
                params.props,
                params.ts,
            )?;
        }
    }
    Ok(())
}

fn delete_vertex_indexes(
    ctx: &GraphStorageContext,
    index_metadata_manager: &crate::core::metadata::IndexManager,
    space_id: u64,
    vertex_id: &Value,
    tag_name: &str,
    ts: u32,
) -> StorageResult<()> {
    let indexes = index_metadata_manager.list_tag_indexes(space_id)?;
    for index in indexes {
        if index.schema_name == tag_name {
            ctx.delete_vertex_indexes_mvcc(space_id, vertex_id, ts)?;
        }
    }
    Ok(())
}

fn delete_edge_indexes(
    ctx: &GraphStorageContext,
    index_metadata_manager: &crate::core::metadata::IndexManager,
    space_id: u64,
    src: &Value,
    dst: &Value,
    edge_type: &str,
    ts: u32,
) -> StorageResult<()> {
    let indexes = index_metadata_manager.list_edge_indexes(space_id)?;
    let index_names: Vec<String> = indexes
        .iter()
        .filter(|index| index.schema_name == edge_type)
        .map(|index| index.name.clone())
        .collect();

    if !index_names.is_empty() {
        ctx.delete_edge_indexes_mvcc(space_id, src, dst, &index_names, ts)?;
    }
    Ok(())
}
