//! 重写规则 trait 定义
//!
//! 该模块提供启发式重写规则的 trait 定义。
//! 启发式规则不依赖代价计算，总是产生更优或等价的计划。
//!
//! 这是从 optimizer 层独立出来的版本，专注于 planner 层的需求。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{MatchedResult, RewriteResult, TransformResult};

/// 重写规则 trait
///
/// 所有启发式重写规则必须实现此 trait。
/// 规则通过模式匹配识别计划树的特定结构，然后应用转换。
///
/// # 示例
/// ```rust
/// use crate::query::planner::rewrite::rule::RewriteRule;
///
/// #[derive(Debug)]
/// struct MyRule;
///
/// impl RewriteRule for MyRule {
///     fn name(&self) -> &str { "MyRule" }
///     
///     fn pattern(&self) -> Pattern {
///         Pattern::new_with_name("Filter")
///     }
///     
///     fn apply(&self, ctx: &mut RewriteContext, node: &PlanNodeEnum) -> RewriteResult<Option<TransformResult>> {
///         // 实现规则逻辑
///         Ok(None)
///     }
/// }
/// ```
pub trait RewriteRule: std::fmt::Debug + Send + Sync {
    /// 规则名称
    fn name(&self) -> &'static str;

    /// 返回规则的模式
    ///
    /// 用于匹配计划树的特定结构
    fn pattern(&self) -> Pattern;

    /// 应用重写规则
    ///
    /// # 参数
    /// - `ctx`: 重写上下文
    /// - `node`: 当前计划节点
    ///
    /// # 返回
    /// - `Ok(Some(result))`: 重写成功，返回转换结果
    /// - `Ok(None)`: 不匹配，保持原节点
    /// - `Err(e)`: 重写失败
    fn apply(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>>;

    /// 匹配模式
    ///
    /// 检查当前节点是否匹配规则的模式
    fn match_pattern(&self, node: &PlanNodeEnum) -> RewriteResult<Option<MatchedResult>> {
        if self.pattern().matches(node) {
            let mut result = MatchedResult::new();
            result.add_node(node.clone());

            // 添加依赖节点
            for dep in node.dependencies() {
                result.add_dependency(dep.as_ref().clone());
            }

            result.set_root_node(node.clone());
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// 检查规则是否匹配
    ///
    /// 便捷方法，只返回是否匹配，不返回详细信息
    fn matches(&self, node: &PlanNodeEnum) -> bool {
        self.pattern().matches(node)
    }
}

/// 基础重写规则 trait
///
/// 标记 trait，用于标识基础重写规则
pub trait BaseRewriteRule: RewriteRule {}

/// 合并规则 trait
///
/// 用于合并两个连续操作的规则
pub trait MergeRule: RewriteRule {
    /// 检查是否可以合并
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool;

    /// 创建合并后的节点
    fn create_merged_node(
        &self,
        ctx: &mut RewriteContext,
        parent: &PlanNodeEnum,
        child: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>>;
}

/// 下推规则 trait
///
/// 用于将操作下推到计划树底层的规则
pub trait PushDownRule: RewriteRule {
    /// 检查是否可以下推
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool;

    /// 执行下推操作
    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>>;
}

/// 消除规则 trait
///
/// 用于消除冗余操作的规则
pub trait EliminationRule: RewriteRule {
    /// 检查是否可以消除
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool;

    /// 执行消除操作
    fn eliminate(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>>;
}

/// 规则包装器
///
/// 用于将具体规则类型包装为统一接口
#[derive(Debug)]
pub struct RuleWrapper<T: RewriteRule> {
    inner: T,
}

impl<T: RewriteRule> RuleWrapper<T> {
    pub fn new(rule: T) -> Self {
        Self { inner: rule }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: RewriteRule> RewriteRule for RuleWrapper<T> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn pattern(&self) -> Pattern {
        self.inner.pattern()
    }

    fn apply(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.inner.apply(ctx, node)
    }
}

/// 规则适配器 trait
///
/// 允许将具体规则转换为包装器
pub trait IntoRuleWrapper: RewriteRule + Sized {
    fn into_wrapper(self) -> RuleWrapper<Self> {
        RuleWrapper::new(self)
    }
}

impl<T: RewriteRule + Sized> IntoRuleWrapper for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;

    #[derive(Debug)]
    struct TestRule;

    impl RewriteRule for TestRule {
        fn name(&self) -> &'static str {
            "TestRule"
        }

        fn pattern(&self) -> Pattern {
            Pattern::new_with_name("ScanVertices")
        }

        fn apply(
            &self,
            _ctx: &mut RewriteContext,
            _node: &PlanNodeEnum,
        ) -> RewriteResult<Option<TransformResult>> {
            Ok(None)
        }
    }

    #[test]
    fn test_rule_name() {
        let rule = TestRule;
        assert_eq!(rule.name(), "TestRule");
    }

    #[test]
    fn test_rule_matches() {
        let rule = TestRule;
        let node = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        
        assert!(rule.matches(&node));
    }

    #[test]
    fn test_rule_wrapper() {
        let rule = TestRule;
        let wrapper = rule.into_wrapper();
        
        assert_eq!(wrapper.name(), "TestRule");
    }
}
