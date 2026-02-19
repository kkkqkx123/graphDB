//! 规则注册初始化
//! 在程序启动时注册所有优化规则

use crate::query::optimizer::rule_registry::RuleRegistry;
use crate::query::optimizer::OptimizationRule;

pub fn register_all_rules() {
    register_logical_rules();
    register_physical_rules();
    register_post_rules();
}

fn register_logical_rules() {
    let _ = RuleRegistry::register(OptimizationRule::ProjectionPushDown, || Box::new(crate::query::optimizer::ProjectionPushDownRule));
    let _ = RuleRegistry::register(OptimizationRule::CombineFilter, || Box::new(crate::query::optimizer::CombineFilterRule));
    let _ = RuleRegistry::register(OptimizationRule::CollapseProject, || Box::new(crate::query::optimizer::CollapseProjectRule));
    let _ = RuleRegistry::register(OptimizationRule::DedupElimination, || Box::new(crate::query::optimizer::DedupEliminationRule));
    let _ = RuleRegistry::register(OptimizationRule::EliminateFilter, || Box::new(crate::query::optimizer::EliminateFilterRule));
    let _ = RuleRegistry::register(OptimizationRule::EliminateRowCollect, || Box::new(crate::query::optimizer::EliminateRowCollectRule));
    let _ = RuleRegistry::register(OptimizationRule::RemoveNoopProject, || Box::new(crate::query::optimizer::RemoveNoopProjectRule));
    let _ = RuleRegistry::register(OptimizationRule::EliminateAppendVertices, || Box::new(crate::query::optimizer::EliminateAppendVerticesRule));
    let _ = RuleRegistry::register(OptimizationRule::RemoveAppendVerticesBelowJoin, || Box::new(crate::query::optimizer::RemoveAppendVerticesBelowJoinRule));
    let _ = RuleRegistry::register(OptimizationRule::PushFilterDownAggregate, || Box::new(crate::query::optimizer::PushFilterDownAggregateRule));
    let _ = RuleRegistry::register(OptimizationRule::TopN, || Box::new(crate::query::optimizer::TopNRule));
    let _ = RuleRegistry::register(OptimizationRule::MergeGetVerticesAndProject, || Box::new(crate::query::optimizer::MergeGetVerticesAndProjectRule));
    let _ = RuleRegistry::register(OptimizationRule::MergeGetVerticesAndDedup, || Box::new(crate::query::optimizer::MergeGetVerticesAndDedupRule));
    let _ = RuleRegistry::register(OptimizationRule::MergeGetNbrsAndProject, || Box::new(crate::query::optimizer::MergeGetNbrsAndProjectRule));
    let _ = RuleRegistry::register(OptimizationRule::MergeGetNbrsAndDedup, || Box::new(crate::query::optimizer::MergeGetNbrsAndDedupRule));
}

fn register_physical_rules() {
    let _ = RuleRegistry::register(OptimizationRule::JoinOptimization, || Box::new(crate::query::optimizer::JoinOptimizationRule));
    let _ = RuleRegistry::register(OptimizationRule::PushLimitDownGetVertices, || Box::new(crate::query::optimizer::PushLimitDownGetVerticesRule));
    let _ = RuleRegistry::register(OptimizationRule::PushLimitDownGetEdges, || Box::new(crate::query::optimizer::PushLimitDownGetEdgesRule));
    let _ = RuleRegistry::register(OptimizationRule::PushLimitDownScanVertices, || Box::new(crate::query::optimizer::PushLimitDownScanVerticesRule));
    let _ = RuleRegistry::register(OptimizationRule::PushLimitDownScanEdges, || Box::new(crate::query::optimizer::PushLimitDownScanEdgesRule));
    let _ = RuleRegistry::register(OptimizationRule::PushLimitDownIndexScan, || Box::new(crate::query::optimizer::PushLimitDownIndexScanRule));
    let _ = RuleRegistry::register(OptimizationRule::ScanWithFilterOptimization, || Box::new(crate::query::optimizer::ScanWithFilterOptimizationRule));
    let _ = RuleRegistry::register(OptimizationRule::IndexFullScan, || Box::new(crate::query::optimizer::IndexFullScanRule));
    let _ = RuleRegistry::register(OptimizationRule::IndexScan, || Box::new(crate::query::optimizer::IndexScanRule));
    let _ = RuleRegistry::register(OptimizationRule::EdgeIndexFullScan, || Box::new(crate::query::optimizer::EdgeIndexFullScanRule));
    let _ = RuleRegistry::register(OptimizationRule::TagIndexFullScan, || Box::new(crate::query::optimizer::TagIndexFullScanRule));
    let _ = RuleRegistry::register(OptimizationRule::UnionAllEdgeIndexScan, || Box::new(crate::query::optimizer::UnionAllEdgeIndexScanRule));
    let _ = RuleRegistry::register(OptimizationRule::UnionAllTagIndexScan, || Box::new(crate::query::optimizer::UnionAllTagIndexScanRule));
    let _ = RuleRegistry::register(OptimizationRule::IndexCoveringScan, || Box::new(crate::query::optimizer::IndexCoveringScanRule));
    let _ = RuleRegistry::register(OptimizationRule::PushTopNDownIndexScan, || Box::new(crate::query::optimizer::PushTopNDownIndexScanRule));
    
}

fn register_post_rules() {
}
