//! YIELD子句规划器
//! 处理YIELD子句的规划
//! 负责规划YIELD子句中的结果产出

use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// YIELD子句规划器
/// 负责规划YIELD子句中的结果产出
#[derive(Debug)]
pub struct YieldClausePlanner;

impl YieldClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for YieldClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
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

        let mut plan = SubPlan::new(None, None);

        // 处理聚合函数
        if yield_clause_ctx.has_agg {
            // 创建聚合节点
            let agg_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Aggregate,
                create_empty_node()?,
            ));

            // TODO: 设置聚合相关的参数
            // 这里需要根据group_keys和group_items设置聚合逻辑

            plan.root = Some(agg_node.clone());
            plan.tail = Some(agg_node);
        }

        // 处理投影（列选择）
        if yield_clause_ctx.need_gen_project {
            // 创建投影节点
            let project_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Project,
                create_empty_node()?,
            ));

            // TODO: 设置投影列
            // 这里需要根据proj_cols设置投影逻辑

            if plan.root.is_none() {
                plan.root = Some(project_node.clone());
                plan.tail = Some(project_node);
            } else {
                // 将投影节点连接到现有计划的尾部
                let connector = crate::query::planner::match_planning::segments_connector::SegmentsConnector::new();
                plan = connector.add_input(
                    SubPlan::new(Some(project_node.clone()), Some(project_node)),
                    plan,
                    true,
                );
            }
        }

        // 处理去重
        if yield_clause_ctx.distinct {
            // 创建去重节点
            let dedup_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Dedup,
                create_empty_node()?,
            ));

            // TODO: 设置去重键

            if plan.root.is_none() {
                plan.root = Some(dedup_node.clone());
                plan.tail = Some(dedup_node);
            } else {
                // 将去重节点连接到现有计划的尾部
                let connector = crate::query::planner::match_planning::segments_connector::SegmentsConnector::new();
                plan = connector.add_input(
                    SubPlan::new(Some(dedup_node.clone()), Some(dedup_node)),
                    plan,
                    true,
                );
            }
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
