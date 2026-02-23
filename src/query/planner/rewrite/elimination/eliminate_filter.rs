//! 消除永假式过滤操作的规则
//!
//! 根据 nebula-graph 的参考实现，此规则处理 Filter(false) 或 Filter(null) 的情况，
//! 将其替换为返回空集的 ValueNode。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   Filter(false)
//!       |
//!   ScanVertices
//! ```
//!
//! After:
//! ```text
//!   Value(空集)
//!       |
//!   Start
//! ```
//!
//! # 适用条件
//!
//! - 过滤条件为永假式（如 FALSE、null 等）

use crate::core::{Expression, Value};
use crate::core::types::operators::BinaryOperator;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::start_node::StartNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 消除永假式过滤操作的规则
///
/// 当 Filter 节点的条件为 false 或 null 时，直接替换为返回空集的节点
#[derive(Debug)]
pub struct EliminateFilterRule;

impl EliminateFilterRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查表达式是否为永假式（false 或 null）
    fn is_contradiction(&self, expression: &Expression) -> bool {
        match expression {
            // 布尔字面量 false
            Expression::Literal(Value::Bool(false)) => true,
            // null 值
            Expression::Literal(Value::Null(_)) => true,
            // 二元表达式：检查 1 = 0 或 0 = 1 等形式
            Expression::Binary { left, op, right } => {
                match (left.as_ref(), op, right.as_ref()) {
                    // 1 = 0 或 0 = 1
                    (
                        Expression::Literal(Value::Int(1)),
                        BinaryOperator::Equal,
                        Expression::Literal(Value::Int(0)),
                    ) => true,
                    (
                        Expression::Literal(Value::Int(0)),
                        BinaryOperator::Equal,
                        Expression::Literal(Value::Int(1)),
                    ) => true,
                    // a != a
                    (
                        Expression::Variable(a),
                        BinaryOperator::NotEqual,
                        Expression::Variable(b),
                    ) if a == b => true,
                    // a = b 其中 a 和 b 是不同的常量
                    (
                        Expression::Literal(a),
                        BinaryOperator::Equal,
                        Expression::Literal(b),
                    ) if a != b => true,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl Default for EliminateFilterRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for EliminateFilterRule {
    fn name(&self) -> &'static str {
        "EliminateFilterRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Filter 节点
        let filter_node = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        // 检查条件是否为永假式
        if !self.is_contradiction(filter_node.condition()) {
            return Ok(None);
        }

        // 创建转换结果：用 StartNode 替换当前 Filter 节点
        // 参考 nebula-graph 的实现，返回空集
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::Start(StartNode::new()));

        Ok(Some(result))
    }
}

impl EliminationRule for EliminateFilterRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Filter(n) => self.is_contradiction(n.condition()),
            _ => false,
        }
    }

    fn eliminate(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_eliminate_filter_rule_name() {
        let rule = EliminateFilterRule::new();
        assert_eq!(rule.name(), "EliminateFilterRule");
    }

    #[test]
    fn test_eliminate_filter_rule_pattern() {
        let rule = EliminateFilterRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_is_contradiction() {
        let rule = EliminateFilterRule::new();

        // 测试 false
        assert!(rule.is_contradiction(&Expression::Literal(Value::Bool(false))));

        // 测试 true（不是永假式）
        assert!(!rule.is_contradiction(&Expression::Literal(Value::Bool(true))));

        // 测试 null
        assert!(rule.is_contradiction(&Expression::Literal(Value::Null(crate::core::value::NullType::Null))));

        // 测试 1 = 0
        assert!(rule.is_contradiction(&Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Value::Int(0))),
        }));

        // 测试 1 = 1（不是永假式）
        assert!(!rule.is_contradiction(&Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Value::Int(1))),
        }));
    }
}
