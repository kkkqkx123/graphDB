//! LIMIT/SKIP 子句规划器
//!
//! 负责规划 LIMIT 和 SKIP 子句的执行，实现结果分页。

use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::sort_node::LimitNode;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::planner::statements::match_planner::PaginationInfo;
use crate::query::validator::structs::CypherClauseKind;

/// LIMIT/SKIP 子句规划器
///
/// 负责规划 LIMIT 和 SKIP 子句的执行，实现结果分页。
#[derive(Debug, Default)]
pub struct PaginationPlanner;

impl PaginationPlanner {
    pub fn new() -> Self {
        Self
    }
}

fn extract_pagination_info(ast_ctx: &AstContext) -> PaginationInfo {
    let stmt = ast_ctx.sentence();
    if let Some(crate::query::parser::ast::Stmt::Match(match_stmt)) = stmt {
        let skip = match_stmt.skip.unwrap_or(0);
        let limit = match_stmt.limit.unwrap_or(100);
        return PaginationInfo { skip, limit };
    }
    PaginationInfo { skip: 0, limit: 100 }
}

impl ClausePlanner for PaginationPlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Pagination
    }

    fn name(&self) -> &'static str {
        "PaginationPlanner"
    }

    fn transform_clause(
        &self,
        _query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let pagination = extract_pagination_info(ast_ctx);

        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("LIMIT/SKIP 子句需要输入计划".to_string())
        })?;

        let limit_node = LimitNode::new(
            input_node.clone(),
            pagination.skip as i64,
            pagination.limit as i64,
        )?;
        Ok(SubPlan::new(Some(limit_node.into_enum()), input_plan.tail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_planner_creation() {
        let planner = PaginationPlanner::new();
        assert_eq!(planner.name(), "PaginationPlanner");
        assert_eq!(planner.clause_kind(), CypherClauseKind::Pagination);
    }

    #[test]
    fn test_supports() {
        let planner = PaginationPlanner::new();
        assert!(planner.supports(CypherClauseKind::Pagination));
        assert!(!planner.supports(CypherClauseKind::Where));
    }
}
