//! Type Conversion Utilities
//!
//! Provides utility functions for converting between different data representations
//! used by the storage engine.

use std::collections::HashMap;

use crate::core::types::{LabelId, VertexId};
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, StorageResult, Value, Vertex};
use crate::storage::edge::EdgeRecord;
use crate::storage::vertex::VertexRecord;

use super::context::GraphStorageContext;

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
        .schema_manager
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
        .schema_manager
        .get_edge_type(space, edge_type_name)?
        .map(|edge_type| edge_type.edge_type_id))
}

pub fn vertex_id_to_string(vid: &VertexId) -> String {
    if let Some(i) = vid.as_int64() {
        i.to_string()
    } else if let Some(s) = vid.as_str() {
        s.to_string()
    } else {
        format!("{:?}", vid)
    }
}

pub fn value_to_string(value: &Value) -> String {
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

pub fn vertex_record_to_vertex(record: &VertexRecord, tag_name: &str) -> Vertex {
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

pub fn edge_record_to_edge(
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

pub fn serialize_properties(props: &[(String, Value)]) -> Vec<u8> {
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
