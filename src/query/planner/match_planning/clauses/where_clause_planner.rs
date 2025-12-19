//! WHERE 子句规划器
//! 架构重构：实现统一的 CypherClausePlanner 接口

use crate::query::planner::match_planning::core::{
    CypherClausePlanner, ClauseType, PlanningContext, DataFlowNode
};
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::CypherClauseContext;
use crate::query::validator::CypherClauseKind;

/// WHERE 子句规划器
/// 
/// 负责规划 WHERE 子句的执行。WHERE 子句是一个转换子句，
/// 它需要输入数据流并根据指定的过滤条件对结果进行过滤。
/// 
/// # 示例
/// 
/// ```cypher
/// MATCH (n:Person)
/// WHERE n.age > 25 AND n.name STARTS WITH 'John'
/// RETURN n.name, n.age
/// ```
#[derive(Debug)]
pub struct WhereClausePlanner {
    #[allow(dead_code)]
    need_stable_filter: bool,
}

impl WhereClausePlanner {
    /// 创建新的 WHERE 子句规划器
    pub fn new(need_stable_filter: bool) -> Self {
        Self { need_stable_filter }
    }

    /// 构建 WHERE 子句的执行计划
    fn build_where(
        &self,
        where_clause_ctx: &crate::query::validator::WhereClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证是否有过滤条件
        if where_clause_ctx.filter.is_none() {
            return Ok(input_plan.clone());
        }

        // 暂时简单实现：直接返回输入计划
        // 实际的过滤逻辑会在这里进行
        Ok(input_plan.clone())
    }
}

impl ClausePlanner for WhereClausePlanner {
    fn name(&self) -> &'static str {
        "WhereClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Where
    }
}

impl CypherClausePlanner for WhereClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("WHERE clause requires input".to_string())
        })?;

        // 验证上下文类型
        let where_clause_ctx = match clause_ctx {
            CypherClauseContext::Where(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "WhereClausePlanner 只能处理 WHERE 子句上下文".to_string(),
                ))
            }
        };

        // 构建 WHERE 子句的执行计划
        self.build_where(where_clause_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Where
    }
}

impl DataFlowNode for WhereClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::match_planning::core::ClauseType;

    #[test]
    fn test_where_clause_planner_interface() {
        let planner = WhereClausePlanner::new(false);
        assert_eq!(planner.clause_type(), ClauseType::Where);
        assert_eq!(planner.name(), "WhereClausePlanner");
    }
}
