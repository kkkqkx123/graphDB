//! 优化规则枚举定义
//! 使用枚举替代字符串匹配规则，提高类型安全性和可维护性

use std::rc::Rc;

use crate::query::optimizer::core::OptimizationPhase;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationRule {
    // 逻辑优化规则
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
    PushFilterDownNode,
    PushEFilterDown,
    PushVFilterDownScanVertices,
    PushFilterDownInnerJoin,
    PushFilterDownHashInnerJoin,
    PushFilterDownHashLeftJoin,
    PushFilterDownCrossJoin,
    PushFilterDownGetNbrs,
    PushFilterDownExpandAll,
    PushFilterDownAllPaths,
    EliminateEmptySetOperation,
    OptimizeSetOperationInputOrder,

    // 物理优化规则
    JoinOptimization,
    PushLimitDownGetVertices,
    PushLimitDownGetEdges,
    PushLimitDownScanVertices,
    PushLimitDownScanEdges,
    PushLimitDownIndexScan,
    ScanWithFilterOptimization,
    IndexFullScan,
    IndexScan,
    EdgeIndexFullScan,
    TagIndexFullScan,
    UnionAllEdgeIndexScan,
    UnionAllTagIndexScan,

    // 索引覆盖扫描和TopN下推规则
    IndexCoveringScan,
    PushTopNDownIndexScan,
}

impl OptimizationRule {
    pub fn phase(&self) -> OptimizationPhase {
        match self {
            Self::ProjectionPushDown |
            Self::CombineFilter | Self::CollapseProject | Self::DedupElimination |
            Self::EliminateFilter | Self::EliminateRowCollect | Self::RemoveNoopProject |
            Self::EliminateAppendVertices | Self::RemoveAppendVerticesBelowJoin |
            Self::PushFilterDownAggregate | Self::TopN |
            Self::MergeGetVerticesAndProject | Self::MergeGetVerticesAndDedup |
            Self::MergeGetNbrsAndProject | Self::MergeGetNbrsAndDedup |
            Self::PushFilterDownNode | Self::PushEFilterDown | Self::PushVFilterDownScanVertices |
            Self::PushFilterDownInnerJoin | Self::PushFilterDownHashInnerJoin |
            Self::PushFilterDownHashLeftJoin | Self::PushFilterDownCrossJoin |
            Self::PushFilterDownGetNbrs | Self::PushFilterDownExpandAll |
            Self::PushFilterDownAllPaths | Self::EliminateEmptySetOperation |
            Self::OptimizeSetOperationInputOrder => OptimizationPhase::Logical,

            Self::JoinOptimization | Self::PushLimitDownGetVertices |
            Self::PushLimitDownGetEdges |
            Self::PushLimitDownScanVertices | Self::PushLimitDownScanEdges |
            Self::PushLimitDownIndexScan |
            Self::ScanWithFilterOptimization | Self::IndexFullScan | Self::IndexScan |
            Self::EdgeIndexFullScan | Self::TagIndexFullScan | Self::UnionAllEdgeIndexScan |
            Self::UnionAllTagIndexScan | Self::IndexCoveringScan | Self::PushTopNDownIndexScan => OptimizationPhase::Physical,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
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
            Self::PushFilterDownNode => "PushFilterDownNodeRule",
            Self::PushEFilterDown => "PushEFilterDownRule",
            Self::PushVFilterDownScanVertices => "PushVFilterDownScanVerticesRule",
            Self::PushFilterDownInnerJoin => "PushFilterDownInnerJoinRule",
            Self::PushFilterDownHashInnerJoin => "PushFilterDownHashInnerJoinRule",
            Self::PushFilterDownHashLeftJoin => "PushFilterDownHashLeftJoinRule",
            Self::PushFilterDownCrossJoin => "PushFilterDownCrossJoinRule",
            Self::PushFilterDownGetNbrs => "PushFilterDownGetNbrsRule",
            Self::PushFilterDownExpandAll => "PushFilterDownExpandAllRule",
            Self::PushFilterDownAllPaths => "PushFilterDownAllPathsRule",
            Self::EliminateEmptySetOperation => "EliminateEmptySetOperationRule",
            Self::OptimizeSetOperationInputOrder => "OptimizeSetOperationInputOrderRule",
            Self::JoinOptimization => "JoinOptimizationRule",
            Self::PushLimitDownGetVertices => "PushLimitDownGetVerticesRule",
            Self::PushLimitDownGetEdges => "PushLimitDownGetEdgesRule",
            Self::PushLimitDownScanVertices => "PushLimitDownScanVerticesRule",
            Self::PushLimitDownScanEdges => "PushLimitDownScanEdgesRule",
            Self::PushLimitDownIndexScan => "PushLimitDownIndexScanRule",
            Self::ScanWithFilterOptimization => "ScanWithFilterOptimizationRule",
            Self::IndexFullScan => "IndexFullScanRule",
            Self::IndexScan => "IndexScanRule",
            Self::EdgeIndexFullScan => "EdgeIndexFullScanRule",
            Self::TagIndexFullScan => "TagIndexFullScanRule",
            Self::UnionAllEdgeIndexScan => "UnionAllEdgeIndexScanRule",
            Self::UnionAllTagIndexScan => "UnionAllTagIndexScanRule",
            Self::IndexCoveringScan => "IndexCoveringScanRule",
            Self::PushTopNDownIndexScan => "PushTopNDownIndexScanRule",
        }
    }
    
