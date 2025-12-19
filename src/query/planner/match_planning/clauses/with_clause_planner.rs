//! WITH 子句规划器
//! 处理WITH子句的规划

use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// WITH 子句规划器
///
/// 负责规划WITH子句的执行。WITH子句允许中间结果投影，用于查询中的流水处理。
#[derive(Debug, Clone)]
pub struct WithClausePlanner;

impl WithClausePlanner {
    /// 创建新的WITH子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建WITH子句的执行计划
    fn build_with(
        &self,
        _with_ctx: &crate::query::validator::WithClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 暂时简单实现：直接返回输入计划
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for WithClausePlanner {
    fn name(&self) -> &'static str {
        "WithClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::With
    }
}

impl CypherClausePlanner for WithClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("WITH clause requires input".to_string())
        })?;

        // 验证上下文类型
        let with_ctx = match clause_ctx {
            CypherClauseContext::With(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "WithClausePlanner 只能处理 WITH 子句上下文".to_string(),
                ))
            }
        };

        // 构建WITH子句的执行计划
        self.build_with(with_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::With
    }
}

impl DataFlowNode for WithClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_clause_planner_interface() {
        let planner = WithClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::With);
        assert_eq!(planner.name(), "WithClausePlanner");
    }
}
