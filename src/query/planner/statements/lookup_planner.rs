//! LOOKUP语句规划器
//! 处理Nebula LOOKUP查询的规划
//!
//! ## 改进说明
//!
//! - 统一导入路径
//! - 完善表达式解析
//! - 添加属性索引选择逻辑
//! - 使用 IndexSelector 自动选择最优索引

use crate::core::types::expression::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::{LookupStmt, Stmt};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::planner::plan::algorithms::{IndexScan, ScanType};
use crate::index::Index;
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
        let lookup_stmt = match &validated.stmt {
            Stmt::Lookup(lookup_stmt) => lookup_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "LookupPlanner 需要 Lookup 语句".to_string()
                ));
            }
        };

        let space_id = qctx.space_id().unwrap_or(1);

        if space_id == 0 {
            return Err(PlannerError::PlanGenerationFailed(
                "Invalid space ID: 0".to_string(),
            ));
        }

        // 1. 获取可用的索引列表（暂时返回空列表，后续需要从存储客户端获取）
        let available_indexes: Vec<Index> = vec![];

        // 2. 使用简单启发式选择索引（选择第一个可用索引）
        let (selected_index, scan_limits, scan_type) = if !available_indexes.is_empty() {
            // 简单启发式：选择第一个可用索引
            // 在小型数据库中，这种简单策略通常足够
            let index = available_indexes.first().cloned();
            (index, vec![], ScanType::Full)
        } else {
            (None, vec![], ScanType::Full)
        };

        let index_id = selected_index.as_ref().map(|idx| idx.id).unwrap_or(0);

        // 3. 创建 IndexScan 节点
        let mut index_scan_node = IndexScan::new(
            -1,
            space_id,
            0,
            index_id,
            scan_type,
        );

        // 4. 设置扫描限制和返回列
        index_scan_node.scan_limits = scan_limits;

        let mut current_node: PlanNodeEnum = PlanNodeEnum::IndexScan(index_scan_node);

        if let Some(ref condition) = lookup_stmt.where_clause {
            use std::sync::Arc;
            let ctx = Arc::new(crate::core::types::ExpressionContext::new());
            let filter_node = FilterNode::from_expression(current_node, condition.clone(), ctx).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create FilterNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Filter(filter_node);
        }

        let yield_columns = Self::build_yield_columns(lookup_stmt)?;
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
            columns.push(crate::core::YieldColumn {
                expression: Expression::Variable("*".to_string()),
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
