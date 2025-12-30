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

use super::order_by_planner::OrderByClausePlanner;
use super::pagination_planner::PaginationPlanner;
use super::yield_planner::YieldClausePlanner;
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::core::ClauseType;
use crate::query::planner::match_planning::utils::connection_strategy::UnifiedConnector;

use crate::query::planner::plan::factory::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

/// RETURN 子句规划器
///
/// 负责规划 RETURN 子句的执行。RETURN 子句是一个输出子句，
/// 它需要输入数据流并根据指定的投影列、排序、分页和去重选项对结果进行处理。
///
/// # 示例
///
/// ```cypher
/// MATCH (n:Person)
/// RETURN n.name, n.age
/// ORDER BY n.age DESC
/// LIMIT 10
/// ```
///
/// 在上面的例子中，RETURN 子句会返回人员的姓名和年龄，按年龄降序排列，并限制返回10条记录。
#[derive(Debug)]
pub struct ReturnClausePlanner;

impl ReturnClausePlanner {
    /// 创建新的 RETURN 子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建 RETURN 子句的执行计划
    ///
    /// # 参数
    ///
    /// * `return_clause_ctx` - RETURN 子句的上下文信息
    /// * `input_plan` - 输入的执行计划
    /// * `context` - 规划上下文
    ///
    /// # 返回值
    ///
    /// 返回包含 RETURN 子句执行计划的 SubPlan
    fn build_return(
        &self,
        return_clause_ctx: &crate::query::validator::structs::clause_structs::ReturnClauseContext,
        input_plan: &SubPlan,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证 RETURN 子句上下文的完整性
        if return_clause_ctx.yield_clause.yield_columns.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "RETURN 子句必须至少包含一个输出列".to_string(),
            ));
        }

        // 步骤1: 处理YIELD子句（RETURN的投影部分）
        let yield_planner = YieldClausePlanner::new();
        let yield_clause_ctx = CypherClauseContext::Yield(return_clause_ctx.yield_clause.clone());
        let mut plan = yield_planner.transform(&yield_clause_ctx, Some(input_plan), context)?;

        // 步骤2: 处理ORDER BY子句（排序）
        if let Some(order_by) = &return_clause_ctx.order_by {
            let order_by_planner = OrderByClausePlanner::new();
            let order_by_clause_ctx = CypherClauseContext::OrderBy(order_by.clone());
            let order_plan =
                order_by_planner.transform(&order_by_clause_ctx, Some(&plan), context)?;

            // 使用新的统一连接器连接排序计划
            let temp_ast_context = crate::query::context::ast::base::AstContext::from_strings(
                &context.query_info.statement_type,
                &context.query_info.query_id,
            );
            plan = UnifiedConnector::add_input(&temp_ast_context, &order_plan, &plan, true)?;
        }

        // 步骤3: 处理分页（LIMIT/OFFSET）
        if let Some(pagination) = &return_clause_ctx.pagination {
            // 验证分页参数的合理性
            validate_pagination_params(pagination.skip, pagination.limit)?;

            // 只有当skip或limit有实际值时才创建分页节点
            if pagination.skip != 0 || pagination.limit != i64::MAX {
                let pagination_planner = PaginationPlanner::new();
                let pagination_clause_ctx = CypherClauseContext::Pagination(pagination.clone());
                let pagination_plan =
                    pagination_planner.transform(&pagination_clause_ctx, Some(&plan), context)?;

                let temp_ast_context = crate::query::context::ast::base::AstContext::from_strings(
                    &context.query_info.statement_type,
                    &context.query_info.query_id,
                );
                plan =
                    UnifiedConnector::add_input(&temp_ast_context, &pagination_plan, &plan, true)?;
            }
        }

        // 步骤4: 处理去重 (DISTINCT)
        if return_clause_ctx.distinct {
            // 创建去重节点 - 使用新的节点类型
            // 注意：这里我们暂时使用占位符节点，因为 DedupNode 还没有实现
            // 在完整的实现中，应该创建一个专门的 DedupNode
            let dedup_node = PlanNodeFactory::create_placeholder_node()?;

            // 设置去重键 - 使用投影列作为去重依据
            // 暂时简化去重节点创建
            // TODO: 实现完整的去重逻辑，创建专门的 DedupNode

            let temp_ast_context = crate::query::context::ast::base::AstContext::from_strings(
                &context.query_info.statement_type,
                &context.query_info.query_id,
            );
            plan = UnifiedConnector::add_input(
                &temp_ast_context,
                &SubPlan::from_single_node(dedup_node),
                &plan,
                true,
            )?;
        }

        // 标记上下文中的变量为输出变量
        context.mark_output_variables();

        Ok(plan)
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
        // 验证数据流：RETURN 子句需要输入
        self.validate_flow(input_plan)?;

        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("RETURN clause requires input".to_string())
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

        // 构建 RETURN 子句的执行计划
        self.build_return(return_clause_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Return
    }
}

impl DataFlowNode for ReturnClausePlanner {
    fn flow_direction(
        &self,
    ) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

/// 获取 YIELD 子句中的列名
/// 用于设置去重键

fn get_yield_columns(
    yield_clause: &crate::query::validator::structs::clause_structs::YieldClauseContext,
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
    use crate::query::planner::match_planning::core::ClauseType;

    #[test]
    fn test_return_clause_planner_interface() {
        let planner = ReturnClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Return);
        assert_eq!(<ReturnClausePlanner as DataFlowNode>::flow_direction(&planner), crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection::Output);
        assert!(planner.requires_input());
    }

    #[test]
    fn test_return_clause_planner_validate_flow() {
        let planner = ReturnClausePlanner::new();

        // 测试没有输入的情况（应该失败）
        let result = planner.validate_flow(None);
        assert!(result.is_err());

        // 测试有输入的情况（应该成功）
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_flow(Some(&dummy_plan));
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
