//! ORDER BY子句规划器
//! 处理ORDER BY子句的规划
//! 负责规划ORDER BY子句中的排序操作

use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::plan::core::PlanNodeMutable;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// ORDER BY子句规划器
/// 负责规划ORDER BY子句中的排序操作
#[derive(Debug)]
pub struct OrderByClausePlanner;

impl OrderByClausePlanner {
    pub fn new() -> Self {
        Self
    }

    /// 构建排序节点
    fn build_sort(
        &mut self,
        order_by_ctx: &crate::query::validator::structs::OrderByClauseContext,
        mut subplan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        // 获取当前的根节点作为输入
        let current_root = subplan
            .root
            .take()
            .unwrap_or_else(|| create_empty_node().unwrap());

        // 创建排序节点，使用当前根节点作为输入
        let sort_node = SingleInputNode::new(PlanNodeKind::Sort, current_root);

        // 将排序因子信息存储在节点的列名中，以便执行阶段使用
        // 在实际执行时，排序逻辑会根据这些信息进行排序
        // indexed_order_factors包含(列索引, 排序类型)的元组
        let mut col_names = Vec::new();
        for (idx, _) in &order_by_ctx.indexed_order_factors {
            // 使用特殊格式存储排序信息，供执行器使用
            col_names.push(format!("sort_factor_{}", idx));
        }

        // 创建新的排序节点并设置属性
        let mut new_sort_node = sort_node.clone();
        new_sort_node.set_col_names(col_names);
        let sort_node = Arc::new(new_sort_node);

        // 更新子计划的根和尾节点
        subplan.root = Some(sort_node.clone());
        subplan.tail = Some(sort_node);

        Ok(subplan)
    }
}

impl CypherClausePlanner for OrderByClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::OrderBy) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for OrderByClausePlanner".to_string(),
            ));
        }

        let order_by_ctx = match clause_ctx {
            CypherClauseContext::OrderBy(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected OrderByClauseContext".to_string(),
                ))
            }
        };

        // 创建一个空的子计划
        let empty_subplan = SubPlan::new(None, None);

        // 构建排序计划
        self.build_sort(order_by_ctx, empty_subplan)
    }
}

/// 创建空节点
fn create_empty_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;

    // 创建一个空的计划节点作为占位符
    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}
