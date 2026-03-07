//! 操作日志回滚处理器
//!
//! 提供基于操作日志的回滚功能

use crate::core::StorageError;
use crate::transaction::OperationLog;

/// 操作日志上下文 trait
///
/// 定义操作日志回滚所需的基本操作
pub trait OperationLogContext {
    /// 获取操作日志长度
    fn operation_log_len(&self) -> usize;
    /// 截断操作日志到指定索引
    fn truncate_operation_log(&self, index: usize);
    /// 获取指定索引的操作日志
    fn get_operation_log(&self, index: usize) -> Option<OperationLog>;
    /// 获取指定范围的操作日志
    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog>;
    /// 清空操作日志
    fn clear_operation_log(&self);
}

impl OperationLogContext for crate::transaction::context::TransactionContext {
    fn operation_log_len(&self) -> usize {
        self.operation_log_len()
    }

    fn truncate_operation_log(&self, index: usize) {
        self.truncate_operation_log(index);
    }

    fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
        self.get_operation_log(index)
    }

    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
        self.get_operation_logs(start, end)
    }

    fn clear_operation_log(&self) {
        self.clear_operation_log();
    }
}

/// 回滚执行器 trait
///
/// 定义如何执行单个操作的逆操作
pub trait RollbackExecutor {
    /// 执行单个操作日志的逆操作
    ///
    /// # Arguments
    /// * `log` - 要回滚的操作日志
    ///
    /// # Returns
    /// * `Ok(())` - 回滚成功
    /// * `Err(StorageError)` - 回滚失败
    fn execute_rollback(&self, log: &OperationLog) -> Result<(), StorageError>;
}

/// 操作日志回滚处理器
///
/// 负责执行基于操作日志的回滚操作
pub struct OperationLogRollback<'a, T: OperationLogContext> {
    ctx: &'a T,
}

impl<'a, T: OperationLogContext> OperationLogRollback<'a, T> {
    /// 创建新的回滚处理器
    pub fn new(ctx: &'a T) -> Self {
        Self { ctx }
    }

    /// 回滚到指定的操作日志索引
    ///
    /// 此方法仅截断操作日志，不执行实际的逆操作。
    /// 实际的回滚需要配合 RollbackExecutor 使用 execute_rollback_with_executor 方法。
    ///
    /// # Arguments
    /// * `index` - 要回滚到的操作日志索引
    ///
    /// # Returns
    /// * `Ok(())` - 回滚成功
    /// * `Err(StorageError)` - 回滚失败
    pub fn rollback_to_index(&self, index: usize) -> Result<(), StorageError> {
        let current_len = self.ctx.operation_log_len();

        if index > current_len {
            return Err(StorageError::DbError(format!(
                "无效的回滚索引: {}, 操作日志长度: {}",
                index, current_len
            )));
        }

        self.ctx.truncate_operation_log(index);

        Ok(())
    }

    /// 使用执行器回滚到指定的操作日志索引
    ///
    /// 此方法会逆序执行从目标索引到当前索引之间的所有操作的逆操作，
    /// 然后截断操作日志。
    ///
    /// # Arguments
    /// * `index` - 要回滚到的操作日志索引
    /// * `executor` - 回滚执行器，用于执行逆操作
    ///
    /// # Returns
    /// * `Ok(())` - 回滚成功
    /// * `Err(StorageError)` - 回滚失败
    pub fn execute_rollback_to_index<E: RollbackExecutor>(
        &self,
        index: usize,
        executor: &E,
    ) -> Result<(), StorageError> {
        let current_len = self.ctx.operation_log_len();

        if index > current_len {
            return Err(StorageError::DbError(format!(
                "无效的回滚索引: {}, 操作日志长度: {}",
                index, current_len
            )));
        }

        // 获取需要回滚的操作（从后往前）
        let logs_to_rollback = self.ctx.get_operation_logs(index, current_len);

        // 逆序执行逆操作
        for log in logs_to_rollback.iter().rev() {
            executor.execute_rollback(log)?;
        }

        // 截断日志
        self.ctx.truncate_operation_log(index);

        Ok(())
    }

    /// 获取操作日志长度
    pub fn operation_log_len(&self) -> usize {
        self.ctx.operation_log_len()
    }

    /// 获取所有操作日志
    pub fn get_all_logs(&self) -> Vec<OperationLog> {
        let len = self.ctx.operation_log_len();
        self.ctx.get_operation_logs(0, len)
    }

    /// 清空所有操作日志
    pub fn clear_logs(&self) {
        self.ctx.clear_operation_log();
    }
}

/// 标准回滚执行器
///
/// 基于 Redb 存储的标准回滚执行器实现
pub struct StandardRollbackExecutor<'a> {
    write_txn: &'a redb::WriteTransaction,
}

impl<'a> StandardRollbackExecutor<'a> {
    /// 创建新的标准回滚执行器
    pub fn new(write_txn: &'a redb::WriteTransaction) -> Self {
        Self { write_txn }
    }
}

