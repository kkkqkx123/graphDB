//! PlanNode 类型到 CoreOperationKind 的转换支持
//!
//! 此模块提供 PlanNodeEnum 到 CoreOperationKind 的类型转换功能。

use crate::query::core::{CoreOperationKind, IntoOperationKind};
use crate::query::planner::plan::core::nodes::PlanNodeEnum;

impl IntoOperationKind for PlanNodeEnum {
    fn into_operation_kind(&self) -> CoreOperationKind {
        match self {
            PlanNodeEnum::Start(_) => CoreOperationKind::Project,
            PlanNodeEnum::Project(_) => CoreOperationKind::Project,
            PlanNodeEnum::Filter(_) => CoreOperationKind::Filter,
            PlanNodeEnum::Sort(_) => CoreOperationKind::Sort,
            PlanNodeEnum::Limit(_) => CoreOperationKind::Limit,
            PlanNodeEnum::TopN(_) => CoreOperationKind::TopN,
            PlanNodeEnum::Sample(_) => CoreOperationKind::Sample,
            PlanNodeEnum::Unwind(_) => CoreOperationKind::Unwind,
            PlanNodeEnum::Dedup(_) => CoreOperationKind::Dedup,
            PlanNodeEnum::Aggregate(_) => CoreOperationKind::Aggregate,
            PlanNodeEnum::InnerJoin(_) => CoreOperationKind::InnerJoin,
            PlanNodeEnum::LeftJoin(_) => CoreOperationKind::LeftJoin,
            PlanNodeEnum::CrossJoin(_) => CoreOperationKind::CrossJoin,
            PlanNodeEnum::HashInnerJoin(_) => CoreOperationKind::HashJoin,
            PlanNodeEnum::HashLeftJoin(_) => CoreOperationKind::HashJoin,
            PlanNodeEnum::Union(_) => CoreOperationKind::Union,
            PlanNodeEnum::ScanVertices(_) => CoreOperationKind::ScanVertices,
            PlanNodeEnum::ScanEdges(_) => CoreOperationKind::ScanEdges,
            PlanNodeEnum::GetVertices(_) => CoreOperationKind::GetVertices,
            PlanNodeEnum::GetEdges(_) => CoreOperationKind::GetEdges,
            PlanNodeEnum::GetNeighbors(_) => CoreOperationKind::GetNeighbors,
            PlanNodeEnum::IndexScan(_) => CoreOperationKind::IndexScan,
            PlanNodeEnum::FulltextIndexScan(_) => CoreOperationKind::FulltextIndexScan,
            PlanNodeEnum::Expand(_) => CoreOperationKind::Expand,
            PlanNodeEnum::ExpandAll(_) => CoreOperationKind::ExpandAll,
            PlanNodeEnum::Traverse(_) => CoreOperationKind::Traverse,
            PlanNodeEnum::AppendVertices(_) => CoreOperationKind::AppendVertices,
            PlanNodeEnum::ShortestPath(_) => CoreOperationKind::ShortestPath,
            PlanNodeEnum::MultiShortestPath(_) => CoreOperationKind::MultiShortestPath,
            PlanNodeEnum::AllPaths(_) => CoreOperationKind::AllPaths,
            PlanNodeEnum::BFSShortest(_) => CoreOperationKind::BFSShortest,
            PlanNodeEnum::Assign(_) => CoreOperationKind::Assign,
            PlanNodeEnum::PatternApply(_) => CoreOperationKind::PatternApply,
            PlanNodeEnum::RollUpApply(_) => CoreOperationKind::RollUpApply,
            PlanNodeEnum::Loop(_) => CoreOperationKind::Loop,
            PlanNodeEnum::Argument(_) => CoreOperationKind::Argument,
            PlanNodeEnum::PassThrough(_) => CoreOperationKind::PassThrough,
            PlanNodeEnum::Select(_) => CoreOperationKind::Select,
            PlanNodeEnum::DataCollect(_) => CoreOperationKind::DataCollect,
            PlanNodeEnum::CreateSpace(_) => CoreOperationKind::CreateSpace,
            PlanNodeEnum::DropSpace(_) => CoreOperationKind::DropSpace,
            PlanNodeEnum::DescSpace(_) => CoreOperationKind::DescribeSpace,
            PlanNodeEnum::ShowSpaces(_) => CoreOperationKind::ShowSpaces,
            PlanNodeEnum::CreateTag(_) => CoreOperationKind::CreateTag,
            PlanNodeEnum::AlterTag(_) => CoreOperationKind::AlterTag,
            PlanNodeEnum::DescTag(_) => CoreOperationKind::DescribeTag,
            PlanNodeEnum::DropTag(_) => CoreOperationKind::DropTag,
            PlanNodeEnum::ShowTags(_) => CoreOperationKind::ShowTags,
            PlanNodeEnum::CreateEdge(_) => CoreOperationKind::CreateEdge,
            PlanNodeEnum::AlterEdge(_) => CoreOperationKind::AlterEdge,
            PlanNodeEnum::DescEdge(_) => CoreOperationKind::DescribeEdge,
            PlanNodeEnum::DropEdge(_) => CoreOperationKind::DropEdge,
            PlanNodeEnum::ShowEdges(_) => CoreOperationKind::ShowEdges,
            PlanNodeEnum::CreateTagIndex(_) => CoreOperationKind::CreateIndex,
            PlanNodeEnum::DropTagIndex(_) => CoreOperationKind::DropIndex,
            PlanNodeEnum::DescTagIndex(_) => CoreOperationKind::DescribeIndex,
            PlanNodeEnum::ShowTagIndexes(_) => CoreOperationKind::Show,
            PlanNodeEnum::CreateEdgeIndex(_) => CoreOperationKind::CreateIndex,
            PlanNodeEnum::DropEdgeIndex(_) => CoreOperationKind::DropIndex,
            PlanNodeEnum::DescEdgeIndex(_) => CoreOperationKind::DescribeIndex,
            PlanNodeEnum::ShowEdgeIndexes(_) => CoreOperationKind::Show,
            PlanNodeEnum::RebuildTagIndex(_) => CoreOperationKind::RebuildIndex,
            PlanNodeEnum::RebuildEdgeIndex(_) => CoreOperationKind::RebuildIndex,
            PlanNodeEnum::CreateUser(_) => CoreOperationKind::CreateUser,
            PlanNodeEnum::AlterUser(_) => CoreOperationKind::AlterUser,
            PlanNodeEnum::DropUser(_) => CoreOperationKind::DropUser,
            PlanNodeEnum::ChangePassword(_) => CoreOperationKind::ChangePassword,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::{StartNode, FilterNode, ProjectNode, PlanNodeClonable};
    use crate::core::Expression;
    
    fn create_start_node() -> StartNode {
        StartNode::new()
    }
    
    #[test]
    fn test_plan_node_to_operation_kind() {
        let start_node = create_start_node();
        let start_enum = start_node.clone_plan_node();
        assert_eq!(start_enum.into_operation_kind(), CoreOperationKind::Project);
    }
    
    #[test]
    fn test_filter_node_operation_kind() {
        let input = create_start_node().clone_plan_node();
        let filter_node = FilterNode::new(input, Expression::bool(true)).expect("Failed to create FilterNode");
        let filter_enum = filter_node.clone_plan_node();
        assert_eq!(filter_enum.into_operation_kind(), CoreOperationKind::Filter);
    }
    
    #[test]
    fn test_project_node_operation_kind() {
        let input = create_start_node().clone_plan_node();
        let project_node = ProjectNode::new(input, vec![]).expect("Failed to create ProjectNode");
        let project_enum = project_node.clone_plan_node();
        assert_eq!(project_enum.into_operation_kind(), CoreOperationKind::Project);
    }
}
