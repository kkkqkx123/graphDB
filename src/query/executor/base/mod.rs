//! Unified Module for Basic Types of Actuators
//!
//! This module defines all the basic types related to actuators in a centralized manner, eliminating duplicate definitions and ensuring type consistency.
//!
//! Module structure:
//! - executor_stats.rs    - 执行器统计信息
//! - execution_result.rs  - 执行结果类型
//! - execution_context.rs - 执行上下文
//! - executor_base.rs     - 基础执行器实现
//! - result_processor.rs  - 结果处理器
//! - config.rs            - 执行器配置结构体

pub mod config;
pub mod execution_context;
pub mod execution_result;
pub mod executor_base;
pub mod executor_stats;
pub mod result_processor;

pub use config::{
    AllPathsConfig, AppendVerticesConfig, BfsShortestConfig, ExecutorConfig, IndexScanConfig,
    JoinConfig, JoinConfigWithDesc, LoopConfig, MultiShortestPathConfig, PathConfig,
    PatternApplyConfig, RollupApplyConfig, ShortestPathConfig,
};
pub use execution_context::ExecutionContext;
pub use execution_result::{DBResult, ExecutionResult, IntoExecutionResult};
pub use executor_base::{
    BaseExecutor, ChainableExecutor, Executor, HasInput, HasStorage, InputExecutor, StartExecutor,
};
pub use executor_stats::ExecutorStats;
pub use result_processor::{BaseResultProcessor, ResultProcessor, ResultProcessorContext};

pub use crate::core::types::EdgeDirection;
