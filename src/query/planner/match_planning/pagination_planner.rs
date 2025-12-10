//! 分页规划器
//! 处理LIMIT和OFFSET子句的规划
//! 负责规划LIMIT和OFFSET子句

use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind, PaginationContext};

/// 分页规划器
/// 负责规划LIMIT和OFFSET子句
#[derive(Debug)]
pub struct PaginationPlanner;

impl PaginationPlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for PaginationPlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::Pagination) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for PaginationPlanner".to_string(),
            ));
        }

        let pagination_ctx = match clause_ctx {
            CypherClauseContext::Pagination(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected PaginationContext".to_string(),
                ))
            }
        };

        let mut plan = SubPlan::new(None, None);

        // 处理OFFSET（跳过）
        if pagination_ctx.skip > 0 {
            // 创建OFFSET节点
            let offset_node = Box::new(SingleInputNode::new(
                PlanNodeKind::Limit, // 使用Limit节点处理OFFSET
                create_empty_node()?,
            ));

            // TODO: 设置OFFSET值
            // 这里需要设置跳过的行数

            plan.root = Some(offset_node.clone());
            plan.tail = Some(offset_node);
        }

        // 处理LIMIT（限制）
        if pagination_ctx.limit < i64::MAX && pagination_ctx.limit > 0 {
            // 创建LIMIT节点
            let limit_node = Box::new(SingleInputNode::new(
                PlanNodeKind::Limit,
                create_empty_node()?,
            ));

            // TODO: 设置LIMIT值
            // 这里需要设置返回的最大行数

            if plan.root.is_none() {
                plan.root = Some(limit_node.clone());
                plan.tail = Some(limit_node);
            } else {
                // 将LIMIT节点连接到现有计划的尾部
                let connector = crate::query::planner::match_planning::segments_connector::SegmentsConnector::new();
                plan = connector.add_input(
                    SubPlan::new(Some(limit_node.clone()), Some(limit_node)),
                    plan,
                    true,
                );
            }
        }

        Ok(plan)
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
