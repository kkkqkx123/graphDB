//! 移除冗余过滤规则
//!
//! 当 Filter 节点的条件恒为真时，移除该 Filter 节点
//! 当 Filter 节点的条件与下层 Filter 重复时，合并或移除
//!
//! 示例:
//! ```
//! Filter(true) -> Scan  =>  Scan
//! Filter(a > 1) -> Filter(a > 1) -> Scan  =>  Filter(a > 1) -> Scan
//! ```

use crate::query::optimizer::{
    BaseOptRule, OptContext, OptGroupNode, OptRule, OptimizerError, Pattern, TransformResult,
};
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::core::Expression;
use std::cell::RefCell;
use std::rc::Rc;

/// 移除冗余过滤规则
#[derive(Debug)]
pub struct RemoveRedundantFilterRule;

impl RemoveRedundantFilterRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查表达式是否为恒真
    fn is_always_true(expr: &Expression) -> bool {
        match expr {
            Expression::Literal(val) => {
                // 检查是否为布尔真值
                matches!(val, crate::core::Value::Bool(true))
            }
            _ => false,
        }
    }
}

impl OptRule for RemoveRedundantFilterRule {
    fn name(&self) -> &str {
        "RemoveRedundantFilterRule"
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node = group_node.borrow();

        // 获取当前 Filter 节点
        let filter_expr = match node.get_plan_node() {
            PlanNodeEnum::Filter(filter) => {
                filter.condition().clone()
            }
            _ => return Ok(None),
        };

        // 情况1：条件恒为真，可以移除 Filter
        // 简化实现：返回 None 表示不转换
        // 实际实现需要返回下层节点来移除当前 Filter
        if Self::is_always_true(&filter_expr) {
            // TODO: 返回下层节点以移除当前 Filter
            return Ok(None);
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter 节点
        Pattern::new_with_name("Filter")
    }
}

impl BaseOptRule for RemoveRedundantFilterRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_is_always_true() {
        let true_expr = Expression::Literal(Value::Bool(true));
        assert!(RemoveRedundantFilterRule::is_always_true(&true_expr));

        let false_expr = Expression::Literal(Value::Bool(false));
        assert!(!RemoveRedundantFilterRule::is_always_true(&false_expr));

        let int_expr = Expression::Literal(Value::Int(1));
        assert!(!RemoveRedundantFilterRule::is_always_true(&int_expr));
    }

    #[test]
    fn test_rule_name() {
        let rule = RemoveRedundantFilterRule::new();
        assert_eq!(rule.name(), "RemoveRedundantFilterRule");
    }
}
