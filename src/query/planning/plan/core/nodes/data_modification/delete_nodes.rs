//! Delete Operation Plan Nodes
//!
//! Provides plan nodes for DELETE VERTEX and DELETE EDGE operations.

use crate::core::types::expr::contextual::ContextualExpression;
use crate::define_plan_node;

use super::info::{EdgeDeleteInfo, VertexDeleteInfo};

define_plan_node! {
    pub struct DeleteVerticesNode {
        info: VertexDeleteInfo,
    }
    enum: DeleteVertices
    input: ZeroInputNode
}

impl DeleteVerticesNode {
    pub fn new(id: i64, info: VertexDeleteInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: vec!["deleted".to_string()],
        }
    }

    pub fn info(&self) -> &VertexDeleteInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn vertex_ids(&self) -> &[ContextualExpression] {
        &self.info.vertex_ids
    }

    pub fn with_edge(&self) -> bool {
        self.info.with_edge
    }

    pub fn condition(&self) -> Option<&ContextualExpression> {
        self.info.condition.as_ref()
    }
}

define_plan_node! {
    pub struct DeleteEdgesNode {
        info: EdgeDeleteInfo,
    }
    enum: DeleteEdges
    input: ZeroInputNode
}

impl DeleteEdgesNode {
    pub fn new(id: i64, info: EdgeDeleteInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: vec!["deleted".to_string()],
        }
    }

    pub fn info(&self) -> &EdgeDeleteInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn edge_type(&self) -> Option<&str> {
        self.info.edge_type.as_deref()
    }

    pub fn edges(&self) -> &[(ContextualExpression, ContextualExpression, Option<ContextualExpression>)] {
        &self.info.edges
    }

    pub fn condition(&self) -> Option<&ContextualExpression> {
        self.info.condition.as_ref()
    }
}
