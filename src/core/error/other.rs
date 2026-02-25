//! 其他错误类型
//!
//! 包含内容较少的错误类型，保持架构对称性

use thiserror::Error;

/// 计划节点访问错误类型
///
/// 涵盖查询计划遍历和验证过程中的错误
#[derive(Error, Debug, Clone)]
pub enum PlanNodeVisitError {
    #[error("访问错误: {0}")]
    VisitError(String),
    #[error("遍历错误: {0}")]
    TraversalError(String),
    #[error("验证错误: {0}")]
    ValidationError(String),
}

/// 锁操作错误类型
///
/// 涵盖并发控制中锁相关的错误
#[derive(Error, Debug, Clone)]
pub enum LockError {
    #[error("Mutex锁被污染: {reason}")]
    MutexPoisoned { reason: String },

    #[error("RwLock读锁被污染: {reason}")]
    RwLockReadPoisoned { reason: String },

    #[error("RwLock写锁被污染: {reason}")]
    RwLockWritePoisoned { reason: String },

    #[error("锁操作超时: {reason}")]
    LockTimeout { reason: String },
}
