// Re-export all executor modules
pub mod base;
pub mod data_access;
pub mod data_modification;
pub mod data_processing;
pub mod factory;
pub mod result_processing;
pub mod tag_filter;
pub mod traits;

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
