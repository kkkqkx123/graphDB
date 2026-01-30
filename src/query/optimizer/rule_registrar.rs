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
    RuleRegistry::register(OptimizationRule::FilterPushDown, || Box::new(crate::query::optimizer::FilterPushDownRule));
    RuleRegistry::register(OptimizationRule::PredicatePushDown, || Box::new(crate::query::optimizer::PredicatePushDownRule));
    RuleRegistry::register(OptimizationRule::ProjectionPushDown, || Box::new(crate::query::optimizer::ProjectionPushDownRule));
    RuleRegistry::register(OptimizationRule::CombineFilter, || Box::new(crate::query::optimizer::CombineFilterRule));
    RuleRegistry::register(OptimizationRule::CollapseProject, || Box::new(crate::query::optimizer::CollapseProjectRule));
    RuleRegistry::register(OptimizationRule::DedupElimination, || Box::new(crate::query::optimizer::DedupEliminationRule));
    RuleRegistry::register(OptimizationRule::EliminateFilter, || Box::new(crate::query::optimizer::EliminateFilterRule));
    RuleRegistry::register(OptimizationRule::EliminateRowCollect, || Box::new(crate::query::optimizer::EliminateRowCollectRule));
    RuleRegistry::register(OptimizationRule::RemoveNoopProject, || Box::new(crate::query::optimizer::RemoveNoopProjectRule));
    RuleRegistry::register(OptimizationRule::EliminateAppendVertices, || Box::new(crate::query::optimizer::EliminateAppendVerticesRule));
    RuleRegistry::register(OptimizationRule::RemoveAppendVerticesBelowJoin, || Box::new(crate::query::optimizer::RemoveAppendVerticesBelowJoinRule));
    RuleRegistry::register(OptimizationRule::PushFilterDownAggregate, || Box::new(crate::query::optimizer::PushFilterDownAggregateRule));
    RuleRegistry::register(OptimizationRule::TopN, || Box::new(crate::query::optimizer::TopNRule));
    RuleRegistry::register(OptimizationRule::MergeGetVerticesAndProject, || Box::new(crate::query::optimizer::MergeGetVerticesAndProjectRule));
    RuleRegistry::register(OptimizationRule::MergeGetVerticesAndDedup, || Box::new(crate::query::optimizer::MergeGetVerticesAndDedupRule));
    RuleRegistry::register(OptimizationRule::MergeGetNbrsAndProject, || Box::new(crate::query::optimizer::MergeGetNbrsAndProjectRule));
    RuleRegistry::register(OptimizationRule::MergeGetNbrsAndDedup, || Box::new(crate::query::optimizer::MergeGetNbrsAndDedupRule));
}

fn register_physical_rules() {
    RuleRegistry::register(OptimizationRule::JoinOptimization, || Box::new(crate::query::optimizer::JoinOptimizationRule));
    RuleRegistry::register(OptimizationRule::PushLimitDown, || Box::new(crate::query::optimizer::PushLimitDownRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownGetVertices, || Box::new(crate::query::optimizer::PushLimitDownGetVerticesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownGetNeighbors, || Box::new(crate::query::optimizer::PushLimitDownGetNeighborsRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownGetEdges, || Box::new(crate::query::optimizer::PushLimitDownGetEdgesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownScanVertices, || Box::new(crate::query::optimizer::PushLimitDownScanVerticesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownScanEdges, || Box::new(crate::query::optimizer::PushLimitDownScanEdgesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownIndexScan, || Box::new(crate::query::optimizer::PushLimitDownIndexScanRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownProjectRule, || Box::new(crate::query::optimizer::PushLimitDownProjectRule));
    RuleRegistry::register(OptimizationRule::ScanWithFilterOptimization, || Box::new(crate::query::optimizer::ScanWithFilterOptimizationRule));
    RuleRegistry::register(OptimizationRule::IndexFullScan, || Box::new(crate::query::optimizer::IndexFullScanRule));
    RuleRegistry::register(OptimizationRule::IndexScan, || Box::new(crate::query::optimizer::IndexScanRule));
    RuleRegistry::register(OptimizationRule::EdgeIndexFullScan, || Box::new(crate::query::optimizer::EdgeIndexFullScanRule));
    RuleRegistry::register(OptimizationRule::TagIndexFullScan, || Box::new(crate::query::optimizer::TagIndexFullScanRule));
    RuleRegistry::register(OptimizationRule::UnionAllEdgeIndexScan, || Box::new(crate::query::optimizer::UnionAllEdgeIndexScanRule));
    RuleRegistry::register(OptimizationRule::UnionAllTagIndexScan, || Box::new(crate::query::optimizer::UnionAllTagIndexScanRule));
    RuleRegistry::register(OptimizationRule::OptimizeEdgeIndexScanByFilter, || Box::new(crate::query::optimizer::OptimizeEdgeIndexScanByFilterRule));
    RuleRegistry::register(OptimizationRule::OptimizeTagIndexScanByFilter, || Box::new(crate::query::optimizer::OptimizeTagIndexScanByFilterRule));
}

fn register_post_rules() {
}