impl<'a> RollbackExecutor for StandardRollbackExecutor<'a> {
    fn execute_rollback(&self, log: &OperationLog) -> Result<(), StorageError> {
        use crate::storage::redb_types::{ByteKey, EDGES_TABLE, NODES_TABLE};

        match log {
            OperationLog::InsertVertex {
                vertex_id,
                previous_state: _,
                ..
            } => {
                // 逆操作：删除插入的顶点
                let mut table = self
                    .write_txn
                    .open_table(NODES_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                table
                    .remove(ByteKey(vertex_id.clone()))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
            OperationLog::UpdateVertex {
                vertex_id,
                previous_data,
                ..
            } => {
                // 逆操作：恢复更新前的数据
                let mut table = self
                    .write_txn
                    .open_table(NODES_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                table
                    .insert(ByteKey(vertex_id.clone()), ByteKey(previous_data.clone()))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
            OperationLog::DeleteVertex {
                vertex_id,
                deleted_data,
                ..
            } => {
                // 逆操作：恢复被删除的顶点
                let mut table = self
                    .write_txn
                    .open_table(NODES_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                table
                    .insert(ByteKey(vertex_id.clone()), ByteKey(deleted_data.clone()))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
            OperationLog::InsertEdge {
                edge_key,
                previous_state: _,
                ..
            } => {
                // 逆操作：删除插入的边
                let mut table = self
                    .write_txn
                    .open_table(EDGES_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                table
                    .remove(ByteKey(edge_key.clone()))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
            OperationLog::DeleteEdge {
                edge_key,
                deleted_data,
                ..
            } => {
                // 逆操作：恢复被删除的边
                let mut table = self
                    .write_txn
                    .open_table(EDGES_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                table
                    .insert(ByteKey(edge_key.clone()), ByteKey(deleted_data.clone()))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
            OperationLog::UpdateIndex {
                space: _,
                index_name: _,
                key,
                previous_value,
            } => {
                // 逆操作：恢复更新前的索引值
                use crate::storage::redb_types::INDEX_DATA_TABLE;
                let mut table = self
                    .write_txn
                    .open_table(INDEX_DATA_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                match previous_value {
                    Some(value) => {
                        table
                            .insert(ByteKey(key.clone()), ByteKey(value.clone()))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                    }
                    None => {
                        table
                            .remove(ByteKey(key.clone()))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                    }
                }
            }
            OperationLog::DeleteIndex {
                space: _,
                index_name: _,
                key,
                deleted_value,
            } => {
                // 逆操作：恢复被删除的索引
                use crate::storage::redb_types::INDEX_DATA_TABLE;
                let mut table = self
                    .write_txn
                    .open_table(INDEX_DATA_TABLE)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                table
                    .insert(ByteKey(key.clone()), ByteKey(deleted_value.clone()))
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockContext {
        logs: RefCell<Vec<OperationLog>>,
    }

    impl MockContext {
        fn new() -> Self {
            Self {
                logs: RefCell::new(Vec::new()),
            }
        }

        fn add_log(&self, log: OperationLog) {
            self.logs.borrow_mut().push(log);
        }
    }

    impl OperationLogContext for MockContext {
        fn operation_log_len(&self) -> usize {
            self.logs.borrow().len()
        }

        fn truncate_operation_log(&self, index: usize) {
            self.logs.borrow_mut().truncate(index);
        }

        fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
            self.logs.borrow().get(index).cloned()
        }

        fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
            let logs = self.logs.borrow();
            if start >= logs.len() {
                return Vec::new();
            }
            let end = end.min(logs.len());
            logs[start..end].to_vec()
        }

        fn clear_operation_log(&self) {
            self.logs.borrow_mut().clear();
        }
    }

    struct MockExecutor;

    impl RollbackExecutor for MockExecutor {
        fn execute_rollback(&self, _log: &OperationLog) -> Result<(), StorageError> {
            Ok(())
        }
    }

    #[test]
    fn test_rollback_to_index() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);

        ctx.add_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_state: None,
        });

        ctx.add_log(OperationLog::UpdateVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_data: vec![4, 5, 6],
        });

        assert_eq!(rollback.operation_log_len(), 2);

        let result = rollback.rollback_to_index(1);
        assert!(result.is_ok());
        assert_eq!(rollback.operation_log_len(), 1);
    }

    #[test]
    fn test_execute_rollback_with_executor() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);
        let executor = MockExecutor;

        ctx.add_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_state: None,
        });

        ctx.add_log(OperationLog::UpdateVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_data: vec![4, 5, 6],
        });

        let result = rollback.execute_rollback_to_index(0, &executor);
        assert!(result.is_ok());
        assert_eq!(rollback.operation_log_len(), 0);
    }

    #[test]
    fn test_rollback_invalid_index() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);

        let result = rollback.rollback_to_index(100);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_logs() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);

        ctx.add_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![1],
            previous_state: None,
        });

        ctx.add_log(OperationLog::DeleteVertex {
            space: "test".to_string(),
            vertex_id: vec![2],
            deleted_data: vec![3],
        });

        let logs = rollback.get_all_logs();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_clear_logs() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);

        ctx.add_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![1],
            previous_state: None,
        });

        assert_eq!(rollback.operation_log_len(), 1);

        rollback.clear_logs();

        assert_eq!(rollback.operation_log_len(), 0);
    }
}
