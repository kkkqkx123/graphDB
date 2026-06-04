use crate::core::types::{EdgeTypeInfo, LabelId, TagInfo, VertexId};
use crate::core::{Edge, EdgeDirection, StorageError, StorageResult, Value, Vertex};
use crate::storage::engine::params::{EdgeOperationParams, EdgeOperationParamsByI64};

use super::context::GraphStorageContext;
use super::type_utils::{
    edge_record_to_edge, endpoint_label_id, serialize_properties, value_to_string,
    vertex_record_to_vertex,
};

pub(crate) fn get_vertex(
    ctx: &GraphStorageContext,
    space: &str,
    id: &VertexId,
) -> StorageResult<Option<Vertex>> {
    let _space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let tags = ctx.schema_manager().list_tags(space)?;
    if tags.is_empty() {
        return Ok(None);
    }

    let ts = ctx.get_read_timestamp();

    for tag in &tags {
        let label_id = tag.tag_id;
        let record = if let Some(id_int) = id.as_int64() {
            ctx.graph().get_vertex_by_i64(label_id, id_int, ts)
        } else {
            let id_str = id.to_string();
            ctx.graph().get_vertex(label_id, &id_str, ts)
        };

        if let Some(record) = record {
            let vertex = vertex_record_to_vertex(&record, &tag.tag_name);
            return Ok(Some(vertex));
        }
    }

    Ok(None)
}

pub(crate) fn scan_vertices(ctx: &GraphStorageContext, space: &str) -> StorageResult<Vec<Vertex>> {
    let tags = ctx.schema_manager().list_tags(space)?;
    let ts = ctx.get_read_timestamp();
    let mut vertices = Vec::new();

    for tag in &tags {
        if let Some(iterator) = ctx.graph().scan_vertices(tag.tag_id, ts) {
            for record in iterator {
                let vertex = vertex_record_to_vertex(&record, &tag.tag_name);
                vertices.push(vertex);
            }
        }
    }

    Ok(vertices)
}

pub(crate) fn scan_vertices_by_tag(
    ctx: &GraphStorageContext,
    space: &str,
    tag: &str,
) -> StorageResult<Vec<Vertex>> {
    let tag_info = ctx.schema_manager().get_tag(space, tag)?.ok_or_else(|| {
        StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
    })?;

    let ts = ctx.get_read_timestamp();
    let mut vertices = Vec::new();

    let label_id = tag_info.tag_id;
    if let Some(iterator) = ctx.graph().scan_vertices(label_id, ts) {
        for record in iterator {
            let vertex = vertex_record_to_vertex(&record, tag);
            vertices.push(vertex);
        }
    }

    Ok(vertices)
}

pub(crate) fn scan_vertices_by_prop(
    ctx: &GraphStorageContext,
    space: &str,
    tag: &str,
    prop: &str,
    value: &Value,
) -> StorageResult<Vec<Vertex>> {
    let tag_info = ctx.schema_manager().get_tag(space, tag)?.ok_or_else(|| {
        StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
    })?;

    let ts = ctx.get_read_timestamp();
    let mut vertices = Vec::new();

    let label_id = tag_info.tag_id;
    if let Some(iterator) = ctx.graph().scan_vertices(label_id, ts) {
        for record in iterator {
            if record
                .properties
                .iter()
                .any(|(k, v)| k == prop && v == value)
            {
                let vertex = vertex_record_to_vertex(&record, tag);
                vertices.push(vertex);
            }
        }
    }

    Ok(vertices)
}

