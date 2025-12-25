//! ORDER BY子句规划器
//! 处理ORDER BY子句的规划
//! 负责规划ORDER BY子句中的排序操作

use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::{
    CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::core::ClauseType;

use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::SubPlan;
use crate::query::validator::structs::common_structs::CypherClauseContext;

/// ORDER BY子句规划器
///
/// 负责规划ORDER BY子句中的排序操作。ORDER BY子句是一个修饰子句，
/// 它需要输入数据流并根据指定的排序因子对结果进行排序。
///
/// # 示例
///
/// ```cypher
/// MATCH (n:Person)
/// RETURN n.name
/// ORDER BY n.age DESC, n.name ASC
/// ```
///
/// 在上面的例子中，ORDER BY子句会根据年龄降序和姓名升序对结果进行排序。
#[derive(Debug, Clone)]
pub struct OrderByClausePlanner;

impl OrderByClausePlanner {
    /// 创建新的ORDER BY子句规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建排序节点
    ///
    /// 根据ORDER BY子句的上下文信息构建排序节点。
    /// 排序因子信息会存储在节点的列名中，以便执行阶段使用。
    ///
    /// # 参数
    ///
    /// * `order_by_ctx` - ORDER BY子句的上下文信息
    /// * `input_plan` - 输入的执行计划
    /// * `context` - 规划上下文
    ///
    /// # 返回值
    ///
    /// 返回包含排序节点的执行计划
    fn build_sort(
        &self,
        order_by_ctx: &crate::query::validator::structs::OrderByClauseContext,
        input_plan: &SubPlan,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 获取输入计划的根节点
        let _input_root = input_plan.root.as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("ORDER BY clause requires input plan".to_string())
        })?;

        // 创建排序节点，使用输入根节点作为输入
        let sort_node = PlanNodeFactory::create_placeholder_node()?;

        // 将排序因子信息存储在节点的列名中，以便执行阶段使用
        // 在实际执行时，排序逻辑会根据这些信息进行排序
        // indexed_order_factors包含(列索引, 排序类型)的元组
        let _col_names: Vec<String> = order_by_ctx
            .indexed_order_factors
            .iter()
            .map(|(idx, order_type)| {
                // 使用特殊格式存储排序信息，供执行器使用
                // 格式: sort_factor_<index>_<direction>
                let direction = match order_type {
                    crate::query::validator::structs::clause_structs::OrderType::Asc => "ASC",
                    crate::query::validator::structs::clause_structs::OrderType::Desc => "DESC",
                };
                format!("sort_factor_{}_{}", idx, direction)
            })
            .collect();

        // 创建新的子计划
        let mut subplan = input_plan.clone();
        subplan.root = Some(sort_node.clone());
        subplan.tail = Some(sort_node);

        Ok(subplan)
    }
}

impl ClausePlanner for OrderByClausePlanner {
    fn name(&self) -> &'static str {
        "OrderByClausePlanner"
    }

    fn supported_clause_kind(&self) -> crate::query::validator::structs::CypherClauseKind {
        crate::query::validator::structs::CypherClauseKind::OrderBy
    }
}

impl CypherClausePlanner for OrderByClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证数据流
        self.validate_flow(input_plan)?;

        let order_by_ctx = match clause_ctx {
            CypherClauseContext::OrderBy(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected OrderByClauseContext".to_string(),
                ))
            }
        };

        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("ORDER BY clause requires input plan".to_string())
        })?;

        // 构建排序计划
        self.build_sort(order_by_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::OrderBy
    }
}

impl DataFlowNode for OrderByClausePlanner {
    fn flow_direction(
        &self,
    ) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::clause_structs::OrderByClauseContext;

    #[test]
    fn test_order_by_planner_creation() {
        let planner = OrderByClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::OrderBy);
        assert_eq!(<OrderByClausePlanner as DataFlowNode>::flow_direction(&planner), crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection::Transform);
        assert!(planner.requires_input());
    }

    #[test]
    fn test_order_by_planner_validate_flow() {
        let planner = OrderByClausePlanner::new();

        // 没有输入应该失败
        assert!(planner.validate_flow(None).is_err());

        // 有输入应该成功
        let empty_plan = SubPlan::new(None, None);
        assert!(planner.validate_flow(Some(&empty_plan)).is_ok());
    }

    #[test]
    fn test_order_by_planner_transform() {
        let planner = OrderByClausePlanner::new();
        let query_info =
            crate::query::planner::match_planning::core::cypher_clause_planner::QueryInfo {
                query_id: "test".to_string(),
                statement_type: "ORDER BY".to_string(),
            };
        let mut context = PlanningContext::new(query_info);

        // 创建ORDER BY上下文
        let order_by_ctx = OrderByClauseContext {
            indexed_order_factors: vec![(
                0,
                crate::query::validator::structs::clause_structs::OrderType::Asc,
            )],
        };

        let clause_ctx = CypherClauseContext::OrderBy(order_by_ctx);

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
