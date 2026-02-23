//! 重写规则 trait 定义
//!
//! 定义所有启发式优化规则需要实现的接口

use crate::query::planner::plan::PlanNodeEnum;

/// 重写规则 trait
///
/// 所有启发式优化规则实现此 trait
/// 这些规则不依赖代价计算，总是产生更优或等价的计划
pub trait RewriteRule {
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
    /// - `Ok(None)`: 不匹配，返回原节点
    /// - `Err(e)`: 重写失败
    fn apply(&self, node: PlanNodeEnum) -> Result<Option<PlanNodeEnum>, RewriteError>;
}

/// 重写规则 trait（可变版本）
///
/// 适用于需要原地修改节点的规则
pub trait RewriteRuleMut {
    /// 规则名称
    fn name(&self) -> &'static str;

    /// 检查是否匹配当前计划节点
    fn matches(&self, node: &PlanNodeEnum) -> bool;

    /// 应用重写规则（原地修改）
    ///
    /// # 参数
    /// - `node`: 当前计划节点的可变引用
    ///
    /// # 返回
    /// - `Ok(true)`: 重写成功
    /// - `Ok(false)`: 不匹配，未修改
    /// - `Err(e)`: 重写失败
    fn apply_mut(&self, node: &mut PlanNodeEnum) -> Result<bool, RewriteError>;
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
}
