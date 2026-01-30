// Re-export all executor modules
pub mod admin;
pub mod base;
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
pub mod operation_kind_support;

// Re-export from base module (统一的基础类型)
pub use base::{
    BaseExecutor, ChainableExecutor, ExecutionContext, ExecutionResult, Executor,
    ExecutorStats, HasInput, HasStorage, InputExecutor, StartExecutor,
};

// Re-export data access executors
pub use data_access::{
    AllPathsExecutor, GetEdgesExecutor, GetNeighborsExecutor, GetPropExecutor, GetVerticesExecutor,
    IndexScanExecutor, ScanVerticesExecutor,
};

// Re-export result processing executors
pub use result_processing::{
    AggregateExecutor, AggregateFunction, AggregateState, DedupExecutor, DedupStrategy,
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
pub use logic::{ForLoopExecutor, LoopExecutor, LoopState, WhileLoopExecutor};

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
    InsertVertexExecutor, InsertEdgeExecutor, DeleteExecutor, UpdateExecutor,
    ChangePasswordExecutor,
};

// Re-export search executors (搜索执行器)
pub use search_executors::{BFSShortestExecutor, FulltextIndexScanExecutor};

// Re-export special executors (特殊执行器)
pub use special_executors::{ArgumentExecutor, DataCollectExecutor, PassThroughExecutor};
