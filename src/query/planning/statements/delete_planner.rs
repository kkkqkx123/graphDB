//! Deletion Operation Planner
//!
//! Query planning for handling DELETE VERTEX/EDGE/TAG statements

use crate::core::types::ContextualExpression;
use crate::core::YieldColumn;
use crate::query::parser::ast::{DeleteStmt, DeleteTarget, Stmt};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, ProjectNode},
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

        // Create a parameter node as the input.
        let arg_node = ArgumentNode::new(next_node_id(), "delete_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // Develop different plans depending on the type of content that needs to be deleted.
        let yield_columns = match &delete_stmt.target {
            DeleteTarget::Vertices(..) => {
                let expr_meta = crate::core::types::expr::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_vertices".to_string()),
                );
                let id = validated.expr_context().register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Edges { .. } => {
                let expr_meta = crate::core::types::expr::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_edges".to_string()),
                );
                let id = validated.expr_context().register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Tags { .. } => {
                let expr_meta = crate::core::types::expr::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_tags".to_string()),
                );
                let id = validated.expr_context().register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Index(..) => {
                let expr_meta = crate::core::types::expr::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_index".to_string()),
                );
                let id = validated.expr_context().register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
        };

        // Create a projection node to output the deletion results.
        let project_node = ProjectNode::new(arg_node_enum.clone(), yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // Create a SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(arg_node_enum));

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
