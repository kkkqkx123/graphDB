//! Type Conversion Helpers
//!
//! Provides utility functions for converting between different data representations
//! used by the storage engine.

use std::collections::HashMap;

use crate::core::{Edge, Value, Vertex};
use crate::core::vertex_edge_path::Tag;
use crate::storage::edge::EdgeRecord;
use crate::storage::vertex::VertexRecord;

pub fn value_to_string(id: &Value) -> String {
    match id {
        Value::String(s) => s.clone(),
        _ => id.to_string().unwrap_or_default(),
    }
}

pub fn vertex_record_to_vertex(record: &VertexRecord, tag_name: &str) -> Vertex {
    let vid_value = Value::String(record.vid.to_string());
    let properties: HashMap<String, Value> = record.properties.iter().cloned().collect();

    Vertex {
        vid: Box::new(vid_value),
        id: record.internal_id as i64,
        tags: vec![Tag {
            name: tag_name.to_string(),
            properties: properties.clone(),
        }],
        properties,
    }
}

pub fn edge_record_to_edge(record: &EdgeRecord, edge_type: &str, src_id: &str, dst_id: &str) -> Edge {
    let props: HashMap<String, Value> = record.properties.iter().cloned().collect();

    Edge {
        src: Box::new(Value::String(src_id.to_string())),
        dst: Box::new(Value::String(dst_id.to_string())),
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