pub(crate) fn get_edge(
    ctx: &GraphStorageContext,
    space: &str,
    src: &VertexId,
    dst: &VertexId,
    edge_type: &str,
    rank: i64,
) -> StorageResult<Option<Edge>> {
    let edge_info = ctx
        .schema_manager()
        .get_edge_type(space, edge_type)?
        .ok_or_else(|| {
            StorageError::not_found(format!(
                "Edge type {} not found in space {}",
                edge_type, space
            ))
        })?;

    let ts = ctx.get_read_timestamp();

    let edge_label_id = edge_info.edge_type_id;
    let src_label_id = match endpoint_label_id(ctx, space, &edge_info.src_tag_name)? {
        Some(id) => id,
        None => return Ok(None),
    };
    let dst_label_id = match endpoint_label_id(ctx, space, &edge_info.dst_tag_name)? {
        Some(id) => id,
        None => return Ok(None),
    };
    let src_str = src.to_string();
    let dst_str = dst.to_string();

    if let Some(record) = ctx.graph().get_edge(
        &EdgeOperationParams {
            edge_label: edge_label_id,
            src_label: src_label_id,
            src_id: &src_str,
            dst_label: dst_label_id,
            dst_id: &dst_str,
            rank,
        },
        ts,
    ) {
        let edge = edge_record_to_edge(&record, edge_type, &src_str, &dst_str);
        return Ok(Some(edge));
    }

    if let (Some(src_int), Some(dst_int)) = (src.as_int64(), dst.as_int64()) {
        if let Some(record) = ctx.graph().get_edge_by_i64(
            &EdgeOperationParamsByI64 {
                edge_label: edge_label_id,
                src_label: src_label_id,
                src_id: src_int,
                dst_label: dst_label_id,
                dst_id: dst_int,
                rank,
            },
            ts,
        ) {
            let edge = edge_record_to_edge(&record, edge_type, &src_str, &dst_str);
            return Ok(Some(edge));
        }
    }

    Ok(None)
}

pub(crate) fn get_node_edges(
    ctx: &GraphStorageContext,
    space: &str,
    node_id: &VertexId,
    direction: EdgeDirection,
) -> StorageResult<Vec<Edge>> {
    let edge_types = ctx.schema_manager().list_edge_types(space)?;
    if edge_types.is_empty() {
        return Ok(Vec::new());
    }

    let ts = ctx.get_read_timestamp();
    let node_str = node_id.to_string();
    let mut edges = Vec::new();

    for edge_info in &edge_types {
        let edge_label_id = edge_info.edge_type_id;
        let edge_type_name = &edge_info.edge_type_name;

        let src_label_id = match endpoint_label_id(ctx, space, &edge_info.src_tag_name)? {
            Some(id) => id,
            None => continue,
        };
        let dst_label_id = match endpoint_label_id(ctx, space, &edge_info.dst_tag_name)? {
            Some(id) => id,
            None => continue,
        };
        match direction {
            EdgeDirection::Out => {
                if let Some(out_edges) =
                    ctx.graph()
                        .out_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                {
                    for record in out_edges {
                        let dst_internal = record.dst_vid.as_int64().unwrap_or(0) as u32;
                        let dst_external = if dst_label_id != 0 {
                            ctx.graph()
                                .get_external_id(dst_label_id, dst_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.dst_vid))
                        } else {
                            ctx.graph()
                                .get_external_id_any(dst_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.dst_vid))
                        };

                        let edge =
                            edge_record_to_edge(&record, edge_type_name, &node_str, &dst_external);
                        edges.push(edge);
                    }
                }
            }
            EdgeDirection::In => {
                if let Some(in_edges) =
                    ctx.graph()
                        .in_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                {
                    for record in in_edges {
                        let src_internal = record.src_vid.as_int64().unwrap_or(0) as u32;
                        let src_external = if src_label_id != 0 {
                            ctx.graph()
                                .get_external_id(src_label_id, src_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.src_vid))
                        } else {
                            ctx.graph()
                                .get_external_id_any(src_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.src_vid))
                        };

                        let edge =
                            edge_record_to_edge(&record, edge_type_name, &src_external, &node_str);
                        edges.push(edge);
                    }
                }
            }
            EdgeDirection::Both => {
                if let Some(out_edges) =
                    ctx.graph()
                        .out_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                {
                    for record in out_edges {
                        let dst_internal = record.dst_vid.as_int64().unwrap_or(0) as u32;
                        let dst_external = if dst_label_id != 0 {
                            ctx.graph()
                                .get_external_id(dst_label_id, dst_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.dst_vid))
                        } else {
                            ctx.graph()
                                .get_external_id_any(dst_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.dst_vid))
                        };

                        let edge =
                            edge_record_to_edge(&record, edge_type_name, &node_str, &dst_external);
                        edges.push(edge);
                    }
                }
                if let Some(in_edges) =
                    ctx.graph()
                        .in_edges(edge_label_id, src_label_id, dst_label_id, &node_str, ts)
                {
                    for record in in_edges {
                        let src_internal = record.src_vid.as_int64().unwrap_or(0) as u32;
                        let src_external = if src_label_id != 0 {
                            ctx.graph()
                                .get_external_id(src_label_id, src_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.src_vid))
                        } else {
                            ctx.graph()
                                .get_external_id_any(src_internal, ts)
                                .unwrap_or_else(|| format!("{}", record.src_vid))
                        };

                        let edge =
                            edge_record_to_edge(&record, edge_type_name, &src_external, &node_str);
                        edges.push(edge);
                    }
                }
            }
        }
    }

    Ok(edges)
}

