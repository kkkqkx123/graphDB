//! 集合操作规划器
//!
//! 处理 UNION, UNION ALL, INTERSECT, MINUS 等集合操作语句的查询规划

use crate::query::QueryContext;
use crate::query::parser::ast::{SetOperationType, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        ArgumentNode, IntersectNode, MinusNode, UnionNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// 集合操作规划器
/// 负责将集合操作语句转换为执行计划
#[derive(Debug, Clone)]
pub struct SetOperationPlanner;

impl SetOperationPlanner {
    /// 创建新的集合操作规划器
    pub fn new() -> Self {
        Self
    }
}

impl Planner for SetOperationPlanner {
    fn transform(&mut self, stmt: &Stmt, _qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError> {
        let set_op_stmt = match stmt {
            Stmt::SetOperation(set_op_stmt) => set_op_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "SetOperationPlanner 需要 SetOperation 语句".to_string()
                ));
            }
        };

        // 创建左右子计划的参数节点
        let left_arg = ArgumentNode::new(next_node_id(), "left_input");
        let right_arg = ArgumentNode::new(next_node_id(), "right_input");

        let left_enum = PlanNodeEnum::Argument(left_arg.clone());
        let right_enum = PlanNodeEnum::Argument(right_arg.clone());

        // 根据集合操作类型创建相应的计划节点
        let final_node = match set_op_stmt.op_type {
            SetOperationType::Union => {
                // UNION (去重) - UnionNode 只接受单个输入，需要特殊处理
                // 这里我们使用 left_enum 作为主输入，distinct=true
                let union_node = UnionNode::new(
                    left_enum,
                    true, // distinct = true
                ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
                    "Failed to create UnionNode: {}",
                    e
                )))?;
                PlanNodeEnum::Union(union_node)
            }
            SetOperationType::UnionAll => {
                // UNION ALL (不去重)
                let union_node = UnionNode::new(
                    left_enum,
                    false, // distinct = false
                ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
                    "Failed to create UnionNode: {}",
                    e
                )))?;
                PlanNodeEnum::Union(union_node)
            }
            SetOperationType::Intersect => {
                // INTERSECT
                let intersect_node = IntersectNode::new(
                    left_enum,
                    right_enum,
                ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
                    "Failed to create IntersectNode: {}",
                    e
                )))?;
                PlanNodeEnum::Intersect(intersect_node)
            }
            SetOperationType::Minus => {
                // MINUS
                let minus_node = MinusNode::new(
                    left_enum,
                    right_enum,
                ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
                    "Failed to create MinusNode: {}",
                    e
                )))?;
                PlanNodeEnum::Minus(minus_node)
            }
        };

        // 创建 SubPlan，使用左输入作为主输入
        let sub_plan = SubPlan::new(
            Some(final_node),
            Some(PlanNodeEnum::Argument(left_arg)),
        );

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::SetOperation(_))
    }
}

impl Default for SetOperationPlanner {
    fn default() -> Self {
        Self::new()
    }
}
