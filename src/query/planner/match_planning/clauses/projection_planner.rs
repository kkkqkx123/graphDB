//! 投影规划器
//! 处理查询的投影操作

use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// 投影规划器
///
/// 负责规划查询结果的投影。
#[derive(Debug, Clone)]
pub struct ProjectionPlanner;

impl ProjectionPlanner {
    /// 创建新的投影规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建投影执行计划
    fn build_projection(
        &self,
        _projection_ctx: &crate::query::validator::ReturnClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 暂时简单实现：直接返回输入计划
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for ProjectionPlanner {
    fn name(&self) -> &'static str {
        "ProjectionPlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Return
    }
}

impl CypherClausePlanner for ProjectionPlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("Projection requires input".to_string())
        })?;

        // 验证上下文类型
        let return_ctx = match clause_ctx {
            CypherClauseContext::Return(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "ProjectionPlanner 只能处理 RETURN 子句上下文".to_string(),
                ))
            }
        };

        // 构建投影计划
        self.build_projection(return_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Return
    }
}

impl DataFlowNode for ProjectionPlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_planner_interface() {
        let planner = ProjectionPlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Return);
        assert_eq!(planner.name(), "ProjectionPlanner");
    }
}
