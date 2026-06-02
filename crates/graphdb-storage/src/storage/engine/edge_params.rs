use crate::core::types::LabelId;
use crate::storage::edge::EdgeStrategy;
use crate::storage::storage_types::StoragePropertyDef;

/// Parameters for creating an edge type
pub struct CreateEdgeTypeParams<'a> {
    pub name: &'a str,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub properties: Vec<StoragePropertyDef>,
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
    pub rank: i64,
}


