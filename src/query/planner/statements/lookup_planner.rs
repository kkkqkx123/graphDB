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
use crate::query::context::ast::{AstContext, LookupContext};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::plan::algorithms::{IndexScan, ScanType};
use crate::query::optimizer::IndexSelector;
use crate::index::Index;

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

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配LOOKUP查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "LOOKUP"
    }
}

impl Planner for LookupPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let lookup_ctx = LookupContext::new(ast_ctx.clone());

        let space_id = ast_ctx.space().space_id.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("Space ID is required for LOOKUP query".to_string())
        })?;

        if space_id == 0 {
            return Err(PlannerError::PlanGenerationFailed(
                "Invalid space ID: 0".to_string(),
            ));
        }

        if lookup_ctx.schema_id == 0 {
            return Err(PlannerError::PlanGenerationFailed(
                "Invalid schema ID: 0".to_string(),
            ));
        }

        // 1. 获取可用的索引列表（从元数据服务）
        let available_indexes = self.get_available_indexes(ast_ctx, space_id as u64, lookup_ctx.schema_id, lookup_ctx.is_edge)?;

        // 2. 使用 IndexSelector 选择最优索引并获取评分详情
        let (selected_index, scan_limits, scan_type, score_detail) = if !available_indexes.is_empty() {
            if let Some((candidate, detail)) = IndexSelector::select_best_index_with_detail(&available_indexes, &lookup_ctx.filter) {
                let scan_limits = IndexSelector::hints_to_limits(&candidate.column_hints);
                let scan_type = if candidate.column_hints.is_empty() {
                    ScanType::Full
                } else {
                    candidate.column_hints[0].scan_type
                };
                
                // 记录评分信息到日志（实际应用中可以用于查询分析）
                log::debug!(
                    "LOOKUP 选择索引: {} (ID: {}), 评分: {}, 匹配率: {:.2}%, 预估行数: {}",
                    detail.index_name,
                    detail.index_id,
                    detail.total_score,
                    detail.match_ratio * 100.0,
                    detail.estimated_rows
                );
                
                (Some(candidate.index), scan_limits, scan_type, Some(detail))
            } else {
                (available_indexes.first().cloned(), vec![], ScanType::Full, None)
            }
        } else {
            (None, vec![], ScanType::Full, None)
        };
        
        // 存储评分详情供后续使用（如查询分析、优化器统计）
        let _score_detail = score_detail;

        let index_id = selected_index.as_ref().map(|idx| idx.id).unwrap_or(lookup_ctx.schema_id);

        // 3. 创建 IndexScan 节点，使用动态选择的扫描类型
        let mut index_scan_node = IndexScan::new(
            -1,
            space_id as i32,
            lookup_ctx.schema_id,
            index_id,
            scan_type,
        );

        // 4. 设置扫描限制和返回列
        index_scan_node.scan_limits = scan_limits;
        index_scan_node.return_columns = lookup_ctx.idx_return_cols.clone();

        let mut current_node: PlanNodeEnum = PlanNodeEnum::IndexScan(index_scan_node);

        if let Some(ref condition) = lookup_ctx.filter {
            let filter_node = FilterNode::new(current_node, condition.clone()).map_err(|e| {
                PlannerError::PlanGenerationFailed(format!("Failed to create FilterNode: {}", e))
            })?;
            current_node = PlanNodeEnum::Filter(filter_node);
        }

        let yield_columns = Self::build_yield_columns(&lookup_ctx)?;
        let project_node = ProjectNode::new(current_node, yield_columns).map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;
        current_node = PlanNodeEnum::Project(project_node);

        let final_node = if lookup_ctx.dedup {
            match DedupNode::new(current_node.clone()) {
                Ok(dedup) => PlanNodeEnum::Dedup(dedup),
                Err(_) => current_node,
            }
        } else {
            current_node
        };

        let arg_node = ArgumentNode::new(0, "lookup_input");
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(PlanNodeEnum::Argument(arg_node)),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl LookupPlanner {
    /// 获取可用的索引列表
    /// 从元数据服务获取真实的索引列表
    fn get_available_indexes(
        &self,
        ast_ctx: &AstContext,
        space_id: u64,
        schema_id: i32,
        is_edge: bool,
    ) -> Result<Vec<Index>, PlannerError> {
        // 从查询上下文中获取索引元数据管理器
        let index_manager = ast_ctx.index_metadata_manager()
            .ok_or_else(|| PlannerError::PlanGenerationFailed(
                "Index metadata manager not available".to_string()
            ))?;

        // 获取schema名称
        let schema_name = if is_edge {
            ast_ctx.get_edge_type_name_by_id(space_id, schema_id)
                .ok_or_else(|| PlannerError::PlanGenerationFailed(
                    format!("Edge type not found for ID: {}", schema_id)
                ))?
        } else {
            ast_ctx.get_tag_name_by_id(space_id, schema_id)
                .ok_or_else(|| PlannerError::PlanGenerationFailed(
                    format!("Tag not found for ID: {}", schema_id)
                ))?
        };

        // 从元数据服务获取索引列表
        let indexes = if is_edge {
            index_manager.list_edge_indexes(space_id as i32)
                .map_err(|e| PlannerError::PlanGenerationFailed(
                    format!("Failed to list edge indexes: {}", e)
                ))?
        } else {
            index_manager.list_tag_indexes(space_id as i32)
                .map_err(|e| PlannerError::PlanGenerationFailed(
                    format!("Failed to list tag indexes: {}", e)
                ))?
        };

        // 过滤出与当前schema相关的索引
        let schema_indexes: Vec<Index> = indexes
            .into_iter()
            .filter(|idx| idx.schema_name == schema_name && idx.status == crate::index::IndexStatus::Active)
            .collect();

        if schema_indexes.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                format!("No active indexes found for {}: {}", 
                    if is_edge { "edge" } else { "tag" }, 
                    schema_name
                )
            ));
        }

        Ok(schema_indexes)
    }

    /// 构建YIELD列
    fn build_yield_columns(
        lookup_ctx: &LookupContext,
    ) -> Result<Vec<crate::query::validator::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        if let Some(ref yield_expression) = lookup_ctx.yield_expression {
            for col in &yield_expression.columns {
                columns.push(crate::query::validator::YieldColumn {
                    expression: Self::parse_yield_expression(&col.name(), lookup_ctx.is_edge)?,
                    alias: col.alias.clone(),
                    is_matched: false,
                });
            }
        } else {
            columns.push(crate::query::validator::YieldColumn {
                expression: Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        if columns.is_empty() {
            columns.push(crate::query::validator::YieldColumn {
                expression: Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        Ok(columns)
    }

    /// 解析YIELD表达式
    fn parse_yield_expression(name: &str, _is_edge: bool) -> Result<Expression, PlannerError> {
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
