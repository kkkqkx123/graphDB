//! Storage Engine Operations
//!
//! Contains type conversion utilities, user operations, and maintenance operations.

use std::collections::HashMap;

use crate::core::types::{LabelId, PasswordInfo, UserAlterInfo, UserInfo, VertexId};
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, RoleType, StorageError, StorageResult, Value, Vertex};
use crate::storage::edge::EdgeRecord;
use crate::storage::vertex::VertexRecord;
use crate::storage::StorageStats;

use super::context::GraphStorageContext;
use super::writer;

// ── Type Conversion Utilities ──

pub(crate) fn vertex_type_storage_name(space_id: u64, tag_name: &str) -> String {
    format!("space_{space_id}:tag:{tag_name}")
}

pub(crate) fn edge_type_storage_name(space_id: u64, edge_type_name: &str) -> String {
    format!("space_{space_id}:edge:{edge_type_name}")
}

pub(crate) fn tag_label_id(
    ctx: &GraphStorageContext,
    space: &str,
    tag_name: &str,
) -> StorageResult<Option<LabelId>> {
    Ok(ctx
        .schema_manager()
        .get_tag(space, tag_name)?
        .map(|tag| tag.tag_id))
}

pub(crate) fn endpoint_label_id(
    ctx: &GraphStorageContext,
    space: &str,
    tag_name: &str,
) -> StorageResult<Option<LabelId>> {
    if tag_name.is_empty() {
        return Ok(Some(0));
    }
    tag_label_id(ctx, space, tag_name)
}

pub(crate) fn edge_label_id(
    ctx: &GraphStorageContext,
    space: &str,
    edge_type_name: &str,
) -> StorageResult<Option<LabelId>> {
    Ok(ctx
        .schema_manager()
        .get_edge_type(space, edge_type_name)?
        .map(|edge_type| edge_type.edge_type_id))
}

