//! WHERE 子句规划器
//!
//! 负责规划 WHERE 子句的执行，过滤输入数据。
//! 实现了 ClausePlanner 接口，提供完整的过滤功能。

use crate::core::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

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

    pub fn from_stmt(stmt: &Stmt) -> Self {
        let filter = extract_where_condition(stmt);
        Self::with_filter(filter)
    }
}

fn extract_where_condition(stmt: &Stmt) -> Expression {
    if let Stmt::Match(match_stmt) = stmt {
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

    fn transform_clause(
        &self,
        _qctx: Arc<QueryContext>,
        stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let condition = self.filter_expression.clone()
            .or_else(|| Some(extract_where_condition(stmt)))
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
        assert_eq!(planner.clause_kind(), CypherClauseKind::Where);
    }

    #[test]
    fn test_where_clause_planner_with_filter() {
        let expr = Expression::Variable("age".to_string());
        let planner = WhereClausePlanner::with_filter(expr);
        assert!(planner.filter_expression.is_some());
    }
}
