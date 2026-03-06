//! 优化决策模块
//!
//! 提供优化决策类型定义。
//!
//! 注意：原包含的决策缓存（DecisionCache）已删除，因为：
//! 1. 决策计算开销不大，缓存收益有限
//! 2. 版本感知机制复杂，维护成本高
//! 3. 实际使用场景少，QueryPlanCache已足够

// 类型定义
pub mod types;

// 重新导出主要类型
pub use types::{
    AccessPath, EntityIndexChoice, EntityType, IndexChoice, IndexSelectionDecision, JoinAlgorithm,
    JoinOrderDecision, OptimizationDecision, RewriteRuleId, TraversalStartDecision,
};
