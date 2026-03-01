//! RETURN 子句规划器
//!
//! 负责规划 RETURN 子句的执行，实现结果投影。

use crate::core::types::{ContextualExpression, ExpressionContext};
use crate::core::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::data_processing_node::DedupNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::core::YieldColumn;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

pub use crate::query::planner::plan::core::PlanNodeEnum;

/// RETURN 子句规划器
///
/// 负责规划 RETURN 子句的执行，实现结果投影。
#[derive(Debug)]
pub struct ReturnClausePlanner {
    distinct: bool,
}

#[derive(Debug, Clone)]
pub struct ReturnItem {
    pub alias: String,
    pub expression: Expression,
    pub is_aggregated: bool,
}

impl ReturnClausePlanner {
    pub fn new() -> Self {
        Self {
            distinct: false,
        }
    }

    pub fn with_distinct(distinct: bool) -> Self {
        Self { distinct }
    }

    pub fn from_stmt(stmt: &Stmt) -> Self {
        let distinct = extract_distinct_flag(stmt);
        Self::with_distinct(distinct)
    }
}

fn extract_distinct_flag(stmt: &Stmt) -> bool {
    if let Stmt::Match(match_stmt) = stmt {
        if let Some(return_clause) = &match_stmt.return_clause {
            return return_clause.distinct;
        }
    }
    false
}

fn extract_return_columns(stmt: &Stmt) -> Vec<YieldColumn> {
    let mut columns = Vec::new();

    if let Stmt::Match(match_stmt) = stmt {
        if let Some(return_clause) = &match_stmt.return_clause {
            for item in &return_clause.items {
                match item {
                    crate::query::parser::ast::stmt::ReturnItem::Expression { expression, alias } => {
                        columns.push(YieldColumn {
                            expression: expression.clone(),
                            alias: alias.clone().unwrap_or_default(),
                            is_matched: false,
                        });
                    }
                    crate::query::parser::ast::stmt::ReturnItem::All => {
                        let ctx = Arc::new(ExpressionContext::new());
                        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                            crate::core::Expression::Variable("*".to_string())
                        );
                        let id = ctx.register_expression(expr_meta);
                        let ctx_expr = ContextualExpression::new(id, ctx);
                        columns.push(YieldColumn {
                            expression: ctx_expr,
                            alias: "*".to_string(),
                            is_matched: false,
                        });
                    }
                }
            }
        }
    }

    if columns.is_empty() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
            crate::core::Expression::Variable("*".to_string())
        );
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, ctx);
        columns.push(YieldColumn {
            expression: ctx_expr,
            alias: "*".to_string(),
            is_matched: false,
        });
    }

    columns
}

impl ClausePlanner for ReturnClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Return
    }

    fn transform_clause(
        &self,
        _qctx: Arc<QueryContext>,
        stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let yield_columns = extract_return_columns(stmt);

        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("RETURN 子句需要输入计划".to_string())
        })?;

        let project_node = ProjectNode::new(input_node.clone(), yield_columns)?;

        let final_node = if self.distinct {
            match DedupNode::new(project_node.clone().into_enum()) {
                Ok(dedup) => dedup.into_enum(),
                Err(_) => project_node.into_enum(),
            }
        } else {
            project_node.into_enum()
        };

        Ok(SubPlan::new(Some(final_node), input_plan.tail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_clause_planner_creation() {
        let planner = ReturnClausePlanner::new();
        assert_eq!(planner.clause_kind(), CypherClauseKind::Return);
    }

    #[test]
    fn test_return_clause_planner_with_distinct() {
        let planner = ReturnClausePlanner::with_distinct(true);
        assert!(planner.distinct);
    }
}
