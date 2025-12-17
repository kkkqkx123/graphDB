//! 新的 WITH子句规划器
//! 实现新的 CypherClausePlanner 接口

use crate::query::planner::match_planning::core::{
    CypherClausePlanner, ClauseType, PlanningContext
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};

/// 新的 WITH子句规划器
/// 实现新的 CypherClausePlanner 接口
#[derive(Debug)]
pub struct WithClausePlanner;

impl WithClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for WithClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError> {
        // 验证输入
        self.validate_input(input_plan)?;
        
        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::missing_input("WITH clause requires input".to_string())
        })?;
        
        // 验证上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::With) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for WithClausePlanner".to_string(),
            ));
        }

        let with_clause_ctx = match clause_ctx {
            CypherClauseContext::With(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected WithClauseContext".to_string(),
                ))
            }
        };

        // 验证 WITH 子句上下文的完整性
        if with_clause_ctx.yield_clause.yield_columns.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "WITH 子句必须至少包含一个输出列".to_string(),
            ));
        }

        // 步骤1: 处理YIELD子句（WITH的投影部分）
        // 暂时跳过YIELD子句处理，因为接口不兼容
        let plan = input_plan.clone();
        
        // TODO: 实现YIELD子句处理逻辑
        // let mut yield_planner = YieldClausePlanner::new();
        // let yield_clause_ctx = CypherClauseContext::Yield(with_clause_ctx.yield_clause.clone());
        // let mut plan = yield_planner.transform(&yield_clause_ctx)?;

        // 步骤2: 处理WHERE子句（如果存在）
        // 暂时跳过WHERE子句处理，因为接口不兼容
        // TODO: 实现WHERE子句处理逻辑
        // if let Some(where_clause) = &with_clause_ctx.where_clause {
        //     let mut where_planner = super::where_clause_planner::WhereClausePlanner::new(false);
        //     let where_clause_ctx = CypherClauseContext::Where(where_clause.clone());
        //     let where_plan = where_planner.transform(&where_clause_ctx)?;
        //     let connector = SegmentsConnector::new();
        //     plan = connector.add_input(where_plan, plan, true);
        // }

        // 步骤3: 更新上下文中的变量
        // WITH 子句会重新定义可用的变量
        for column in &with_clause_ctx.yield_clause.yield_columns {
            if !column.alias.is_empty() {
                context.add_variable(column.alias.clone());
            }
        }

        Ok(plan)
    }
    
    fn validate_input(&self, input_plan: Option<&SubPlan>) -> Result<(), crate::query::planner::planner::PlannerError> {
        if input_plan.is_none() {
            return Err(PlannerError::missing_input(
                "WITH clause requires input from previous clauses".to_string()
            ));
        }
        Ok(())
    }
    
    fn clause_type(&self) -> ClauseType {
        ClauseType::Transform
    }
    
    fn can_start_flow(&self) -> bool {
        false  // WITH 不能开始数据流
    }
    
    fn requires_input(&self) -> bool {
        true   // WITH 需要输入
    }
    
    fn input_requirements(&self) -> Vec<crate::query::planner::match_planning::core::VariableRequirement> {
        // WITH 子句需要输入数据，但不强制要求特定变量
        vec![]
    }
    
    fn output_provides(&self) -> Vec<crate::query::planner::match_planning::core::VariableProvider> {
        // WITH 子句的输出取决于具体的投影列
        // 这里返回空列表，实际实现中应该根据 yield_clause 来确定
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::match_planning::core::ClauseType;
    
    #[test]
    fn test_with_clause_planner_interface() {
        let planner = WithClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Transform);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
    }
    
    #[test]
    fn test_with_clause_planner_validate_input() {
        let planner = WithClausePlanner::new();
        
        // 测试没有输入的情况
        let result = planner.validate_input(None);
        assert!(result.is_err());
        
        // 测试有输入的情况
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_input(Some(&dummy_plan));
        assert!(result.is_ok());
    }
}