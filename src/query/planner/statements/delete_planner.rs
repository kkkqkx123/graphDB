//! 删除操作规划器
//!
//! 处理 DELETE VERTEX/EDGE/TAG 语句的查询规划

use crate::query::QueryContext;
use crate::query::parser::ast::{DeleteStmt, DeleteTarget, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::core::YieldColumn;
use crate::core::types::{ContextualExpression, ExpressionContext};
use std::sync::Arc;

/// 删除操作规划器
/// 负责将 DELETE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct DeletePlanner;

impl DeletePlanner {
    /// 创建新的删除规划器
    pub fn new() -> Self {
        Self
    }

    /// 从 Stmt 提取 DeleteStmt
    fn extract_delete_stmt(&self, stmt: &Stmt) -> Result<DeleteStmt, PlannerError> {
        match stmt {
            Stmt::Delete(delete_stmt) => Ok(delete_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 DELETE".to_string(),
            )),
        }
    }
}

impl Planner for DeletePlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let delete_stmt = self.extract_delete_stmt(&validated.stmt)?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "delete_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // 根据删除目标类型构建不同的计划
        let ctx = Arc::new(ExpressionContext::new());
        let yield_columns = match &delete_stmt.target {
            DeleteTarget::Vertices(..) => {
                let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_vertices".to_string())
                );
                let id = ctx.register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, ctx.clone());
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Edges { .. } => {
                let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_edges".to_string())
                );
                let id = ctx.register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, ctx.clone());
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Tags { .. } => {
                let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_tags".to_string())
                );
                let id = ctx.register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, ctx.clone());
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Index(..) => {
                let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                    crate::core::Expression::Variable("deleted_index".to_string())
                );
                let id = ctx.register_expression(expr_meta);
                let ctx_expr = ContextualExpression::new(id, ctx);
                vec![YieldColumn {
                    expression: ctx_expr,
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
        };

        // 创建投影节点输出删除结果
        let project_node = ProjectNode::new(
            arg_node_enum.clone(),
            yield_columns,
        ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
            "Failed to create ProjectNode: {}",
            e
        )))?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(
            Some(final_node),
            Some(arg_node_enum),
        );

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Delete(_))
    }
}

impl Default for DeletePlanner {
    fn default() -> Self {
        Self::new()
    }
}
