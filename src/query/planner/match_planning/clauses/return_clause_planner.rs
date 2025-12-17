//! RETURN 子句规划器
//!
//! 负责将 Cypher 查询中的 RETURN 子句转换为执行计划。
//!
//! # 功能概述
//!
//! RETURN 子句是 Cypher 查询的最后一步，负责：
//! - 结果投影（选择输出列）
//! - 聚合计算（如 COUNT、SUM 等）
//! - 排序（ORDER BY）
//! - 分页（LIMIT/OFFSET）
//! - 去重（DISTINCT）
//!
//! # 处理顺序
//!
//! 按照 Cypher 语义，处理顺序为：
//! 1. 投影（YIELD） - 选择要输出的列
//! 2. 排序（ORDER BY） - 对结果进行排序
//! 3. 分页（LIMIT/OFFSET） - 限制结果数量
//! 4. 去重（DISTINCT） - 去除重复行
//!
//! # 设计原则
//!
//! - 模块化设计：每个功能由专门的规划器处理
//! - 顺序保证：严格按照 Cypher 语义顺序执行
//! - 性能优化：避免不必要的节点创建
//! - 错误处理：完善的边界检查和错误报告
//!
//! # 示例
//!
//! ```cypher
//! MATCH (n:Person)-[:KNOWS]->(m:Person)
//! WHERE n.age > 25
//! RETURN DISTINCT n.name, COUNT(m) as friend_count
//! ORDER BY friend_count DESC
//! LIMIT 10
//! ```
//!
//! 对应的执行计划将包含：Project -> Aggregate -> Sort -> Limit -> Dedup 节点

use super::order_by_planner::OrderByClausePlanner;
use super::pagination_planner::PaginationPlanner;
use super::yield_planner::YieldClausePlanner;
use crate::query::planner::match_planning::core::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::utils::connector::SegmentsConnector;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::plan::core::plan_node_traits::PlanNodeMutable;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// RETURN子句规划器
/// 负责规划RETURN子句中的结果投影
#[derive(Debug)]
pub struct ReturnClausePlanner;

