//! 执行器执行模块
//!
//! 负责执行执行计划，管理执行器树的生命周期

pub mod plan_executor;

pub use plan_executor::PlanExecutor;
