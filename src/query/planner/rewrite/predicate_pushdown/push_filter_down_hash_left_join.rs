//! 将过滤条件下推到哈希左连接操作的规则
//!
//! 该规则识别 Filter -> HashLeftJoin 模式，
//! 并将过滤条件下推到连接的两侧。

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::core::Expression;
use crate::query::optimizer::expression_utils::{check_col_name, split_filter};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到哈希左连接操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(a.col1 > 10 AND b.col2 < 20)
///           |
///   HashLeftJoin
///   /          \
/// Left      Right
/// ```
///
/// After:
/// ```text
///   HashLeftJoin
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
pub struct PushFilterDownHashLeftJoinRule;

impl PushFilterDownHashLeftJoinRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownHashLeftJoinRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownHashLeftJoinRule {
    fn name(&self) -> &'static str {
        "PushFilterDownHashLeftJoinRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("HashLeftJoin")
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

        // 检查输入节点是否为 HashLeftJoin
        let join = match input {
            PlanNodeEnum::HashLeftJoin(n) => n,
            _ => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

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
        let (left_picked, left_remained) = split_filter(filter_condition, left_picker);
        let (right_picked, right_remained) = split_filter(filter_condition, right_picker);

        // 如果没有可以下推的条件，则不进行转换
        if left_picked.is_none() && right_picked.is_none() {
            return Ok(None);
        }

        // 创建新的 HashLeftJoin 节点
        let mut new_join = join.clone();
        let mut new_left = join.left_input().clone();
        let mut new_right = join.right_input().clone();

        // 处理左侧下推
        let left_pushed = left_picked.is_some();
        if let Some(left_filter) = left_picked {
            let left_filter_node = FilterNode::new(new_left, left_filter)
                .map_err(|e| crate::query::planner::rewrite::result::RewriteError::rewrite_failed(
                    format!("创建FilterNode失败: {:?}", e)
                ))?;
            new_left = PlanNodeEnum::Filter(left_filter_node);
        }

        // 处理右侧下推
        let right_pushed = right_picked.is_some();
        if let Some(right_filter) = right_picked {
            let right_filter_node = FilterNode::new(new_right, right_filter)
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
            new_filter.set_condition(remained);
            result.add_new_node(PlanNodeEnum::Filter(new_filter));
        } else {
            result.erase_curr = true;
        }

        result.add_new_node(PlanNodeEnum::HashLeftJoin(new_join));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownHashLeftJoinRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::HashLeftJoin(_)))
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
    use crate::query::planner::plan::core::nodes::join_node::HashLeftJoinNode;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownHashLeftJoinRule::new();
        assert_eq!(rule.name(), "PushFilterDownHashLeftJoinRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownHashLeftJoinRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_can_push_down() {
        let rule = PushFilterDownHashLeftJoinRule::new();

        let start = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start);

        let condition = Expression::Variable("test".to_string());
        let filter = FilterNode::new(start_enum.clone(), condition).expect("创建FilterNode失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        let join = HashLeftJoinNode::new(
            start_enum.clone(),
            start_enum,
            vec![],
            vec![]
        ).expect("创建HashLeftJoinNode失败");
        let join_enum = PlanNodeEnum::HashLeftJoin(join);

        assert!(rule.can_push_down(&filter_enum, &join_enum));
    }
}
