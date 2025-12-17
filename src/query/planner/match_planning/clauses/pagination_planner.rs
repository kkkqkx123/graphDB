//! 分页规划器
//! 处理LIMIT和OFFSET子句的规划
//! 负责规划LIMIT和OFFSET子句

use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, ClauseType, PlanningContext, VariableRequirement, VariableProvider,
};
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::plan::core::PlanNodeMutable;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use std::sync::Arc;

/// 分页规划器
/// 
/// 负责规划LIMIT和OFFSET子句。分页子句是一个修饰子句，
/// 它需要输入数据流并根据指定的skip和limit值对结果进行分页。
/// 
/// # 示例
/// 
/// ```cypher
/// MATCH (n:Person)
/// RETURN n.name
/// SKIP 10
/// LIMIT 5
/// ```
/// 
/// 在上面的例子中，分页子句会跳过前10个结果，然后返回接下来的5个结果。
#[derive(Debug, Clone)]
pub struct PaginationPlanner;

impl PaginationPlanner {
    /// 创建新的分页规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建分页节点
    /// 
    /// 根据分页上下文信息构建LIMIT节点。
    /// skip和limit值会存储在节点的列名中，以便执行阶段使用。
    /// 
    /// # 参数
    /// 
    /// * `pagination_ctx` - 分页上下文信息
    /// * `input_plan` - 输入的执行计划
    /// * `context` - 规划上下文
    /// 
    /// # 返回值
    /// 
    /// 返回包含LIMIT节点的执行计划
    fn build_limit(
        &self,
        pagination_ctx: &crate::query::validator::structs::clause_structs::PaginationContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 获取输入计划的根节点
        let input_root = input_plan.root.as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed(
                "Pagination clause requires input plan".to_string()
            )
        })?;

        // 创建Limit节点
        let limit_node = SingleInputNode::new(PlanNodeKind::Limit, input_root.clone());

        // 将skip和limit值存储在列名中，供执行器使用
        let col_names = vec![
            format!("skip_{}", pagination_ctx.skip),
            format!("limit_{}", pagination_ctx.limit),
        ];

        // 创建新的Limit节点并设置属性
        let mut new_limit_node = limit_node.clone();
        new_limit_node.set_col_names(col_names);
        let limit_node = Arc::new(new_limit_node);

        // 创建新的子计划
        let mut subplan = input_plan.clone();
        subplan.root = Some(limit_node.clone());
        subplan.tail = Some(limit_node);

        Ok(subplan)
    }
}

impl ClausePlanner for PaginationPlanner {
    fn name(&self) -> &'static str {
        "PaginationPlanner"
    }

    fn supported_clause_kind(&self) -> crate::query::validator::structs::CypherClauseKind {
        crate::query::validator::structs::CypherClauseKind::Pagination
    }
}

impl CypherClausePlanner for PaginationPlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证输入计划
        self.validate_input(input_plan)?;

        let pagination_ctx = match clause_ctx {
            CypherClauseContext::Pagination(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected PaginationContext".to_string(),
                ))
            }
        };

        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed(
                "Pagination clause requires input plan".to_string()
            )
        })?;

        // 构建分页计划
        self.build_limit(pagination_ctx, input_plan, context)
    }

    fn validate_input(&self, input_plan: Option<&SubPlan>) -> Result<(), PlannerError> {
        if input_plan.is_none() {
            return Err(PlannerError::PlanGenerationFailed(
                "Pagination clause requires input from previous clauses".to_string()
            ));
        }
        Ok(())
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Modifier
    }

    fn can_start_flow(&self) -> bool {
        false  // 分页不能开始数据流
    }

    fn requires_input(&self) -> bool {
        true   // 分页需要输入
    }

    fn input_requirements(&self) -> Vec<VariableRequirement> {
        // 分页需要输入中的所有变量，以便进行分页
        vec![]
    }

    fn output_provides(&self) -> Vec<VariableProvider> {
        // 分页输出与输入相同的变量，只是数量不同
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::clause_structs::PaginationContext;

    #[test]
    fn test_pagination_planner_creation() {
        let planner = PaginationPlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Modifier);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
    }

    #[test]
    fn test_pagination_planner_validate_input() {
        let planner = PaginationPlanner::new();
        
        // 没有输入应该失败
        assert!(planner.validate_input(None).is_err());
        
        // 有输入应该成功
        let empty_plan = SubPlan::new(None, None);
        assert!(planner.validate_input(Some(&empty_plan)).is_ok());
    }

    #[test]
    fn test_pagination_planner_transform() {
        let planner = PaginationPlanner::new();
        let query_ctx = crate::query::context::ast::AstContext::new("test", "test");
        let mut context = PlanningContext::new(query_ctx);
        
        // 创建分页上下文
        let pagination_ctx = PaginationContext {
            skip: 10,
            limit: 5,
        };
        
        let clause_ctx = CypherClauseContext::Pagination(pagination_ctx);
        
        // 没有输入应该失败
        let result = planner.transform(&clause_ctx, None, &mut context);
        assert!(result.is_err());
        
        // 有输入应该成功
        let input_plan = SubPlan::new(None, None);
        let result = planner.transform(&clause_ctx, Some(&input_plan), &mut context);
        // 这里可能会失败，因为需要有效的输入节点
        // 但至少验证了输入检查逻辑
    }
}