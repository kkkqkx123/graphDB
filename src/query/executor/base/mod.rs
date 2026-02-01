//! 执行器基础类型统一模块
//!
//! 本模块集中定义所有执行器相关的基础类型，消除重复定义，确保类型一致性。
//!
//! 模块结构：
//! - executor_stats.rs    - 执行器统计信息
//! - execution_result.rs  - 执行结果类型
//! - execution_context.rs - 执行上下文
//! - executor_base.rs     - 基础执行器实现

pub mod executor_stats;
pub mod execution_result;
pub mod execution_context;
pub mod executor_base;
pub mod storage_processor_executor;

pub use executor_stats::ExecutorStats;
pub use execution_result::{ExecutionResult, DBResult};
pub use execution_context::ExecutionContext;
pub use executor_base::{
    BaseExecutor, ChainableExecutor, Executor, HasStorage, HasInput, InputExecutor, StartExecutor
};
pub use storage_processor_executor::{StorageProcessorExecutor, StorageProcessorExecutorImpl, ProcessorExecutorCounters};

pub use crate::core::types::EdgeDirection;
