//! 分页操作执行器模块
//!
//! 包含分页相关的执行器，包括：
//! - Limit（限制和偏移）

pub mod limit;

pub use limit::LimitExecutor;