pub(crate) fn value_to_string(value: &Value) -> String {
    match value {
        Value::SmallInt(i) => i.to_string(),
        Value::Int(i) => i.to_string(),
        Value::BigInt(i) => i.to_string(),
        Value::String(s) => s.clone(),
        Value::Float(f) => f.to_string(),
        Value::Double(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => format!("{:?}", value),
    }
}

pub(crate) fn vertex_record_to_vertex(record: &VertexRecord, tag_name: &str) -> Vertex {
    let vid = record.vid;
    let properties: HashMap<String, Value> = record.properties.iter().cloned().collect();

    Vertex {
        vid,
        id: record.internal_id as i64,
        tags: vec![Tag {
            name: tag_name.to_string(),
            properties: properties.clone(),
        }],
        properties,
    }
}

pub(crate) fn edge_record_to_edge(
    record: &EdgeRecord,
    edge_type: &str,
    src_id: &str,
    dst_id: &str,
) -> Edge {
    let props: HashMap<String, Value> = record.properties.iter().cloned().collect();

    let src_vid = if let Ok(id) = src_id.parse::<i64>() {
        VertexId::from_int64(id)
    } else {
        VertexId::from_string(src_id)
    };

    let dst_vid = if let Ok(id) = dst_id.parse::<i64>() {
        VertexId::from_int64(id)
    } else {
        VertexId::from_string(dst_id)
    };

    Edge {
        src: src_vid,
        dst: dst_vid,
        edge_type: edge_type.to_string(),
        ranking: record.rank,
        id: record.edge_id as i64,
        props,
    }
}

pub(crate) fn serialize_properties(props: &[(String, Value)]) -> Vec<u8> {
    let mut data = Vec::new();
    for (key, value) in props {
        data.extend_from_slice(key.as_bytes());
        data.push(0);
        match value {
            Value::String(s) => {
                data.push(1);
                data.extend_from_slice(s.as_bytes());
            }
            Value::Int(i) => {
                data.push(2);
                data.extend_from_slice(&i.to_le_bytes());
            }
            Value::Float(f) => {
                data.push(3);
                data.extend_from_slice(&f.to_le_bytes());
            }
            Value::Bool(b) => {
                data.push(4);
                data.push(if *b { 1 } else { 0 });
            }
            _ => {
                data.push(0);
            }
        }
        data.push(0);
    }
    data
}

// ── User Operations ──

pub(crate) fn create_user(ctx: &GraphStorageContext, info: &UserInfo) -> StorageResult<bool> {
    ctx.user_storage().create_user(info)
}

pub(crate) fn drop_user(ctx: &GraphStorageContext, username: &str) -> StorageResult<bool> {
    ctx.user_storage().drop_user(username)
}

pub(crate) fn alter_user(ctx: &GraphStorageContext, info: &UserAlterInfo) -> StorageResult<bool> {
    ctx.user_storage().alter_user(info)
}

pub(crate) fn grant_role(
    ctx: &GraphStorageContext,
    username: &str,
    space_id: u64,
    role: RoleType,
) -> StorageResult<bool> {
    ctx.user_storage().grant_role(username, space_id, role)
}

pub(crate) fn revoke_role(
    ctx: &GraphStorageContext,
    username: &str,
    space_id: u64,
) -> StorageResult<bool> {
    ctx.user_storage().revoke_role(username, space_id)
}

pub(crate) fn change_password(
    ctx: &GraphStorageContext,
    info: &PasswordInfo,
) -> StorageResult<bool> {
    ctx.user_storage().change_password(info)
}

// ── Maintenance Operations ──

pub(crate) fn get_storage_stats(ctx: &GraphStorageContext) -> StorageStats {
    let total_vertices = ctx.total_vertex_count();
    let total_edges = ctx.total_edge_count();

    let spaces = ctx.schema_manager().list_spaces().unwrap_or_default();
    let tags = spaces
        .iter()
        .filter_map(|s| ctx.schema_manager().list_tags(&s.space_name).ok())
        .flatten()
        .count();

    let edge_types = spaces
        .iter()
        .filter_map(|s| ctx.schema_manager().list_edge_types(&s.space_name).ok())
        .flatten()
        .count();

    let total_size = ctx.storage_size() as u64;
    let data_size = ctx.used_storage_size() as u64;

    StorageStats {
        total_vertices,
        total_edges,
        total_spaces: spaces.len(),
        total_tags: tags,
        total_edge_types: edge_types,
        total_size_bytes: total_size,
        data_size_bytes: data_size,
        index_size_bytes: total_size.saturating_sub(data_size),
    }
}

pub(crate) fn find_dangling_edges(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<Vec<Edge>> {
    let _space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_read_timestamp();
    let mut dangling_edges = Vec::new();
    let edge_type_names: std::collections::HashMap<_, _> = ctx
        .schema_manager()
        .list_edge_types(space)?
        .into_iter()
        .map(|edge_type| (edge_type.edge_type_id, edge_type.edge_type_name))
        .collect();

    let edge_records = ctx.collect_all_edge_records(ts);
    for (src_label_id, dst_label_id, edge_label_id, record) in edge_records {
        let Some(edge_type_name) = edge_type_names.get(&edge_label_id) else {
            continue;
        };
        let src_exists = ctx
            .get_vertex_by_internal_id(
                src_label_id,
                record.src_vid.as_int64().unwrap_or(0) as u32,
                ts,
            )
            .is_some();
        let dst_exists = ctx
            .get_vertex_by_internal_id(
                dst_label_id,
                record.dst_vid.as_int64().unwrap_or(0) as u32,
                ts,
            )
            .is_some();

        if !src_exists || !dst_exists {
            let src_external = ctx
                .get_vertex_by_internal_id(
                    src_label_id,
                    record.src_vid.as_int64().unwrap_or(0) as u32,
                    ts,
                )
                .map(|vr| vr.vid)
                .or_else(|| {
                    ctx.get_external_id_by_internal_id(
                        src_label_id,
                        record.src_vid.as_int64().unwrap_or(0) as u32,
                    )
                })
                .unwrap_or(record.src_vid);
            let dst_external = ctx
                .get_vertex_by_internal_id(
                    dst_label_id,
                    record.dst_vid.as_int64().unwrap_or(0) as u32,
                    ts,
                )
                .map(|vr| vr.vid)
                .or_else(|| {
                    ctx.get_external_id_by_internal_id(
                        dst_label_id,
                        record.dst_vid.as_int64().unwrap_or(0) as u32,
                    )
                })
                .unwrap_or(record.dst_vid);
            let edge = edge_record_to_edge(
                &record,
                edge_type_name,
                &format!("{}", src_external),
                &format!("{}", dst_external),
            );
            dangling_edges.push(edge);
        }
    }

    Ok(dangling_edges)
}

pub(crate) fn repair_dangling_edges(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<usize> {
    let dangling_edges = find_dangling_edges(ctx, space)?;
    let mut repaired_count = 0;

    for edge in &dangling_edges {
        if writer::delete_edge(
            ctx,
            space,
            &edge.src,
            &edge.dst,
            &edge.edge_type,
            edge.ranking,
        )
        .is_ok()
        {
            repaired_count += 1;
        }
    }

    Ok(repaired_count)
}