pub(crate) fn scan_edges_by_type(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type: &str,
) -> StorageResult<Vec<Edge>> {
    let edge_info = ctx
        .schema_manager()
        .get_edge_type(space, edge_type)?
        .ok_or_else(|| {
            StorageError::not_found(format!(
                "Edge type {} not found in space {}",
                edge_type, space
            ))
        })?;

    let ts = ctx.get_read_timestamp();
    let mut edges = Vec::new();

    let edge_label_id = edge_info.edge_type_id;

    let src_label_id: LabelId = match endpoint_label_id(ctx, space, &edge_info.src_tag_name)? {
        Some(id) => id,
        None => return Ok(edges),
    };
    let dst_label_id: LabelId = match endpoint_label_id(ctx, space, &edge_info.dst_tag_name)? {
        Some(id) => id,
        None => return Ok(edges),
    };

    let records = if edge_info.src_tag_name.is_empty() || edge_info.dst_tag_name.is_empty() {
        ctx.graph().scan_edges_by_label(edge_label_id, ts)
    } else {
        ctx.graph()
            .scan_edges(src_label_id, dst_label_id, edge_label_id, ts)
    };

    for record in records {
        let src_internal = record.src_vid.as_int64().unwrap_or(0) as u32;
        let dst_internal = record.dst_vid.as_int64().unwrap_or(0) as u32;

        let src_external = if src_label_id != 0 {
            ctx.graph()
                .get_external_id(src_label_id, src_internal, ts)
                .unwrap_or_else(|| format!("{}", record.src_vid))
        } else {
            ctx.graph()
                .get_external_id_any(src_internal, ts)
                .unwrap_or_else(|| format!("{}", record.src_vid))
        };

        let dst_external = if dst_label_id != 0 {
            ctx.graph()
                .get_external_id(dst_label_id, dst_internal, ts)
                .unwrap_or_else(|| format!("{}", record.dst_vid))
        } else {
            ctx.graph()
                .get_external_id_any(dst_internal, ts)
                .unwrap_or_else(|| format!("{}", record.dst_vid))
        };

        let edge = edge_record_to_edge(&record, edge_type, &src_external, &dst_external);
        edges.push(edge);
    }

    Ok(edges)
}

pub(crate) fn scan_all_edges(ctx: &GraphStorageContext, space: &str) -> StorageResult<Vec<Edge>> {
    let _space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let mut edges = Vec::new();
    let edge_types = ctx.schema_manager().list_edge_types(space)?;

    for et in edge_types {
        let type_edges = scan_edges_by_type(ctx, space, &et.edge_type_name)?;
        edges.extend(type_edges);
    }

    Ok(edges)
}

