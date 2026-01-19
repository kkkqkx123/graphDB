// Re-export all executor modules
pub mod aggregation;
pub mod base;
pub mod data_access;
pub mod data_modification;
pub mod data_processing;
pub mod factory;
pub mod graph_query_executor;
pub mod object_pool;
pub mod recursion_detector;
pub mod result_processing;
pub mod tag_filter;
pub mod traits;

// Re-export the new trait types
pub use traits::{
    BaseExecutor as NewBaseExecutor, Executor, HasInput, HasStorage, ExecutorStats, ExecutionResult,
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

// Re-export graph query executor
pub use graph_query_executor::GraphQueryExecutor;
