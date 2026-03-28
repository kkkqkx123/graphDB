//! Set Operation Planner
//!
//! Query planning for set operation statements such as UNION, UNION ALL, INTERSECT, and MINUS

use crate::query::parser::ast::{SetOperationType, Stmt};
use crate::query::planning::plan::core::{
    node_id_generator::next_node_id,
    nodes::{ArgumentNode, IntersectNode, MinusNode, UnionNode},
};
use crate::query::planning::plan::{PlanNodeEnum, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// Set Operation Planner
/// Responsible for converting set operation statements into execution plans
#[derive(Debug, Clone)]
pub struct SetOperationPlanner;

impl SetOperationPlanner {
    /// Create a new set of operation planners.
    pub fn new() -> Self {
        Self
    }
}

impl Planner for SetOperationPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let set_op_stmt = match validated.stmt() {
            Stmt::SetOperation(set_op_stmt) => set_op_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "SetOperationPlanner 需要 SetOperation 语句".to_string(),
                ));
            }
        };

        // Parameter nodes for creating left and right sub-plans
        let left_arg = ArgumentNode::new(next_node_id(), "left_input");
        let right_arg = ArgumentNode::new(next_node_id(), "right_input");

        let left_enum = PlanNodeEnum::Argument(left_arg.clone());
        let right_enum = PlanNodeEnum::Argument(right_arg.clone());

        // Create the corresponding planning nodes based on the type of set operation.
        let final_node = match set_op_stmt.op_type {
            SetOperationType::Union => {
                // UNION (Deduplication) – The UnionNode only accepts a single input and requires special processing.
                // Here, we use `left_enum` as the main input, with `distinct=true`.
                let union_node = UnionNode::new(
                    left_enum, true, // distinct = true
                )
                .map_err(|e| {
                    PlannerError::PlanGenerationFailed(format!("Failed to create UnionNode: {}", e))
                })?;
                PlanNodeEnum::Union(union_node)
            }
            SetOperationType::UnionAll => {
                // UNION ALL (without duplicates)
                let union_node = UnionNode::new(
                    left_enum, false, // distinct = false
                )
                .map_err(|e| {
                    PlannerError::PlanGenerationFailed(format!("Failed to create UnionNode: {}", e))
                })?;
                PlanNodeEnum::Union(union_node)
            }
            SetOperationType::Intersect => {
                // INTERSECT
                let intersect_node = IntersectNode::new(left_enum, right_enum).map_err(|e| {
                    PlannerError::PlanGenerationFailed(format!(
                        "Failed to create IntersectNode: {}",
                        e
                    ))
                })?;
                PlanNodeEnum::Intersect(intersect_node)
            }
            SetOperationType::Minus => {
                // MINUS
                let minus_node = MinusNode::new(left_enum, right_enum).map_err(|e| {
                    PlannerError::PlanGenerationFailed(format!("Failed to create MinusNode: {}", e))
                })?;
                PlanNodeEnum::Minus(minus_node)
            }
        };

        // Create a SubPlan, using the left input as the main input.
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(left_arg)));

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
