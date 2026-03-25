//! Update Operation Planner
//!
//! Query planning for processing UPDATE VERTEX/EDGE statements

use crate::core::types::ContextualExpression;
use crate::core::YieldColumn;
use crate::query::parser::ast::{Stmt, UpdateStmt, UpdateTarget};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, ProjectNode},
};
use crate::query::planning::plan::{PlanNodeEnum, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// Update Operation Planner
/// Responsible for converting UPDATE statements into execution plans.
#[derive(Debug, Clone)]
pub struct UpdatePlanner;

impl UpdatePlanner {
    /// Create a new update planner.
    pub fn new() -> Self {
        Self
    }

    /// Extract the UpdateStmt from the Stmt.
    fn extract_update_stmt(&self, stmt: &Stmt) -> Result<UpdateStmt, PlannerError> {
        match stmt {
            Stmt::Update(update_stmt) => Ok(update_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 UPDATE".to_string(),
            )),
        }
    }
}

impl Planner for UpdatePlanner {
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
            log::debug!("UPDATE 引用的标签: {:?}", referenced_tags);
        }

        let referenced_edges = &validation_info.semantic_info.referenced_edges;
        if !referenced_edges.is_empty() {
            log::debug!("UPDATE 引用的边类型: {:?}", referenced_edges);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("UPDATE 引用的属性: {:?}", referenced_properties);
        }

        let update_stmt = self.extract_update_stmt(validated.stmt())?;

        // Create a parameter node as the input.
        let arg_node = ArgumentNode::new(next_node_id(), "update_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // Construct the output column based on the type of update target.
        let target_name = match &update_stmt.target {
            UpdateTarget::Vertex(..) => "vertex",
            UpdateTarget::Edge { .. } => "edge",
            UpdateTarget::Tag(..) => "tag",
            UpdateTarget::TagOnVertex { .. } => "vertex_tag",
        };

        let expr_meta = crate::core::types::expr::ExpressionMeta::new(
            crate::core::Expression::Variable(format!("updated_{}", target_name)),
        );
        let id = validated.expr_context().register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());

        let yield_columns = vec![YieldColumn {
            expression: ctx_expr,
            alias: "updated_count".to_string(),
            is_matched: false,
        }];

        // Create a projection node to output the updated results.
        let project_node = ProjectNode::new(arg_node_enum.clone(), yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // Create a SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(arg_node_enum));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Update(_))
    }
}

impl Default for UpdatePlanner {
    fn default() -> Self {
        Self::new()
    }
}
