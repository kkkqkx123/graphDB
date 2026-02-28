//! 将过滤条件下推到哈希内连接操作的规则
//!
//! 该规则识别 Filter -> HashInnerJoin 模式，
//! 并将过滤条件下推到连接的两侧。

use std::sync::Arc;

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::core::Expression;
use crate::core::types::{ContextualExpression, ExpressionContext};
use crate::query::planner::rewrite::expression_utils::{check_col_name, split_filter};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到哈希内连接操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(a.col1 > 10 AND b.col2 < 20)
///           |
///   HashInnerJoin
///   /          \
/// Left      Right
/// ```
///
/// After:
/// ```text
///   HashInnerJoin
///   /          \
/// Filter      Filter
/// (a.col1>10) (b.col2<20)
///   |            |
/// Left        Right
/// ```
///
/// # 适用条件
///
/// - 过滤条件可以分离为左右两侧的条件
/// - 可以安全地将条件下推到两侧
#[derive(Debug)]
pub struct PushFilterDownHashInnerJoinRule;

impl PushFilterDownHashInnerJoinRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownHashInnerJoinRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownHashInnerJoinRule {
    fn name(&self) -> &'static str {
        "PushFilterDownHashInnerJoinRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("HashInnerJoin")
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

        // 检查输入节点是否为 HashInnerJoin
        let join = match input {
            PlanNodeEnum::HashInnerJoin(n) => n,
            _ => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 获取表达式用于处理
        let filter_expr = match filter_condition.expression() {
            Some(meta) => meta.inner().clone(),
            None => return Ok(None),
        };

        // 获取上下文用于创建 ContextualExpression
        let ctx = filter_condition.context().clone();

        // 获取左右输入的列名
        let left_col_names = join.left_input().col_names().to_vec();
        let right_col_names = join.right_input().col_names().to_vec();

        // 定义左侧选择器函数
        let left_picker = |expr: &Expression| -> bool {
            check_col_name(&left_col_names, expr)
        };

        // 定义右侧选择器函数
        let right_picker = |expr: &Expression| -> bool {
            check_col_name(&right_col_names, expr)
        };

        // 分割过滤条件
        let (left_picked, left_remained) = split_filter(&filter_expr, left_picker);
        let (right_picked, right_remained) = split_filter(&filter_expr, right_picker);

        // 如果没有可以下推的条件，则不进行转换
        if left_picked.is_none() && right_picked.is_none() {
            return Ok(None);
        }

        // 创建新的 HashInnerJoin 节点
        let mut new_join = join.clone();
        let mut new_left = join.left_input().clone();
        let mut new_right = join.right_input().clone();

        // 处理左侧下推
        let left_pushed = left_picked.is_some();
        if let Some(left_filter) = left_picked {
            let left_filter_node = FilterNode::from_expression(new_left, left_filter, ctx.clone())
                .map_err(|e| crate::query::planner::rewrite::result::RewriteError::rewrite_failed(
                    format!("创建FilterNode失败: {:?}", e)
                ))?;
            new_left = PlanNodeEnum::Filter(left_filter_node);
        }

        // 处理右侧下推
        let right_pushed = right_picked.is_some();
        if let Some(right_filter) = right_picked {
            let right_filter_node = FilterNode::from_expression(new_right, right_filter, ctx.clone())
                .map_err(|e| crate::query::planner::rewrite::result::RewriteError::rewrite_failed(
                    format!("创建FilterNode失败: {:?}", e)
                ))?;
            new_right = PlanNodeEnum::Filter(right_filter_node);
        }

        // 更新 Join 节点的输入
        new_join.set_left_input(new_left);
        new_join.set_right_input(new_right);

        // 构建转换结果
        let mut result = TransformResult::new();

        // 检查是否有剩余的过滤条件
        let remaining_condition = if left_pushed && right_pushed {
            None
        } else if left_pushed {
            right_remained
        } else {
            left_remained
        };

        if let Some(remained) = remaining_condition {
            result.erase_curr = false;
            let mut new_filter = filter_node.clone();
            let remained_meta = crate::core::types::ExpressionMeta::new(remained);
            let remained_id = ctx.register_expression(remained_meta);
            let remained_ctx_expr = ContextualExpression::new(remained_id, ctx);
            new_filter.set_condition(remained_ctx_expr);
            result.add_new_node(PlanNodeEnum::Filter(new_filter));
        } else {
            result.erase_curr = true;
        }

        result.add_new_node(PlanNodeEnum::HashInnerJoin(new_join));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownHashInnerJoinRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::HashInnerJoin(_)))
    }

    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use crate::query::planner::plan::core::nodes::join_node::HashInnerJoinNode;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownHashInnerJoinRule::new();
        assert_eq!(rule.name(), "PushFilterDownHashInnerJoinRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownHashInnerJoinRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_can_push_down() {
        let rule = PushFilterDownHashInnerJoinRule::new();

        let start = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start);

        let condition = Expression::Variable("test".to_string());
        let ctx = Arc::new(ExpressionContext::new());
        let filter = FilterNode::from_expression(start_enum.clone(), condition, ctx).expect("创建FilterNode失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        let join = HashInnerJoinNode::new(
            start_enum.clone(),
            start_enum,
            vec![],
            vec![]
        ).expect("创建HashInnerJoinNode失败");
        let join_enum = PlanNodeEnum::HashInnerJoin(join);

        assert!(rule.can_push_down(&filter_enum, &join_enum));
    }
}
