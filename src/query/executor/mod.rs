// Re-export all executor modules
pub mod base;
pub mod data_access;
pub mod data_modification;
pub mod data_processing;
pub mod result_processing;
pub mod traits;

// Re-export the new trait types
pub use traits::{
    BaseExecutor as NewBaseExecutor, ExecutionResult, Executor, ExecutorCore, ExecutorLifecycle,
    ExecutorMetadata,
};

// Re-export the base types
pub use base::{
    BaseExecutor, ChainableExecutor, EdgeDirection, ExecutionContext, InputExecutor, StartExecutor,
};
