use crate::core::types::LabelId;
use crate::storage::edge::{PropertyDef as EdgePropertyDef, EdgeStrategy};

/// Parameters for creating an edge type
pub struct CreateEdgeTypeParams<'a> {
    pub name: &'a str,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub properties: Vec<EdgePropertyDef>,
    pub oe_strategy: EdgeStrategy,
    pub ie_strategy: EdgeStrategy,
}

/// Parameters for edge operations that need vertex/edge labels and IDs
pub struct EdgeOperationParams<'a> {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub src_id: &'a str,
    pub dst_label: LabelId,
    pub dst_id: &'a str,
}

/// Parameters for edge traversal operations
pub struct EdgeTraversalParams {
    pub edge_label: LabelId,
    pub src_label: LabelId,
    pub dst_label: LabelId,
}