pub(crate) fn get_vertex_with_schema(
    ctx: &GraphStorageContext,
    space: &str,
    tag: &str,
    id: &Value,
) -> StorageResult<Option<(TagInfo, Vec<u8>)>> {
    let tag_info = ctx.schema_manager().get_tag(space, tag)?.ok_or_else(|| {
        StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
    })?;

    let ts = ctx.get_read_timestamp();
    let id_str = value_to_string(id);

    let label_id = tag_info.tag_id;
    if let Some(record) = ctx.graph().get_vertex(label_id, &id_str, ts) {
        let data = serialize_properties(&record.properties);
        return Ok(Some((tag_info, data)));
    }

    Ok(None)
}

pub(crate) fn get_edge_with_schema(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type: &str,
    src: &Value,
    dst: &Value,
) -> StorageResult<Option<(EdgeTypeInfo, Vec<u8>)>> {
    let edge_info = ctx
        .schema_manager()
        .get_edge_type(space, edge_type)?
        .ok_or_else(|| {
            StorageError::not_found(format!(
                "Edge type {} not found in space {}",
                edge_type, space
            ))
        })?;

    let ts = ctx.get_read_timestamp();
    let src_str = value_to_string(src);
    let dst_str = value_to_string(dst);

    let edge_label_id = edge_info.edge_type_id;
    let src_label_id = match endpoint_label_id(ctx, space, &edge_info.src_tag_name)? {
        Some(id) => id,
        None => return Ok(None),
    };
    let dst_label_id = match endpoint_label_id(ctx, space, &edge_info.dst_tag_name)? {
        Some(id) => id,
        None => return Ok(None),
    };
    if let Some(record) = ctx.graph().get_edge(
        &EdgeOperationParams {
            edge_label: edge_label_id,
            src_label: src_label_id,
            src_id: &src_str,
            dst_label: dst_label_id,
            dst_id: &dst_str,
            rank: 0,
        },
        ts,
    ) {
        let data = serialize_properties(&record.properties);
        return Ok(Some((edge_info, data)));
    }

    Ok(None)
}

pub(crate) fn scan_vertices_with_schema(
    ctx: &GraphStorageContext,
    space: &str,
    tag: &str,
) -> StorageResult<Vec<(TagInfo, Vec<u8>)>> {
    let tag_info = ctx.schema_manager().get_tag(space, tag)?.ok_or_else(|| {
        StorageError::not_found(format!("Tag {} not found in space {}", tag, space))
    })?;

    let ts = ctx.get_read_timestamp();
    let mut results = Vec::new();

    let label_id = tag_info.tag_id;
    if let Some(iterator) = ctx.graph().scan_vertices(label_id, ts) {
        for record in iterator {
            let data = serialize_properties(&record.properties);
            results.push((tag_info.clone(), data));
        }
    }

    Ok(results)
}

pub(crate) fn scan_edges_with_schema(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type: &str,
) -> StorageResult<Vec<(EdgeTypeInfo, Vec<u8>)>> {
    let edge_info = ctx
        .schema_manager()
        .get_edge_type(space, edge_type)?
        .ok_or_else(|| {
            StorageError::not_found(format!(
                "Edge type {} not found in space {}",
                edge_type, space
            ))
        })?;

    let ts = ctx.get_read_timestamp();
    let mut results = Vec::new();

    let edge_label_id = edge_info.edge_type_id;
    let src_label_id: LabelId;
    let dst_label_id: LabelId;

    if edge_info.src_tag_name.is_empty() || edge_info.dst_tag_name.is_empty() {
        let records = ctx.graph().scan_edges_by_label(edge_label_id, ts);
        for record in records {
            let data = serialize_properties(&record.properties);
            results.push((edge_info.clone(), data));
        }
    } else {
        src_label_id = match endpoint_label_id(ctx, space, &edge_info.src_tag_name)? {
            Some(id) => id,
            None => return Ok(results),
        };
        dst_label_id = match endpoint_label_id(ctx, space, &edge_info.dst_tag_name)? {
            Some(id) => id,
            None => return Ok(results),
        };
        let records = ctx
            .graph()
            .scan_edges(src_label_id, dst_label_id, edge_label_id, ts);
        for record in records {
            let data = serialize_properties(&record.properties);
            results.push((edge_info.clone(), data));
        }
    }

    Ok(results)
}
