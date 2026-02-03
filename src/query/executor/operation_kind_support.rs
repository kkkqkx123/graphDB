//! Executor 类型到 CoreOperationKind 的转换支持
//!
//! 此模块提供 ExecutorEnum 到 CoreOperationKind 的类型转换功能。

use crate::query::core::{CoreOperationKind, IntoOperationKind};
use crate::storage::StorageClient;

use super::executor_enum::ExecutorEnum;

impl<S: StorageClient + Send + 'static> IntoOperationKind for ExecutorEnum<S> {
    fn into_operation_kind(&self) -> CoreOperationKind {
        match self {
            ExecutorEnum::Start(_) => CoreOperationKind::Project,
            ExecutorEnum::Base(_) => CoreOperationKind::Project,
            ExecutorEnum::GetVertices(_) => CoreOperationKind::GetVertices,
            ExecutorEnum::GetNeighbors(_) => CoreOperationKind::GetNeighbors,
            ExecutorEnum::GetProp(_) => CoreOperationKind::Project,
            ExecutorEnum::AllPaths(_) => CoreOperationKind::AllPaths,
            ExecutorEnum::Expand(_) => CoreOperationKind::Expand,
            ExecutorEnum::ExpandAll(_) => CoreOperationKind::ExpandAll,
            ExecutorEnum::Traverse(_) => CoreOperationKind::Traverse,
            ExecutorEnum::ShortestPath(_) => CoreOperationKind::ShortestPath,
            ExecutorEnum::MultiShortestPath(_) => CoreOperationKind::MultiShortestPath,
            ExecutorEnum::InnerJoin(_) => CoreOperationKind::InnerJoin,
            ExecutorEnum::HashInnerJoin(_) => CoreOperationKind::HashJoin,
            ExecutorEnum::LeftJoin(_) => CoreOperationKind::LeftJoin,
            ExecutorEnum::HashLeftJoin(_) => CoreOperationKind::HashJoin,
            ExecutorEnum::CrossJoin(_) => CoreOperationKind::CrossJoin,
            ExecutorEnum::Union(_) => CoreOperationKind::Union,
            ExecutorEnum::UnionAll(_) => CoreOperationKind::Union,
            ExecutorEnum::Minus(_) => CoreOperationKind::Minus,
            ExecutorEnum::Intersect(_) => CoreOperationKind::Intersect,
            ExecutorEnum::Filter(_) => CoreOperationKind::Filter,
            ExecutorEnum::Project(_) => CoreOperationKind::Project,
            ExecutorEnum::Limit(_) => CoreOperationKind::Limit,
            ExecutorEnum::Sort(_) => CoreOperationKind::Sort,
            ExecutorEnum::TopN(_) => CoreOperationKind::TopN,
            ExecutorEnum::Sample(_) => CoreOperationKind::Sample,
            ExecutorEnum::Aggregate(_) => CoreOperationKind::Aggregate,
            ExecutorEnum::GroupBy(_) => CoreOperationKind::Aggregate,
            ExecutorEnum::Having(_) => CoreOperationKind::Having,
            ExecutorEnum::Dedup(_) => CoreOperationKind::Dedup,
            ExecutorEnum::Unwind(_) => CoreOperationKind::Unwind,
            ExecutorEnum::Assign(_) => CoreOperationKind::Assign,
            ExecutorEnum::AppendVertices(_) => CoreOperationKind::AppendVertices,
            ExecutorEnum::RollUpApply(_) => CoreOperationKind::RollUpApply,
            ExecutorEnum::PatternApply(_) => CoreOperationKind::PatternApply,
            ExecutorEnum::Loop(_) => CoreOperationKind::Loop,
            ExecutorEnum::ForLoop(_) => CoreOperationKind::ForLoop,
            ExecutorEnum::WhileLoop(_) => CoreOperationKind::WhileLoop,
            ExecutorEnum::Select(_) => CoreOperationKind::Select,
            ExecutorEnum::ScanEdges(_) => CoreOperationKind::ScanEdges,
            ExecutorEnum::ScanVertices(_) => CoreOperationKind::ScanVertices,
            ExecutorEnum::IndexScan(_) => CoreOperationKind::IndexScan,
            ExecutorEnum::Argument(_) => CoreOperationKind::Argument,
            ExecutorEnum::PassThrough(_) => CoreOperationKind::PassThrough,
            ExecutorEnum::DataCollect(_) => CoreOperationKind::DataCollect,
            ExecutorEnum::FulltextIndexScan(_) => CoreOperationKind::FulltextIndexScan,
            ExecutorEnum::BFSShortest(_) => CoreOperationKind::BFSShortest,
            ExecutorEnum::ShowSpaces(_) => CoreOperationKind::ShowSpaces,
            ExecutorEnum::ShowTags(_) => CoreOperationKind::ShowTags,
            ExecutorEnum::ShowEdges(_) => CoreOperationKind::ShowEdges,
            ExecutorEnum::CreateTagIndex(_) => CoreOperationKind::CreateIndex,
            ExecutorEnum::DropTagIndex(_) => CoreOperationKind::DropIndex,
            ExecutorEnum::DescTagIndex(_) => CoreOperationKind::DescribeIndex,
            ExecutorEnum::ShowTagIndexes(_) => CoreOperationKind::Show,
            ExecutorEnum::RebuildTagIndex(_) => CoreOperationKind::RebuildIndex,
            ExecutorEnum::CreateEdgeIndex(_) => CoreOperationKind::CreateIndex,
            ExecutorEnum::DropEdgeIndex(_) => CoreOperationKind::DropIndex,
            ExecutorEnum::DescEdgeIndex(_) => CoreOperationKind::DescribeIndex,
            ExecutorEnum::ShowEdgeIndexes(_) => CoreOperationKind::Show,
            ExecutorEnum::RebuildEdgeIndex(_) => CoreOperationKind::RebuildIndex,
            ExecutorEnum::CreateSpace(_) => CoreOperationKind::CreateSpace,
            ExecutorEnum::DropSpace(_) => CoreOperationKind::DropSpace,
            ExecutorEnum::DescSpace(_) => CoreOperationKind::DescribeSpace,
            ExecutorEnum::CreateTag(_) => CoreOperationKind::CreateTag,
            ExecutorEnum::AlterTag(_) => CoreOperationKind::AlterTag,
            ExecutorEnum::DescTag(_) => CoreOperationKind::DescribeTag,
            ExecutorEnum::DropTag(_) => CoreOperationKind::DropTag,
            ExecutorEnum::CreateEdge(_) => CoreOperationKind::CreateEdge,
            ExecutorEnum::AlterEdge(_) => CoreOperationKind::AlterEdge,
            ExecutorEnum::DescEdge(_) => CoreOperationKind::DescribeEdge,
            ExecutorEnum::DropEdge(_) => CoreOperationKind::DropEdge,
            ExecutorEnum::InsertVertex(_) => CoreOperationKind::Insert,
            ExecutorEnum::InsertEdge(_) => CoreOperationKind::Insert,
            ExecutorEnum::Update(_) => CoreOperationKind::Update,
            ExecutorEnum::ChangePassword(_) => CoreOperationKind::ChangePassword,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::executor::StartExecutor;
    use std::sync::{Arc, Mutex};
    use crate::storage::test_mock::MockStorage;
    
    #[test]
    fn test_executor_enum_to_operation_kind() {
        let storage = Arc::new(Mutex::new(MockStorage::new().expect("Failed to create MockStorage")));
        
        let start_executor = StartExecutor::new(1);
        
        let executor_enum: ExecutorEnum<MockStorage> = ExecutorEnum::Start(start_executor);
        
        assert_eq!(executor_enum.into_operation_kind(), CoreOperationKind::Project);
    }
}
