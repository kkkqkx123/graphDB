//! Delete Operation Plan Nodes
//!
//! Provides plan nodes for DELETE VERTEX and DELETE EDGE operations.
//! Supports both standalone deletion and pipe-based deletion (e.g., GO ... | DELETE VERTEX $-.id).

use crate::core::types::expr::contextual::ContextualExpression;
use crate::define_plan_node_with_deps;

use super::info::{EdgeDeleteInfo, VertexDeleteInfo};

define_plan_node_with_deps! {
    /// Delete vertices node
    /// 
    /// Supports both:
    /// - Standalone: DELETE VERTEX "vid1", "vid2"
    /// - Pipe-based: GO FROM "vid" OVER edge YIELD dst(edge) AS id | DELETE VERTEX $-.id
    pub struct DeleteVerticesNode {
        info: VertexDeleteInfo,
    }
    enum: DeleteVertices
    input: SingleInputNode
}

impl DeleteVerticesNode {
    pub fn new(id: i64, info: VertexDeleteInfo) -> Self {
        Self {
            id,
            input: None,
            deps: vec![],
            info,
            output_var: None,
            col_names: vec!["deleted".to_string()],
        }
    }

    pub fn with_input(id: i64, info: VertexDeleteInfo, input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum) -> Self {
        Self {
            id,
            input: Some(Box::new(input.clone())),
            deps: vec![input],
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

    pub fn has_input(&self) -> bool {
        self.input.is_some()
    }
}

define_plan_node_with_deps! {
    /// Delete edges node
    /// 
    /// Supports both:
    /// - Standalone: DELETE EDGE edge_type "src" -> "dst"
    /// - Pipe-based: GO FROM "vid" OVER edge YIELD src(edge) AS s, dst(edge) AS d | DELETE EDGE type $-.s -> $-.d
    pub struct DeleteEdgesNode {
        info: EdgeDeleteInfo,
    }
    enum: DeleteEdges
    input: SingleInputNode
}

impl DeleteEdgesNode {
    pub fn new(id: i64, info: EdgeDeleteInfo) -> Self {
        Self {
            id,
            input: None,
            deps: vec![],
            info,
            output_var: None,
            col_names: vec!["deleted".to_string()],
        }
    }

    pub fn with_input(id: i64, info: EdgeDeleteInfo, input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum) -> Self {
        Self {
            id,
            input: Some(Box::new(input.clone())),
            deps: vec![input],
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

    pub fn edges(
        &self,
    ) -> &[(
        ContextualExpression,
        ContextualExpression,
        Option<ContextualExpression>,
    )] {
        &self.info.edges
    }

    pub fn condition(&self) -> Option<&ContextualExpression> {
        self.info.condition.as_ref()
    }

    pub fn has_input(&self) -> bool {
        self.input.is_some()
    }
}
