//! WAL Redo Log Types
//!
//! Redo log entry types for WAL replay during recovery.

use serde::{Deserialize, Serialize};

use crate::core::types::LabelId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertVertexRedo {
    pub label: LabelId,
    pub oid: Vec<u8>,
    pub properties: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertEdgeRedo {
    pub src_label: LabelId,
    pub src_oid: Vec<u8>,
    pub dst_label: LabelId,
    pub dst_oid: Vec<u8>,
    pub edge_label: LabelId,
    pub properties: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVertexPropRedo {
    pub label: LabelId,
    pub oid: Vec<u8>,
    pub prop_name: String,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEdgePropRedo {
    pub src_label: LabelId,
    pub src_oid: Vec<u8>,
    pub dst_label: LabelId,
    pub dst_oid: Vec<u8>,
    pub edge_label: LabelId,
    pub prop_name: String,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVertexTypeRedo {
    pub label_name: String,
    pub schema: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEdgeTypeRedo {
    pub src_label: String,
    pub dst_label: String,
    pub edge_label: String,
    pub schema: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteVertexRedo {
    pub label: LabelId,
    pub oid: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEdgeRedo {
    pub src_label: LabelId,
    pub src_oid: Vec<u8>,
    pub dst_label: LabelId,
    pub dst_oid: Vec<u8>,
    pub edge_label: LabelId,
}