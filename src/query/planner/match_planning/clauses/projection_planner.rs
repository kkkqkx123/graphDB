/// 投影规划器
/// 提供RETURN和WITH子句的公共逻辑
/// 处理结果投影、排序、分页等公共功能
use super::order_by_planner::OrderByClausePlanner;
use super::pagination_planner::PaginationPlanner;
use super::where_clause_planner::WhereClausePlanner;
use super::yield_planner::YieldClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::utils::connection_strategy::UnifiedConnector;

use crate::query::planner::plan::factory::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{
    clause_structs::{
        OrderByClauseContext, PaginationContext, WhereClauseContext, YieldClauseContext,
    },
    CypherClauseContext,
};

/// 投影规划器
/// 提供RETURN和WITH子句的公共逻辑
#[derive(Debug)]
pub struct ProjectionPlanner;

impl ProjectionPlanner {
    /// 创建新的投影规划器
    pub fn new() -> Self {
        Self
    }

    /// 构建投影计划
    ///
    /// 处理YIELD、ORDER BY、分页和去重等公共逻辑
    pub fn build_projection_plan(
        &self,
        yield_clause: &YieldClauseContext,
        order_by: Option<&OrderByClauseContext>,
        pagination: Option<&PaginationContext>,
        where_clause: Option<&WhereClauseContext>,
        distinct: bool,
        need_stable_filter: bool,
    ) -> Result<SubPlan, PlannerError> {
        // 创建规划上下文
        let query_info =
            crate::query::planner::match_planning::core::cypher_clause_planner::QueryInfo {
                query_id: "test".to_string(),
                statement_type: "PROJECTION".to_string(),
            };
        let mut context = crate::query::planner::match_planning::core::cypher_clause_planner::PlanningContext::new(query_info);

        // 首先处理YIELD子句（投影部分）
        let yield_planner = YieldClausePlanner::new();
        let yield_clause_ctx = CypherClauseContext::Yield(yield_clause.clone());

        // 创建空的输入计划用于YIELD子句
        let empty_input_plan = SubPlan::new(None, None);
        let mut plan =
            yield_planner.transform(&yield_clause_ctx, Some(&empty_input_plan), &mut context)?;

        // 处理ORDER BY子句
        if let Some(order_by) = order_by {
            let order_by_planner = OrderByClausePlanner::new();
            let order_by_clause_ctx = CypherClauseContext::OrderBy(order_by.clone());

            // 确保plan有有效的root节点
            if plan.root.is_none() {
                // 如果YIELD没有产生有效的计划，创建一个空的起始节点
                let start_node = PlanNodeFactory::create_placeholder_node()?;
                plan = SubPlan::new(Some(start_node.clone()), Some(start_node));
            }

            let order_plan =
                order_by_planner.transform(&order_by_clause_ctx, Some(&plan), &mut context)?;

            // 使用新的统一连接器
            plan = UnifiedConnector::add_input(
                &crate::query::context::ast::AstContext::from_strings("PROJECTION", "test"),
                &order_plan,
                &plan,
                true,
            )?;
        }

        // 处理分页（LIMIT/OFFSET）
        if let Some(pagination) = pagination {
            if pagination.skip != 0 || pagination.limit != i64::MAX {
                let pagination_planner = PaginationPlanner::new();
                let pagination_clause_ctx = CypherClauseContext::Pagination(pagination.clone());

                // 确保plan有有效的root节点
                if plan.root.is_none() {
                    // 如果前面的步骤没有产生有效的计划，创建一个空的起始节点
                    let start_node = PlanNodeFactory::create_placeholder_node()?;
                    plan = SubPlan::new(Some(start_node.clone()), Some(start_node));
                }

                let pagination_plan = pagination_planner.transform(
                    &pagination_clause_ctx,
                    Some(&plan),
                    &mut context,
                )?;

                // 使用新的统一连接器
                plan = UnifiedConnector::add_input(
                    &crate::query::context::ast::AstContext::from_strings("PROJECTION", "test"),
                    &pagination_plan,
                    &plan,
                    true,
                )?;
            }
        }

        // 处理WHERE子句（主要用于WITH子句）
        if let Some(where_clause) = where_clause {
            let where_planner = WhereClausePlanner::new(need_stable_filter);
            let where_clause_ctx = CypherClauseContext::Where(where_clause.clone());

            // 确保plan有有效的root节点
            if plan.root.is_none() {
                // 如果前面的步骤没有产生有效的计划，创建一个空的起始节点
                let start_node = PlanNodeFactory::create_placeholder_node()?;
                plan = SubPlan::new(Some(start_node.clone()), Some(start_node));
            }

            let where_plan =
                where_planner.transform(&where_clause_ctx, Some(&plan), &mut context)?;

            // 使用新的统一连接器
            plan = UnifiedConnector::add_input(
                &crate::query::context::ast::AstContext::from_strings("PROJECTION", "test"),
                &where_plan,
                &plan,
                true,
            )?;
        }

        // 处理去重（主要用于RETURN子句）
        if distinct {
            // 创建去重节点
            let dedup_node = PlanNodeFactory::create_placeholder_node()?;

            // TODO: 设置去重键

            let dedup_plan = SubPlan::new(Some(dedup_node.clone()), Some(dedup_node));
            // 使用新的统一连接器
            plan = UnifiedConnector::add_input(
                &crate::query::context::ast::AstContext::from_strings("PROJECTION", "test"),
                &dedup_plan,
                &plan,
                true,
            )?;
        }

        Ok(plan)
    }

