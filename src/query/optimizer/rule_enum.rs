//! 优化规则枚举定义
//! 使用枚举替代字符串匹配规则，提高类型安全性和可维护性

use std::rc::Rc;

use crate::query::optimizer::core::OptimizationPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationRule {
    // 逻辑优化规则
    FilterPushDown,
    PredicatePushDown,
    ProjectionPushDown,
    CombineFilter,
    CollapseProject,
    DedupElimination,
    EliminateFilter,
    EliminateRowCollect,
    RemoveNoopProject,
    EliminateAppendVertices,
    RemoveAppendVerticesBelowJoin,
    PushFilterDownAggregate,
    TopN,
    MergeGetVerticesAndProject,
    MergeGetVerticesAndDedup,
    MergeGetNbrsAndProject,
    MergeGetNbrsAndDedup,

    // 后优化规则
    ConstantFolding,
    SubQueryOptimization,
    LoopUnrolling,
    PredicateReorder,
    
    // 物理优化规则
    JoinOptimization,
    PushLimitDown,
    PushLimitDownGetVertices,
    PushLimitDownGetNeighbors,
    PushLimitDownGetEdges,
    PushLimitDownScanVertices,
    PushLimitDownScanEdges,
    PushLimitDownIndexScan,
    PushLimitDownProjectRule,
    ScanWithFilterOptimization,
    IndexFullScan,
    IndexScan,
    EdgeIndexFullScan,
    TagIndexFullScan,
    UnionAllEdgeIndexScan,
    UnionAllTagIndexScan,
    OptimizeEdgeIndexScanByFilter,
    OptimizeTagIndexScanByFilter,
}

impl OptimizationRule {
    pub fn phase(&self) -> OptimizationPhase {
        match self {
            Self::FilterPushDown | Self::PredicatePushDown | Self::ProjectionPushDown |
            Self::CombineFilter | Self::CollapseProject | Self::DedupElimination |
            Self::EliminateFilter | Self::EliminateRowCollect | Self::RemoveNoopProject |
            Self::EliminateAppendVertices | Self::RemoveAppendVerticesBelowJoin |
            Self::PushFilterDownAggregate | Self::TopN |
            Self::MergeGetVerticesAndProject | Self::MergeGetVerticesAndDedup |
            Self::MergeGetNbrsAndProject | Self::MergeGetNbrsAndDedup => OptimizationPhase::Logical,

            Self::ConstantFolding | Self::SubQueryOptimization |
            Self::LoopUnrolling | Self::PredicateReorder => OptimizationPhase::Unknown,

            Self::JoinOptimization | Self::PushLimitDown | Self::PushLimitDownGetVertices |
            Self::PushLimitDownGetNeighbors | Self::PushLimitDownGetEdges |
            Self::PushLimitDownScanVertices | Self::PushLimitDownScanEdges |
            Self::PushLimitDownIndexScan | Self::PushLimitDownProjectRule |
            Self::ScanWithFilterOptimization | Self::IndexFullScan | Self::IndexScan |
            Self::EdgeIndexFullScan | Self::TagIndexFullScan | Self::UnionAllEdgeIndexScan |
            Self::UnionAllTagIndexScan | Self::OptimizeEdgeIndexScanByFilter |
            Self::OptimizeTagIndexScanByFilter => OptimizationPhase::Physical,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            Self::FilterPushDown => "FilterPushDownRule",
            Self::PredicatePushDown => "PredicatePushDownRule",
            Self::ProjectionPushDown => "ProjectionPushDownRule",
            Self::CombineFilter => "CombineFilterRule",
            Self::CollapseProject => "CollapseProjectRule",
            Self::DedupElimination => "DedupEliminationRule",
            Self::EliminateFilter => "EliminateFilterRule",
            Self::EliminateRowCollect => "EliminateRowCollectRule",
            Self::RemoveNoopProject => "RemoveNoopProjectRule",
            Self::EliminateAppendVertices => "EliminateAppendVerticesRule",
            Self::RemoveAppendVerticesBelowJoin => "RemoveAppendVerticesBelowJoinRule",
            Self::PushFilterDownAggregate => "PushFilterDownAggregateRule",
            Self::TopN => "TopNRule",
            Self::MergeGetVerticesAndProject => "MergeGetVerticesAndProjectRule",
            Self::MergeGetVerticesAndDedup => "MergeGetVerticesAndDedupRule",
            Self::MergeGetNbrsAndProject => "MergeGetNbrsAndProjectRule",
            Self::MergeGetNbrsAndDedup => "MergeGetNbrsAndDedupRule",

            Self::ConstantFolding => "ConstantFoldingRule",
            Self::SubQueryOptimization => "SubQueryOptimizationRule",
            Self::LoopUnrolling => "LoopUnrollingRule",
            Self::PredicateReorder => "PredicateReorderRule",

            Self::JoinOptimization => "JoinOptimizationRule",
            Self::PushLimitDown => "PushLimitDownRule",
            Self::PushLimitDownGetVertices => "PushLimitDownGetVerticesRule",
            Self::PushLimitDownGetNeighbors => "PushLimitDownGetNeighborsRule",
            Self::PushLimitDownGetEdges => "PushLimitDownGetEdgesRule",
            Self::PushLimitDownScanVertices => "PushLimitDownScanVerticesRule",
            Self::PushLimitDownScanEdges => "PushLimitDownScanEdgesRule",
            Self::PushLimitDownIndexScan => "PushLimitDownIndexScanRule",
            Self::PushLimitDownProjectRule => "PushLimitDownProjectRule",
            Self::ScanWithFilterOptimization => "ScanWithFilterOptimizationRule",
            Self::IndexFullScan => "IndexFullScanRule",
            Self::IndexScan => "IndexScanRule",
            Self::EdgeIndexFullScan => "EdgeIndexFullScanRule",
            Self::TagIndexFullScan => "TagIndexFullScanRule",
            Self::UnionAllEdgeIndexScan => "UnionAllEdgeIndexScanRule",
            Self::UnionAllTagIndexScan => "UnionAllTagIndexScanRule",
            Self::OptimizeEdgeIndexScanByFilter => "OptimizeEdgeIndexScanByFilterRule",
            Self::OptimizeTagIndexScanByFilter => "OptimizeTagIndexScanByFilterRule",
        }
    }
    
