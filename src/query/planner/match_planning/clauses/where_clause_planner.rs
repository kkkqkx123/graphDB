//! WHERE子句规划器
//! 处理WHERE条件的规划
//! 负责规划WHERE子句中的过滤条件

use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::paths::match_path_planner::MatchPathPlanner;
use crate::query::planner::match_planning::utils::connector::SegmentsConnector;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// WHERE子句规划器
/// 负责规划WHERE子句中的过滤条件
#[derive(Debug)]
pub struct WhereClausePlanner {
    need_stable_filter: bool, // 是否需要稳定的过滤器（用于ORDER BY场景）
}

impl WhereClausePlanner {
    pub fn new(need_stable_filter: bool) -> Self {
        Self { need_stable_filter }
    }
}

impl CypherClausePlanner for WhereClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::Where) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for WhereClausePlanner".to_string(),
            ));
        }

        let where_clause_ctx = match clause_ctx {
            CypherClauseContext::Where(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected WhereClauseContext".to_string(),
                ))
            }
        };

        let mut plan = SubPlan::new(None, None);

        // 处理路径表达式（模式谓词）
        if !where_clause_ctx.paths.is_empty() {
            let mut paths_plan = SubPlan::new(None, None);

            // 为模式表达式构建计划
            for path in &where_clause_ctx.paths {
                let mut path_planner = MatchPathPlanner::new(
                    // 这里需要创建一个临时的MatchClauseContext
                    crate::query::validator::structs::MatchClauseContext {
                        paths: vec![path.clone()],
                        aliases_available: where_clause_ctx.aliases_available.clone(),
                        aliases_generated: where_clause_ctx.aliases_generated.clone(),
                        where_clause: None,
                        is_optional: false,
                        skip: None,
                        limit: None,
                    },
                    path.clone(),
                );

                let path_plan =
                    path_planner.transform(None, &mut std::collections::HashSet::new())?;

                let connector = SegmentsConnector::new();
                if path.is_pred {
                    // 构建模式谓词的计划
                    paths_plan = connector.pattern_apply(paths_plan, path_plan, path);
                } else {
                    // 构建路径收集的计划
                    paths_plan = connector.roll_up_apply(paths_plan, path_plan, path);
                }
            }

            plan = paths_plan;
        }

        // 处理过滤条件
        if let Some(_filter) = &where_clause_ctx.filter {
            let mut where_plan = SubPlan::new(None, None);

            // 创建过滤器节点
            let filter_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Filter,
                create_empty_node()?,
            ));

            // TODO: 设置过滤条件表达式
            // 这里需要根据filter表达式创建相应的计划节点

            where_plan.root = Some(filter_node.clone());
            where_plan.tail = Some(filter_node);

            if plan.root.is_none() {
                return Ok(where_plan);
            }

            let connector = SegmentsConnector::new();
            plan = connector.add_input(where_plan, plan, true);
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
