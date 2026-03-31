//! Deletion Operation Planner
//!
//! Query planning for handling DELETE VERTEX/EDGE/TAG statements

use crate::core::types::ContextualExpression;
use crate::query::parser::ast::{DeleteStmt, DeleteTarget, Stmt};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{DeleteEdgesNode, DeleteVerticesNode, EdgeDeleteInfo, VertexDeleteInfo},
};
use crate::query::planning::plan::{PlanNodeEnum, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// Deletion Operation Planner
/// Responsible for converting DELETE statements into execution plans.
#[derive(Debug, Clone)]
pub struct DeletePlanner;

impl DeletePlanner {
    /// Create a new deletion planner.
    pub fn new() -> Self {
        Self
    }

    /// Extract the DeleteStmt from the Stmt.
    fn extract_delete_stmt(&self, stmt: &Stmt) -> Result<DeleteStmt, PlannerError> {
        match stmt {
            Stmt::Delete(delete_stmt) => Ok(delete_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 DELETE".to_string(),
            )),
        }
    }
}

impl Planner for DeletePlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let _ = qctx;

        // Use the verification information to optimize the planning process.
        let validation_info = &validated.validation_info;

        // Check the semantic information.
        let referenced_tags = &validation_info.semantic_info.referenced_tags;
        if !referenced_tags.is_empty() {
            log::debug!("DELETE 引用的标签: {:?}", referenced_tags);
        }

        let referenced_edges = &validation_info.semantic_info.referenced_edges;
        if !referenced_edges.is_empty() {
            log::debug!("DELETE 引用的边类型: {:?}", referenced_edges);
        }

        let delete_stmt = self.extract_delete_stmt(validated.stmt())?;

        // Get current space name from query context or use default
        let space_name = qctx.space_name().unwrap_or_else(|| "default".to_string());

        // Create the appropriate delete node based on target type
        let final_node = match &delete_stmt.target {
            DeleteTarget::Vertices(vertex_ids) => {
                let info = VertexDeleteInfo {
                    space_name,
                    vertex_ids: vertex_ids.clone(),
                    with_edge: false, // TODO: support WITH EDGE option
                    condition: None,  // TODO: support WHERE clause
                };
                let node = DeleteVerticesNode::new(next_node_id(), info);
                PlanNodeEnum::DeleteVertices(node)
            }
            DeleteTarget::Edges { edge_type, edges } => {
                let info = EdgeDeleteInfo {
                    space_name,
                    edge_type: edge_type.clone(),
                    edges: edges
                        .iter()
                        .map(|(src, dst, rank)| (src.clone(), dst.clone(), rank.clone()))
                        .collect(),
                    condition: None, // TODO: support WHERE clause
                };
                let node = DeleteEdgesNode::new(next_node_id(), info);
                PlanNodeEnum::DeleteEdges(node)
            }
            DeleteTarget::Tags { .. } => {
                // TODO: Implement DELETE TAG
                return Err(PlannerError::PlanGenerationFailed(
                    "DELETE TAG not yet implemented".to_string(),
                ));
            }
            DeleteTarget::Index(..) => {
                // TODO: Implement DELETE INDEX
                return Err(PlannerError::PlanGenerationFailed(
                    "DELETE INDEX not yet implemented".to_string(),
                ));
            }
        };

        // Create a SubPlan
        let sub_plan = SubPlan::new(Some(final_node), None);

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Delete(_))
    }
}

impl Default for DeletePlanner {
    fn default() -> Self {
        Self::new()
    }
}
