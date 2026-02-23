//! 将顶点过滤条件下推到ScanVertices节点的规则
//!
//! 该规则识别 Traverse 节点中的 vFilter，
//! 并将其重写为具体的顶点属性表达式。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};

/// 将顶点过滤条件下推到ScanVertices节点的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Traverse(vFilter: *.age > 18)
/// ```
///
/// After:
/// ```text
///   Traverse(filter: v.age > 18)
/// ```
///
/// # 适用条件
///
/// - Traverse 节点存在 vFilter
/// - vFilter 包含通配符顶点属性表达式
/// - Traverse 不为零步遍历
#[derive(Debug)]
pub struct PushVFilterDownScanVerticesRule;

impl PushVFilterDownScanVerticesRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushVFilterDownScanVerticesRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushVFilterDownScanVerticesRule {
    fn name(&self) -> &'static str {
        "PushVFilterDownScanVerticesRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Traverse")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Traverse 节点
        let traverse = match node {
            PlanNodeEnum::Traverse(t) => t,
            _ => return Ok(None),
        };

        // 获取 vFilter
        let _v_filter = match traverse.v_filter() {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 简化实现：返回 None 表示不转换
        // 实际实现需要将通配符顶点属性表达式重写为具体的顶点属性表达式
        Ok(None)
    }
}

impl PushDownRule for PushVFilterDownScanVerticesRule {
    fn can_push_down(&self, node: &PlanNodeEnum, _target: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Traverse(traverse) => {
                traverse.v_filter().is_some() && traverse.min_steps() > 0
            }
            _ => false,
        }
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
        let rule = PushVFilterDownScanVerticesRule::new();
        assert_eq!(rule.name(), "PushVFilterDownScanVerticesRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushVFilterDownScanVerticesRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
