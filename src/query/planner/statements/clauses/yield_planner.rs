//! YIELD 子句规划器
//!
//! 负责将 YIELD 子句转换为执行计划节点
//! 支持 YIELD ... WHERE ... 语法

use crate::query::planner::statements::clauses::clause_planner::ClausePlanner;
use crate::query::planner::statements::core::cypher_clause_planner::{
    ClauseType, CypherClausePlanner, DataFlowNode, FlowDirection, PlanningContext,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::{
    FilterNode, LimitNode, PlanNodeEnum, ProjectNode,
};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::validator::YieldColumn;

/// YIELD 子句规划器
#[derive(Debug)]
pub struct YieldClausePlanner {}

impl YieldClausePlanner {
    pub fn new() -> Self {
        Self {}
    }

    /// 规划 YIELD 子句
    ///
    /// 处理流程：
    /// 1. 构建投影节点（YIELD 列）
    /// 2. 如有 WHERE 条件，添加 Filter 节点
    /// 3. 如有 LIMIT/SKIP，添加分页节点
    pub fn plan_yield_clause(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        let mut current_plan = input_plan.clone();

        // 获取 YIELD 上下文
        let yield_ctx = clause_ctx
            .yield_clause()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("缺少 YIELD 上下文".to_string()))?;

        // 1. 构建投影节点（如果有具体的 YIELD 列）
        if !yield_ctx.yield_columns.is_empty() {
            let project_node = self.create_project_node(&current_plan, &yield_ctx.yield_columns)?;
            current_plan = SubPlan::new(Some(project_node), current_plan.tail.clone());
        }

        // 2. 如有 WHERE 条件，添加 Filter 节点
        if let Some(ref filter_condition) = yield_ctx.filter_condition {
            let filter_node = self.create_filter_node(&current_plan, filter_condition.clone())?;
            current_plan = SubPlan::new(Some(filter_node), current_plan.tail.clone());
        }

        // 3. 处理分页（LIMIT/SKIP）
        if yield_ctx.limit.is_some() || yield_ctx.skip.is_some() {
            current_plan =
                self.apply_pagination(current_plan, yield_ctx.skip, yield_ctx.limit)?;
        }

        Ok(current_plan)
    }

    /// 创建投影节点
    fn create_project_node(
        &self,
        input_plan: &SubPlan,
        columns: &[YieldColumn],
    ) -> Result<PlanNodeEnum, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        ProjectNode::new(input_node.clone(), columns.to_vec())
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建投影节点失败: {}", e)))
            .map(|node| PlanNodeEnum::Project(node))
    }

    /// 创建过滤节点
    fn create_filter_node(
        &self,
        input_plan: &SubPlan,
        condition: crate::core::Expression,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        FilterNode::new(input_node.clone(), condition)
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建过滤节点失败: {}", e)))
            .map(|node| PlanNodeEnum::Filter(node))
    }

    /// 应用分页
    fn apply_pagination(
        &self,
        input_plan: SubPlan,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        let offset = skip.unwrap_or(0) as i64;
        let count = limit.map(|l| l as i64).unwrap_or(i64::MAX);

        let limit_node = LimitNode::new(input_node.clone(), offset, count)
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建分页节点失败: {}", e)))?;

        Ok(SubPlan::new(
            Some(PlanNodeEnum::Limit(limit_node)),
            input_plan.tail.clone(),
        ))
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

impl DataFlowNode for YieldClausePlanner {
    fn flow_direction(&self) -> FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for YieldClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Yield
    }

    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("YIELD 子句需要输入计划".to_string())
        })?;
        self.plan_yield_clause(clause_ctx, input_plan, context)
    }
}

impl Default for YieldClausePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_clause_planner_creation() {
        let planner = YieldClausePlanner::new();
        assert_eq!(planner.name(), "YieldClausePlanner");
    }
}
