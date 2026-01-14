// Re-export all executor modules
pub mod aggregation;
pub mod base;
pub mod data_access;
pub mod data_modification;
pub mod data_processing;
pub mod factory;
pub mod memory_manager;
pub mod object_pool;
pub mod recursion_detector;
pub mod result_processing;
pub mod tag_filter;
pub mod traits;

// Cypher执行器模块
pub mod cypher;

// Re-export the new trait types
pub use traits::{
    BaseExecutor as NewBaseExecutor, ExecutionResult, Executor, HasInput, HasStorage, ExecutorStats,
};

// Re-export the base types
pub use base::{BaseExecutor, ChainableExecutor, ExecutionContext, InputExecutor, StartExecutor};

// Re-export result processing executors
pub use result_processing::{
    AggregateExecutor, AggregateFunction, AggregateState, DedupExecutor, DedupStrategy,
    FilterExecutor, GroupAggregateState, GroupByExecutor, HavingExecutor, LimitExecutor,
    ProjectExecutor, ResultProcessor, ResultProcessorContext, SampleExecutor, SampleMethod,
    SortExecutor, SortKey, SortOrder, TopNExecutor,
};

pub use result_processing::traits::ResultProcessorFactory;

// Re-export Cypher executor types
pub use cypher::{
    CypherExecutionContext, CypherExecutor, CypherExecutorFactory, CypherExecutorTrait,
};

// Re-export Cypher clause executors
pub use cypher::clauses::MatchClauseExecutor;
