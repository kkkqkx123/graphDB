// Re-export all executor modules
pub mod traits;
pub mod base;
pub mod data_access;
pub mod data_processing;
pub mod data_modification;
pub mod result_processing;

// Re-export the new trait types
pub use traits::{
    ExecutorCore, ExecutorLifecycle, ExecutorMetadata, Executor,
    ExecutionResult, BaseExecutor as NewBaseExecutor
};

// Re-export the base types
pub use base::{
    ExecutionContext, BaseExecutor,
    InputExecutor, ChainableExecutor, EdgeDirection, StartExecutor
};