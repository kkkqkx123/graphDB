//! ORDER BY子句规划器
//! 处理ORDER BY子句的规划
//! 负责规划ORDER BY子句中的排序操作

use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{
    CypherClauseContext, CypherClauseKind,
};

/// ORDER BY子句规划器
/// 负责规划ORDER BY子句中的排序操作
#[derive(Debug)]
pub struct OrderByClausePlanner;

impl OrderByClausePlanner {
    pub fn new() -> Self {
        Self
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

        // 创建排序节点
        let sort_node = Box::new(SingleInputNode::new(
            PlanNodeKind::Sort,
            create_empty_node()?,
        ));

        // TODO: 设置排序因子
        // 这里需要根据 indexed_order_factors 设置排序逻辑
        // indexed_order_factors 包含 (列索引, 排序类型) 的元组

        Ok(SubPlan::new(Some(sort_node.clone()), Some(sort_node)))
    }
}

/// 创建空节点
fn create_empty_node() -> Result<Box<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;

    // 创建一个空的计划节点作为占位符
    Ok(Box::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}