    pub fn create_instance(&self) -> Option<Rc<dyn super::OptRule>> {
        match self {
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
            Self::PushFilterDownNode => Some(Rc::new(super::PushFilterDownNodeRule)),
            Self::PushEFilterDown => Some(Rc::new(super::PushEFilterDownRule)),
            Self::PushVFilterDownScanVertices => Some(Rc::new(super::PushVFilterDownScanVerticesRule)),
            Self::PushFilterDownInnerJoin => Some(Rc::new(super::PushFilterDownInnerJoinRule)),
            Self::PushFilterDownHashInnerJoin => Some(Rc::new(super::PushFilterDownHashInnerJoinRule)),
            Self::PushFilterDownHashLeftJoin => Some(Rc::new(super::PushFilterDownHashLeftJoinRule)),
            Self::PushFilterDownCrossJoin => Some(Rc::new(super::PushFilterDownCrossJoinRule)),
            Self::PushFilterDownGetNbrs => Some(Rc::new(super::PushFilterDownGetNbrsRule)),
            Self::PushFilterDownExpandAll => Some(Rc::new(super::PushFilterDownExpandAllRule)),
            Self::PushFilterDownAllPaths => Some(Rc::new(super::PushFilterDownAllPathsRule)),
            Self::EliminateEmptySetOperation => Some(Rc::new(super::EliminateEmptySetOperationRule)),
            Self::OptimizeSetOperationInputOrder => Some(Rc::new(super::OptimizeSetOperationInputOrderRule)),

            Self::JoinOptimization => Some(Rc::new(super::JoinOptimizationRule)),
            Self::PushLimitDownGetVertices => Some(Rc::new(super::PushLimitDownGetVerticesRule)),
            Self::PushLimitDownGetEdges => Some(Rc::new(super::PushLimitDownGetEdgesRule)),
            Self::PushLimitDownScanVertices => Some(Rc::new(super::PushLimitDownScanVerticesRule)),
            Self::PushLimitDownScanEdges => Some(Rc::new(super::PushLimitDownScanEdgesRule)),
            Self::PushLimitDownIndexScan => Some(Rc::new(super::PushLimitDownIndexScanRule)),
            Self::ScanWithFilterOptimization => Some(Rc::new(super::ScanWithFilterOptimizationRule)),
            Self::IndexFullScan => Some(Rc::new(super::IndexFullScanRule)),
            Self::IndexScan => Some(Rc::new(super::IndexScanRule)),
            Self::EdgeIndexFullScan => Some(Rc::new(super::EdgeIndexFullScanRule)),
            Self::TagIndexFullScan => Some(Rc::new(super::TagIndexFullScanRule)),
            Self::UnionAllEdgeIndexScan => Some(Rc::new(super::UnionAllEdgeIndexScanRule)),
            Self::UnionAllTagIndexScan => Some(Rc::new(super::UnionAllTagIndexScanRule)),
            Self::IndexCoveringScan => Some(Rc::new(super::IndexCoveringScanRule)),
            Self::PushTopNDownIndexScan => Some(Rc::new(super::PushTopNDownIndexScanRule)),
        }
    }
    
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "ProjectionPushDownRule" => Some(Self::ProjectionPushDown),
            "CombineFilterRule" => Some(Self::CombineFilter),
            "CollapseProjectRule" => Some(Self::CollapseProject),
            "DedupEliminationRule" => Some(Self::DedupElimination),
            "EliminateFilterRule" => Some(Self::EliminateFilter),
            "EliminateRowCollectRule" => Some(Self::EliminateRowCollect),
            "RemoveNoopProjectRule" => Some(Self::RemoveNoopProject),
            "EliminateAppendVerticesRule" => Some(Self::EliminateAppendVertices),
            "RemoveAppendVerticesBelowJoinRule" => Some(Self::RemoveAppendVerticesBelowJoin),
            "PushFilterDownAggregateRule" => Some(Self::PushFilterDownAggregate),
            "TopNRule" => Some(Self::TopN),
            "MergeGetVerticesAndProjectRule" => Some(Self::MergeGetVerticesAndProject),
            "MergeGetVerticesAndDedupRule" => Some(Self::MergeGetVerticesAndDedup),
            "MergeGetNbrsAndProjectRule" => Some(Self::MergeGetNbrsAndProject),
            "MergeGetNbrsAndDedupRule" => Some(Self::MergeGetNbrsAndDedup),
            "PushFilterDownNodeRule" => Some(Self::PushFilterDownNode),
            "PushEFilterDownRule" => Some(Self::PushEFilterDown),
            "PushVFilterDownScanVerticesRule" => Some(Self::PushVFilterDownScanVertices),
            "PushFilterDownInnerJoinRule" => Some(Self::PushFilterDownInnerJoin),
            "PushFilterDownHashInnerJoinRule" => Some(Self::PushFilterDownHashInnerJoin),
            "PushFilterDownHashLeftJoinRule" => Some(Self::PushFilterDownHashLeftJoin),
            "PushFilterDownCrossJoinRule" => Some(Self::PushFilterDownCrossJoin),
            "PushFilterDownGetNbrsRule" => Some(Self::PushFilterDownGetNbrs),
            "PushFilterDownExpandAllRule" => Some(Self::PushFilterDownExpandAll),
            "PushFilterDownAllPathsRule" => Some(Self::PushFilterDownAllPaths),
            "EliminateEmptySetOperationRule" => Some(Self::EliminateEmptySetOperation),
            "OptimizeSetOperationInputOrderRule" => Some(Self::OptimizeSetOperationInputOrder),

