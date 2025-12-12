//! Optimizer module for optimizing execution plans
//! Contains the Optimizer implementation and various optimization rules

pub mod filter_rules;
pub mod projection_rules;
pub mod general_rules;
pub mod index_rules;
pub mod index_scan_rules;
pub mod join_rules;
pub mod limit_rules;
pub mod optimizer;

// Re-export all rule structs for convenient access
pub use filter_rules::{
    FilterPushDownRule, PushFilterDownTraverseRule, PushFilterDownExpandRule,
    CombineFilterRule, EliminateFilterRule, PredicatePushDownRule
};
pub use projection_rules::{
    ProjectionPushDownRule, CollapseProjectRule, RemoveNoopProjectRule,
    PushProjectDownRule
};
pub use general_rules::{
    DedupEliminationRule, JoinOptimizationRule, LimitOptimizationRule,
    IndexFullScanRule, TopNRule, EliminateAppendVerticesRule,
    MergeGetVerticesAndProjectRule, ScanWithFilterOptimizationRule
};
pub use index_rules::{
    OptimizeEdgeIndexScanByFilterRule, OptimizeTagIndexScanByFilterRule,
    PushLimitDownRule
};
pub use index_scan_rules::{
    EdgeIndexFullScanRule, TagIndexFullScanRule, IndexScanRule,
    UnionAllEdgeIndexScanRule, UnionAllTagIndexScanRule
};
pub use join_rules::{
    PushFilterDownHashInnerJoinRule, PushFilterDownHashLeftJoinRule,
    PushFilterDownInnerJoinRule, MergeGetVerticesAndDedupRule,
    MergeGetNbrsAndDedupRule,
    MergeGetNbrsAndProjectRule, RemoveAppendVerticesBelowJoinRule
};
pub use limit_rules::{
    PushLimitDownGetVerticesRule, PushLimitDownGetNeighborsRule,
    PushLimitDownGetEdgesRule, PushLimitDownScanVerticesRule,
    PushLimitDownScanEdgesRule, PushLimitDownIndexScanRule,
    PushLimitDownProjectRule, PushLimitDownAllPathsRule,
    PushLimitDownExpandAllRule
};