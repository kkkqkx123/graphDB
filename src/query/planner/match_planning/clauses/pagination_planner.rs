//! PAGINATION 子句规划器
//! 处理LIMIT和SKIP的规划

use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// PAGINATION 子句规划器
///
/// 负责规划分页操作，包括 LIMIT 和 OFFSET/SKIP。
#[derive(Debug, Clone)]
pub struct PaginationPlanner;

impl PaginationPlanner {
    /// 创建新的PAGINATION规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建分页节点
    fn build_pagination(
        &self,
        pagination_ctx: &crate::query::validator::PaginationContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证分页参数
        if let Some(skip) = pagination_ctx.skip {
            if skip == 0 && pagination_ctx.limit.is_none() {
                // 如果skip为0且没有limit，则不需要分页
                return Ok(input_plan.clone());
            }
        }

        // 暂时简单实现：直接返回输入计划
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for PaginationPlanner {
    fn name(&self) -> &'static str {
        "PaginationPlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Pagination
    }
}

impl CypherClausePlanner for PaginationPlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("PAGINATION clause requires input".to_string())
        })?;

        // 验证上下文类型
        let pagination_ctx = match clause_ctx {
            CypherClauseContext::Pagination(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "PaginationPlanner 只能处理 Pagination 子句上下文".to_string(),
                ))
            }
        };

        // 构建分页计划
        self.build_pagination(pagination_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Limit
    }
}

impl DataFlowNode for PaginationPlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_planner_interface() {
        let planner = PaginationPlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Limit);
        assert_eq!(planner.name(), "PaginationPlanner");
    }
}