            "JoinOptimizationRule" => Some(Self::JoinOptimization),
            "PushLimitDownGetVerticesRule" => Some(Self::PushLimitDownGetVertices),
            "PushLimitDownGetEdgesRule" => Some(Self::PushLimitDownGetEdges),
            "PushLimitDownScanVerticesRule" => Some(Self::PushLimitDownScanVertices),
            "PushLimitDownScanEdgesRule" => Some(Self::PushLimitDownScanEdges),
            "PushLimitDownIndexScanRule" => Some(Self::PushLimitDownIndexScan),
            "ScanWithFilterOptimizationRule" => Some(Self::ScanWithFilterOptimization),
            "IndexFullScanRule" => Some(Self::IndexFullScan),
            "IndexScanRule" => Some(Self::IndexScan),
            "EdgeIndexFullScanRule" => Some(Self::EdgeIndexFullScan),
            "TagIndexFullScanRule" => Some(Self::TagIndexFullScan),
            "UnionAllEdgeIndexScanRule" => Some(Self::UnionAllEdgeIndexScan),
            "UnionAllTagIndexScanRule" => Some(Self::UnionAllTagIndexScan),
            "IndexCoveringScanRule" => Some(Self::IndexCoveringScan),
            "PushTopNDownIndexScanRule" => Some(Self::PushTopNDownIndexScan),
            _ => None,
        }
    }
}
