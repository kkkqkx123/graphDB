//! 将过滤条件下推到GetNeighbors操作的规则
//!
//! 该规则识别 Filter -> GetNeighbors 模式，
//! 并将过滤条件下推到 GetNeighbors 节点中。

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{PushDownRule, RewriteRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到GetNeighbors操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   GetNeighbors
/// ```
///
/// After:
/// ```text
///   GetNeighbors(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - GetNeighbors 节点获取边属性
/// - 过滤条件可以下推到存储层
#[derive(Debug)]
pub struct PushFilterDownGetNbrsRule;

impl PushFilterDownGetNbrsRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownGetNbrsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownGetNbrsRule {
    fn name(&self) -> &'static str {
        "PushFilterDownGetNbrsRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("GetNeighbors")
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

        // 检查输入节点是否为 GetNeighbors
        let get_nbrs = match input {
            PlanNodeEnum::GetNeighbors(n) => n,
            _ => return Ok(None),
        };

        // 检查 GetNeighbors 是否获取边属性
        let edge_props = get_nbrs.edge_props();
        if edge_props.is_empty() {
            return Ok(None);
        }

        // 获取过滤条件
        let condition = filter_node.condition();

        // 将 Expression 序列化为字符串
        let condition_str = match serde_json::to_string(condition) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        // 创建新的 GetNeighbors 节点
        let mut new_get_nbrs = get_nbrs.clone();

        // 合并现有过滤条件
        let new_filter = if let Some(existing) = get_nbrs.expression() {
            format!("{{\"and\": [{}, {}]}}", existing, condition_str)
        } else {
            condition_str
        };

        new_get_nbrs.set_expression(new_filter);

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::GetNeighbors(new_get_nbrs));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownGetNbrsRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!(
            (node, target),
            (PlanNodeEnum::Filter(_), PlanNodeEnum::GetNeighbors(_))
        )
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
    use crate::core::Expression;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;
    use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
    use crate::query::planner::plan::core::nodes::graph_scan_node::GetNeighborsNode;

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownGetNbrsRule::new();
        assert_eq!(rule.name(), "PushFilterDownGetNbrsRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownGetNbrsRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_can_push_down() {
        let rule = PushFilterDownGetNbrsRule::new();

        let start = StartNode::new();
        let start_enum = PlanNodeEnum::Start(start);

        let condition = Expression::Variable("test".to_string());
        let filter = FilterNode::new(start_enum.clone(), condition).expect("创建FilterNode失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        let get_nbrs = GetNeighborsNode::new(1, "v");
        let get_nbrs_enum = PlanNodeEnum::GetNeighbors(get_nbrs);

        assert!(rule.can_push_down(&filter_enum, &get_nbrs_enum));
    }
}
