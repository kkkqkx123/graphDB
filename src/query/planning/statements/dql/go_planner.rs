//! GO Statement Planner
//! Planning for handling Nebula GO queries
//!
//! ## Improvement Notes
//!
//! Implement the complete logic for filtering expressions.
//! Improving the handling of JOIN operations
//! - Add support for attribute projection.

use crate::core::types::{ContextualExpression, EdgeDirection};
use crate::query::parser::ast::{GoStmt, Stmt};
use crate::query::planning::plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::QueryContext;
use std::sync::Arc;

pub use crate::query::planning::plan::core::nodes::{
    ArgumentNode, DedupNode, ExpandAllNode, FilterNode, GetNeighborsNode, HashInnerJoinNode,
    ProjectNode, StartNode,
};
pub use crate::query::planning::plan::core::PlanNodeEnum;

/// GO Query Planner
/// Responsible for converting GO statements into execution plans.
#[derive(Debug, Clone)]
pub struct GoPlanner {}

impl GoPlanner {
    /// Create a new GO planner.
    pub fn new() -> Self {
        Self {}
    }
}

impl Planner for GoPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let _ = qctx;

        let go_stmt = match validated.stmt() {
            Stmt::Go(go_stmt) => go_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "GoPlanner 需要 Go 语句".to_string(),
                ));
            }
        };

        // Use the verification information to optimize the planning process.
        let validation_info = &validated.validation_info;

        // 1. Check the optimization suggestions.
        for hint in &validation_info.optimization_hints {
            log::debug!("GO 优化提示: {:?}", hint);
        }

        // 2. Use the path analysis information
        for path_analysis in &validation_info.path_analysis {
            if path_analysis.edge_count > 5 {
                log::warn!(
                    "GO 路径包含 {} 条边，可能影响性能",
                    path_analysis.edge_count
                );
            }
        }

        // 3. Use semantic information
        let referenced_edges = &validation_info.semantic_info.referenced_edges;
        if !referenced_edges.is_empty() {
            log::debug!("GO 引用的边类型: {:?}", referenced_edges);
        }

        // Handle FROM clause - extract source vertex IDs
        let from_vertices = &go_stmt.from.vertices;
        if from_vertices.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "GO statement must have FROM clause".to_string(),
            ));
        }

        // Check if the first from expression is a literal (vertex ID)
        let first_from = &from_vertices[0];
        let (use_start_node, from_var) = if first_from.is_literal() {
            // If it's a literal like "1", we need to create a variable in context
            // Use StartNode as the tail and set the variable in execution context
            (true, "v".to_string())
        } else if let Some(var_name) = first_from.as_variable() {
            // If it's already a variable, use ArgumentNode
            (false, var_name.clone())
        } else {
            // For other expressions, use ArgumentNode with a default variable name
            (false, "v".to_string())
        };

        // Create the tail node
        let tail_node = if use_start_node {
            PlanNodeEnum::Start(StartNode::new())
        } else {
            PlanNodeEnum::Argument(ArgumentNode::new(0, &from_var))
        };

        let (direction_str, edge_types) = if let Some(over_clause) = &go_stmt.over {
            let direction_str = match over_clause.direction {
                EdgeDirection::Out => "out",
                EdgeDirection::In => "in",
                EdgeDirection::Both => "both",
            };
            (direction_str, over_clause.edge_types.clone())
        } else {
            ("both", vec![])
        };

        // Create ExpandAllNode to traverse edges
        let mut expand_all_node = ExpandAllNode::new(1, edge_types, direction_str);

        // Set step_limit to 1 for GO FROM (one step traversal)
        expand_all_node.set_step_limit(1);

        // Don't include empty paths for GO FROM queries
        expand_all_node.set_include_empty_paths(false);

        // Set src_vids from FROM clause if they are literals
        if use_start_node {
            let src_vids: Vec<crate::core::Value> = from_vertices
                .iter()
                .filter_map(|expr| expr.as_literal())
                .collect();
            if !src_vids.is_empty() {
                expand_all_node.set_src_vids(src_vids);
            }
        }

        let input_for_join = PlanNodeEnum::ExpandAll(expand_all_node);

        let filter_node = if let Some(ref condition) = go_stmt.where_clause {
            match FilterNode::new(input_for_join, condition.clone()) {
                Ok(filter) => PlanNodeEnum::Filter(filter),
                Err(e) => {
                    return Err(PlannerError::PlanGenerationFailed(format!(
                        "Failed to create filter node: {}",
                        e
                    )));
                }
            }
        } else {
            input_for_join
        };

        let project_columns = Self::build_yield_columns(go_stmt, validated.expr_context())?;
        let project_node = match ProjectNode::new(filter_node, project_columns) {
            Ok(project) => PlanNodeEnum::Project(project),
            Err(e) => {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "Failed to create project node: {}",
                    e
                )));
            }
        };

        let sub_plan = SubPlan {
            root: Some(project_node),
            tail: Some(tail_node),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Go(_))
    }
}

impl GoPlanner {
    /// Create the YIELD column
    fn build_yield_columns(
        go_stmt: &GoStmt,
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Result<Vec<crate::core::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        if let Some(ref yield_clause) = go_stmt.yield_clause {
            for item in &yield_clause.items {
                columns.push(crate::core::YieldColumn {
                    expression: item.expression.clone(),
                    alias: item.alias.clone().unwrap_or_default(),
                    is_matched: false,
                });
            }
        } else {
            let expr_meta = crate::core::types::expr::ExpressionMeta::new(
                crate::core::Expression::Variable("_expandall_dst".to_string()),
            );
            let id = expr_context.register_expression(expr_meta);
            let ctx_expr = ContextualExpression::new(id, expr_context.clone());
            columns.push(crate::core::YieldColumn {
                expression: ctx_expr,
                alias: "dst".to_string(),
                is_matched: false,
            });

            let expr_meta = crate::core::types::expr::ExpressionMeta::new(
                crate::core::Expression::Variable("_expandall_props".to_string()),
            );
            let id = expr_context.register_expression(expr_meta);
            let ctx_expr = ContextualExpression::new(id, expr_context.clone());
            columns.push(crate::core::YieldColumn {
                expression: ctx_expr,
                alias: "properties".to_string(),
                is_matched: false,
            });
        }

        if columns.is_empty() {
            let expr_meta = crate::core::types::expr::ExpressionMeta::new(
                crate::core::Expression::Variable("*".to_string()),
            );
            let id = expr_context.register_expression(expr_meta);
            let ctx_expr = ContextualExpression::new(id, expr_context.clone());
            columns.push(crate::core::YieldColumn {
                expression: ctx_expr,
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        Ok(columns)
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}
