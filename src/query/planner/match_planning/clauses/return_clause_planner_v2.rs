//! 新的 RETURN 子句规划器
//! 实现新的 CypherClausePlanner 接口

use super::order_by_planner::OrderByClausePlanner;
use super::pagination_planner::PaginationPlanner;
use super::yield_planner::YieldClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner as _;
use crate::query::planner::match_planning::core::{
    ClauseType, CypherClausePlanner, PlanningContext, VariableProvider, VariableRequirement,
};
use crate::query::planner::match_planning::utils::connector::SegmentsConnector;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// 新的 RETURN子句规划器
/// 实现新的 CypherClausePlanner 接口
#[derive(Debug)]
pub struct ReturnClausePlannerV2;

impl ReturnClausePlannerV2 {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for ReturnClausePlannerV2 {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError> {
        // 验证输入
        self.validate_input(input_plan)?;

        // 确保有输入计划
        let _input_plan = input_plan.ok_or_else(|| {
            PlannerError::missing_input("RETURN clause requires input".to_string())
        })?;

        // 验证上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::Return) {
            return Err(PlannerError::InvalidAstContext(
                "ReturnClausePlanner 只能处理 RETURN 子句上下文".to_string(),
            ));
        }

        // 提取具体的 RETURN 子句上下文
        let return_clause_ctx = match clause_ctx {
            CypherClauseContext::Return(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "无法提取 ReturnClauseContext".to_string(),
                ))
            }
        };

        // 验证 RETURN 子句上下文的完整性
        if return_clause_ctx.yield_clause.yield_columns.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "RETURN 子句必须至少包含一个输出列".to_string(),
            ));
        }

        // 步骤1: 处理YIELD子句（RETURN的投影部分）
        let mut yield_planner = YieldClausePlanner::new();
        let yield_clause_ctx = CypherClauseContext::Yield(return_clause_ctx.yield_clause.clone());
        // 暂时使用旧的接口，因为其他规划器还没有更新
        let mut plan = yield_planner.transform(&yield_clause_ctx)?;

        // 步骤2: 处理ORDER BY子句（排序）
        if let Some(order_by) = &return_clause_ctx.order_by {
            let mut order_by_planner = OrderByClausePlanner::new();
            let order_by_clause_ctx = CypherClauseContext::OrderBy(order_by.clone());
            // 暂时使用旧的接口，因为其他规划器还没有更新
            let order_plan = order_by_planner.transform(&order_by_clause_ctx)?;

            // 使用新的连接机制
            let connector = SegmentsConnector::new();
            plan = connector.add_input(order_plan, plan, true);
        }

        // 步骤3: 处理分页（LIMIT/OFFSET）
        if let Some(pagination) = &return_clause_ctx.pagination {
            // 验证分页参数的合理性
            validate_pagination_params(pagination.skip, pagination.limit)?;

            // 只有当skip或limit有实际值时才创建分页节点
            if pagination.skip != 0 || pagination.limit != i64::MAX {
                let mut pagination_planner = PaginationPlanner::new();
                let pagination_clause_ctx = CypherClauseContext::Pagination(pagination.clone());
                // 暂时使用旧的接口，因为其他规划器还没有更新
                let pagination_plan = pagination_planner.transform(&pagination_clause_ctx)?;

                let connector = SegmentsConnector::new();
                plan = connector.add_input(pagination_plan, plan, true);
            }
        }

        // 步骤4: 处理去重 (DISTINCT)
        if return_clause_ctx.distinct {
            let current_root = plan.root.as_ref().unwrap().clone();
            let dedup_node = Arc::new(SingleInputNode::new(PlanNodeKind::Dedup, current_root));

            // 设置去重键 - 使用投影列作为去重依据
            // 暂时简化去重节点创建
            // TODO: 实现完整的去重逻辑

            let connector = SegmentsConnector::new();
            plan = connector.add_input(
                SubPlan::new(Some(dedup_node.clone()), Some(dedup_node)),
                plan,
                true,
            );
        }

        Ok(plan)
    }

    fn validate_input(
        &self,
        input_plan: Option<&SubPlan>,
    ) -> Result<(), crate::query::planner::planner::PlannerError> {
        if input_plan.is_none() {
            return Err(PlannerError::missing_input(
                "RETURN clause requires input from previous clauses".to_string(),
            ));
        }
        Ok(())
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Output
    }

    fn can_start_flow(&self) -> bool {
        false // RETURN 不能开始数据流
    }

    fn requires_input(&self) -> bool {
        true // RETURN 需要输入
    }

    fn input_requirements(&self) -> Vec<VariableRequirement> {
        // RETURN 子句需要输入数据，但不强制要求特定变量
        vec![]
    }

    fn output_provides(&self) -> Vec<VariableProvider> {
        // RETURN 子句的输出取决于具体的投影列
        // 这里返回空列表，实际实现中应该根据 yield_clause 来确定
        vec![]
    }
}

/// 获取 YIELD 子句中的列名
/// 用于设置去重键
#[allow(dead_code)]
fn get_yield_columns(
    yield_clause: &crate::query::validator::structs::YieldClauseContext,
) -> Option<Vec<String>> {
    // 优先使用投影输出列名
    if !yield_clause.proj_output_column_names.is_empty() {
        return Some(yield_clause.proj_output_column_names.clone());
    }

    // 如果没有投影列名，尝试从 yield_columns 中提取
    if !yield_clause.yield_columns.is_empty() {
        let columns: Vec<String> = yield_clause
            .yield_columns
            .iter()
            .map(|col| col.alias.clone())
            .collect();
        return Some(columns);
    }

    // 如果都没有，返回 None
    None
}

/// 验证分页参数的合理性
fn validate_pagination_params(skip: i64, limit: i64) -> Result<(), PlannerError> {
    if skip < 0 {
        return Err(PlannerError::PlanGenerationFailed(format!(
            "OFFSET 值不能为负数: {}",
            skip
        )));
    }

    if limit <= 0 && limit != i64::MAX {
        return Err(PlannerError::PlanGenerationFailed(format!(
            "LIMIT 值必须为正数: {}",
            limit
        )));
    }

    if skip > i64::MAX / 2 {
        return Err(PlannerError::PlanGenerationFailed(
            "OFFSET 值过大，可能导致内存问题".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::AstContext;
    use crate::query::planner::match_planning::core::ClauseType;

    #[test]
    fn test_return_clause_planner_v2_interface() {
        let planner = ReturnClausePlannerV2::new();
        assert_eq!(planner.clause_type(), ClauseType::Output);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
    }

    #[test]
    fn test_return_clause_planner_v2_validate_input() {
        let planner = ReturnClausePlannerV2::new();

        // 测试没有输入的情况
        let result = planner.validate_input(None);
        assert!(result.is_err());

        // 测试有输入的情况
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_input(Some(&dummy_plan));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pagination_params() {
        // 测试正常参数
        assert!(validate_pagination_params(0, 10).is_ok());
        assert!(validate_pagination_params(5, i64::MAX).is_ok());

        // 测试无效参数
        assert!(validate_pagination_params(-1, 10).is_err());
        assert!(validate_pagination_params(0, 0).is_err());
        assert!(validate_pagination_params(0, -5).is_err());
        assert!(validate_pagination_params(i64::MAX / 2 + 1, 10).is_err());
    }
}
