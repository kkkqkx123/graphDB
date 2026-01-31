//! Optimizer module for optimizing execution plans
//! Contains the Optimizer implementation and various optimization rules

// 基础设施模块
pub mod rule_patterns;
pub mod rule_traits;

// 核心类型模块
pub mod core;

// 规则枚举和配置模块
pub mod rule_enum;
pub mod rule_config;
pub mod rule_registry;
pub mod rule_registrar;
pub mod optimizer_config;
pub mod plan_node_visitor;

// 执行计划表示模块
pub mod plan;

// 优化引擎模块
pub mod engine;

// 优化策略模块
pub mod elimination_rules;
pub mod index_optimization;
pub mod join_optimization;
pub mod limit_pushdown;
pub mod operation_merge;
pub mod plan_validator;
pub mod predicate_pushdown;
pub mod projection_pushdown;
pub mod property_tracker;
pub mod prune_properties_visitor;
pub mod push_filter_down_aggregate;
pub mod scan_optimization;
pub mod transformation_rules;
pub mod predicate_reorder;
pub mod constant_folding;
pub mod subquery_optimization;
pub mod loop_unrolling;

// Re-export core types
pub use core::{Cost, OptimizationConfig, OptimizationPhase, OptimizationStats, Statistics};

// Re-export rule enum and config
pub use rule_enum::OptimizationRule;
pub use rule_config::RuleConfig;
pub use rule_registry::RuleRegistry;
pub use optimizer_config::{load_optimizer_config, OptimizerConfigInfo};
pub use plan_node_visitor::{PlanNodeVisitor, PlanNodeVisitable};

// Re-export plan types
pub use plan::{
    OptContext, OptGroup, OptGroupNode, MatchedResult, MatchNode, OptRule, Pattern,
    PlanCandidate, PlanNodeProperties, TransformResult, OptimizerError,
};
pub use crate::utils::ObjectPool;

// Re-export engine types
pub use engine::{ExplorationState, Optimizer, RuleSet};

// Re-export all rule structs for convenient access
pub use elimination_rules::{
    DedupEliminationRule, EliminateAppendVerticesRule, EliminateFilterRule,
    EliminateRowCollectRule, RemoveAppendVerticesBelowJoinRule, RemoveNoopProjectRule,
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
pub use property_tracker::PropertyTracker;
pub use prune_properties_visitor::PrunePropertiesVisitor;
pub use push_filter_down_aggregate::PushFilterDownAggregateRule;
pub use rule_traits::{BaseOptRule, EliminationRule, MergeRule, PushDownRule};
pub use scan_optimization::{IndexFullScanRule, ScanWithFilterOptimizationRule};
pub use transformation_rules::TopNRule;
pub use predicate_reorder::PredicateReorderRule;
pub use constant_folding::ConstantFoldingRule;
pub use subquery_optimization::SubQueryOptimizationRule;
pub use loop_unrolling::LoopUnrollingRule;

// Re-export PlanValidator
pub use plan_validator::PlanValidator;
