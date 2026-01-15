//! Optimizer module for optimizing execution plans
//! Contains the Optimizer implementation and various optimization rules

// 基础设施模块
pub mod rule_patterns;
pub mod rule_traits;

// 优化策略模块
pub mod elimination_rules;
pub mod index_optimization;
pub mod join_optimization;
pub mod limit_pushdown;
pub mod operation_merge;
pub mod optimizer;
pub mod plan_validator;
pub mod predicate_pushdown;
pub mod projection_pushdown;
pub mod property_tracker;
pub mod prune_properties_visitor;
pub mod scan_optimization;
pub mod transformation_rules;

// Re-export all rule structs for convenient access
pub use elimination_rules::{
    DedupEliminationRule, EliminateAppendVerticesRule, EliminateFilterRule,
    RemoveAppendVerticesBelowJoinRule, RemoveNoopProjectRule,
};
pub use index_optimization::{
    EdgeIndexFullScanRule, IndexScanRule, OptimizeEdgeIndexScanByFilterRule,
    OptimizeTagIndexScanByFilterRule, TagIndexFullScanRule, UnionAllEdgeIndexScanRule,
    UnionAllTagIndexScanRule,
};
pub use join_optimization::JoinOptimizationRule;
pub use limit_pushdown::{
    PushLimitDownGetEdgesRule, PushLimitDownGetNeighborsRule, PushLimitDownGetVerticesRule,
    PushLimitDownIndexScanRule, PushLimitDownProjectRule, PushLimitDownRule,
    PushLimitDownScanEdgesRule, PushLimitDownScanVerticesRule,
};
pub use operation_merge::{
    CollapseProjectRule, CombineFilterRule, MergeGetNbrsAndDedupRule, MergeGetNbrsAndProjectRule,
    MergeGetVerticesAndDedupRule, MergeGetVerticesAndProjectRule,
};
pub use predicate_pushdown::{
    FilterPushDownRule, PredicatePushDownRule, PushFilterDownExpandRule,
    PushFilterDownHashInnerJoinRule, PushFilterDownHashLeftJoinRule, PushFilterDownInnerJoinRule,
    PushFilterDownTraverseRule,
};
pub use projection_pushdown::{ProjectionPushDownRule, PushProjectDownRule};
pub use rule_traits::{BaseOptRule, EliminationRule, MergeRule, PushDownRule};
pub use scan_optimization::{IndexFullScanRule, ScanWithFilterOptimizationRule};
pub use optimizer::OptimizerError;
pub use transformation_rules::TopNRule;

// Re-export the main Optimizer struct
pub use optimizer::Optimizer;
pub use plan_validator::PlanValidator;