    pub fn create_instance(&self) -> Option<Rc<dyn super::OptRule>> {
        match self {
            Self::FilterPushDown => Some(Rc::new(super::FilterPushDownRule)),
            Self::PredicatePushDown => Some(Rc::new(super::PredicatePushDownRule)),
            Self::ProjectionPushDown => Some(Rc::new(super::ProjectionPushDownRule)),
            Self::CombineFilter => Some(Rc::new(super::CombineFilterRule)),
            Self::CollapseProject => Some(Rc::new(super::CollapseProjectRule)),
            Self::DedupElimination => Some(Rc::new(super::DedupEliminationRule)),
            Self::EliminateFilter => Some(Rc::new(super::EliminateFilterRule)),
            Self::EliminateRowCollect => Some(Rc::new(super::EliminateRowCollectRule)),
            Self::RemoveNoopProject => Some(Rc::new(super::RemoveNoopProjectRule)),
            Self::EliminateAppendVertices => Some(Rc::new(super::EliminateAppendVerticesRule)),
            Self::RemoveAppendVerticesBelowJoin => Some(Rc::new(super::RemoveAppendVerticesBelowJoinRule)),
            Self::PushFilterDownAggregate => Some(Rc::new(super::PushFilterDownAggregateRule)),
            Self::TopN => Some(Rc::new(super::TopNRule)),
            Self::MergeGetVerticesAndProject => Some(Rc::new(super::MergeGetVerticesAndProjectRule)),
            Self::MergeGetVerticesAndDedup => Some(Rc::new(super::MergeGetVerticesAndDedupRule)),
            Self::MergeGetNbrsAndProject => Some(Rc::new(super::MergeGetNbrsAndProjectRule)),
            Self::MergeGetNbrsAndDedup => Some(Rc::new(super::MergeGetNbrsAndDedupRule)),

            Self::ConstantFolding => Some(Rc::new(super::ConstantFoldingRule)),
            Self::SubQueryOptimization => Some(Rc::new(super::SubQueryOptimizationRule)),
            Self::LoopUnrolling => Some(Rc::new(super::LoopUnrollingRule)),
            Self::PredicateReorder => Some(Rc::new(super::PredicateReorderRule)),

            Self::JoinOptimization => Some(Rc::new(super::JoinOptimizationRule)),
            Self::PushLimitDown => Some(Rc::new(super::PushLimitDownRule)),
            Self::PushLimitDownGetVertices => Some(Rc::new(super::PushLimitDownGetVerticesRule)),
            Self::PushLimitDownGetNeighbors => Some(Rc::new(super::PushLimitDownGetNeighborsRule)),
            Self::PushLimitDownGetEdges => Some(Rc::new(super::PushLimitDownGetEdgesRule)),
            Self::PushLimitDownScanVertices => Some(Rc::new(super::PushLimitDownScanVerticesRule)),
            Self::PushLimitDownScanEdges => Some(Rc::new(super::PushLimitDownScanEdgesRule)),
            Self::PushLimitDownIndexScan => Some(Rc::new(super::PushLimitDownIndexScanRule)),
            Self::PushLimitDownProjectRule => Some(Rc::new(super::PushLimitDownProjectRule)),
            Self::ScanWithFilterOptimization => Some(Rc::new(super::ScanWithFilterOptimizationRule)),
            Self::IndexFullScan => Some(Rc::new(super::IndexFullScanRule)),
            Self::IndexScan => Some(Rc::new(super::IndexScanRule)),
            Self::EdgeIndexFullScan => Some(Rc::new(super::EdgeIndexFullScanRule)),
            Self::TagIndexFullScan => Some(Rc::new(super::TagIndexFullScanRule)),
            Self::UnionAllEdgeIndexScan => Some(Rc::new(super::UnionAllEdgeIndexScanRule)),
            Self::UnionAllTagIndexScan => Some(Rc::new(super::UnionAllTagIndexScanRule)),
            Self::OptimizeEdgeIndexScanByFilter => Some(Rc::new(super::OptimizeEdgeIndexScanByFilterRule)),
            Self::OptimizeTagIndexScanByFilter => Some(Rc::new(super::OptimizeTagIndexScanByFilterRule)),
        }
    }
    
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "FilterPushDownRule" => Some(Self::FilterPushDown),
            "PredicatePushDownRule" => Some(Self::PredicatePushDown),
            "ProjectionPushDownRule" => Some(Self::ProjectionPushDown),
            "CombineFilterRule" => Some(Self::CombineFilter),
            "CollapseProjectRule" => Some(Self::CollapseProject),
            "DedupEliminationRule" => Some(Self::DedupElimination),
            "EliminateFilterRule" => Some(Self::EliminateFilter),
            "EliminateRowCollectRule" => Some(Self::EliminateRowCollect),
            "RemoveNoopProjectRule" => Some(Self::RemoveNoopProject),
            "EliminateAppendVerticesRule" => Some(Self::EliminateAppendVertices),
            "RemoveAppendVerticesBelowJoinRule" => Some(Self::RemoveAppendVerticesBelowJoin),
            "TopNRule" => Some(Self::TopN),
            "MergeGetVerticesAndProjectRule" => Some(Self::MergeGetVerticesAndProject),
            "MergeGetVerticesAndDedupRule" => Some(Self::MergeGetVerticesAndDedup),
            "MergeGetNbrsAndProjectRule" => Some(Self::MergeGetNbrsAndProject),
            "MergeGetNbrsAndDedupRule" => Some(Self::MergeGetNbrsAndDedup),

