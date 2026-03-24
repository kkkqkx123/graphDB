//! 删除属性/标签规划器
//!
//! 处理 REMOVE 语句的查询规划

use crate::core::types::ContextualExpression;
use crate::core::YieldColumn;
use crate::query::parser::ast::{RemoveStmt, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, ProjectNode, RemoveNode},
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// 删除属性/标签规划器
/// 负责将 REMOVE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct RemovePlanner;

impl RemovePlanner {
    /// 创建新的删除规划器
    pub fn new() -> Self {
        Self
    }

    /// 从 Stmt 提取 RemoveStmt
    fn extract_remove_stmt(&self, stmt: &Stmt) -> Result<RemoveStmt, PlannerError> {
        match stmt {
            Stmt::Remove(remove_stmt) => Ok(remove_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 REMOVE".to_string(),
            )),
        }
    }
}

impl Planner for RemovePlanner {
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
            log::debug!("REMOVE 引用的标签: {:?}", referenced_tags);
        }

        let referenced_properties = &validation_info.semantic_info.referenced_properties;
        if !referenced_properties.is_empty() {
            log::debug!("REMOVE 引用的属性: {:?}", referenced_properties);
        }

        let remove_stmt = self.extract_remove_stmt(validated.stmt())?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "remove_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // 解析 REMOVE 项，确定是删除属性还是标签
        let mut remove_items = Vec::new();
        for item in &remove_stmt.items {
            // 根据表达式类型判断是属性还是标签
            let expr = item.get_expression();
            if let Some(expression) = expr {
                let item_type = match expression {
                    crate::core::Expression::Property { .. } => "property",
                    crate::core::Expression::Label { .. } => "tag",
                    _ => "property",
                };
                remove_items.push((item_type.to_string(), item.clone()));
            }
        }

        // 创建 Remove 节点
        let remove_node = RemoveNode::new(arg_node_enum.clone(), remove_items).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create RemoveNode: {}", e))
        })?;

        let remove_node_enum = PlanNodeEnum::Remove(remove_node);

        // 构建输出列 - 返回删除的属性/标签数量
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
            crate::core::Expression::Variable("removed_count".to_string()),
        );
        let id = validated.expr_context().register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, validated.expr_context().clone());

        let yield_columns = vec![YieldColumn {
            expression: ctx_expr,
            alias: "removed_count".to_string(),
            is_matched: false,
        }];

        // 创建投影节点输出删除结果
        let project_node =
            ProjectNode::new(remove_node_enum.clone(), yield_columns).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
            })?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(arg_node_enum));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Remove(_))
    }
}

impl Default for RemovePlanner {
    fn default() -> Self {
        Self::new()
    }
}
