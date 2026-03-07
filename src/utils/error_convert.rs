//! 错误类型转换工具
//!
//! 提供 TransactionError 和 StorageError 之间的转换功能

use crate::core::StorageError;
use crate::transaction::TransactionError;

/// 错误转换 trait
///
/// 提供统一的错误转换接口
pub trait ErrorConvert<T> {
    /// 将错误转换为目标类型
    fn convert_error(self) -> T;
}

impl ErrorConvert<StorageError> for TransactionError {
    fn convert_error(self) -> StorageError {
        match self {
            TransactionError::BeginFailed(msg) => StorageError::DbError(format!("事务开始失败: {}", msg)),
            TransactionError::CommitFailed(msg) => StorageError::DbError(format!("事务提交失败: {}", msg)),
            TransactionError::AbortFailed(msg) => StorageError::DbError(format!("事务中止失败: {}", msg)),
            TransactionError::TransactionNotFound(id) => {
                StorageError::DbError(format!("事务未找到: {}", id))
            }
            TransactionError::TransactionNotPrepared(id) => {
                StorageError::DbError(format!("事务未准备: {}", id))
            }
            TransactionError::InvalidStateTransition { from, to } => {
                StorageError::DbError(format!("无效的状态转换: 从 {} 到 {}", from, to))
            }
            TransactionError::InvalidStateForCommit(state) => {
                StorageError::DbError(format!("无效的状态用于提交: {}", state))
            }
            TransactionError::InvalidStateForAbort(state) => {
                StorageError::DbError(format!("无效的状态用于中止: {}", state))
            }
            TransactionError::TransactionTimeout => StorageError::LockTimeout("事务超时".to_string()),
            TransactionError::TransactionExpired => StorageError::LockTimeout("事务已过期".to_string()),
            TransactionError::SavepointFailed(msg) => {
                StorageError::DbError(format!("保存点创建失败: {}", msg))
            }
            TransactionError::SavepointNotFound(id) => {
                StorageError::DbError(format!("保存点未找到: {}", id))
            }
            TransactionError::SavepointNotActive(id) => {
                StorageError::DbError(format!("保存点未激活: {}", id))
            }
            TransactionError::NoSavepointsInTransaction => {
                StorageError::DbError("事务中无保存点".to_string())
            }
            TransactionError::TwoPhaseNotFound(id) => {
                StorageError::DbError(format!("2PC事务未找到: {}", id))
            }
            TransactionError::RollbackFailed(msg) => {
                StorageError::DbError(format!("回滚失败: {}", msg))
            }
            TransactionError::TooManyTransactions => {
                StorageError::DbError("并发事务数过多".to_string())
            }
            TransactionError::WriteTransactionConflict => {
                StorageError::DbError("写事务冲突，已有活跃的写事务".to_string())
            }
            TransactionError::ReadOnlyTransaction => {
                StorageError::DbError("只读事务不允许写操作".to_string())
            }
            TransactionError::RecoveryFailed(msg) => {
                StorageError::DbError(format!("恢复失败: {}", msg))
            }
            TransactionError::PersistenceFailed(msg) => {
                StorageError::DbError(format!("持久化失败: {}", msg))
            }
            TransactionError::SerializationFailed(msg) => {
                StorageError::DbError(format!("序列化失败: {}", msg))
            }
            TransactionError::Internal(msg) => StorageError::DbError(format!("内部错误: {}", msg)),
        }
    }
}

/// 错误转换扩展 trait
///
/// 为 Result 类型提供便捷的错误转换方法
pub trait ResultErrorConvert<T, E> {
    /// 将 Result 中的错误转换为目标类型
    fn map_storage_error(self) -> Result<T, StorageError>;
}

impl<T> ResultErrorConvert<T, TransactionError> for Result<T, TransactionError> {
    fn map_storage_error(self) -> Result<T, StorageError> {
        self.map_err(|e| e.convert_error())
    }
}

/// 错误上下文扩展
///
/// 为错误添加上下文信息
pub trait ErrorContext<T> {
    /// 为错误添加上下文信息
    fn with_context<F>(self, f: F) -> T
    where
        F: FnOnce() -> String;
}

impl<T> ErrorContext<Result<T, StorageError>> for Result<T, StorageError> {
    fn with_context<F>(self, f: F) -> Result<T, StorageError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| StorageError::DbError(format!("{}: {}", f(), e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_error_to_storage_error() {
        let txn_err = TransactionError::BeginFailed("测试错误".to_string());
        let storage_err: StorageError = txn_err.convert_error();

        match storage_err {
            StorageError::DbError(msg) => {
                assert!(msg.contains("事务开始失败"));
                assert!(msg.contains("测试错误"));
            }
            _ => panic!("期望 DbError"),
        }
    }

    #[test]
    fn test_result_map_storage_error() {
        let result: Result<i32, TransactionError> = Err(TransactionError::TransactionTimeout);
        let converted: Result<i32, StorageError> = result.map_storage_error();

        match converted {
            Err(StorageError::LockTimeout(msg)) => {
                assert!(msg.contains("事务超时"));
            }
            _ => panic!("期望 LockTimeout 错误"),
        }
    }

    #[test]
    fn test_error_context() {
        let result: Result<i32, StorageError> = Err(StorageError::DbError("原始错误".to_string()));
        let with_context = result.with_context(|| "操作失败".to_string());

        match with_context {
            Err(StorageError::DbError(msg)) => {
                assert!(msg.contains("操作失败"));
                assert!(msg.contains("原始错误"));
            }
            _ => panic!("期望 DbError"),
        }
    }
}
