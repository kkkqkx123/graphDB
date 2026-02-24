//! 重写规则 trait 定义
//!
//! 该模块提供启发式重写规则的 trait 定义。
//! 启发式规则不依赖代价计算，总是产生更优或等价的计划。
//!
//! 注意：当前实现使用 planner 层独立的类型，不再依赖 optimizer 模块。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::RewriteRule;

/// 启发式重写规则 trait（兼容层）
///
/// 为不需要完整优化器上下文的简单重写规则提供的简化接口。
/// 这些规则总是产生更优或等价的计划，不需要代价计算。
///
/// 对于需要访问优化器状态或代价信息的规则，请使用 `RewriteRule` trait。
pub trait HeuristicRule: std::fmt::Debug + Send + Sync {
    /// 规则名称
    fn name(&self) -> &'static str;

    /// 检查是否匹配当前计划节点
    fn matches(&self, node: &PlanNodeEnum) -> bool;

    /// 应用重写规则
    ///
    /// # 参数
    /// - `ctx`: 重写上下文
    /// - `node`: 当前计划节点
    ///
    /// # 返回
    /// - `Ok(Some(node))`: 重写成功，返回新节点
    /// - `Ok(None)`: 不匹配，保持原节点
    /// - `Err(e)`: 重写失败
    fn apply(&self, ctx: &mut RewriteContext, node: &PlanNodeEnum) -> RewriteResult<Option<PlanNodeEnum>>;
}

/// 将 HeuristicRule 适配为 RewriteRule 的包装器
#[derive(Debug)]
pub struct HeuristicRuleAdapter<T: HeuristicRule> {
    inner: T,
}

impl<T: HeuristicRule> HeuristicRuleAdapter<T> {
    pub fn new(rule: T) -> Self {
        Self { inner: rule }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: HeuristicRule> RewriteRule for HeuristicRuleAdapter<T> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn pattern(&self) -> Pattern {
        // 启发式规则使用通配模式，由 matches 方法进行精确匹配
        Pattern::new()
    }

    fn apply(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        match self.inner.apply(ctx, node)? {
            Some(new_node) => {
                let mut result = TransformResult::new();
                result.add_new_node(new_node);
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    fn matches(&self, node: &PlanNodeEnum) -> bool {
        self.inner.matches(node)
    }
}

/// 为 HeuristicRule 提供适配器构造函数的 trait
pub trait IntoOptRule: HeuristicRule + Sized {
    fn into_opt_rule(self) -> HeuristicRuleAdapter<Self> {
        HeuristicRuleAdapter::new(self)
    }
}

impl<T: HeuristicRule + Sized> IntoOptRule for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;

    #[derive(Debug)]
    struct TestHeuristicRule;

    impl HeuristicRule for TestHeuristicRule {
        fn name(&self) -> &'static str {
            "TestHeuristicRule"
        }

        fn matches(&self, node: &PlanNodeEnum) -> bool {
            node.is_scan_vertices()
        }

        fn apply(
            &self,
            _ctx: &mut RewriteContext,
            node: &PlanNodeEnum,
        ) -> RewriteResult<Option<PlanNodeEnum>> {
            if self.matches(node) {
                // 返回原节点（实际规则会有更复杂的逻辑）
                Ok(Some(node.clone()))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_heuristic_rule_adapter() {
        let rule = TestHeuristicRule;
        let adapter = rule.into_opt_rule();
        
        assert_eq!(adapter.name(), "TestHeuristicRule");
        
        let node = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        assert!(adapter.matches(&node));
    }

    #[test]
    fn test_heuristic_rule_apply() {
        let rule = TestHeuristicRule;
        let mut ctx = RewriteContext::new();
        let node = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        
        let result = rule.apply(&mut ctx, &node).expect("Failed to apply rewrite rule");
        assert!(result.is_some());
    }
}
