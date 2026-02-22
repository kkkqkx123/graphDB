//! 统一 MATCH 语句规划器
//!
//! 实现 StatementPlanner 接口，处理完整的 MATCH 查询规划。
//! 整合了以下功能：
//! - 节点和边模式匹配
//! - WHERE 条件过滤
//! - RETURN 投影
//! - ORDER BY 排序
//! - LIMIT/SKIP 分页

use crate::core::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::{LimitNode, ProjectNode, ScanVerticesNode, SortNode, SortItem};
use crate::core::types::graph_schema::OrderDirection;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::statements::statement_planner::StatementPlanner;
use crate::core::YieldColumn;
use crate::query::validator::structs::OrderByItem;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

/// 分页信息结构体
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub skip: usize,
    pub limit: usize,
}

/// MATCH 语句规划器
///
/// 负责将 MATCH 查询转换为可执行的执行计划。
/// 实现 StatementPlanner 接口，提供统一的规划入口。
#[derive(Debug, Clone)]
pub struct MatchStatementPlanner {
    config: MatchPlannerConfig,
}

#[derive(Debug, Clone, Default)]
pub struct MatchPlannerConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub enable_index_optimization: bool,
}

impl MatchStatementPlanner {
    pub fn new() -> Self {
        Self {
            config: MatchPlannerConfig::default(),
        }
    }

    pub fn with_config(config: MatchPlannerConfig) -> Self {
        Self { config }
    }
}

impl Planner for MatchStatementPlanner {
    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Match(_))
    }

    fn transform(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError> {
        let space_id = qctx.rctx().space_id().unwrap_or(1) as u64;
        self.plan_match_pattern(stmt, space_id)
    }

    fn transform_with_full_context(
        &mut self,
        qctx: Arc<QueryContext>,
        stmt: &Stmt,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(stmt, qctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }
}

impl StatementPlanner for MatchStatementPlanner {
    fn statement_type(&self) -> &'static str {
        "MATCH"
    }

    fn supported_clause_kinds(&self) -> &[CypherClauseKind] {
        const SUPPORTED_CLAUSES: &[CypherClauseKind] = &[
            CypherClauseKind::Match,
            CypherClauseKind::Where,
            CypherClauseKind::Return,
            CypherClauseKind::OrderBy,
            CypherClauseKind::Pagination,
        ];
        SUPPORTED_CLAUSES
    }
}

impl MatchStatementPlanner {
    fn plan_match_pattern(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(_match_stmt) => {
                let mut plan = self.plan_node_pattern(space_id)?;

                if let Some(condition) = self.extract_where_condition(stmt)? {
                    plan = self.plan_filter(plan, condition, space_id)?;
                }

                if let Some(columns) = self.extract_return_columns(stmt)? {
                    plan = self.plan_project(plan, columns, space_id)?;
                }

                if let Some(order_by) = self.extract_order_by(stmt)? {
                    plan = self.plan_sort(plan, order_by, space_id)?;
                }

                if let Some(pagination) = self.extract_pagination(stmt)? {
                    plan = self.plan_limit(plan, pagination)?;
                }

                Ok(plan)
            }
            _ => Err(PlannerError::InvalidOperation(
                "Expected MATCH statement".to_string()
            ))
        }
    }

    fn plan_node_pattern(&self, space_id: u64) -> Result<SubPlan, PlannerError> {
        let scan_node = ScanVerticesNode::new(space_id);
        Ok(SubPlan::from_root(scan_node.into_enum()))
    }

    fn plan_filter(
        &self,
        input_plan: SubPlan,
        condition: Expression,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let filter_node = FilterNode::new(input_node.clone(), condition)?;
        Ok(SubPlan::new(Some(filter_node.into_enum()), input_plan.tail))
    }

    fn plan_project(
        &self,
        input_plan: SubPlan,
        columns: Vec<YieldColumn>,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let project_node = ProjectNode::new(input_node.clone(), columns)?;
        Ok(SubPlan::new(Some(project_node.into_enum()), input_plan.tail))
    }

    fn plan_sort(
        &self,
        input_plan: SubPlan,
        order_by: Vec<OrderByItem>,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let sort_items: Vec<SortItem> = order_by
            .into_iter()
            .map(|item| {
                let column = self.expression_to_string(&item.expression);
                let direction = if item.desc { OrderDirection::Desc } else { OrderDirection::Asc };
                SortItem::new(column, direction)
            })
            .collect();

        let sort_node = SortNode::new(input_node.clone(), sort_items)?;
        Ok(SubPlan::new(Some(sort_node.into_enum()), input_plan.tail))
    }

    /// 将表达式转换为字符串表示
    /// 
    /// 使用 Expression::to_expression_string() 方法
    fn expression_to_string(&self, expr: &Expression) -> String {
        expr.to_expression_string()
    }

    fn plan_limit(
        &self,
        input_plan: SubPlan,
        pagination: PaginationInfo,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let limit_node = LimitNode::new(input_node.clone(), pagination.skip as i64, pagination.limit as i64)?;
        let limit_node_enum = limit_node.into_enum();
        Ok(SubPlan::new(Some(limit_node_enum), input_plan.tail))
    }

    fn extract_where_condition(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Expression>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                Ok(match_stmt.where_clause.clone())
            }
            _ => Ok(None),
        }
    }

    fn extract_return_columns(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Vec<YieldColumn>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(return_clause) = &match_stmt.return_clause {
                    let mut columns = Vec::new();
                    for item in &return_clause.items {
                        match item {
                            crate::query::parser::ast::stmt::ReturnItem::Expression { expression, alias } => {
                                columns.push(YieldColumn {
                                    expression: expression.clone(),
                                    alias: alias.clone().unwrap_or_default(),
                                    is_matched: false,
                                });
                            }
                            crate::query::parser::ast::stmt::ReturnItem::All => {
                                columns.push(YieldColumn {
                                    expression: crate::core::Expression::Variable("*".to_string()),
                                    alias: "*".to_string(),
                                    is_matched: false,
                                });
                            }
                        }
                    }
                    if columns.is_empty() {
                        columns.push(YieldColumn {
                            expression: crate::core::Expression::Variable("*".to_string()),
                            alias: "*".to_string(),
                            is_matched: false,
                        });
                    }
                    Ok(Some(columns))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn extract_order_by(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Vec<OrderByItem>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(order_by_clause) = &match_stmt.order_by {
                    let items = order_by_clause.items.iter().map(|item| {
                        OrderByItem {
                            expression: item.expression.clone(),
                            desc: item.direction == crate::query::parser::ast::types::OrderDirection::Desc,
                        }
                    }).collect();
                    Ok(Some(items))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn extract_pagination(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<PaginationInfo>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                let skip = match_stmt.skip.unwrap_or(0);
                let limit = match_stmt.limit.unwrap_or(self.config.default_limit);
                Ok(Some(PaginationInfo { skip, limit }))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_statement_planner_creation() {
        let planner = MatchStatementPlanner::new();
        assert_eq!(planner.statement_type(), "MATCH");
        // name() 返回完整的类型路径，检查是否包含类型名称
        assert!(planner.name().contains("MatchStatementPlanner"));
    }

    #[test]
    fn test_supported_clauses() {
        let planner = MatchStatementPlanner::new();
        let clauses = planner.supported_clause_kinds();
        assert!(clauses.contains(&CypherClauseKind::Match));
        assert!(clauses.contains(&CypherClauseKind::Where));
        assert!(clauses.contains(&CypherClauseKind::Return));
        assert!(clauses.contains(&CypherClauseKind::OrderBy));
        assert!(clauses.contains(&CypherClauseKind::Pagination));
    }
}