    /// 构建RETURN投影计划
    ///
    /// 专门用于RETURN子句的投影计划构建
    pub fn build_return_projection(
        &self,
        yield_clause: &YieldClauseContext,
        order_by: Option<&OrderByClauseContext>,
        pagination: Option<&PaginationContext>,
        distinct: bool,
    ) -> Result<SubPlan, PlannerError> {
        self.build_projection_plan(yield_clause, order_by, pagination, None, distinct, false)
    }

    /// 构建WITH投影计划
    ///
    /// 专门用于WITH子句的投影计划构建
    pub fn build_with_projection(
        &self,
        yield_clause: &YieldClauseContext,
        order_by: Option<&OrderByClauseContext>,
        pagination: Option<&PaginationContext>,
        where_clause: Option<&WhereClauseContext>,
    ) -> Result<SubPlan, PlannerError> {
        let need_stable_filter = order_by.is_some(); // 如果有ORDER BY，需要稳定的过滤器
        self.build_projection_plan(
            yield_clause,
            order_by,
            pagination,
            where_clause,
            false,
            need_stable_filter,
        )
    }

    /// 检查是否需要处理ORDER BY
    pub fn needs_order_by(order_by: Option<&OrderByClauseContext>) -> bool {
        order_by.is_some()
    }

    /// 检查是否需要处理分页
    pub fn needs_pagination(pagination: Option<&PaginationContext>) -> bool {
        if let Some(pagination) = pagination {
            pagination.skip != 0 || pagination.limit != i64::MAX
        } else {
            false
        }
    }

    /// 检查是否需要处理WHERE
    pub fn needs_where(where_clause: Option<&WhereClauseContext>) -> bool {
        where_clause.is_some()
    }

    /// 检查是否需要处理去重
    pub fn needs_distinct(distinct: bool) -> bool {
        distinct
    }

    /// 估算投影计划的成本
    ///
    /// 根据投影操作的复杂性估算成本
    pub fn estimate_projection_cost(
        order_by: Option<&OrderByClauseContext>,
        pagination: Option<&PaginationContext>,
        where_clause: Option<&WhereClauseContext>,
        distinct: bool,
    ) -> f64 {
        let mut cost = 0.0;

        // ORDER BY 成本
        if Self::needs_order_by(order_by) {
            cost += 100.0; // 排序操作成本较高
        }

        // 分页成本
        if Self::needs_pagination(pagination) {
            cost += 10.0; // 分页操作成本较低
        }

        // WHERE 过滤成本
        if Self::needs_where(where_clause) {
            cost += 50.0; // 过滤操作成本中等
        }

        // 去重成本
        if Self::needs_distinct(distinct) {
            cost += 80.0; // 去重操作成本较高
        }

        cost
    }
}

