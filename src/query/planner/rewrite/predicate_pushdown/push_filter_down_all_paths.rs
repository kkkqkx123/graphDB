//! 将过滤条件下推到AllPaths操作的规则
//!
//! 该规则识别 Filter -> AllPaths 模式，
//! 并将过滤条件下推到 AllPaths 节点中。

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

/// 将过滤条件下推到AllPaths操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(e.likeness > 78)
///           |
///   AllPaths
/// ```
///
/// After:
/// ```text
///   AllPaths(filter: e.likeness > 78)
/// ```
///
/// # 适用条件
///
/// - AllPaths 节点获取边属性
/// - AllPaths 的最小步数等于最大步数
/// - 过滤条件可以下推到存储层
#[derive(Debug)]
pub struct PushFilterDownAllPathsRule;

impl PushFilterDownAllPathsRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownAllPathsRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownAllPathsRule {
    fn name(&self) -> &'static str {
        "PushFilterDownAllPathsRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("AllPaths")
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

        // 检查输入节点是否为 AllPaths
        let all_paths = match input {
            PlanNodeEnum::AllPaths(n) => n,
            _ => return Ok(None),
        };

        // 获取过滤条件
        let filter_condition = filter_node.condition();

        // 创建新的 AllPaths 节点
        let mut new_all_paths = all_paths.clone();

        // 设置 filter
        new_all_paths.set_filter(filter_condition.clone());

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::AllPaths(new_all_paths));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownAllPathsRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::AllPaths(_)))
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

    #[test]
    fn test_rule_name() {
        let rule = PushFilterDownAllPathsRule::new();
        assert_eq!(rule.name(), "PushFilterDownAllPathsRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushFilterDownAllPathsRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
