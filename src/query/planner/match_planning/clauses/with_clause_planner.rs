//! WITH子句规划器
//! 处理WITH子句的规划
//! 负责规划WITH子句用于链接多个查询部分

use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use super::order_by_planner::OrderByClausePlanner;
use super::pagination_planner::PaginationPlanner;
use crate::query::planner::match_planning::utils::connector::SegmentsConnector;
use super::where_clause_planner::WhereClausePlanner;
use super::yield_planner::YieldClausePlanner;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind, WithClauseContext};

/// WITH子句规划器
/// 负责规划WITH子句用于链接多个查询部分
#[derive(Debug)]
pub struct WithClausePlanner;

impl WithClausePlanner {
    pub fn new() -> Self {
        Self
    }

    /// 构建WITH子句计划
    fn build_with(
        &mut self,
        wctx: &WithClauseContext,
        sub_plan: &mut SubPlan,
    ) -> Result<(), PlannerError> {
        // 首先处理YIELD子句
        let mut yield_planner = YieldClausePlanner::new();
        let yield_clause_ctx = CypherClauseContext::Yield(wctx.yield_clause.clone());
        let yield_plan = yield_planner.transform(&yield_clause_ctx)?;

        sub_plan.tail = yield_plan.tail;
        sub_plan.root = yield_plan.root;

        // 处理ORDER BY子句
        if let Some(order_by) = &wctx.order_by {
            let mut order_by_planner = OrderByClausePlanner::new();
            let order_by_clause_ctx = CypherClauseContext::OrderBy(order_by.clone());
            let order_plan = order_by_planner.transform(&order_by_clause_ctx)?;

            let connector = SegmentsConnector::new();
            *sub_plan = connector.add_input(order_plan, sub_plan.clone(), true);
        }

        // 处理分页（LIMIT/OFFSET）
        if let Some(pagination) = &wctx.pagination {
            if pagination.skip != 0 || pagination.limit != i64::MAX {
                let mut pagination_planner = PaginationPlanner::new();
                let pagination_clause_ctx = CypherClauseContext::Pagination(pagination.clone());
                let pagination_plan = pagination_planner.transform(&pagination_clause_ctx)?;

                let connector = SegmentsConnector::new();
                *sub_plan = connector.add_input(pagination_plan, sub_plan.clone(), true);
            }
        }

        // 处理WHERE子句
        if let Some(where_clause) = &wctx.where_clause {
            let need_stable_filter = wctx.order_by.is_some(); // 如果有ORDER BY，需要稳定的过滤器
            let mut where_planner = WhereClausePlanner::new(need_stable_filter);
            let where_clause_ctx = CypherClauseContext::Where(where_clause.clone());
            let where_plan = where_planner.transform(&where_clause_ctx)?;

            let connector = SegmentsConnector::new();
            *sub_plan = connector.add_input(where_plan, sub_plan.clone(), true);
        }

        Ok(())
    }
}

impl CypherClausePlanner for WithClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::With) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for WithClausePlanner".to_string(),
            ));
        }

        let with_clause_ctx = match clause_ctx {
            CypherClauseContext::With(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected WithClauseContext".to_string(),
                ))
            }
        };

        let mut with_plan = SubPlan::new(None, None);
        self.build_with(with_clause_ctx, &mut with_plan)?;

        Ok(with_plan)
    }
}
