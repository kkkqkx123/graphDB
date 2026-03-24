//! 计划验证器
//!
//! 负责验证计划节点的有效性，检查计划节点的约束条件

use crate::core::error::QueryError;
use crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum;

/// 计划验证器
pub struct PlanValidator;

impl PlanValidator {
    /// 创建新的计划验证器
    pub fn new() -> Self {
        Self
    }

    /// 验证计划节点
    pub fn validate(&self, plan_node: &PlanNodeEnum) -> Result<(), QueryError> {
        match plan_node {
            PlanNodeEnum::Expand(node) => {
                let step_limit = node
                    .step_limit()
                    .and_then(|s| usize::try_from(s).ok())
                    .unwrap_or(10);
                if step_limit > 1000 {
                    return Err(QueryError::ExecutionError(format!(
                        "Expand执行器的步数限制{}超过安全阈值1000",
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
