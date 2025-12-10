//! 分页规划器
//! 处理LIMIT和OFFSET子句的规划
//! 负责规划LIMIT和OFFSET子句

use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::plan::{PlanNode, PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};

/// 分页规划器
/// 负责规划LIMIT和OFFSET子句
#[derive(Debug)]
pub struct PaginationPlanner;

impl PaginationPlanner {
    pub fn new() -> Self {
        Self
    }

    /// 构建分页节点
    fn build_limit(&mut self, pagination_ctx: &crate::query::validator::structs::PaginationContext, mut subplan: SubPlan) -> Result<SubPlan, PlannerError> {
        let current_root = subplan.root.take().unwrap_or_else(|| create_empty_node().unwrap());

        // 创建Limit节点
        let mut limit_node = SingleInputNode::new(
            PlanNodeKind::Limit,
            current_root,
        );

        // 将skip和limit值存储在列名中，供执行器使用
        let col_names = vec![
            format!("skip_{}", pagination_ctx.skip),
            format!("limit_{}", pagination_ctx.limit)
        ];
        limit_node.set_col_names(col_names);

        // 更新子计划的根和尾节点
        subplan.root = Some(Box::new(limit_node));
        subplan.tail = Some(subplan.root.as_ref().unwrap().clone_plan_node());

        Ok(subplan)
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

        // 创建一个空的子计划
        let empty_subplan = SubPlan::new(None, None);

        // 构建分页计划
        self.build_limit(pagination_ctx, empty_subplan)
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