            "ConstantFoldingRule" => Some(Self::ConstantFolding),
            "SubQueryOptimizationRule" => Some(Self::SubQueryOptimization),
            "LoopUnrollingRule" => Some(Self::LoopUnrolling),
            "PredicateReorderRule" => Some(Self::PredicateReorder),

            "JoinOptimizationRule" => Some(Self::JoinOptimization),
            "PushLimitDownRule" => Some(Self::PushLimitDown),
            "PushLimitDownGetVerticesRule" => Some(Self::PushLimitDownGetVertices),
            "PushLimitDownGetNeighborsRule" => Some(Self::PushLimitDownGetNeighbors),
            "PushLimitDownGetEdgesRule" => Some(Self::PushLimitDownGetEdges),
            "PushLimitDownScanVerticesRule" => Some(Self::PushLimitDownScanVertices),
            "PushLimitDownScanEdgesRule" => Some(Self::PushLimitDownScanEdges),
            "PushLimitDownIndexScanRule" => Some(Self::PushLimitDownIndexScan),
            "PushLimitDownProjectRule" => Some(Self::PushLimitDownProjectRule),
            "ScanWithFilterOptimizationRule" => Some(Self::ScanWithFilterOptimization),
            "IndexFullScanRule" => Some(Self::IndexFullScan),
            "IndexScanRule" => Some(Self::IndexScan),
            "EdgeIndexFullScanRule" => Some(Self::EdgeIndexFullScan),
            "TagIndexFullScanRule" => Some(Self::TagIndexFullScan),
            "UnionAllEdgeIndexScanRule" => Some(Self::UnionAllEdgeIndexScan),
            "UnionAllTagIndexScanRule" => Some(Self::UnionAllTagIndexScan),
            "OptimizeEdgeIndexScanByFilterRule" => Some(Self::OptimizeEdgeIndexScanByFilter),
            "OptimizeTagIndexScanByFilterRule" => Some(Self::OptimizeTagIndexScanByFilter),
            
            _ => None,
        }
    }
}
