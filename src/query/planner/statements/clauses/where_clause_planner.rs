//! WHERE 子句规划器
//!
//! 负责规划 WHERE 子句的执行，过滤输入数据。
//! 实现了 ClausePlanner 接口，提供完整的过滤功能。

use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::structs::CypherClauseKind;

/// WHERE 子句规划器
///
/// 负责规划 WHERE 子句的执行，过滤输入数据。
#[derive(Debug)]
pub struct WhereClausePlanner {
    filter_expression: Option<Expression>,
}

impl WhereClausePlanner {
    pub fn new() -> Self {
        Self {
            filter_expression: None,
        }
    }

    pub fn with_filter(filter_expression: Expression) -> Self {
        Self {
            filter_expression: Some(filter_expression),
        }
    }

    pub fn from_ast(ast_ctx: &AstContext) -> Self {
        let filter = extract_where_condition(ast_ctx);
        Self::with_filter(filter)
    }
}

fn extract_where_condition(ast_ctx: &AstContext) -> Expression {
    let stmt = ast_ctx.sentence();
    if let Some(crate::query::parser::ast::Stmt::Match(match_stmt)) = stmt {
        if let Some(where_expr) = &match_stmt.where_clause {
            return where_expr.clone();
        }
    }
    Expression::Variable("true".to_string())
}

impl ClausePlanner for WhereClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Where
    }

    fn name(&self) -> &'static str {
        "WhereClausePlanner"
    }

    fn transform_clause(
        &self,
        _query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let condition = self.filter_expression.clone()
            .or_else(|| extract_where_condition(ast_ctx).into())
            .unwrap_or_else(|| Expression::Variable("true".to_string()));

        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("WHERE 子句需要输入计划".to_string())
        })?;

        let filter_node = FilterNode::new(input_node.clone(), condition)?;
        Ok(SubPlan::new(Some(filter_node.into_enum()), input_plan.tail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_where_clause_planner_creation() {
        let planner = WhereClausePlanner::new();
        assert_eq!(planner.name(), "WhereClausePlanner");
        assert_eq!(planner.clause_kind(), CypherClauseKind::Where);
    }

    #[test]
    fn test_where_clause_planner_with_filter() {
        let expr = Expression::Variable("age".to_string());
        let planner = WhereClausePlanner::with_filter(expr);
        assert!(planner.filter_expression.is_some());
    }

    #[test]
    fn test_supports() {
        let planner = WhereClausePlanner::new();
        assert!(planner.supports(CypherClauseKind::Where));
        assert!(!planner.supports(CypherClauseKind::Return));
    }
}
