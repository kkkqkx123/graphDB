//! The FETCH EDGES query planner
//! Planning for the execution of the FETCH EDGES query

use crate::core::types::expr::common_utils::extract_string_from_expr;
use crate::query::parser::ast::{FetchTarget, Stmt};
use crate::query::planning::plan::core::nodes::{
    ArgumentNode, FilterNode, GetEdgesNode, ProjectNode,
};
use crate::query::planning::plan::core::PlanNodeEnum;
use crate::query::planning::plan::execution_plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// The FETCH EDGES query planner
/// Responsible for converting the FETCH EDGES query into an execution plan.
#[derive(Debug, Clone)]
pub struct FetchEdgesPlanner;

impl FetchEdgesPlanner {
    /// Create a new FETCH EDGES planner.
    pub fn new() -> Self {
        Self
    }
}

impl Planner for FetchEdgesPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let _ = qctx;

        let fetch_stmt = match validated.stmt() {
            Stmt::Fetch(fetch_stmt) => fetch_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchEdgesPlanner 需要 Fetch 语句".to_string(),
                ));
            }
        };

        // Check whether it is "FETCH EDGES".
        let (src, dst, edge_type, rank) = match &fetch_stmt.target {
            FetchTarget::Edges {
                src,
                dst,
                edge_type,
                rank,
                ..
            } => (src, dst, edge_type, rank),
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchEdgesPlanner 需要 FETCH EDGES 语句".to_string(),
                ));
            }
        };

        let var_name = "e";

        // 1. Create a parameter node to define the conditions for obtaining the edges.
        let arg_node = ArgumentNode::new(1, var_name);

        // Extract string values from the expression.
        let src_str = extract_string_from_expr(src)?;
        let dst_str = extract_string_from_expr(dst)?;
        let rank_str = rank
            .as_ref()
            .map(extract_string_from_expr)
            .transpose()?
            .unwrap_or_else(|| "0".to_string());

        // 2. Create nodes for retrieving the edges.
        let get_edges_node = PlanNodeEnum::GetEdges(GetEdgesNode::new(
            1, // space_id
            &src_str, edge_type, &rank_str, &dst_str,
        ));

        // 3. Create nodes that filter out empty edges.
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(
            crate::core::Expression::Variable(format!("{} IS NOT EMPTY", var_name)),
        );
        let id = validated.expr_context().register_expression(expr_meta);
        let ctx_expr =
            crate::core::types::ContextualExpression::new(id, validated.expr_context().clone());
        let filter_node = match FilterNode::new(get_edges_node.clone(), ctx_expr) {
            Ok(node) => PlanNodeEnum::Filter(node),
            Err(_) => get_edges_node.clone(),
        };

        // 4. Create a projection node.
        let project_node = match ProjectNode::new(filter_node.clone(), vec![]) {
            Ok(node) => PlanNodeEnum::Project(node),
            Err(e) => {
                println!("Failed to create project node: {:?}", e);
                filter_node
            }
        };

        // 5. Create a SubPlan
        let arg_node = PlanNodeEnum::Argument(arg_node);
        let sub_plan = SubPlan::new(Some(project_node), Some(arg_node));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Fetch(fetch_stmt) => {
                matches!(&fetch_stmt.target, FetchTarget::Edges { .. })
            }
            _ => false,
        }
    }
}

impl Default for FetchEdgesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
