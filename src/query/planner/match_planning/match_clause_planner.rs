//! MATCH子句规划器
//! 处理MATCH语句中的个别匹配子句

use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::match_path_planner::MatchPathPlanner;
use crate::query::planner::match_planning::segments_connector::SegmentsConnector;
use crate::query::planner::match_planning::shortest_path_planner::ShortestPathPlanner;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{
    clause_structs::WhereClauseContext, CypherClauseContext, CypherClauseKind, MatchClauseContext,
};
use std::collections::HashSet;

/// MATCH子句的规划器
/// 负责规划MATCH语句中的模式匹配部分
#[derive(Debug)]
pub struct MatchClausePlanner;

impl MatchClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for MatchClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::Match) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for MatchClausePlanner".to_string(),
            ));
        }

        let match_clause_ctx = match clause_ctx {
            CypherClauseContext::Match(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected MatchClauseContext".to_string(),
                ))
            }
        };

        let mut match_clause_plan = SubPlan::new(None, None);
        // 所有在当前MATCH子句中见过的节点别名
        let mut node_aliases_seen = HashSet::new();

        // TODO: 可能需要重建图并找到所有连通分量
        for path_info in &match_clause_ctx.paths {
            let mut path_plan = SubPlan::new(None, None);

            // 根据路径类型选择不同的规划器
            if path_info.is_default_path() {
                let mut match_path_planner =
                    MatchPathPlanner::new(match_clause_ctx.clone(), path_info.clone());
                let result = match_path_planner.transform(
                    match_clause_ctx.where_clause.as_ref(),
                    &mut node_aliases_seen,
                );
                match result {
                    Ok(plan) => path_plan = plan,
                    Err(e) => return Err(e),
                }
            } else {
                let mut shortest_path_planner =
                    ShortestPathPlanner::new(match_clause_ctx.clone(), path_info.clone());
                let result = shortest_path_planner.transform(
                    match_clause_ctx.where_clause.as_ref(),
                    &mut node_aliases_seen,
                );
                match result {
                    Ok(plan) => path_plan = plan,
                    Err(e) => return Err(e),
                }
            }

            // 连接路径计划
            match connect_path_plan(
                &path_info.node_infos(),
                match_clause_ctx,
                &path_plan,
                &mut node_aliases_seen,
                &mut match_clause_plan,
            ) {
                Ok(_) => {}
                Err(e) => return Err(e),
            }
        }

        Ok(match_clause_plan)
    }
}

/// 连接路径计划
fn connect_path_plan(
    node_infos: &[crate::query::validator::structs::path_structs::NodeInfo],
    match_clause_ctx: &MatchClauseContext,
    subplan: &SubPlan,
    node_aliases_seen: &mut HashSet<String>,
    match_clause_plan: &mut SubPlan,
) -> Result<(), PlannerError> {
    let mut intersected_aliases = HashSet::new();

    for info in node_infos {
        if node_aliases_seen.contains(&info.alias) {
            intersected_aliases.insert(info.alias.clone());
        }
        if !info.anonymous {
            node_aliases_seen.insert(info.alias.clone());
        }
    }

    if match_clause_plan.root.is_none() {
        *match_clause_plan = subplan.clone();
        return Ok(());
    }

    let connector = SegmentsConnector::new();

    if intersected_aliases.is_empty() {
        // 笛卡尔积
        *match_clause_plan =
            connector.cartesian_product(match_clause_plan.clone(), subplan.clone());
    } else {
        // 内连接
        *match_clause_plan = connector.inner_join(
            match_clause_plan.clone(),
            subplan.clone(),
            intersected_aliases.into_iter().collect(),
        );
    }

    Ok(())
}
