//! 优化策略模块
//!
//! 提供查询优化策略，包括遍历起点选择、索引选择和基于代价的优化策略
//!
//! ## 模块结构
//!
//! - `traversal_start` - 遍历起点选择器
//! - `index` - 索引选择器
//! - `aggregate_strategy` - 聚合策略选择器
//! - `join_order` - 连接顺序优化器
//! - `traversal_direction` - 图遍历方向优化器
//! - `bidirectional_traversal` - 双向遍历优化器
//! - `topn_optimization` - TopN 优化器（Sort + Limit 到 TopN 的转换）
//! - `subquery_unnesting` - 子查询去关联化优化器
//! - `materialization` - CTE 物化优化器
//!
//! 注意：CTE结果缓存管理器已移至 `crate::query::cache` 模块

pub mod aggregate_strategy;
pub mod bidirectional_traversal;
pub mod index;
pub mod join_order;
pub mod materialization;
pub mod subquery_unnesting;
pub mod topn_optimization;
pub mod traversal_direction;
pub mod traversal_start;

pub use traversal_start::{
    CandidateStart, SelectionReason as TraversalSelectionReason, TraversalStartSelector,
};

pub use index::{IndexSelection, IndexSelector, PredicateOperator, PropertyPredicate};

pub use aggregate_strategy::{
    AggregateContext, AggregateStrategy, AggregateStrategyDecision, AggregateStrategySelector,
    SelectionReason as AggregateSelectionReason,
};

pub use join_order::{
    JoinCondition, JoinOrderOptimizer, JoinOrderResult, OptimizationMethod, TableInfo,
};

pub use bidirectional_traversal::{
    BidirectionalDecision, BidirectionalTraversalOptimizer, DepthAllocationContext,
};

pub use traversal_direction::{
    DegreeInfo, DirectionContext, DirectionSelectionReason, TraversalDirection,
    TraversalDirectionDecision, TraversalDirectionOptimizer,
};

pub use topn_optimization::{
    SortContext, SortEliminationDecision, SortEliminationOptimizer, SortKeepReason,
    TopNConversionReason,
};

pub use subquery_unnesting::{
    KeepReason, SubqueryUnnestingOptimizer, UnnestDecision, UnnestReason,
};

pub use materialization::{
    MaterializationDecision, MaterializationOptimizer, MaterializeReason, NoMaterializeReason,
};

// 从cache模块重新导出CTE缓存类型（向后兼容）
pub use crate::query::cache::{
    CteCacheConfig, CteCacheDecision, CteCacheDecisionMaker, CteCacheEntry, CteCacheManager,
    CteCacheStats,
};