impl ReturnClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for ReturnClausePlanner {
    /// 将 RETURN 子句上下文转换为执行计划
    ///
    /// # 参数
    /// * `clause_ctx` - Cypher 子句上下文，必须是 Return 类型
    ///
    /// # 返回
    /// * `Result<SubPlan, PlannerError>` - 执行计划或错误
    ///
    /// # 错误处理
    /// * 如果上下文不是 Return 类型，返回 InvalidAstContext 错误
    /// * 如果无法提取 ReturnClauseContext，返回 InvalidAstContext 错误
    /// * 如果子规划器失败，返回相应的 PlannerError
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        // 验证输入上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::Return) {
            return Err(PlannerError::InvalidAstContext(
                "ReturnClausePlanner 只能处理 RETURN 子句上下文".to_string(),
            ));
        }

        // 提取具体的 RETURN 子句上下文
        let return_clause_ctx = match clause_ctx {
            CypherClauseContext::Return(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "无法提取 ReturnClauseContext".to_string(),
                ))
            }
        };

        // 验证 RETURN 子句上下文的完整性
        if return_clause_ctx.yield_clause.yield_columns.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "RETURN 子句必须至少包含一个输出列".to_string(),
            ));
        }

        // 步骤1: 处理YIELD子句（RETURN的投影部分）
        // 这是RETURN子句的核心，负责选择输出列和执行聚合操作
        let mut yield_planner = YieldClausePlanner::new();
        let yield_clause_ctx = CypherClauseContext::Yield(return_clause_ctx.yield_clause.clone());
        let mut plan = yield_planner.transform(&yield_clause_ctx)?;

        // 步骤2: 处理ORDER BY子句（排序）
        // 注意：排序必须在投影之后、分页之前执行
        if let Some(order_by) = &return_clause_ctx.order_by {
            let mut order_by_planner = OrderByClausePlanner::new();
            let order_by_clause_ctx = CypherClauseContext::OrderBy(order_by.clone());
            let order_plan = order_by_planner.transform(&order_by_clause_ctx)?;

            // 将排序节点连接到现有计划的顶部
            // 数据流：现有计划 -> 排序节点
            let connector = SegmentsConnector::new();
            plan = connector.add_input(order_plan, plan, true);
        }

        // 步骤3: 处理分页（LIMIT/OFFSET）
        // 分页必须在排序之后执行，以确保结果的正确性
        if let Some(pagination) = &return_clause_ctx.pagination {
            // 验证分页参数的合理性
            validate_pagination_params(pagination.skip, pagination.limit)?;
            
            // 只有当skip或limit有实际值时才创建分页节点
            if pagination.skip != 0 || pagination.limit != i64::MAX {
                let mut pagination_planner = PaginationPlanner::new();
                let pagination_clause_ctx = CypherClauseContext::Pagination(pagination.clone());
                let pagination_plan = pagination_planner.transform(&pagination_clause_ctx)?;

                // 将分页节点连接到现有计划的顶部
                // 数据流：现有计划 -> 分页节点
                let connector = SegmentsConnector::new();
                plan = connector.add_input(pagination_plan, plan, true);
            }
        }

        // 处理去重 (DISTINCT)
        if return_clause_ctx.distinct {
            // RETURN 子句不应该创建起始节点
            // 如果计划为空，说明这是一个错误的情况，因为 RETURN 必须有输入
            if plan.root.is_none() {
                return Err(PlannerError::PlanGenerationFailed(
                    "RETURN 子句必须有输入数据源，不能作为查询的起始".to_string(),
                ));
            }

            // 创建去重节点，使用当前计划的根节点作为输入
            let current_root = plan.root.as_ref().unwrap().clone();
            let dedup_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Dedup,
                current_root,
            ));

            // 设置去重键 - 使用投影列作为去重依据
            let mut dedup_node_mut = (*dedup_node).clone();
            if let Some(yield_cols) = get_yield_columns(&return_clause_ctx.yield_clause) {
                dedup_node_mut.set_col_names(yield_cols);
            }
            let dedup_node = Arc::new(dedup_node_mut);

            // 将去重节点连接到计划中
            let connector = SegmentsConnector::new();
            plan = connector.add_input(
                SubPlan::new(Some(dedup_node.clone()), Some(dedup_node)),
                plan,
                true,
            );
        }

        Ok(plan)
    }
}

/// 注意：RETURN 子句规划器不应该创建起始节点
/// 起始节点应该在查询的最开始（如 MATCH 子句）创建
/// RETURN 子句必须接收来自上游子句的输入数据

/// 获取 YIELD 子句中的列名
/// 用于设置去重键
///
/// # 参数
/// * `yield_clause` - YIELD 子句上下文
///
/// # 返回
/// * `Option<Vec<String>>` - 列名列表，如果没有列则返回 None
fn get_yield_columns(yield_clause: &crate::query::validator::structs::YieldClauseContext) -> Option<Vec<String>> {
    // 优先使用投影输出列名
    if !yield_clause.proj_output_column_names.is_empty() {
        return Some(yield_clause.proj_output_column_names.clone());
    }
    
    // 如果没有投影列名，尝试从 yield_columns 中提取
    if !yield_clause.yield_columns.is_empty() {
        let columns: Vec<String> = yield_clause.yield_columns
            .iter()
            .map(|col| col.alias.clone())
            .collect();
        return Some(columns);
    }
    
    // 如果都没有，返回 None
    None
}

/// 验证分页参数的合理性
///
/// # 参数
/// * `skip` - 跳过的记录数
/// * `limit` - 限制的记录数
///
/// # 返回
/// * `Result<(), PlannerError>` - 验证结果
fn validate_pagination_params(skip: i64, limit: i64) -> Result<(), PlannerError> {
    if skip < 0 {
        return Err(PlannerError::PlanGenerationFailed(
            format!("OFFSET 值不能为负数: {}", skip)
        ));
    }
    
    if limit <= 0 && limit != i64::MAX {
        return Err(PlannerError::PlanGenerationFailed(
            format!("LIMIT 值必须为正数: {}", limit)
        ));
    }
    
    if skip > i64::MAX / 2 {
        return Err(PlannerError::PlanGenerationFailed(
            "OFFSET 值过大，可能导致内存问题".to_string()
        ));
    }
    
    Ok(())
}
