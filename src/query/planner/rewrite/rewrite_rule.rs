//! 重写规则 trait 定义
//!
//! 该模块提供启发式重写规则的 trait 定义。
//! 启发式规则不依赖代价计算，总是产生更优或等价的计划。
//!
//! 注意：当前实现复用 optimizer 模块的 OptRule trait，
//! 但提供一个简化的 HeuristicRule trait 用于不需要完整优化器上下文的场景。

pub use crate::query::optimizer::plan::{OptRule, OptContext, OptGroupNode, TransformResult, OptimizerError, Pattern, MatchedResult};
use crate::query::planner::plan::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 启发式重写规则 trait
///
/// 为不需要完整优化器上下文的简单重写规则提供的简化接口。
/// 这些规则总是产生更优或等价的计划，不需要代价计算。
///
/// 对于需要访问优化器状态或代价信息的规则，请直接使用 `OptRule`。
pub trait HeuristicRule: std::fmt::Debug + Send + Sync {
    /// 规则名称
    fn name(&self) -> &'static str;

    /// 检查是否匹配当前计划节点
    fn matches(&self, node: &PlanNodeEnum) -> bool;

    /// 应用重写规则
    ///
    /// # 参数
    /// - `node`: 当前计划节点（所有权转移）
    ///
    /// # 返回
    /// - `Ok(Some(node))`: 重写成功，返回新节点
    /// - `Ok(None)`: 不匹配，保持原节点
    /// - `Err(e)`: 重写失败
    fn apply(&self, node: PlanNodeEnum) -> Result<Option<PlanNodeEnum>, RewriteError>;
}

/// 重写错误类型
#[derive(Debug, thiserror::Error)]
pub enum RewriteError {
    #[error("无效的计划节点: {0}")]
    InvalidNode(String),

    #[error("重写失败: {0}")]
    RewriteFailed(String),

    #[error("不支持的节点类型: {0}")]
    UnsupportedNodeType(String),

    #[error("优化器错误: {0}")]
    OptimizerError(#[from] OptimizerError),
}

/// 将 HeuristicRule 适配为 OptRule 的包装器
#[derive(Debug)]
pub struct HeuristicRuleAdapter<T: HeuristicRule> {
    inner: T,
}

impl<T: HeuristicRule> HeuristicRuleAdapter<T> {
    pub fn new(rule: T) -> Self {
        Self { inner: rule }
    }
}

impl<T: HeuristicRule + 'static> OptRule for HeuristicRuleAdapter<T> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn apply(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        let plan_node = node_ref.plan_node.clone();
        drop(node_ref);

        match self.inner.apply(plan_node) {
            Ok(Some(new_node)) => {
                let mut result = TransformResult::new();
                let new_group_node = OptGroupNode::new(
                    group_node.borrow().id,
                    new_node,
                );
                result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));
                Ok(Some(result))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(OptimizerError::rule_application_failed(
                self.inner.name().to_string(),
                e.to_string(),
            )),
        }
    }

    fn pattern(&self) -> Pattern {
        // 启发式规则使用通配模式，由 matches 方法进行精确匹配
        Pattern::new()
    }

    fn match_pattern(
        &self,
        _ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<MatchedResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        if self.inner.matches(&node_ref.plan_node) {
            let mut result = MatchedResult::new();
            result.add_group_node(group_node.clone());
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

/// 为 HeuristicRule 提供适配器构造函数的 trait
pub trait IntoOptRule: HeuristicRule + Sized + 'static {
    fn into_opt_rule(self) -> HeuristicRuleAdapter<Self> {
        HeuristicRuleAdapter::new(self)
    }
}

impl<T: HeuristicRule + 'static> IntoOptRule for T {}
