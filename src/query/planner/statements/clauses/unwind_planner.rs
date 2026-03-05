//! UNWIND 子句规划器
//!
//! 负责规划 UNWIND 子句的执行，将列表展开为多行。

use crate::core::types::ContextualExpression;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::core::nodes::data_processing_node::UnwindNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::QueryContext;
use std::sync::Arc;

/// UNWIND 子句规划器
///
/// 负责将 UNWIND 子句转换为执行计划节点。
/// UNWIND 语法：UNWIND [expression] AS [variable]
#[derive(Debug)]
pub struct UnwindClausePlanner;

impl UnwindClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl ClausePlanner for UnwindClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Unwind
    }

    fn transform_clause(
        &self,
        _qctx: Arc<QueryContext>,
        stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let (expression, variable) = extract_unwind_info(stmt)?;

        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("UNWIND 子句需要输入计划".to_string())
        })?;

        let unwind_node = UnwindNode::new(input_node.clone(), &variable, expression)?;
        Ok(SubPlan::new(Some(unwind_node.into_enum()), input_plan.tail))
    }
}

/// 从语句中提取 UNWIND 子句信息
fn extract_unwind_info(stmt: &Stmt) -> Result<(ContextualExpression, String), PlannerError> {
    if let Stmt::Unwind(unwind_stmt) = stmt {
        return Ok((unwind_stmt.expression.clone(), unwind_stmt.variable.clone()));
    }
    Err(PlannerError::PlanGenerationFailed(
        "期望 UNWIND 语句，但得到了其他类型的语句".to_string(),
    ))
}

impl Default for UnwindClausePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwind_clause_planner_creation() {
        let planner = UnwindClausePlanner::new();
        assert_eq!(planner.clause_kind(), CypherClauseKind::Unwind);
    }

    #[test]
    fn test_extract_unwind_info() {
        use crate::core::Expression;
        use crate::query::parser::ast::Span;
        use std::sync::Arc;
        use ExpressionAnalysisContext;

        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::List(vec![]);
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let unwind_stmt = Stmt::Unwind(crate::query::parser::ast::stmt::UnwindStmt {
            span: Span::default(),
            expression: ctx_expr.clone(),
            variable: "x".to_string(),
        });

        let (_expr, var) = extract_unwind_info(&unwind_stmt).expect("提取失败");
        assert_eq!(var, "x");
    }

    #[test]
    fn test_extract_unwind_info_invalid_stmt() {
        use crate::query::parser::ast::Span;

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

        let result = extract_unwind_info(&match_stmt);
        assert!(result.is_err());
    }
}
