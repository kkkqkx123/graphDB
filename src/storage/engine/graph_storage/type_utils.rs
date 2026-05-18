//! Type Conversion Utilities
//!
//! Provides utility functions for converting between different data representations
//! used by the storage engine.

use std::collections::HashMap;

use crate::core::types::VertexId;
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, Value, Vertex};
use crate::storage::edge::EdgeRecord;
use crate::storage::vertex::VertexRecord;

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

    Edge {
        src: VertexId::from_string(src_id),
        dst: VertexId::from_string(dst_id),
        edge_type: edge_type.to_string(),
        ranking: 0,
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
