//! WHERE 子句规划器
//!
//! 负责规划 WHERE 子句的执行，过滤输入数据。
//! 实现了 ClausePlanner 接口，提供完整的过滤功能。

use crate::core::types::ContextualExpression;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::QueryContext;
use std::sync::Arc;

/// WHERE 子句规划器
///
/// 负责规划 WHERE 子句的执行，过滤输入数据。
#[derive(Debug)]
pub struct WhereClausePlanner;

impl WhereClausePlanner {
    pub fn new() -> Self {
        Self
    }
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
        let condition = extract_where_condition(stmt)?;

        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("WHERE 子句需要输入计划".to_string())
        })?;

        let filter_node = FilterNode::new(input_node.clone(), condition)?;
        Ok(SubPlan::new(Some(filter_node.into_enum()), input_plan.tail))
    }
}

fn extract_where_condition(stmt: &Stmt) -> Result<ContextualExpression, PlannerError> {
    if let Stmt::Match(match_stmt) = stmt {
        if let Some(ref where_expr) = match_stmt.where_clause {
            return Ok(where_expr.clone());
        }
    }
    Err(PlannerError::PlanGenerationFailed(
        "WHERE 子句应该在 Parser 层创建默认表达式".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::parser::ast::Span;
    use crate::query::planner::plan::core::nodes::StartNode;
    use crate::query::planner::plan::core::PlanNodeEnum;
    use std::sync::Arc;
    use ExpressionAnalysisContext;

    #[test]
    fn test_where_clause_planner_creation() {
        let planner = WhereClausePlanner::new();
        assert_eq!(planner.clause_kind(), CypherClauseKind::Where);
    }

    #[test]
    fn test_extract_where_condition() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("age".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: Some(ctx_expr.clone()),
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let condition = extract_where_condition(&match_stmt).expect("提取失败");
        assert_eq!(condition.id(), ctx_expr.id());
    }

    #[test]
    fn test_extract_where_condition_none() {
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

        let result = extract_where_condition(&match_stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_clause() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("age".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: Some(ctx_expr),
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

        let planner = WhereClausePlanner::new();
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

        if let Some(PlanNodeEnum::Filter(_)) = sub_plan.root {
        } else {
            panic!("Expected FilterNode");
        }
    }

    #[test]
    fn test_transform_clause_invalid_stmt() {
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

        let planner = WhereClausePlanner::new();
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
