//! ORDER BY 子句规划器
//!
//! 负责规划 ORDER BY 子句的执行，对结果进行排序。

use crate::core::types::ContextualExpression;
use crate::query::parser::ast::Stmt;
use crate::query::parser::OrderByItem;
use crate::query::planner::plan::core::nodes::base::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::operation::sort_node::{SortItem, SortNode};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::QueryContext;
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
    use crate::core::types::OrderDirection;
    use crate::core::Expression;
    use crate::query::parser::ast::{OrderByItem, Span};
    use crate::query::planner::plan::core::nodes::StartNode;
    use crate::query::planner::plan::core::PlanNodeEnum;
    use std::sync::Arc;
    use ExpressionAnalysisContext;

    #[test]
    fn test_order_by_clause_planner_creation() {
        let planner = OrderByClausePlanner::new();
        assert_eq!(planner.clause_kind(), CypherClauseKind::OrderBy);
    }

    #[test]
    fn test_extract_order_by_items() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("age".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: Some(crate::query::parser::ast::stmt::OrderByClause {
                span: Span::default(),
                items: vec![OrderByItem {
                    expression: ctx_expr.clone(),
                    direction: OrderDirection::Asc,
                }],
            }),
            limit: None,
            skip: None,
            optional: false,
        });

        let items = extract_order_by_items(&match_stmt);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].direction, OrderDirection::Asc);
    }

    #[test]
    fn test_extract_order_by_items_empty() {
        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let items = extract_order_by_items(&match_stmt);
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_expression_to_string() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("age".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let result = expression_to_string(&ctx_expr);
        assert_eq!(result, "age");
    }

    #[test]
    fn test_expression_to_string_complex() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "name".to_string(),
        };
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let result = expression_to_string(&ctx_expr);
        assert_eq!(result, "n.name");
    }

    #[test]
    fn test_transform_clause() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("age".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: Some(crate::query::parser::ast::stmt::OrderByClause {
                span: Span::default(),
                items: vec![OrderByItem {
                    expression: ctx_expr.clone(),
                    direction: OrderDirection::Asc,
                }],
            }),
            limit: None,
            skip: None,
            optional: false,
        });

        let start_node = StartNode::new();
        let start_node_enum = PlanNodeEnum::Start(start_node.clone());
        let input_plan = SubPlan {
            root: Some(start_node_enum.clone()),
            tail: Some(start_node_enum),
        };

        let planner = OrderByClausePlanner::new();
        let qctx = Arc::new(crate::query::QueryContext::new(Arc::new(
            crate::query::query_request_context::QueryRequestContext {
                session_id: None,
                user_name: None,
                space_name: None,
                query: String::new(),
                parameters: std::collections::HashMap::new(),
            },
        )));

        let result = planner.transform_clause(qctx, &match_stmt, input_plan);
        assert!(result.is_ok());

        let sub_plan = result.expect("transform_clause should succeed");
        assert!(sub_plan.root.is_some());

        if let Some(PlanNodeEnum::Sort(_)) = sub_plan.root {
        } else {
            panic!("Expected SortNode");
        }
    }

    #[test]
    fn test_transform_clause_empty_order_by() {
        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let start_node = StartNode::new();
        let start_node_enum = PlanNodeEnum::Start(start_node.clone());
        let input_plan = SubPlan {
            root: Some(start_node_enum.clone()),
            tail: Some(start_node_enum),
        };

        let planner = OrderByClausePlanner::new();
        let qctx = Arc::new(crate::query::QueryContext::new(Arc::new(
            crate::query::query_request_context::QueryRequestContext {
                session_id: None,
                user_name: None,
                space_name: None,
                query: String::new(),
                parameters: std::collections::HashMap::new(),
            },
        )));

        let result = planner.transform_clause(qctx, &match_stmt, input_plan);
        assert!(result.is_ok());

        let sub_plan = result.expect("transform_clause should succeed");
        assert!(sub_plan.root.is_some());
    }

    #[test]
    fn test_transform_clause_empty_input_plan() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("age".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: Some(crate::query::parser::ast::stmt::OrderByClause {
                span: Span::default(),
                items: vec![OrderByItem {
                    expression: ctx_expr.clone(),
                    direction: OrderDirection::Asc,
                }],
            }),
            limit: None,
            skip: None,
            optional: false,
        });

        let input_plan = SubPlan {
            root: None,
            tail: None,
        };

        let planner = OrderByClausePlanner::new();
        let qctx = Arc::new(crate::query::QueryContext::new(Arc::new(
            crate::query::query_request_context::QueryRequestContext {
                session_id: None,
                user_name: None,
                space_name: None,
                query: String::new(),
                parameters: std::collections::HashMap::new(),
            },
        )));

        let result = planner.transform_clause(qctx, &match_stmt, input_plan);
        assert!(result.is_err());
    }
}
