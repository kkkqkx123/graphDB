//! 优化规则枚举定义
//! 使用枚举替代字符串匹配规则，提高类型安全性和可维护性

use crate::query::optimizer::core::phase::OptimizationPhase;

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
    RemoveNoopProject,
    EliminateAppendVertices,
    RemoveAppendVerticesBelowJoin,
    TopN,
    MergeGetVerticesAndProject,
    MergeGetVerticesAndDedup,
    MergeGetNbrsAndProject,
    MergeGetNbrsAndDedup,
    
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
    
    // 后优化规则
    RemoveUselessNode,
}

impl OptimizationRule {
    pub fn phase(&self) -> OptimizationPhase {
        match self {
            Self::FilterPushDown | Self::PredicatePushDown | Self::ProjectionPushDown |
            Self::CombineFilter | Self::CollapseProject | Self::DedupElimination |
            Self::EliminateFilter | Self::RemoveNoopProject | Self::EliminateAppendVertices |
            Self::RemoveAppendVerticesBelowJoin | Self::TopN | Self::MergeGetVerticesAndProject |
            Self::MergeGetVerticesAndDedup | Self::MergeGetNbrsAndProject |
            Self::MergeGetNbrsAndDedup => OptimizationPhase::LogicalOptimization,
            
            Self::JoinOptimization | Self::PushLimitDown | Self::PushLimitDownGetVertices |
            Self::PushLimitDownGetNeighbors | Self::PushLimitDownGetEdges |
            Self::PushLimitDownScanVertices | Self::PushLimitDownScanEdges |
            Self::PushLimitDownIndexScan | Self::PushLimitDownProjectRule |
            Self::ScanWithFilterOptimization | Self::IndexFullScan | Self::IndexScan |
            Self::EdgeIndexFullScan | Self::TagIndexFullScan | Self::UnionAllEdgeIndexScan |
            Self::UnionAllTagIndexScan | Self::OptimizeEdgeIndexScanByFilter |
            Self::OptimizeTagIndexScanByFilter => OptimizationPhase::PhysicalOptimization,
            
            Self::RemoveUselessNode => OptimizationPhase::PostOptimization,
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
            Self::RemoveNoopProject => "RemoveNoopProjectRule",
            Self::EliminateAppendVertices => "EliminateAppendVerticesRule",
            Self::RemoveAppendVerticesBelowJoin => "RemoveAppendVerticesBelowJoinRule",
            Self::TopN => "TopNRule",
            Self::MergeGetVerticesAndProject => "MergeGetVerticesAndProjectRule",
            Self::MergeGetVerticesAndDedup => "MergeGetVerticesAndDedupRule",
            Self::MergeGetNbrsAndProject => "MergeGetNbrsAndProjectRule",
            Self::MergeGetNbrsAndDedup => "MergeGetNbrsAndDedupRule",
            
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
            
            Self::RemoveUselessNode => "RemoveUselessNodeRule",
        }
    }
    
    pub fn create_instance(&self) -> Option<Box<dyn super::OptRule>> {
        match self {
            Self::FilterPushDown => Some(Box::new(super::FilterPushDownRule)),
            Self::PredicatePushDown => Some(Box::new(super::PredicatePushDownRule)),
            Self::ProjectionPushDown => Some(Box::new(super::ProjectionPushDownRule)),
            Self::CombineFilter => Some(Box::new(super::CombineFilterRule)),
            Self::CollapseProject => Some(Box::new(super::CollapseProjectRule)),
            Self::DedupElimination => Some(Box::new(super::DedupEliminationRule)),
            Self::EliminateFilter => Some(Box::new(super::EliminateFilterRule)),
            Self::RemoveNoopProject => Some(Box::new(super::RemoveNoopProjectRule)),
            Self::EliminateAppendVertices => Some(Box::new(super::EliminateAppendVerticesRule)),
            Self::RemoveAppendVerticesBelowJoin => Some(Box::new(super::RemoveAppendVerticesBelowJoinRule)),
            Self::TopN => Some(Box::new(super::TopNRule)),
            Self::MergeGetVerticesAndProject => Some(Box::new(super::MergeGetVerticesAndProjectRule)),
            Self::MergeGetVerticesAndDedup => Some(Box::new(super::MergeGetVerticesAndDedupRule)),
            Self::MergeGetNbrsAndProject => Some(Box::new(super::MergeGetNbrsAndProjectRule)),
            Self::MergeGetNbrsAndDedup => Some(Box::new(super::MergeGetNbrsAndDedupRule)),
            
            Self::JoinOptimization => Some(Box::new(super::JoinOptimizationRule)),
            Self::PushLimitDown => Some(Box::new(super::PushLimitDownRule)),
            Self::PushLimitDownGetVertices => Some(Box::new(super::PushLimitDownGetVerticesRule)),
            Self::PushLimitDownGetNeighbors => Some(Box::new(super::PushLimitDownGetNeighborsRule)),
            Self::PushLimitDownGetEdges => Some(Box::new(super::PushLimitDownGetEdgesRule)),
            Self::PushLimitDownScanVertices => Some(Box::new(super::PushLimitDownScanVerticesRule)),
            Self::PushLimitDownScanEdges => Some(Box::new(super::PushLimitDownScanEdgesRule)),
            Self::PushLimitDownIndexScan => Some(Box::new(super::PushLimitDownIndexScanRule)),
            Self::PushLimitDownProjectRule => Some(Box::new(super::PushLimitDownProjectRule)),
            Self::ScanWithFilterOptimization => Some(Box::new(super::ScanWithFilterOptimizationRule)),
            Self::IndexFullScan => Some(Box::new(super::IndexFullScanRule)),
            Self::IndexScan => Some(Box::new(super::IndexScanRule)),
            Self::EdgeIndexFullScan => Some(Box::new(super::EdgeIndexFullScanRule)),
            Self::TagIndexFullScan => Some(Box::new(super::TagIndexFullScanRule)),
            Self::UnionAllEdgeIndexScan => Some(Box::new(super::UnionAllEdgeIndexScanRule)),
            Self::UnionAllTagIndexScan => Some(Box::new(super::UnionAllTagIndexScanRule)),
            Self::OptimizeEdgeIndexScanByFilter => Some(Box::new(super::OptimizeEdgeIndexScanByFilterRule)),
            Self::OptimizeTagIndexScanByFilter => Some(Box::new(super::OptimizeTagIndexScanByFilterRule)),
            
            Self::RemoveUselessNode => None,
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
            "RemoveNoopProjectRule" => Some(Self::RemoveNoopProject),
            "EliminateAppendVerticesRule" => Some(Self::EliminateAppendVertices),
            "RemoveAppendVerticesBelowJoinRule" => Some(Self::RemoveAppendVerticesBelowJoin),
            "TopNRule" => Some(Self::TopN),
            "MergeGetVerticesAndProjectRule" => Some(Self::MergeGetVerticesAndProject),
            "MergeGetVerticesAndDedupRule" => Some(Self::MergeGetVerticesAndDedup),
            "MergeGetNbrsAndProjectRule" => Some(Self::MergeGetNbrsAndProject),
            "MergeGetNbrsAndDedupRule" => Some(Self::MergeGetNbrsAndDedup),
            
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
            
            "RemoveUselessNodeRule" => Some(Self::RemoveUselessNode),
            
            _ => None,
        }
    }
}
