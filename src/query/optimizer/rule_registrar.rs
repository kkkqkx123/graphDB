//! 规则注册初始化
//! 在程序启动时注册所有优化规则

use crate::query::optimizer::rule_registry::RuleRegistry;
use crate::query::optimizer::OptimizationRule;

#[cfg(feature = "optimizer_registration")]
pub fn register_all_rules() {
    register_logical_rules();
    register_physical_rules();
    register_post_rules();
}

#[cfg(feature = "optimizer_registration")]
fn register_logical_rules() {
    use crate::query::optimizer::FilterPushDownRule;
    use crate::query::optimizer::PredicatePushDownRule;
    use crate::query::optimizer::PushFilterDownTraverseRule;
    use crate::query::optimizer::PushFilterDownExpandRule;
    use crate::query::optimizer::PushFilterDownInnerJoinRule;
    use crate::query::optimizer::PushFilterDownHashInnerJoinRule;
    use crate::query::optimizer::PushFilterDownHashLeftJoinRule;
    use crate::query::optimizer::ProjectionPushDownRule;
    use crate::query::optimizer::PushProjectDownRule;
    use crate::query::optimizer::CombineFilterRule;
    use crate::query::optimizer::CollapseProjectRule;
    use crate::query::optimizer::MergeGetVerticesAndProjectRule;
    use crate::query::optimizer::MergeGetVerticesAndDedupRule;
    use crate::query::optimizer::MergeGetNbrsAndDedupRule;
    use crate::query::optimizer::MergeGetNbrsAndProjectRule;
    use crate::query::optimizer::DedupEliminationRule;
    use crate::query::optimizer::EliminateFilterRule;
    use crate::query::optimizer::RemoveNoopProjectRule;
    use crate::query::optimizer::EliminateAppendVerticesRule;
    use crate::query::optimizer::RemoveAppendVerticesBelowJoinRule;
    use crate::query::optimizer::TopNRule;
    
    RuleRegistry::register(OptimizationRule::FilterPushDown, || Box::new(FilterPushDownRule));
    RuleRegistry::register(OptimizationRule::PredicatePushDown, || Box::new(PredicatePushDownRule));
    RuleRegistry::register(OptimizationRule::ProjectionPushDown, || Box::new(ProjectionPushDownRule));
    RuleRegistry::register(OptimizationRule::CombineFilter, || Box::new(CombineFilterRule));
    RuleRegistry::register(OptimizationRule::CollapseProject, || Box::new(CollapseProjectRule));
    RuleRegistry::register(OptimizationRule::DedupElimination, || Box::new(DedupEliminationRule));
    RuleRegistry::register(OptimizationRule::EliminateFilter, || Box::new(EliminateFilterRule));
    RuleRegistry::register(OptimizationRule::RemoveNoopProject, || Box::new(RemoveNoopProjectRule));
    RuleRegistry::register(OptimizationRule::EliminateAppendVertices, || Box::new(EliminateAppendVerticesRule));
    RuleRegistry::register(OptimizationRule::RemoveAppendVerticesBelowJoin, || Box::new(RemoveAppendVerticesBelowJoinRule));
    RuleRegistry::register(OptimizationRule::TopN, || Box::new(TopNRule));
    RuleRegistry::register(OptimizationRule::MergeGetVerticesAndProject, || Box::new(MergeGetVerticesAndProjectRule));
    RuleRegistry::register(OptimizationRule::MergeGetVerticesAndDedup, || Box::new(MergeGetVerticesAndDedupRule));
    RuleRegistry::register(OptimizationRule::MergeGetNbrsAndProject, || Box::new(MergeGetNbrsAndProjectRule));
    RuleRegistry::register(OptimizationRule::MergeGetNbrsAndDedup, || Box::new(MergeGetNbrsAndDedupRule));
}

