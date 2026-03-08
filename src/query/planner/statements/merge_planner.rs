//! 合并操作规划器
//!
//! 处理 MERGE 语句的查询规划

use crate::core::types::ContextualExpression;
use crate::core::YieldColumn;
use crate::query::parser::ast::{MergeStmt, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, ProjectNode},
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// 合并操作规划器
/// 负责将 MERGE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct MergePlanner;

impl MergePlanner {
    /// 创建新的合并规划器
    pub fn new() -> Self {
        Self
    }

    /// 从 Stmt 提取 MergeStmt
    fn extract_merge_stmt(&self, stmt: &Stmt) -> Result<MergeStmt, PlannerError> {
        match stmt {
            Stmt::Merge(merge_stmt) => Ok(merge_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 MERGE".to_string(),
            )),
        }
    }
}

impl Planner for MergePlanner {
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
            log::debug!("MERGE 引用的标签: {:?}", referenced_tags);
        }

        let referenced_edges = &validation_info.semantic_info.referenced_edges;
        if !referenced_edges.is_empty() {
            log::debug!("MERGE 引用的边类型: {:?}", referenced_edges);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("MERGE 引用的属性: {:?}", referenced_properties);
        }

        let _merge_stmt = self.extract_merge_stmt(validated.stmt())?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "merge_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // 构建输出列 - 返回合并的结果
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
            crate::core::Expression::Variable("merged_count".to_string()),
        );
        let id = validated.expr_context().register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());

        let yield_columns = vec![YieldColumn {
            expression: ctx_expr,
            alias: "merged_count".to_string(),
            is_matched: false,
        }];

        // 创建投影节点输出合并结果
        let project_node = ProjectNode::new(arg_node_enum.clone(), yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(arg_node_enum));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Merge(_))
    }
}

impl Default for MergePlanner {
    fn default() -> Self {
        Self::new()
    }
}
