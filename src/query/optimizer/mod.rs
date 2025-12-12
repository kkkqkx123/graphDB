//! Optimizer module for optimizing execution plans
//! Contains the Optimizer implementation and various optimization rules

// 基础设施模块
pub mod rule_traits;
pub mod rule_patterns;

// 优化策略模块
pub mod elimination_rules;
pub mod operation_merge;
pub mod predicate_pushdown;
pub mod limit_pushdown;
pub mod projection_pushdown;
pub mod scan_optimization;
pub mod index_optimization;
pub mod join_optimization;
pub mod optimizer;

// Re-export all rule structs for convenient access
pub use elimination_rules::{
    EliminateFilterRule, DedupEliminationRule, RemoveNoopProjectRule,
    EliminateAppendVerticesRule, RemoveAppendVerticesBelowJoinRule, TopNRule
};
pub use operation_merge::{
    CombineFilterRule, CollapseProjectRule, MergeGetVerticesAndProjectRule,
    MergeGetVerticesAndDedupRule, MergeGetNbrsAndDedupRule, MergeGetNbrsAndProjectRule
};
pub use predicate_pushdown::{
    FilterPushDownRule, PushFilterDownTraverseRule, PushFilterDownExpandRule,
    PushFilterDownHashInnerJoinRule, PushFilterDownHashLeftJoinRule,
    PushFilterDownInnerJoinRule, PredicatePushDownRule
};
pub use limit_pushdown::{
    PushLimitDownRule, PushLimitDownGetVerticesRule, PushLimitDownGetNeighborsRule,
    PushLimitDownGetEdgesRule, PushLimitDownScanVerticesRule,
    PushLimitDownScanEdgesRule, PushLimitDownIndexScanRule,
    PushLimitDownProjectRule
};
pub use projection_pushdown::{
    ProjectionPushDownRule, PushProjectDownRule
};
pub use scan_optimization::{
    IndexFullScanRule, ScanWithFilterOptimizationRule
};
pub use index_optimization::{
    OptimizeEdgeIndexScanByFilterRule, OptimizeTagIndexScanByFilterRule,
    EdgeIndexFullScanRule, TagIndexFullScanRule, IndexScanRule,
    UnionAllEdgeIndexScanRule, UnionAllTagIndexScanRule
};
pub use join_optimization::{
    JoinOptimizationRule
};