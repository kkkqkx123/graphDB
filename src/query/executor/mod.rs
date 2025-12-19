// Re-export all executor modules
pub mod base;
pub mod data_access;
pub mod data_modification;
pub mod data_processing;
pub mod result_processing;
pub mod tag_filter;
pub mod traits;
pub mod factory;

// Cypher执行器模块
pub mod cypher;

// Re-export the new trait types
pub use traits::{
    BaseExecutor as NewBaseExecutor, ExecutionResult, Executor, ExecutorCore, ExecutorLifecycle,
    ExecutorMetadata,
};

// Re-export the base types
pub use base::{
    BaseExecutor, ChainableExecutor, EdgeDirection, ExecutionContext, InputExecutor, StartExecutor,
};

// Re-export factory types
pub use factory::{BaseExecutorFactory, ExecutorCreator};

// Re-export result processing executors
pub use result_processing::{
    ProjectExecutor, SortExecutor, LimitExecutor, AggregateExecutor, GroupByExecutor, HavingExecutor,
    DedupExecutor, FilterExecutor, SampleExecutor, TopNExecutor,
    SortKey, SortOrder, AggregateFunction, AggregateState, GroupAggregateState, DedupStrategy, SampleMethod,
    ResultProcessor, ResultProcessorContext,
};

pub use result_processing::traits::ResultProcessorFactory;

// Re-export Cypher executor types
pub use cypher::{
    CypherExecutor, CypherExecutionContext, CypherExecutorFactory, CypherExecutorTrait,
};

// Re-export Cypher clause executors
pub use cypher::clauses::MatchClauseExecutor;
