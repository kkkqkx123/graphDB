//! YIELD 语句规划器
//!
//! 处理 YIELD 语句的查询规划

use crate::core::YieldColumn;
use crate::query::parser::ast::{Stmt, YieldItem, YieldStmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, DedupNode, FilterNode, LimitNode, ProjectNode, SortNode},
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// YIELD 语句规划器
/// 负责将 YIELD 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct YieldPlanner;

impl YieldPlanner {
    /// 创建新的 YIELD 规划器
    pub fn new() -> Self {
        Self
    }

    /// 从 Stmt 提取 YieldStmt
    fn extract_yield_stmt(&self, stmt: &Stmt) -> Result<YieldStmt, PlannerError> {
        match stmt {
            Stmt::Yield(yield_stmt) => Ok(yield_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 YIELD".to_string(),
            )),
        }
    }

    /// 将 YieldItem 转换为 YieldColumn
    fn convert_yield_item_to_yield_column(
        &self,
        item: &YieldItem,
        _validated: &ValidatedStatement,
    ) -> YieldColumn {
        let expression = item.expression.clone();
        let alias = item.alias.clone().unwrap_or_else(|| {
            expression
                .get_expression()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "_".to_string())
        });
        YieldColumn {
            expression,
            alias,
            is_matched: false,
        }
    }
}

impl Planner for YieldPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let _ = qctx;

        // 使用验证信息进行优化规划
        let validation_info = &validated.validation_info;

        // 检查语义信息
        let referenced_tags = &validation_info.semantic_info.referenced_tags;
        if !referenced_tags.is_empty() {
            log::debug!("YIELD 引用的标签: {:?}", referenced_tags);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("YIELD 引用的属性: {:?}", referenced_properties);
        }

        let yield_stmt = self.extract_yield_stmt(validated.stmt())?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "yield_input");
        let mut current_node = PlanNodeEnum::Argument(arg_node.clone());

        // 转换返回项为投影列
        let yield_columns: Vec<YieldColumn> = yield_stmt
            .items
            .iter()
            .map(|item| self.convert_yield_item_to_yield_column(item, validated))
            .collect();

        // 创建投影节点
        let project_node = ProjectNode::new(current_node.clone(), yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;
        current_node = PlanNodeEnum::Project(project_node);

        // 如果有 WHERE 子句，创建过滤节点
        if let Some(where_clause) = &yield_stmt.where_clause {
            let filter_node =
                FilterNode::new(current_node.clone(), where_clause.clone()).map_err(|e| {
                    PlannerError::PlanGenerationFailed(format!(
                        "Failed to create FilterNode: {}",
                        e
                    ))
                })?;
            current_node = PlanNodeEnum::Filter(filter_node);
        }

        // 如果需要去重，创建去重节点
        if yield_stmt.distinct {
            let dedup_node = DedupNode::new(current_node.clone()).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create DedupNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Dedup(dedup_node);
        }

        // 如果有 ORDER BY 子句，创建排序节点
        if let Some(order_by) = &yield_stmt.order_by {
            let sort_items: Vec<crate::query::planner::plan::core::nodes::SortItem> = order_by
                .items
                .iter()
                .map(|item| {
                    let direction = match item.direction {
                        crate::query::parser::ast::OrderDirection::Asc => {
                            crate::core::types::graph_schema::OrderDirection::Asc
                        }
                        crate::query::parser::ast::OrderDirection::Desc => {
                            crate::core::types::graph_schema::OrderDirection::Desc
                        }
                    };
                    let column = item.expression.to_expression_string();
                    crate::query::planner::plan::core::nodes::SortItem::new(column, direction)
                })
                .collect();
            let sort_node = SortNode::new(current_node.clone(), sort_items).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create SortNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Sort(sort_node);
        }

        // 如果有 SKIP 子句，创建限制节点
        if let Some(skip) = yield_stmt.skip {
            let limit_node = LimitNode::new(current_node.clone(), skip as i64, 0).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create LimitNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Limit(limit_node);
        }

        // 如果有 LIMIT 子句，创建限制节点
        if let Some(limit) = yield_stmt.limit {
            let limit_node =
                LimitNode::new(current_node.clone(), 0, limit as i64).map_err(|e| {
                    PlannerError::PlanGenerationFailed(format!("Failed to create LimitNode: {}", e))
                })?;
            current_node = PlanNodeEnum::Limit(limit_node);
        }

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(current_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Yield(_))
    }
}

impl Default for YieldPlanner {
    fn default() -> Self {
        Self::new()
    }
}
