use crate::core::metadata::index_manager::IndexMetadataManager;
use crate::core::types::{
    InsertEdgeInfo, InsertVertexInfo, LabelId, UpdateInfo, UpdateOp, UpdateTarget, VertexId,
};
use crate::core::{Edge, EdgeDirection, StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::property_graph::{InsertEdgeParams, InsertEdgeParamsByI64};

use super::context::GraphStorageContext;
use super::reader;
use super::type_utils::{edge_label_id, endpoint_label_id, tag_label_id};

pub(crate) fn insert_vertex(
    ctx: &GraphStorageContext,
    space: &str,
    vertex: Vertex,
) -> StorageResult<VertexId> {
    let space_info = ctx
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();

    let mut inserted_tags: Vec<(LabelId, String)> = Vec::new();

    for tag in &vertex.tags {
        if let Some(label_id) = tag_label_id(ctx, space, &tag.name)? {
            let props: Vec<(String, Value)> = tag
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            let insert_result = if let Some(vid_int) = vertex.vid.as_int64() {
                ctx.graph
                    .insert_vertex_by_i64(label_id, vid_int, &props, ts)
            } else {
                let id_str = vertex.vid.to_string();
                ctx.graph.insert_vertex(label_id, &id_str, &props, ts)
            };

            if let Err(e) = insert_result {
                for (rollback_label, rollback_id) in inserted_tags.iter().rev() {
                    let _ = ctx.graph.delete_vertex(*rollback_label, rollback_id, ts);
                }
                return Err(e);
            }

            let vid_value = Value::from(vertex.vid);
            if let Err(e) = update_vertex_indexes(
                &ctx.graph,
                &ctx.index_metadata_manager,
                space_info.space_id,
                &vid_value,
                &tag.name,
                &props,
                ts,
            ) {
                for (rollback_label, rollback_id) in inserted_tags.iter().rev() {
                    let _ = ctx.graph.delete_vertex(*rollback_label, rollback_id, ts);
                }
                let id_str = vertex.vid.to_string();
                let _ = ctx.graph.delete_vertex(label_id, &id_str, ts);
                return Err(e);
            }

            inserted_tags.push((label_id, vertex.vid.to_string()));
        }
    }

    Ok(vertex.vid)
}

pub(crate) fn update_vertex(
    ctx: &GraphStorageContext,
    space: &str,
    vertex: Vertex,
) -> StorageResult<()> {
    let space_info = ctx
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();
    let vid_int = vertex.vid.as_int64();

    for tag in &vertex.tags {
        if let Some(label_id) = tag_label_id(ctx, space, &tag.name)? {
            for (prop_name, value) in &tag.properties {
                if let Some(id_int) = vid_int {
                    ctx.graph
                        .update_vertex_property_by_i64(label_id, id_int, prop_name, value, ts)?;
                } else {
                    let id_str = vertex.vid.to_string();
                    ctx.graph
                        .update_vertex_property(label_id, &id_str, prop_name, value, ts)?;
                }
            }

            let props: Vec<(String, Value)> = tag
                .properties
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            let vid_value = Value::from(vertex.vid);
            update_vertex_indexes(
                &ctx.graph,
                &ctx.index_metadata_manager,
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
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let tags = ctx.schema_manager.list_tags(space)?;
    let ts = ctx.get_write_timestamp();
    let id_int = id.as_int64();

    for tag in &tags {
        let label_id = tag.tag_id;
        if let Some(vid_int) = id_int {
            let _ = ctx.graph.delete_vertex_by_i64(label_id, vid_int, ts);
        } else {
            let id_str = id.to_string();
            let _ = ctx.graph.delete_vertex(label_id, &id_str, ts);
        }

        let id_value = Value::from(*id);
        delete_vertex_indexes(
            &ctx.graph,
            &ctx.index_metadata_manager,
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
    let mut ids = Vec::with_capacity(vertices.len());
    for vertex in vertices {
        let id = insert_vertex(ctx, space, vertex)?;
        ids.push(id);
    }
    Ok(ids)
}

pub(crate) fn delete_tags(
    ctx: &GraphStorageContext,
    space: &str,
    vertex_id: &VertexId,
    tag_names: &[String],
) -> StorageResult<usize> {
    let space_info = ctx
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();
    let mut deleted_count = 0;

    let id_str = vertex_id.to_string();

    for tag_name in tag_names {
        if let Some(label_id) = tag_label_id(ctx, space, tag_name)? {
            if ctx.graph.delete_vertex(label_id, &id_str, ts).is_ok() {
                let vertex_id_value = Value::from(*vertex_id);
                delete_vertex_indexes(
                    &ctx.graph,
                    &ctx.index_metadata_manager,
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
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();

    if let Some(edge_label_id) = edge_label_id(ctx, space, &edge.edge_type)? {
        let edge_types = ctx.schema_manager.list_edge_types(space)?;
        for et in edge_types {
            if et.edge_type_name == edge.edge_type {
                let src_label_id = match endpoint_label_id(ctx, space, &et.src_tag_name)? {
                    Some(id) => id,
                    None => break,
                };
                let dst_label_id = match endpoint_label_id(ctx, space, &et.dst_tag_name)? {
                    Some(id) => id,
                    None => break,
                };
                let props: Vec<(String, Value)> = edge
                    .props
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                let src_int = edge.src.as_int64();
                let dst_int = edge.dst.as_int64();

                if let (Some(src_id), Some(dst_id)) = (src_int, dst_int) {
                    ctx.graph.insert_edge_by_i64(InsertEdgeParamsByI64 {
                        edge_label: edge_label_id,
                        src_label: src_label_id,
                        src_id,
                        dst_label: dst_label_id,
                        dst_id,
                        properties: &props,
                        ts,
                    })?;
                } else {
                    let src_str = edge.src.to_string();
                    let dst_str = edge.dst.to_string();
                    ctx.graph.insert_edge(InsertEdgeParams {
                        edge_label: edge_label_id,
                        src_label: src_label_id,
                        src_id: &src_str,
                        dst_label: dst_label_id,
                        dst_id: &dst_str,
                        properties: &props,
                        ts,
                    })?;
                }

                let src_value = Value::from(edge.src);
                let dst_value = Value::from(edge.dst);
                update_edge_indexes(EdgeIndexUpdateParams {
                    graph: &ctx.graph,
                    index_metadata_manager: &ctx.index_metadata_manager,
                    space_id: space_info.space_id,
                    src: &src_value,
                    dst: &dst_value,
                    edge_type: &edge.edge_type,
                    props: &props,
                    ts,
                })?;
                break;
            }
        }
    }

    Ok(())
}

pub(crate) fn delete_edge(
    ctx: &GraphStorageContext,
    space: &str,
    src: &VertexId,
    dst: &VertexId,
    edge_type: &str,
    _rank: i64,
) -> StorageResult<()> {
    let space_info = ctx
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_write_timestamp();

    if let Some(edge_label_id) = edge_label_id(ctx, space, edge_type)? {
        let edge_types = ctx.schema_manager.list_edge_types(space)?;
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
                let src_str = src.to_string();
                let dst_str = dst.to_string();

                ctx.graph.delete_edge(
                    edge_label_id,
                    src_label_id,
                    &src_str,
                    dst_label_id,
                    &dst_str,
                    ts,
                )?;

                let src_value = Value::from(*src);
                let dst_value = Value::from(*dst);
                delete_edge_indexes(
                    &ctx.graph,
                    &ctx.index_metadata_manager,
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
    for edge in edges {
        insert_edge(ctx, space, edge)?;
    }
    Ok(())
}

pub(crate) fn insert_vertex_data(
    ctx: &GraphStorageContext,
    space: &str,
    info: &InsertVertexInfo,
) -> StorageResult<bool> {
    let space_info = ctx
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let tag = ctx
        .schema_manager
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

    let result = ctx.graph.insert_vertex(label_id, &id_str, &info.props, ts);
    match result {
        Ok(_) => {
            update_vertex_indexes(
                &ctx.graph,
                &ctx.index_metadata_manager,
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
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let edge_type = ctx
        .schema_manager
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
    let src_int = src_vid.as_int64();
    let dst_int = dst_vid.as_int64();

    let result = if let (Some(src_id), Some(dst_id)) = (src_int, dst_int) {
        ctx.graph.insert_edge_by_i64(InsertEdgeParamsByI64 {
            edge_label: edge_label_id,
            src_label: src_label_id,
            src_id,
            dst_label: dst_label_id,
            dst_id,
            properties: &info.props,
            ts,
        })
    } else {
        let src_id = src_vid.to_string();
        let dst_id = dst_vid.to_string();
        ctx.graph.insert_edge(InsertEdgeParams {
            edge_label: edge_label_id,
            src_label: src_label_id,
            src_id: &src_id,
            dst_label: dst_label_id,
            dst_id: &dst_id,
            properties: &info.props,
            ts,
        })
    };

    match result {
        Ok(_) => {
            update_edge_indexes(EdgeIndexUpdateParams {
                graph: &ctx.graph,
                index_metadata_manager: &ctx.index_metadata_manager,
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
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let tags = ctx.schema_manager.list_tags(space)?;
    let ts = ctx.get_write_timestamp();
    let mut deleted = false;

    for tag in tags {
        let label_id = tag.tag_id;
        if ctx.graph.delete_vertex(label_id, vertex_id, ts).is_ok() {
            delete_vertex_indexes(
                &ctx.graph,
                &ctx.index_metadata_manager,
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
    _rank: i64,
) -> StorageResult<bool> {
    let space_info = ctx
        .schema_manager
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let edge_types = ctx.schema_manager.list_edge_types(space)?;
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
        if ctx
            .graph
            .delete_edge(edge_label_id, src_label_id, src, dst_label_id, dst, ts)
            .is_ok()
        {
            delete_edge_indexes(
                &ctx.graph,
                &ctx.index_metadata_manager,
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
        .schema_manager
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
                if let Some(current) = ctx.graph.get_vertex(label_id, &id_str, ts) {
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
                if let Some(current) = ctx.graph.get_vertex(label_id, &id_str, ts) {
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

        ctx.graph
            .update_vertex_property(label_id, &id_str, prop, &value, ts)?;

        let props = vec![(prop.clone(), value)];
        update_vertex_indexes(
            &ctx.graph,
            &ctx.index_metadata_manager,
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
    graph: &crate::storage::engine::PropertyGraph,
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
            graph.update_vertex_indexes_mvcc(space_id, vertex_id, &index.name, props, ts)?;
        }
    }
    Ok(())
}

struct EdgeIndexUpdateParams<'a> {
    graph: &'a crate::storage::engine::PropertyGraph,
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
            params.graph.update_edge_indexes_mvcc(
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
    graph: &crate::storage::engine::PropertyGraph,
    index_metadata_manager: &crate::core::metadata::IndexManager,
    space_id: u64,
    vertex_id: &Value,
    tag_name: &str,
    ts: u32,
) -> StorageResult<()> {
    let indexes = index_metadata_manager.list_tag_indexes(space_id)?;
    for index in indexes {
        if index.schema_name == tag_name {
            graph.delete_vertex_indexes_mvcc(space_id, vertex_id, ts)?;
        }
    }
    Ok(())
}

fn delete_edge_indexes(
    graph: &crate::storage::engine::PropertyGraph,
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
        graph.delete_edge_indexes_mvcc(space_id, src, dst, &index_names, ts)?;
    }
    Ok(())
}
