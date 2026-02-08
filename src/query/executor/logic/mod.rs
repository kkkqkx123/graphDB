//! 逻辑控制执行器模块
//!
//! 包含所有与逻辑控制相关的执行器，包括：
//! - LoopExecutor（通用循环控制）
//! - WhileLoopExecutor（条件循环）
//! - ForLoopExecutor（计数循环）
//!
//! 对应 NebulaGraph 实现：
//! nebula-3.8.0/src/graph/executor/logic/LoopExecutor.cpp

pub mod loops;

pub use loops::{ForLoopExecutor, LoopExecutor, LoopState, SelectExecutor, WhileLoopExecutor};
