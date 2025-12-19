//! UNWIND 子句规划器
//! 处理UNWIND子句的规划

use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// UNWIND 子句规划器
///
/// 负责规划UNWIND子句的执行。UNWIND子句将列表中的元素展开为单独的行。
#[derive(Debug, Clone)]
pub struct UnwindClausePlanner;

impl UnwindClausePlanner {
    /// 创建新的UNWIND子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建UNWIND子句的执行计划
    fn build_unwind(
        &self,
        _unwind_ctx: &crate::query::validator::UnwindClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 暂时简单实现：直接返回输入计划
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for UnwindClausePlanner {
    fn name(&self) -> &'static str {
        "UnwindClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Unwind
    }
}

impl CypherClausePlanner for UnwindClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // UNWIND 可能没有输入计划，但暂时要求有
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("UNWIND clause requires input".to_string())
        })?;

        // 验证上下文类型
        let unwind_ctx = match clause_ctx {
            CypherClauseContext::Unwind(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "UnwindClausePlanner 只能处理 UNWIND 子句上下文".to_string(),
                ))
            }
        };

        // 构建UNWIND子句的执行计划
        self.build_unwind(unwind_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Unwind
    }
}

impl DataFlowNode for UnwindClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwind_clause_planner_interface() {
        let planner = UnwindClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Unwind);
        assert_eq!(planner.name(), "UnwindClausePlanner");
    }
}
