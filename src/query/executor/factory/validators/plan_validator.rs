//! Plan Validator
//!
//! Responsible for verifying the validity of the planned nodes and checking the constraints associated with these nodes.

use crate::core::error::QueryError;
use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;

/// Plan Validator
pub struct PlanValidator;

impl PlanValidator {
    /// Create a new plan validator.
    pub fn new() -> Self {
        Self
    }

    /// Verify the plan nodes.
    pub fn validate(&self, plan_node: &PlanNodeEnum) -> Result<(), QueryError> {
        match plan_node {
            PlanNodeEnum::Expand(node) => {
                let step_limit = node
                    .step_limit()
                    .and_then(|s| usize::try_from(s).ok())
                    .unwrap_or(10);
                if step_limit > 1000 {
                    return Err(QueryError::ExecutionError(format!(
                        "Expand actuator step limit {} exceeds safety threshold 1000",
                        step_limit
                    )));
                }
            }
            PlanNodeEnum::Loop(_) => {
                return Err(QueryError::ExecutionError(
                    "循环执行器需要手动构建，不支持通过工厂自动创建".to_string(),
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

impl Default for PlanValidator {
    fn default() -> Self {
        Self::new()
    }
}
