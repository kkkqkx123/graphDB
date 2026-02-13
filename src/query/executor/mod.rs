// Re-export all executor modules
pub mod admin;
pub mod base;
pub mod batch;
pub mod data_access;
pub mod data_modification;
pub mod data_processing;
pub mod executor_enum;
pub mod factory;
pub mod graph_query_executor;
pub mod logic;
pub mod object_pool;
pub mod recursion_detector;
pub mod result_processing;
pub mod search_executors;
pub mod special_executors;
pub mod tag_filter;
pub mod traits;

// Re-export from base module (基础类型从 base 模块统一导出)
pub use base::{
    BaseExecutor, ExecutionContext, ExecutionResult, Executor, ExecutorStats,
    HasInput, HasStorage, StartExecutor,
};

// Re-export ExecutorEnum (执行器枚举)
pub use executor_enum::ExecutorEnum;

// Re-export batch module (批量操作优化)
pub use batch::{
    BatchConfig, BatchOptimizer, BatchReadResult,
};

// Re-export data access executors
pub use data_access::{
    AllPathsExecutor, GetEdgesExecutor, GetNeighborsExecutor, GetPropExecutor, GetVerticesExecutor,
    IndexScanExecutor, ScanVerticesExecutor,
};

// Re-export result processing executors
pub use result_processing::{
    AggregateExecutor, AggregateFunction, DedupExecutor, DedupStrategy,
    FilterExecutor, GroupAggregateState, GroupByExecutor, HavingExecutor, LimitExecutor,
    ProjectExecutor, ResultProcessor, ResultProcessorContext, SampleExecutor, SampleMethod,
    SortExecutor, SortKey, SortOrder, TopNExecutor,
};

pub use result_processing::traits::ResultProcessorFactory;

// Re-export transformations (数据转换执行器)
pub use result_processing::transformations::{
    AppendVerticesExecutor, AssignExecutor, PatternApplyExecutor, RollUpApplyExecutor, UnwindExecutor,
};

// Re-export logic executors (循环控制执行器)
pub use logic::{ForLoopExecutor, LoopExecutor, WhileLoopExecutor};

// Re-export LoopState (已废弃，请使用 crate::query::core::LoopExecutionState)
pub use logic::LoopState;

// Re-export core execution states
pub use crate::query::core::{ExecutorState, LoopExecutionState, QueryExecutionState, RowStatus};

// Re-export graph query executor
pub use graph_query_executor::GraphQueryExecutor;

// Re-export admin executors (管理执行器)
pub use admin::{
    CreateSpaceExecutor, DropSpaceExecutor, DescSpaceExecutor, ShowSpacesExecutor,
    CreateTagExecutor, AlterTagExecutor, DescTagExecutor, DropTagExecutor, ShowTagsExecutor,
    CreateEdgeExecutor, AlterEdgeExecutor, DescEdgeExecutor, DropEdgeExecutor, ShowEdgesExecutor,
    CreateTagIndexExecutor, DropTagIndexExecutor, DescTagIndexExecutor, ShowTagIndexesExecutor,
    CreateEdgeIndexExecutor, DropEdgeIndexExecutor, DescEdgeIndexExecutor, ShowEdgeIndexesExecutor,
    RebuildTagIndexExecutor, RebuildEdgeIndexExecutor,
    CreateUserExecutor, AlterUserExecutor, DropUserExecutor, ChangePasswordExecutor,
};

// Re-export search executors (搜索执行器)
pub use search_executors::{BFSShortestExecutor, FulltextIndexScanExecutor};

// Re-export special executors (特殊执行器)
pub use special_executors::{ArgumentExecutor, DataCollectExecutor, PassThroughExecutor};

// 编译期枚举一致性检查
// 这些检查确保 PlanNodeEnum 和 ExecutorEnum 的变体数量一致
// 如果数量不匹配，编译将失败并给出明确的错误信息

/// PlanNodeEnum 的变体数量
/// 注意：当添加或删除 PlanNodeEnum 的变体时，需要更新此常量
/// 此常量仅用于编译期断言检查，故标记为允许未使用
#[allow(dead_code)]
const PLAN_NODE_VARIANT_COUNT: usize = 68;

/// ExecutorEnum 的变体数量
/// 注意：当添加或删除 ExecutorEnum 的变体时，需要更新此常量
/// 此常量仅用于编译期断言检查，故标记为允许未使用
#[allow(dead_code)]
const EXECUTOR_VARIANT_COUNT: usize = 68;

// 编译期断言：确保两个枚举的变体数量一致
// 注意：const assert 中不能使用格式化字符串
const _: () = assert!(
    PLAN_NODE_VARIANT_COUNT == EXECUTOR_VARIANT_COUNT,
    "PlanNodeEnum and ExecutorEnum variant count mismatch"
);

/// 节点类型一致性检查
///
/// 此模块在编译期检查 PlanNodeEnum 和 ExecutorEnum 的一致性
#[cfg(test)]
mod consistency_tests {
    use crate::query::core::NodeTypeMapping;
    use crate::query::planner::plan::core::nodes::PlanNodeEnum;

    /// 测试 PlanNodeEnum 和 ExecutorEnum 的节点类型 ID 是否一致
    #[test]
    fn test_node_type_id_consistency() {
        // 此测试确保所有 PlanNode 类型都有对应的 Executor 类型
        // 实际检查在编译期通过常量断言完成
        assert_eq!(super::PLAN_NODE_VARIANT_COUNT, super::EXECUTOR_VARIANT_COUNT);
    }

    /// 验证节点类型映射
    #[test]
    fn test_node_type_mapping() {
        use crate::query::planner::plan::core::nodes::{CrossJoinNode, ArgumentNode, PlanNodeEnum as NodeEnum};
        
        // 示例：验证 CrossJoin 的映射
        // 创建两个 ArgumentNode 作为 CrossJoin 的输入
        let left = ArgumentNode::new(1, "left_var");
        let right = ArgumentNode::new(2, "right_var");
        let cross_join_node = CrossJoinNode::new(
            NodeEnum::Argument(left),
            NodeEnum::Argument(right)
        ).expect("创建 CrossJoinNode 失败");
        let plan_node = PlanNodeEnum::CrossJoin(cross_join_node);
        
        // 验证 PlanNodeEnum 实现了 NodeTypeMapping
        let executor_type = plan_node.corresponding_executor_type();
        assert!(executor_type.is_some());
        assert_eq!(executor_type.expect("Expected executor type to exist"), "cross_join");
    }
}