impl Default for ProjectionPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::clause_structs::{
        OrderByClauseContext, PaginationContext, WhereClauseContext, YieldClauseContext,
    };

    fn create_test_yield_clause() -> YieldClauseContext {
        YieldClauseContext {
            yield_columns: vec![],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
            query_parts: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn create_test_order_by() -> OrderByClauseContext {
        OrderByClauseContext {
            indexed_order_factors: vec![],
        }
    }

    fn create_test_pagination() -> PaginationContext {
        PaginationContext {
            skip: 10,
            limit: 100,
        }
    }

    fn create_test_where() -> WhereClauseContext {
        WhereClauseContext {
            filter: None,
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![],
            query_parts: Vec::new(),
            errors: Vec::new(),
        }
    }

    #[test]
    fn test_projection_planner_new() {
        let _planner = ProjectionPlanner::new();
        // 测试创建实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_projection_planner_default() {
        let _planner = ProjectionPlanner::default();
        // 测试默认创建实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_build_return_projection() {
        let planner = ProjectionPlanner::new();
        let yield_clause = create_test_yield_clause();
        let order_by = Some(create_test_order_by());
        let pagination = Some(create_test_pagination());

        let result = planner.build_return_projection(
            &yield_clause,
            order_by.as_ref(),
            pagination.as_ref(),
            true,
        );
        if let Err(e) = &result {
            println!("Error in test_build_return_projection: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_with_projection() {
        let planner = ProjectionPlanner::new();
        let yield_clause = create_test_yield_clause();
        let order_by = Some(create_test_order_by());
        let pagination = Some(create_test_pagination());
        let where_clause = Some(create_test_where());

        let result = planner.build_with_projection(
            &yield_clause,
            order_by.as_ref(),
            pagination.as_ref(),
            where_clause.as_ref(),
        );
        if let Err(e) = &result {
            println!("Error in test_build_with_projection: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_needs_order_by() {
        assert!(ProjectionPlanner::needs_order_by(Some(
            &create_test_order_by()
        )));
        assert!(!ProjectionPlanner::needs_order_by(None));
    }

    #[test]
    fn test_needs_pagination() {
        assert!(ProjectionPlanner::needs_pagination(Some(
            &create_test_pagination()
        )));

        let empty_pagination = PaginationContext {
            skip: 0,
            limit: i64::MAX,
        };
        assert!(!ProjectionPlanner::needs_pagination(Some(
            &empty_pagination
        )));
        assert!(!ProjectionPlanner::needs_pagination(None));
    }

    #[test]
    fn test_needs_where() {
        assert!(ProjectionPlanner::needs_where(Some(&create_test_where())));
        assert!(!ProjectionPlanner::needs_where(None));
    }

    #[test]
    fn test_needs_distinct() {
        assert!(ProjectionPlanner::needs_distinct(true));
        assert!(!ProjectionPlanner::needs_distinct(false));
    }

    #[test]
    fn test_estimate_projection_cost() {
        let order_by = Some(create_test_order_by());
        let pagination = Some(create_test_pagination());
        let where_clause = Some(create_test_where());

        let cost = ProjectionPlanner::estimate_projection_cost(
            order_by.as_ref(),
            pagination.as_ref(),
            where_clause.as_ref(),
            true,
        );

        // 预期成本 = 100 (ORDER BY) + 10 (分页) + 50 (WHERE) + 80 (去重) = 240
        assert_eq!(cost, 240.0);
    }

    #[test]
    fn test_estimate_projection_cost_minimal() {
        let cost = ProjectionPlanner::estimate_projection_cost(None, None, None, false);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_build_projection_plan_all_features() {
        let planner = ProjectionPlanner::new();
        let yield_clause = create_test_yield_clause();
        let order_by = Some(create_test_order_by());
        let pagination = Some(create_test_pagination());
        let where_clause = Some(create_test_where());

        let result = planner.build_projection_plan(
            &yield_clause,
            order_by.as_ref(),
            pagination.as_ref(),
            where_clause.as_ref(),
            true,
            true,
        );

        if let Err(e) = &result {
            println!("Error in test_build_projection_plan_all_features: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_projection_plan_yield_only() {
        let planner = ProjectionPlanner::new();
        let yield_clause = create_test_yield_clause();

        let result = planner.build_projection_plan(&yield_clause, None, None, None, false, false);

        if let Err(e) = &result {
            println!("Error in test_build_projection_plan_yield_only: {:?}", e);
        }
        assert!(result.is_ok());
    }
}
