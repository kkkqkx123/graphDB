//! RETURN 子句规划器
//!
//! 负责规划 RETURN 子句的执行，实现结果投影。

use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::data_processing_node::DedupNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::YieldColumn;
use crate::query::validator::structs::CypherClauseKind;

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

    pub fn from_ast(ast_ctx: &AstContext) -> Self {
        Self::new()
    }

    fn is_aggregated_expression(name: &str) -> bool {
        let upper = name.to_uppercase();
        upper.starts_with("COUNT(")
            || upper.starts_with("SUM(")
            || upper.starts_with("AVG(")
            || upper.starts_with("MAX(")
            || upper.starts_with("MIN(")
            || upper.starts_with("COLLECT(")
    }
}

fn extract_return_columns(ast_ctx: &AstContext) -> Vec<YieldColumn> {
    let mut columns = Vec::new();
    let stmt = ast_ctx.sentence();

    if let Some(crate::query::parser::ast::Stmt::Match(match_stmt)) = stmt {
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
                        columns.push(YieldColumn {
                            expression: Expression::Variable("*".to_string()),
                            alias: "*".to_string(),
                            is_matched: false,
                        });
                    }
                }
            }
        }
    }

    if columns.is_empty() {
        columns.push(YieldColumn {
            expression: Expression::Variable("*".to_string()),
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

    fn name(&self) -> &'static str {
        "ReturnClausePlanner"
    }

    fn transform_clause(
        &self,
        _query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let yield_columns = extract_return_columns(ast_ctx);

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
        assert_eq!(planner.name(), "ReturnClausePlanner");
        assert_eq!(planner.clause_kind(), CypherClauseKind::Return);
    }

    #[test]
    fn test_return_clause_planner_with_distinct() {
        let planner = ReturnClausePlanner::with_distinct(true);
        assert!(planner.distinct);
    }

    #[test]
    fn test_supports() {
        let planner = ReturnClausePlanner::new();
        assert!(planner.supports(CypherClauseKind::Return));
        assert!(!planner.supports(CypherClauseKind::Where));
    }

    #[test]
    fn test_is_aggregated_expression() {
        assert!(ReturnClausePlanner::is_aggregated_expression("COUNT(*)"));
        assert!(ReturnClausePlanner::is_aggregated_expression("SUM(x)"));
        assert!(!ReturnClausePlanner::is_aggregated_expression("name"));
    }
}
