//! RETURN子句规划器
//! 处理RETURN子句的规划
//! 负责规划RETURN子句中的结果投影

use super::order_by_planner::OrderByClausePlanner;
use super::pagination_planner::PaginationPlanner;
use super::yield_planner::YieldClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::utils::connector::SegmentsConnector;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// RETURN子句规划器
/// 负责规划RETURN子句中的结果投影
#[derive(Debug)]
pub struct ReturnClausePlanner;

impl ReturnClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for ReturnClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::Return) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for ReturnClausePlanner".to_string(),
            ));
        }

        let return_clause_ctx = match clause_ctx {
            CypherClauseContext::Return(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected ReturnClauseContext".to_string(),
                ))
            }
        };

        // 首先处理YIELD子句（RETURN的投影部分）
        let mut yield_planner = YieldClausePlanner::new();
        let yield_clause_ctx = CypherClauseContext::Yield(return_clause_ctx.yield_clause.clone());
        let mut plan = yield_planner.transform(&yield_clause_ctx)?;

        // 处理ORDER BY子句
        if let Some(order_by) = &return_clause_ctx.order_by {
            let mut order_by_planner = OrderByClausePlanner::new();
            let order_by_clause_ctx = CypherClauseContext::OrderBy(order_by.clone());
            let order_plan = order_by_planner.transform(&order_by_clause_ctx)?;

            let connector = SegmentsConnector::new();
            plan = connector.add_input(order_plan, plan, true);
        }

        // 处理分页（LIMIT/OFFSET）
        if let Some(pagination) = &return_clause_ctx.pagination {
            if pagination.skip != 0 || pagination.limit != i64::MAX {
                let mut pagination_planner = PaginationPlanner::new();
                let pagination_clause_ctx = CypherClauseContext::Pagination(pagination.clone());
                let pagination_plan = pagination_planner.transform(&pagination_clause_ctx)?;

                let connector = SegmentsConnector::new();
                plan = connector.add_input(pagination_plan, plan, true);
            }
        }

        // 处理去重
        if return_clause_ctx.distinct {
            // 创建去重节点
            let dedup_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Dedup,
                create_empty_node()?,
            ));

            // TODO: 设置去重键

            let connector = SegmentsConnector::new();
            plan = connector.add_input(
                SubPlan::new(Some(dedup_node.clone()), Some(dedup_node)),
                plan,
                true,
            );
        }

        Ok(plan)
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