#[cfg(feature = "optimizer_registration")]
fn register_physical_rules() {
    use crate::query::optimizer::JoinOptimizationRule;
    use crate::query::optimizer::PushLimitDownRule;
    use crate::query::optimizer::PushLimitDownGetVerticesRule;
    use crate::query::optimizer::PushLimitDownGetNeighborsRule;
    use crate::query::optimizer::PushLimitDownGetEdgesRule;
    use crate::query::optimizer::PushLimitDownScanVerticesRule;
    use crate::query::optimizer::PushLimitDownScanEdgesRule;
    use crate::query::optimizer::PushLimitDownIndexScanRule;
    use crate::query::optimizer::PushLimitDownProjectRule;
    use crate::query::optimizer::ScanWithFilterOptimizationRule;
    use crate::query::optimizer::IndexFullScanRule;
    use crate::query::optimizer::IndexScanRule;
    use crate::query::optimizer::EdgeIndexFullScanRule;
    use crate::query::optimizer::TagIndexFullScanRule;
    use crate::query::optimizer::UnionAllEdgeIndexScanRule;
    use crate::query::optimizer::UnionAllTagIndexScanRule;
    use crate::query::optimizer::OptimizeEdgeIndexScanByFilterRule;
    use crate::query::optimizer::OptimizeTagIndexScanByFilterRule;
    
    RuleRegistry::register(OptimizationRule::JoinOptimization, || Box::new(JoinOptimizationRule));
    RuleRegistry::register(OptimizationRule::PushLimitDown, || Box::new(PushLimitDownRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownGetVertices, || Box::new(PushLimitDownGetVerticesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownGetNeighbors, || Box::new(PushLimitDownGetNeighborsRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownGetEdges, || Box::new(PushLimitDownGetEdgesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownScanVertices, || Box::new(PushLimitDownScanVerticesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownScanEdges, || Box::new(PushLimitDownScanEdgesRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownIndexScan, || Box::new(PushLimitDownIndexScanRule));
    RuleRegistry::register(OptimizationRule::PushLimitDownProjectRule, || Box::new(PushLimitDownProjectRule));
    RuleRegistry::register(OptimizationRule::ScanWithFilterOptimization, || Box::new(ScanWithFilterOptimizationRule));
    RuleRegistry::register(OptimizationRule::IndexFullScan, || Box::new(IndexFullScanRule));
    RuleRegistry::register(OptimizationRule::IndexScan, || Box::new(IndexScanRule));
    RuleRegistry::register(OptimizationRule::EdgeIndexFullScan, || Box::new(EdgeIndexFullScanRule));
    RuleRegistry::register(OptimizationRule::TagIndexFullScan, || Box::new(TagIndexFullScanRule));
    RuleRegistry::register(OptimizationRule::UnionAllEdgeIndexScan, || Box::new(UnionAllEdgeIndexScanRule));
    RuleRegistry::register(OptimizationRule::UnionAllTagIndexScan, || Box::new(UnionAllTagIndexScanRule));
    RuleRegistry::register(OptimizationRule::OptimizeEdgeIndexScanByFilter, || Box::new(OptimizeEdgeIndexScanByFilterRule));
    RuleRegistry::register(OptimizationRule::OptimizeTagIndexScanByFilter, || Box::new(OptimizeTagIndexScanByFilterRule));
}

#[cfg(feature = "optimizer_registration")]
fn register_post_rules() {
    use crate::query::optimizer::RemoveUselessNodeRule;
    
    RuleRegistry::register(OptimizationRule::RemoveUselessNode, || Box::new(RemoveUselessNodeRule));
}

#[cfg(not(feature = "optimizer_registration"))]
pub fn register_all_rules() {
    // 当特性未启用时，不执行任何操作
}

#[cfg(not(feature = "optimizer_registration"))]
fn register_logical_rules() {}
#[cfg(not(feature = "optimizer_registration"))]
fn register_physical_rules() {}
#[cfg(not(feature = "optimizer_registration"))]
fn register_post_rules() {}
