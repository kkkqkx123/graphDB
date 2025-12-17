use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::PlanNodeKind;
/// YIELD子句规划器
/// 处理YIELD子句的规划
/// 负责规划YIELD子句中的结果产出
///
/// YIELD子句是Cypher查询中的核心投影操作，负责选择和计算要输出的列。
/// 它可以包含聚合函数、投影列和去重操作。

use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, ClauseType, PlanningContext, VariableRequirement, VariableProvider,
};
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::utils::connection_strategy::UnifiedConnector;
use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

/// YIELD子句规划器
/// 
/// 负责规划YIELD子句的执行。YIELD子句是一个转换子句，
/// 它需要输入数据流并根据指定的投影列和聚合函数对结果进行处理。
/// 
/// # 示例
/// 
/// ```cypher
/// MATCH (n:Person)
/// YIELD n.name, count(*) AS count
/// ```
/// 
/// 在上面的例子中，YIELD子句会输出人员的姓名和数量统计。
#[derive(Debug, Clone)]
pub struct YieldClausePlanner;

impl YieldClausePlanner {
    /// 创建新的YIELD子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建YIELD子句的执行计划
    /// 
    /// # 参数
    /// 
    /// * `yield_clause_ctx` - YIELD子句的上下文信息
    /// * `input_plan` - 输入的执行计划
    /// * `context` - 规划上下文
    /// 
    /// # 返回值
    /// 
    /// 返回包含YIELD子句执行计划的SubPlan
    fn build_yield(
        &self,
        yield_clause_ctx: &crate::query::validator::structs::clause_structs::YieldClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        let mut plan = input_plan.clone();

        // 处理聚合函数
        if yield_clause_ctx.has_agg {
            // 创建聚合节点
            let agg_node = PlanNodeFactory::create_placeholder_node()?;

            // TODO: 设置聚合相关的参数
            // 这里需要根据group_keys和group_items设置聚合逻辑

            plan = SubPlan::new(Some(agg_node.clone_plan_node()), Some(agg_node));
        }

        // 处理投影（列选择）
        if yield_clause_ctx.need_gen_project {
            // 创建投影节点
            let input_root = plan.root.as_ref().ok_or_else(|| {
                PlannerError::PlanGenerationFailed(
                    "YIELD clause requires input plan for projection".to_string()
                )
            })?;
            
            let project_node = PlanNodeFactory::create_placeholder_node()?;

            // TODO: 设置投影列
            // 这里需要根据proj_cols设置投影逻辑

            if plan.root.is_none() {
                plan.root = Some(project_node.clone_plan_node());
                plan.tail = Some(project_node);
            } else {
                // 使用新的统一连接器将投影节点连接到现有计划的尾部
                plan = UnifiedConnector::add_input(
                    &crate::query::context::ast::base::AstContext::new("YIELD", "test"),
                    &SubPlan::new(Some(project_node.clone_plan_node()), Some(project_node)),
                    &plan,
                    true,
                )?;
            }
        }

        // 处理去重
        if yield_clause_ctx.distinct {
            // 创建去重节点
            let input_root = plan.root.as_ref().ok_or_else(|| {
                PlannerError::PlanGenerationFailed(
                    "YIELD clause requires input plan for deduplication".to_string()
                )
            })?;
            
            let dedup_node = PlanNodeFactory::create_placeholder_node()?;

            // TODO: 设置去重键

            if plan.root.is_none() {
                plan.root = Some(dedup_node.clone_plan_node());
                plan.tail = Some(dedup_node);
            } else {
                // 使用新的统一连接器将去重节点连接到现有计划的尾部
                plan = UnifiedConnector::add_input(
                    &crate::query::context::ast::base::AstContext::new("YIELD", "test"),
                    &SubPlan::new(Some(dedup_node.clone_plan_node()), Some(dedup_node)),
                    &plan,
                    true,
                )?;
            }
        }

        Ok(plan)
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
        // 验证输入
        self.validate_input(input_plan)?;

        // 验证上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::Yield) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for YieldClausePlanner".to_string(),
            ));
        }

        let yield_clause_ctx = match clause_ctx {
            CypherClauseContext::Yield(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected YieldClauseContext".to_string(),
                ))
            }
        };

        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed(
                "YIELD clause requires input plan".to_string()
            )
        })?;

        // 构建YIELD子句的执行计划
        self.build_yield(yield_clause_ctx, input_plan, context)
    }

    fn validate_input(&self, input_plan: Option<&SubPlan>) -> Result<(), PlannerError> {
        if input_plan.is_none() {
            return Err(PlannerError::PlanGenerationFailed(
                "YIELD clause requires input from previous clauses".to_string()
            ));
        }
        Ok(())
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Transform
    }

    fn can_start_flow(&self) -> bool {
        false  // YIELD 不能开始数据流
    }

    fn requires_input(&self) -> bool {
        true   // YIELD 需要输入
    }

    fn input_requirements(&self) -> Vec<VariableRequirement> {
        // YIELD 需要输入中的变量进行投影和聚合
        vec![]
    }

    fn output_provides(&self) -> Vec<VariableProvider> {
        // YIELD 输出投影后的变量
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::match_planning::core::ClauseType;

    #[test]
    fn test_yield_clause_planner_creation() {
        let planner = YieldClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Transform);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
    }

    #[test]
    fn test_yield_clause_planner_validate_input() {
        let planner = YieldClausePlanner::new();
        
        // 没有输入应该失败
        assert!(planner.validate_input(None).is_err());
        
        // 有输入应该成功
        let empty_plan = SubPlan::new(None, None);
        assert!(planner.validate_input(Some(&empty_plan)).is_ok());
    }
}