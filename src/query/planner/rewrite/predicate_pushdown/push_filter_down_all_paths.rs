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
        let _all_paths = match input {
            PlanNodeEnum::AllPaths(n) => n,
            _ => return Ok(None),
        };

        // 注意：AllPaths 节点目前没有 filter 字段
        // 如果需要支持下推，需要在 AllPaths 结构中添加 filter 字段
        // 目前返回 None 表示不转换
        Ok(None)
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
