//! RETURN Statement Planner
//!
//! Query planning for statements that handle the RETURN command

use crate::core::YieldColumn;
use crate::query::parser::ast::{ReturnItem, ReturnStmt, Stmt};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, DedupNode, LimitNode, ProjectNode, SortNode},
};
use crate::query::planning::plan::{PlanNodeEnum, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// RETURN statement planner
/// Responsible for converting the RETURN statement into an execution plan.
#[derive(Debug, Clone)]
pub struct ReturnPlanner;

impl ReturnPlanner {
    /// Create a new RETURN planner.
    pub fn new() -> Self {
        Self
    }

    /// Extract the ReturnStmt from the Stmt.
    fn extract_return_stmt(&self, stmt: &Stmt) -> Result<ReturnStmt, PlannerError> {
        match stmt {
            Stmt::Return(return_stmt) => Ok(return_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 RETURN".to_string(),
            )),
        }
    }

    /// Convert “ReturnItem” to “YieldColumn”.
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

impl Planner for ReturnPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let _ = qctx;

        // Use the verification information to optimize the planning process.
        let validation_info = &validated.validation_info;

        // Check the semantic information.
        let referenced_tags = &validation_info.semantic_info.referenced_tags;
        if !referenced_tags.is_empty() {
            log::debug!("RETURN 引用的标签: {:?}", referenced_tags);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("RETURN 引用的属性: {:?}", referenced_properties);
        }

        let return_stmt = self.extract_return_stmt(validated.stmt())?;

        // Create a parameter node as the input.
        let arg_node = ArgumentNode::new(next_node_id(), "return_input");
        let mut current_node = PlanNodeEnum::Argument(arg_node.clone());

        // Translate the given text into English:  
“Convert the returned items into projection columns.”
        let yield_columns: Vec<YieldColumn> = return_stmt
            .items
            .iter()
            .map(|item| self.convert_return_item_to_yield_column(item, validated))
            .collect();

        // Create a projection node.
        let project_node = ProjectNode::new(current_node.clone(), yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;
        current_node = PlanNodeEnum::Project(project_node);

        // If deduplication is required, create a deduplication node.
        if return_stmt.distinct {
            let dedup_node = DedupNode::new(current_node.clone()).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create DedupNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Dedup(dedup_node);
        }

        // If there is an ORDER BY clause, create a sorting node.
        if let Some(order_by) = &return_stmt.order_by {
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

        // If there is a SKIP clause, create a restriction node.
        if let Some(skip) = return_stmt.skip {
            let limit_node = LimitNode::new(current_node.clone(), skip as i64, 0).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create LimitNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Limit(limit_node);
        }

        // If there is a LIMIT clause, create a limit node.
        if let Some(limit) = return_stmt.limit {
            let limit_node =
                LimitNode::new(current_node.clone(), 0, limit as i64).map_err(|e| {
                    PlannerError::PlanGenerationFailed(format!("Failed to create LimitNode: {}", e))
                })?;
            current_node = PlanNodeEnum::Limit(limit_node);
        }

        // Create a SubPlan
        let sub_plan = SubPlan::new(Some(current_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Return(_))
    }
}

impl Default for ReturnPlanner {
    fn default() -> Self {
        Self::new()
    }
}
