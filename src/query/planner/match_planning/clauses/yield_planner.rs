//! YIELD子句规划器
//! 处理YIELD子句的规划

use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// YIELD子句规划器
/// 
/// 负责规划YIELD子句的执行。YIELD子句是一个转换子句，
/// 它需要输入数据流并根据指定的投影列对结果进行处理。
#[derive(Debug, Clone)]
pub struct YieldClausePlanner;

impl YieldClausePlanner {
    /// 创建新的YIELD子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建YIELD子句的执行计划
    fn build_yield(
        &self,
        _yield_clause_ctx: &crate::query::validator::YieldClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证YIELD子句上下文的完整性
        // 暂时简单实现：直接返回输入计划
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for YieldClausePlanner {
    fn name(&self) -> &'static str {
        "YieldClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Yield
    }
}

impl CypherClausePlanner for YieldClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("YIELD clause requires input".to_string())
        })?;

        // 验证上下文类型
        let yield_clause_ctx = match clause_ctx {
            CypherClauseContext::Yield(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "YieldClausePlanner 只能处理 YIELD 子句上下文".to_string(),
                ))
            }
        };

        // 构建YIELD子句的执行计划
        self.build_yield(yield_clause_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Yield
    }
}

impl DataFlowNode for YieldClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_clause_planner_interface() {
        let planner = YieldClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Yield);
        assert_eq!(planner.name(), "YieldClausePlanner");
    }
}
