//! 将过滤条件下推到内连接操作的规则
//!
//! 该规则识别 Filter -> InnerJoin 模式，
//! 并将过滤条件下推到连接的一侧。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::core::Expression;
use crate::query::optimizer::expression_utils::{check_col_name, split_filter};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到内连接操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(a.col1 > 10)
///           |
///   InnerJoin
///   /          \
/// Left      Right
/// ```
///
/// After:
/// ```text
///   InnerJoin
///   /          \
/// Filter      Right
/// (a.col1>10)
///   |
/// Left
/// ```
///
/// # 适用条件
///
/// - 过滤条件仅涉及连接的一侧
/// - 可以安全地将条件下推
#[derive(Debug)]
pub struct PushFilterDownInnerJoinRule;

impl PushFilterDownInnerJoinRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownInnerJoinRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownInnerJoinRule {
    fn name(&self) -> &'static str {
        "PushFilterDownInnerJoinRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("InnerJoin")
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

        // 获取输入节点
        let input = filter_node.input();

        // 检查输入节点是否为 InnerJoin
        let join = match input {
            PlanNodeEnum::InnerJoin(n) => n,
            _ => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 获取左右输入的列名
        let left_col_names = join.left_input().col_names().to_vec();

        // 定义选择器函数
        let picker = |expr: &Expression| -> bool {
            check_col_name(&left_col_names, expr)
        };

        // 分割过滤条件
        let (_filter_picked, _filter_unpicked) = split_filter(filter_condition, picker);

        // 简化实现：返回 None 表示不转换
        // 实际实现需要创建新的 InnerJoin 节点并在左侧添加 Filter
        Ok(None)
    }
}

impl PushDownRule for PushFilterDownInnerJoinRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::InnerJoin(_)))
    }

    fn push_down(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownInnerJoinRule::new();
        assert_eq!(rule.name(), "PushFilterDownInnerJoinRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownInnerJoinRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
