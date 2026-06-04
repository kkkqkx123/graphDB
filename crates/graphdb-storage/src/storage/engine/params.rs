use crate::core::types::LabelId;
use crate::core::types::Timestamp;
use crate::core::Value;
use crate::storage::edge::EdgeStrategy;
use crate::storage::types::StoragePropertyDef;

/// Parameters for creating an edge type
pub struct CreateEdgeTypeParams<'a> {
    pub name: &'a str,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub properties: Vec<StoragePropertyDef>,
    pub oe_strategy: EdgeStrategy,
    pub ie_strategy: EdgeStrategy,
}

/// Parameters for edge operations that need vertex/edge labels and IDs (string-based)
pub struct EdgeOperationParams<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: &'a str,
    pub dst_label: LabelId,
    pub dst_id: &'a str,
    pub rank: i64,
}

/// Parameters for edge operations with i64 vertex IDs
pub struct EdgeOperationParamsByI64 {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: i64,
    pub dst_label: LabelId,
    pub dst_id: i64,
    pub rank: i64,
}

/// Parameters for insert_edge operation.
pub struct InsertEdgeParams<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: &'a str,
    pub dst_label: LabelId,
    pub dst_id: &'a str,
    pub rank: i64,
    pub properties: &'a [(String, Value)],
    pub ts: Timestamp,
}

/// Parameters for insert_edge operation with i64 vertex IDs.
pub struct InsertEdgeParamsByI64<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: i64,
    pub dst_label: LabelId,
    pub dst_id: i64,
    pub rank: i64,
    pub properties: &'a [(String, Value)],
    pub ts: Timestamp,
}

/// Parameters for update_edge_property operation in PropertyGraph.
pub struct PropertyGraphUpdateEdgePropertyParams<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: &'a str,
    pub dst_label: LabelId,
    pub dst_id: &'a str,
    pub rank: i64,
    pub prop_name: &'a str,
    pub value: &'a Value,
    pub ts: Timestamp,
}
