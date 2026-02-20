//! ORDER BY 子句规划器
//!
//! 负责规划 ORDER BY 子句的执行，对结果进行排序。

use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::sort_node::{SortNode, SortItem};
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::structs::OrderByItem;
use crate::query::validator::structs::CypherClauseKind;
use crate::core::types::graph_schema::OrderDirection;

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

fn extract_order_by_items(ast_ctx: &AstContext) -> Vec<OrderByItem> {
    let stmt = ast_ctx.sentence();
    if let Some(crate::query::parser::ast::Stmt::Match(match_stmt)) = stmt {
        if let Some(order_by_clause) = &match_stmt.order_by {
            return order_by_clause.items.iter().map(|item| {
                OrderByItem {
                    expression: item.expression.clone(),
                    desc: item.direction == crate::query::parser::ast::types::OrderDirection::Desc,
                }
            }).collect();
        }
    }
    Vec::new()
}

/// 将表达式转换为字符串表示
/// 
/// 使用 Expression::to_expression_string() 方法
fn expression_to_string(expr: &Expression) -> String {
    expr.to_expression_string()
}

impl ClausePlanner for OrderByClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::OrderBy
    }

    fn name(&self) -> &'static str {
        "OrderByClausePlanner"
    }

    fn transform_clause(
        &self,
        _query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let order_by_items = extract_order_by_items(ast_ctx);

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
                let direction = if item.desc { OrderDirection::Desc } else { OrderDirection::Asc };
                SortItem::new(column, direction)
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
        assert_eq!(planner.name(), "OrderByClausePlanner");
        assert_eq!(planner.clause_kind(), CypherClauseKind::OrderBy);
    }

    #[test]
    fn test_supports() {
        let planner = OrderByClausePlanner::new();
        assert!(planner.supports(CypherClauseKind::OrderBy));
        assert!(!planner.supports(CypherClauseKind::Where));
    }
}
