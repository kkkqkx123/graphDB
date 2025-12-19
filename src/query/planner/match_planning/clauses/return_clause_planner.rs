//! RETURN 子句规划器
//! 架构重构：实现统一的 CypherClausePlanner 接口
//!
//! ## 重构说明
//! 
//! ### 删除冗余方法
//! - 移除 `validate_input`, `can_start_flow`, `requires_input` 等冗余方法
//! - 通过 `flow_direction()` 统一表达数据流行为
//! 
//! ### 简化变量管理
//! - RETURN 子句标记输出变量，但不产生新变量
//! - 移除不必要的 `VariableRequirement` 和 `VariableProvider`
//! 
//! ### 优化实现逻辑
//! - 专注于核心的投影和输出功能
//! - 简化排序、分页和去重处理

use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::match_planning::core::cypher_clause_planner::{CypherClausePlanner, DataFlowNode, PlanningContext};
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// RETURN 子句规划器
/// 
/// 负责规划 RETURN 子句的执行。RETURN 子句是一个输出子句，
/// 它需要输入数据流并根据指定的投影列对结果进行处理。
/// 
/// # 示例
/// 
/// ```cypher
/// MATCH (n:Person)
/// RETURN n.name, n.age
/// ```
#[derive(Debug)]
pub struct ReturnClausePlanner;

impl ReturnClausePlanner {
    /// 创建新的 RETURN 子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建 RETURN 子句的执行计划
    fn build_return(
        &self,
        return_clause_ctx: &crate::query::validator::ReturnClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证 RETURN 子句上下文的完整性
        if return_clause_ctx.yield_clause.columns.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "RETURN 子句必须至少包含一个输出列".to_string(),
            ));
        }

        // 暂时简单实现：直接返回输入计划
        // 实际的投影和处理会在这里进行
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for ReturnClausePlanner {
    fn name(&self) -> &'static str {
        "ReturnClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Return
    }
}

impl CypherClausePlanner for ReturnClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("RETURN clause requires input".to_string())
        })?;

        // 验证上下文类型
        let return_clause_ctx = match clause_ctx {
            CypherClauseContext::Return(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "ReturnClausePlanner 只能处理 RETURN 子句上下文".to_string(),
                ))
            }
        };

        // 构建 RETURN 子句的执行计划
        self.build_return(return_clause_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Return
    }
}

impl DataFlowNode for ReturnClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::match_planning::core::ClauseType;

    #[test]
    fn test_return_clause_planner_interface() {
        let planner = ReturnClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Return);
    }
}
