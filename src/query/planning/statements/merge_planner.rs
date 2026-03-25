//! Merge Operation Planner
//!
//! Query planning for handling MERGE statements

use crate::core::types::ContextualExpression;
use crate::core::YieldColumn;
use crate::query::parser::ast::{MergeStmt, Stmt};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, ProjectNode},
};
use crate::query::planning::plan::{PlanNodeEnum, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// Merge Operation Planner
/// Responsible for converting MERGE statements into execution plans.
#[derive(Debug, Clone)]
pub struct MergePlanner;

impl MergePlanner {
    /// Create a new merge planner.
    pub fn new() -> Self {
        Self
    }

    /// Extract the MergeStmt from the Stmt.
    fn extract_merge_stmt(&self, stmt: &Stmt) -> Result<MergeStmt, PlannerError> {
        match stmt {
            Stmt::Merge(merge_stmt) => Ok(merge_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 MERGE".to_string(),
            )),
        }
    }
}

impl Planner for MergePlanner {
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
            log::debug!("MERGE 引用的标签: {:?}", referenced_tags);
        }

        let referenced_edges = &validation_info.semantic_info.referenced_edges;
        if !referenced_edges.is_empty() {
            log::debug!("MERGE 引用的边类型: {:?}", referenced_edges);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("MERGE 引用的属性: {:?}", referenced_properties);
        }

        let _merge_stmt = self.extract_merge_stmt(validated.stmt())?;

        // Create a parameter node as the input.
        let arg_node = ArgumentNode::new(next_node_id(), "merge_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // Construct the output column – Return the merged result
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(
            crate::core::Expression::Variable("merged_count".to_string()),
        );
        let id = validated.expr_context().register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());

        let yield_columns = vec![YieldColumn {
            expression: ctx_expr,
            alias: "merged_count".to_string(),
            is_matched: false,
        }];

        // Create a projection node to output the combined result.
        let project_node = ProjectNode::new(arg_node_enum.clone(), yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // Create a SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(arg_node_enum));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Merge(_))
    }
}

impl Default for MergePlanner {
    fn default() -> Self {
        Self::new()
    }
}
