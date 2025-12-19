//! ORDER BY子句规划器
//! 处理ORDER BY子句的规划

use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// ORDER BY子句规划器
///
/// 负责规划ORDER BY子句中的排序操作。ORDER BY子句是一个修饰子句，
/// 它需要输入数据流并根据指定的排序列对结果进行排序。
#[derive(Debug, Clone)]
pub struct OrderByClausePlanner;

impl OrderByClausePlanner {
    /// 创建新的ORDER BY子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建排序节点
    fn build_sort(
        &self,
        _order_by_ctx: &crate::query::validator::OrderByClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 暂时简单实现：直接返回输入计划
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for OrderByClausePlanner {
    fn name(&self) -> &'static str {
        "OrderByClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::OrderBy
    }
}

impl CypherClausePlanner for OrderByClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("ORDER BY clause requires input".to_string())
        })?;

        // 验证上下文类型
        let order_by_ctx = match clause_ctx {
            CypherClauseContext::OrderBy(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "OrderByClausePlanner 只能处理 ORDER BY 子句上下文".to_string(),
                ))
            }
        };

        // 构建排序计划
        self.build_sort(order_by_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::OrderBy
    }
}

impl DataFlowNode for OrderByClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_by_clause_planner_interface() {
        let planner = OrderByClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::OrderBy);
        assert_eq!(planner.name(), "OrderByClausePlanner");
    }
}
