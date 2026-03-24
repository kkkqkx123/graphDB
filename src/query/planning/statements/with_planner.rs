//! WITH 语句规划器
//!
//! 处理 WITH 语句的查询规划

use crate::core::YieldColumn;
use crate::query::parser::ast::{ReturnItem, Stmt, WithStmt};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, DedupNode, FilterNode, LimitNode, ProjectNode, SortNode},
};
use crate::query::planning::plan::{PlanNodeEnum, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// WITH 语句规划器
/// 负责将 WITH 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct WithPlanner;

impl WithPlanner {
    /// 创建新的 WITH 规划器
    pub fn new() -> Self {
        Self
    }

    /// 从 Stmt 提取 WithStmt
    fn extract_with_stmt(&self, stmt: &Stmt) -> Result<WithStmt, PlannerError> {
        match stmt {
            Stmt::With(with_stmt) => Ok(with_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 WITH".to_string(),
            )),
        }
    }

    /// 将 ReturnItem 转换为 YieldColumn
    fn convert_return_item_to_yield_column(
        &self,
        item: &ReturnItem,
        _validated: &ValidatedStatement,
    ) -> YieldColumn {
        let (expression, alias) = match item {
            ReturnItem::Expression { expression, alias } => (expression.clone(), alias.clone()),
        };
        let alias = alias.unwrap_or_else(|| {
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

impl Planner for WithPlanner {
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
            log::debug!("WITH 引用的标签: {:?}", referenced_tags);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("WITH 引用的属性: {:?}", referenced_properties);
        }

        let with_stmt = self.extract_with_stmt(validated.stmt())?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "with_input");
        let mut current_node = PlanNodeEnum::Argument(arg_node.clone());

        // 转换返回项为投影列
        let yield_columns: Vec<YieldColumn> = with_stmt
            .items
            .iter()
            .map(|item| self.convert_return_item_to_yield_column(item, validated))
            .collect();

        // 创建投影节点
        let project_node = ProjectNode::new(current_node.clone(), yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;
        current_node = PlanNodeEnum::Project(project_node);

        // 如果有 WHERE 子句，创建过滤节点
        if let Some(where_clause) = &with_stmt.where_clause {
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
        if with_stmt.distinct {
            let dedup_node = DedupNode::new(current_node.clone()).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create DedupNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Dedup(dedup_node);
        }

        // 如果有 ORDER BY 子句，创建排序节点
        if let Some(order_by) = &with_stmt.order_by {
            let sort_items: Vec<crate::query::planning::plan::core::nodes::SortItem> = order_by
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
                    crate::query::planning::plan::core::nodes::SortItem::new(column, direction)
                })
                .collect();
            let sort_node = SortNode::new(current_node.clone(), sort_items).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create SortNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Sort(sort_node);
        }

        // 如果有 SKIP 子句，创建限制节点
        if let Some(skip) = with_stmt.skip {
            let limit_node = LimitNode::new(current_node.clone(), skip as i64, 0).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create LimitNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Limit(limit_node);
        }

        // 如果有 LIMIT 子句，创建限制节点
        if let Some(limit) = with_stmt.limit {
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
        matches!(stmt, Stmt::With(_))
    }
}

impl Default for WithPlanner {
    fn default() -> Self {
        Self::new()
    }
}
