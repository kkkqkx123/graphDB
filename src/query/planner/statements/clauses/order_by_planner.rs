//! ORDER BY 子句规划器
//!
//! 负责规划 ORDER BY 子句的执行，对结果进行排序。

use crate::core::types::ContextualExpression;
use crate::core::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::sort_node::{SortNode, SortItem};
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::parser::OrderByItem;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

/// ORDER BY 子句规划器
///
/// 负责规划 ORDER BY 子句的执行，对结果进行排序。
#[derive(Debug)]
pub struct OrderByClausePlanner {}

impl OrderByClausePlanner {
    pub fn new() -> Self {
        Self {}
    }
}

fn extract_order_by_items(stmt: &Stmt) -> Vec<OrderByItem> {
    if let Stmt::Match(match_stmt) = stmt {
        if let Some(order_by_clause) = &match_stmt.order_by {
            return order_by_clause.items.clone();
        }
    }
    Vec::new()
}

/// 将表达式转换为字符串表示
/// 
/// 使用 Expression::to_expression_string() 方法
fn expression_to_string(expr: &ContextualExpression) -> String {
    if let Some(expr_meta) = expr.expression() {
        expr_meta.inner().to_expression_string()
    } else {
        String::new()
    }
}

impl ClausePlanner for OrderByClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::OrderBy
    }

    fn transform_clause(
        &self,
        _qctx: Arc<QueryContext>,
        stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let order_by_items = extract_order_by_items(stmt);

        if order_by_items.is_empty() {
            return Ok(input_plan);
        }

        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("ORDER BY 子句需要输入计划".to_string())
        })?;

        let sort_items: Vec<SortItem> = order_by_items
            .into_iter()
            .map(|item| {
                let column = expression_to_string(&item.expression);
                SortItem::new(column, item.direction)
            })
            .collect();

        let sort_node = SortNode::new(input_node.clone(), sort_items)?;
        Ok(SubPlan::new(Some(sort_node.into_enum()), input_plan.tail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_by_clause_planner_creation() {
        let planner = OrderByClausePlanner::new();
        assert_eq!(planner.clause_kind(), CypherClauseKind::OrderBy);
    }
}
