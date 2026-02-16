//! 合并连续投影规则
//!
//! 当多个 Project 节点连续出现时，合并为一个 Project 节点
//! 减少不必要的中间结果生成
//!
//! 示例:
//! ```
//! Project(a, b) -> Project(c, d)  =>  Project(c, d)
//! ```
//!
//! 适用条件:
//! - 两个 Project 节点连续出现
//! - 上层 Project 不依赖下层 Project 的别名解析

use crate::query::optimizer::{
    BaseOptRule, OptContext, OptGroupNode, OptRule, OptimizerError, Pattern, TransformResult,
};
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use std::cell::RefCell;
use std::rc::Rc;

/// 合并连续投影规则
#[derive(Debug)]
pub struct CollapseConsecutiveProjectRule;

impl CollapseConsecutiveProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl OptRule for CollapseConsecutiveProjectRule {
    fn name(&self) -> &str {
        "CollapseConsecutiveProjectRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node = group_node.borrow();

        // 获取当前节点
        let current = node.get_plan_node();

        // 检查是否是 Project 节点
        match current {
            PlanNodeEnum::Project(_) => {
                // 简化实现：返回 None 表示不转换
                // 实际实现需要检查下层节点并执行合并
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Project 节点
        Pattern::new_with_name("Project")
    }
}

impl BaseOptRule for CollapseConsecutiveProjectRule {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = CollapseConsecutiveProjectRule::new();
        assert_eq!(rule.name(), "CollapseConsecutiveProjectRule");
    }
}
