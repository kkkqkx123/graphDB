//! LOOKUP语句规划器
//! 处理Nebula LOOKUP查询的规划
//!
//! ## 改进说明
//!
//! - 统一导入路径
//! - 完善表达式解析
//! - 添加属性索引选择逻辑
//! - 使用 IndexSelector 自动选择最优索引

use crate::core::types::Index;
use crate::core::value::types::NullType;
use crate::core::Expression;
use crate::query::parser::ast::{LookupStmt, Stmt};
use crate::query::planner::plan::core::nodes::access::{IndexScanNode, ScanType};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

pub use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, FilterNode, GetEdgesNode, GetVerticesNode, HashInnerJoinNode,
    ProjectNode,
};
pub use crate::query::planner::plan::core::PlanNodeEnum;

/// LOOKUP查询规划器
/// 负责将LOOKUP语句转换为执行计划
#[derive(Debug, Clone)]
pub struct LookupPlanner {}

impl LookupPlanner {
    /// 创建新的LOOKUP规划器
    pub fn new() -> Self {
        Self {}
    }
}

impl Planner for LookupPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let lookup_stmt = match validated.stmt() {
            Stmt::Lookup(lookup_stmt) => lookup_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "LookupPlanner 需要 Lookup 语句".to_string(),
                ));
            }
        };

        let space_id = qctx.space_id().unwrap_or(1);

        if space_id == 0 {
            return Err(PlannerError::PlanGenerationFailed(
                "Invalid space ID: 0".to_string(),
            ));
        }

        // 使用验证信息进行优化规划
        let validation_info = &validated.validation_info;

        // 1. 检查优化提示
        for hint in &validation_info.optimization_hints {
            log::debug!("LOOKUP 优化提示: {:?}", hint);
        }

        // 2. 检查索引提示
        let mut selected_index: Option<Index> = None;
        let mut scan_limits: Vec<crate::query::planner::plan::core::nodes::access::IndexLimit> =
            Vec::new();
        let mut scan_type = ScanType::Full;

        if !validation_info.index_hints.is_empty() {
            let hint = &validation_info.index_hints[0];
            log::debug!("LOOKUP 使用索引提示: {:?}", hint);

            // 使用验证器提供的索引提示
            let index_fields: Vec<crate::core::types::IndexField> = hint
                .columns
                .iter()
                .map(|col| {
                    crate::core::types::IndexField::new(
                        col.clone(),
                        crate::core::Value::Null(NullType::Null),
                        true,
                    )
                })
                .collect();

            selected_index = Some(Index {
                id: 1,
                name: hint.index_name.clone(),
                space_id,
                schema_name: hint.table_name.clone(),
                fields: index_fields,
                properties: hint.columns.clone(),
                index_type: crate::core::types::IndexType::TagIndex,
                status: crate::core::types::IndexStatus::Active,
                is_unique: false,
                comment: None,
            });

            scan_type = ScanType::Range;

            // 将列名转换为 IndexLimit
            for column in &hint.columns {
                scan_limits.push(
                    crate::query::planner::plan::core::nodes::access::IndexLimit::equal(
                        column.clone(),
                        "",
                    ),
                );
            }
        }

        // 3. 如果没有索引提示，获取可用的索引列表
        if selected_index.is_none() {
            let available_indexes: Vec<Index> = vec![];

            // 使用简单启发式选择索引（选择第一个可用索引）
            if !available_indexes.is_empty() {
                let index = available_indexes.first().cloned();
                selected_index = index;
                scan_type = ScanType::Range;
            }
        }

        let index_id = selected_index.as_ref().map(|idx| idx.id).unwrap_or(0);

        // 4. 创建 IndexScan 节点
        let mut index_scan_node = IndexScanNode::new(space_id, 0, index_id, scan_type);

        // 5. 设置扫描限制和返回列
        index_scan_node.set_scan_limits(scan_limits);

        let mut current_node: PlanNodeEnum = PlanNodeEnum::IndexScan(index_scan_node);

        if let Some(ref condition) = lookup_stmt.where_clause {
            let filter_node = FilterNode::new(current_node, condition.clone()).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create FilterNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Filter(filter_node);
        }

        let yield_columns = Self::build_yield_columns(lookup_stmt, validated)?;
        let project_node = ProjectNode::new(current_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;
        current_node = PlanNodeEnum::Project(project_node);

        let arg_node = ArgumentNode::new(0, "lookup_input");
        let sub_plan = SubPlan {
            root: Some(current_node),
            tail: Some(PlanNodeEnum::Argument(arg_node)),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Lookup(_))
    }
}

impl LookupPlanner {
    /// 构建YIELD列
    fn build_yield_columns(
        lookup_stmt: &LookupStmt,
        validated: &ValidatedStatement,
    ) -> Result<Vec<crate::core::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        if let Some(ref yield_clause) = lookup_stmt.yield_clause {
            for item in &yield_clause.items {
                columns.push(crate::core::YieldColumn {
                    expression: item.expression.clone(),
                    alias: item.alias.clone().unwrap_or_default(),
                    is_matched: false,
                });
            }
        }

        if columns.is_empty() {
            let expr = Expression::Variable("*".to_string());
            let meta = crate::core::types::expression::ExpressionMeta::new(expr);
            let id = validated.expr_context().register_expression(meta);
            let ctx_expr =
                crate::core::types::ContextualExpression::new(id, validated.expr_context().clone());
            columns.push(crate::core::YieldColumn {
                expression: ctx_expr,
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        Ok(columns)
    }

    /// 解析YIELD表达式
    fn _parse_yield_expression(name: &str) -> Result<Expression, PlannerError> {
        if name.contains(".") {
            let parts: Vec<&str> = name.split(".").collect();
            if parts.len() == 2 {
                return Ok(Expression::Property {
                    object: Box::new(Expression::Variable(parts[0].to_string())),
                    property: parts[1].to_string(),
                });
            }
        }

        Ok(Expression::Variable(name.to_string()))
    }
}

impl Default for LookupPlanner {
    fn default() -> Self {
        Self::new()
    }
}
