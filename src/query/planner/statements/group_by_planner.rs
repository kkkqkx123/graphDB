//! GroupBy 操作规划器
//!
//! 处理 GROUP BY 语句的查询规划

use crate::query::context::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        AggregateNode, ArgumentNode, FilterNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::core::Expression;
use crate::core::types::operators::AggregateFunction;
use std::sync::Arc;

/// GroupBy 操作规划器
/// 负责将 GROUP BY 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct GroupByPlanner;

impl GroupByPlanner {
    /// 创建新的 GroupBy 规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 获取匹配和实例化函数（静态注册版本）
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::GroupBy(Self::new())
    }

    /// 从表达式中提取聚合函数
    fn extract_aggregate_functions(&self, _expr: &Expression) -> Vec<AggregateFunction> {
        // TODO: 实现从表达式中提取聚合函数的逻辑
        // 这里简化处理，返回空列表
        vec![]
    }
}

impl Planner for GroupByPlanner {
    fn transform(&mut self, stmt: &Stmt, _qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError> {
        let group_by_stmt = match stmt {
            Stmt::GroupBy(group_by_stmt) => group_by_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "GroupByPlanner 需要 GroupBy 语句".to_string()
                ));
            }
        };

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "group_by_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // 提取分组键 - 使用表达式描述作为键
        let group_keys: Vec<String> = group_by_stmt.group_items
            .iter()
            .enumerate()
            .map(|(i, _)| format!("group_key_{}", i))
            .collect();

        // 提取聚合函数
        let mut aggregation_functions = Vec::new();
        for item in &group_by_stmt.yield_clause.items {
            let funcs = self.extract_aggregate_functions(&item.expression);
            aggregation_functions.extend(funcs);
        }

        // 创建聚合节点
        let aggregate_node = AggregateNode::new(
            arg_node_enum.clone(),
            group_keys,
            aggregation_functions,
        ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
            "Failed to create AggregateNode: {}",
            e
        )))?;

        let mut final_node = PlanNodeEnum::Aggregate(aggregate_node);

        // 如果有 HAVING 子句，添加 FilterNode
        if let Some(ref having_expr) = group_by_stmt.having_clause {
            let filter_node = FilterNode::new(
                final_node.clone(),
                having_expr.clone(),
            ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
                "Failed to create FilterNode: {}",
                e
            )))?;
            final_node = PlanNodeEnum::Filter(filter_node);
        }

        // 创建 SubPlan
        let sub_plan = SubPlan::new(
            Some(final_node),
            Some(arg_node_enum),
        );

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::GroupBy(_))
    }
}

impl Default for GroupByPlanner {
    fn default() -> Self {
        Self::new()
    }
}
