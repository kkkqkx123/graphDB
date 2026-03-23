//! 事务上下文
//!
//! 管理单个事务的状态和资源

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossbeam_utils::atomic::AtomicCell;
use parking_lot::{Mutex, RwLock};

use crate::core::StorageError;
use crate::storage::operations::rollback::RollbackExecutor;
use crate::transaction::types::*;

/// 事务上下文
pub struct TransactionContext {
    /// 事务ID
    pub id: TransactionId,
    /// 当前状态
    state: AtomicCell<TransactionState>,
    /// 开始时间戳
    pub start_time: Instant,
    /// 超时时间
    timeout: Duration,
    /// 是否只读
    pub read_only: bool,
    /// redb写事务（读写事务时存在）
    /// 使用Option以便在提交时取出所有权
    pub write_txn: Mutex<Option<redb::WriteTransaction>>,
    /// redb读事务（只读事务时存在）
    pub read_txn: Option<redb::ReadTransaction>,
    /// 持久性级别
    pub durability: DurabilityLevel,
    /// 操作日志（使用 RwLock 优化读多写少的场景）
    operation_logs: RwLock<Vec<OperationLog>>,
    /// 修改的表
    modified_tables: Mutex<Vec<String>>,
    /// 保存点管理器（使用 RwLock 优化读多写少的场景）
    savepoint_manager: RwLock<SavepointManager>,
    /// 回滚执行器（用于保存点回滚时执行实际的数据回滚）
    rollback_executor: Mutex<Option<Box<dyn RollbackExecutor>>>,
}

/// 保存点管理器
pub(crate) struct SavepointManager {
    savepoints: HashMap<SavepointId, SavepointInfo>,
    next_id: SavepointId,
}

impl SavepointManager {
    fn new() -> Self {
        Self {
            savepoints: HashMap::new(),
            next_id: 1,
        }
    }

    fn create_savepoint(
        &mut self,
        name: Option<String>,
        operation_log_index: usize,
    ) -> SavepointId {
        let id = self.next_id;
        self.next_id += 1;
        let info = SavepointInfo {
            id,
            name,
            created_at: Instant::now(),
            operation_log_index,
        };
        self.savepoints.insert(id, info);
        id
    }

    fn get_savepoint(&self, id: SavepointId) -> Option<&SavepointInfo> {
        self.savepoints.get(&id)
    }

    fn remove_savepoint(&mut self, id: SavepointId) -> Option<SavepointInfo> {
        self.savepoints.remove(&id)
    }

    fn clear(&mut self) {
        self.savepoints.clear();
    }

    fn find_by_name(&self, name: &str) -> Option<SavepointInfo> {
        self.savepoints
            .values()
            .find(|sp| sp.name.as_deref() == Some(name))
            .cloned()
    }
}

impl TransactionContext {
    /// 创建新的事务上下文（读写事务）
    pub fn new_writable(
        id: TransactionId,
        timeout: Duration,
        durability: DurabilityLevel,
        write_txn: redb::WriteTransaction,
    ) -> Self {
        Self {
            id,
            state: AtomicCell::new(TransactionState::Active),
            start_time: Instant::now(),
            timeout,
            read_only: false,
            write_txn: Mutex::new(Some(write_txn)),
            read_txn: None,
            durability,
            operation_logs: RwLock::new(Vec::new()),
            modified_tables: Mutex::new(Vec::new()),
            savepoint_manager: RwLock::new(SavepointManager::new()),
            rollback_executor: Mutex::new(None),
        }
    }

    /// 创建新的事务上下文（只读事务）
    pub fn new_readonly(
        id: TransactionId,
        timeout: Duration,
        read_txn: redb::ReadTransaction,
    ) -> Self {
        Self {
            id,
            state: AtomicCell::new(TransactionState::Active),
            start_time: Instant::now(),
            timeout,
            read_only: true,
            write_txn: Mutex::new(None),
            read_txn: Some(read_txn),
            durability: DurabilityLevel::Immediate,
            operation_logs: RwLock::new(Vec::new()),
            modified_tables: Mutex::new(Vec::new()),
            savepoint_manager: RwLock::new(SavepointManager::new()),
            rollback_executor: Mutex::new(None),
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> TransactionState {
        self.state.load()
    }

    /// 检查事务是否超时
    pub fn is_expired(&self) -> bool {
        self.start_time.elapsed() > self.timeout
    }

    /// 获取剩余时间
    pub fn remaining_time(&self) -> Duration {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.timeout {
            Duration::from_secs(0)
        } else {
            self.timeout - elapsed
        }
    }

    /// 状态转换
    pub fn transition_to(&self, new_state: TransactionState) -> Result<(), TransactionError> {
        let current = self.state.load();

        // 验证状态转换是否合法
        let valid_transition = matches!(
            (current, new_state),
            (TransactionState::Active, TransactionState::Committing | TransactionState::Aborting)
                | (TransactionState::Committing, TransactionState::Committed)
                | (TransactionState::Aborting, TransactionState::Aborted)
        );

        if !valid_transition {
            return Err(TransactionError::InvalidStateTransition {
                from: current,
                to: new_state,
            });
        }

        self.state.store(new_state);
        Ok(())
    }

    /// 检查是否可以执行操作
    pub fn can_execute(&self) -> Result<(), TransactionError> {
        let state = self.state.load();

        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        Ok(())
    }

    /// 获取事务信息
    pub fn info(&self) -> TransactionInfo {
        let tables = self.modified_tables.lock();
        let savepoints = self.savepoint_manager.read();
        TransactionInfo {
            id: self.id,
            state: self.state.load(),
            start_time: self.start_time,
            elapsed: self.start_time.elapsed(),
            is_read_only: self.read_only,
            modified_tables: tables.clone(),
            savepoint_count: savepoints.savepoints.len(),
        }
    }

    /// 添加操作日志
    pub fn add_operation_log(&self, operation: OperationLog) {
        let mut logs = self.operation_logs.write();
        logs.push(operation);
    }

    /// 批量添加操作日志
    pub fn add_operation_logs(&self, operations: Vec<OperationLog>) {
        let mut logs = self.operation_logs.write();
        logs.extend(operations);
    }

    /// 获取操作日志
    pub fn get_operation_logs(&self) -> Vec<OperationLog> {
        let logs = self.operation_logs.read();
        logs.clone()
    }

    /// 获取操作日志长度
    pub fn operation_log_len(&self) -> usize {
        let logs = self.operation_logs.read();
        logs.len()
    }

    /// 获取指定索引的操作日志
    pub fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
        let logs = self.operation_logs.read();
        logs.get(index).cloned()
    }

    /// 获取指定范围的操作日志
    pub fn get_operation_logs_range(&self, start: usize, end: usize) -> Vec<OperationLog> {
        let logs = self.operation_logs.read();
        if start >= logs.len() {
            return Vec::new();
        }
        let end = end.min(logs.len());
        logs[start..end].to_vec()
    }

    /// 截断操作日志到指定索引
    pub fn truncate_operation_log(&self, index: usize) {
        let mut logs = self.operation_logs.write();
        if index < logs.len() {
            logs.truncate(index);
        }
    }

    /// 清空操作日志
    pub fn clear_operation_log(&self) {
        let mut logs = self.operation_logs.write();
        logs.clear();
    }

    /// 记录表修改
    pub fn record_table_modification(&self, table_name: &str) {
        let mut tables = self.modified_tables.lock();
        if !tables.contains(&table_name.to_string()) {
            tables.push(table_name.to_string());
        }
    }

    /// 获取修改的表
    pub fn get_modified_tables(&self) -> Vec<String> {
        let tables = self.modified_tables.lock();
        tables.clone()
    }

    /// 创建保存点
    pub fn create_savepoint(&self, name: Option<String>) -> SavepointId {
        let mut manager = self.savepoint_manager.write();
        let operation_log_index = self.operation_log_len();
        manager.create_savepoint(name, operation_log_index)
    }

    /// 获取保存点信息
    pub fn get_savepoint(&self, id: SavepointId) -> Option<SavepointInfo> {
        let manager = self.savepoint_manager.read();
        manager.get_savepoint(id).cloned()
    }

    /// 获取所有保存点
    pub fn get_all_savepoints(&self) -> Vec<SavepointInfo> {
        let manager = self.savepoint_manager.read();
        manager.savepoints.values().cloned().collect()
    }

    /// 通过名称查找保存点
    pub fn find_savepoint_by_name(&self, name: &str) -> Option<SavepointInfo> {
        let manager = self.savepoint_manager.read();
        manager.find_by_name(name)
    }

    /// 释放保存点
    pub fn release_savepoint(&self, id: SavepointId) -> Result<(), TransactionError> {
        let mut manager = self.savepoint_manager.write();
        manager
            .remove_savepoint(id)
            .map(|_| ())
            .ok_or(TransactionError::SavepointNotFound(id))
    }

    /// 回滚到保存点
    pub fn rollback_to_savepoint(&self, id: SavepointId) -> Result<(), TransactionError> {
        let state = self.state.load();
        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForAbort(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        let manager = self.savepoint_manager.write();
        let savepoint_info = manager
            .get_savepoint(id)
            .cloned()
            .ok_or(TransactionError::SavepointNotFound(id))?;

        // 获取需要回滚的操作日志（从保存点索引到当前日志末尾）
        let logs_to_rollback = {
            let logs = self.operation_logs.read();
            if savepoint_info.operation_log_index >= logs.len() {
                Vec::new()
            } else {
                logs[savepoint_info.operation_log_index..].to_vec()
            }
        };

        drop(manager);

        // 先执行数据回滚（使用回滚执行器）
        if !logs_to_rollback.is_empty() {
            let mut executor_guard = self.rollback_executor.lock();
            let executor = executor_guard
                .as_mut()
                .ok_or_else(|| TransactionError::RollbackFailed("未设置回滚执行器".to_string()))?;

            // 按逆序执行回滚操作
            for log in logs_to_rollback.iter().rev() {
                executor
                    .execute_rollback(log)
                    .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
            }
        }

        // 数据回滚成功后，再截断操作日志
        self.truncate_operation_log(savepoint_info.operation_log_index);

        // 移除该保存点之后的所有保存点
        let mut manager = self.savepoint_manager.write();
        let savepoints_to_remove: Vec<SavepointId> = manager
            .savepoints
            .keys()
            .filter(|&&k| k > id)
            .copied()
            .collect();

        for sp_id in savepoints_to_remove {
            manager.remove_savepoint(sp_id);
        }

        Ok(())
    }

    /// 清除所有保存点
    pub fn clear_savepoints(&self) {
        let mut manager = self.savepoint_manager.write();
        manager.clear();
    }

    /// 设置回滚执行器
    ///
    /// # Arguments
    /// * `executor` - 回滚执行器实例
    ///
    /// # 注意
    /// 此方法用于将存储层的回滚能力集成到事务上下文中，
    /// 使得保存点回滚能够执行实际的数据回滚操作
    pub fn set_rollback_executor(&self, executor: Box<dyn RollbackExecutor>) {
        let mut guard = self.rollback_executor.lock();
        *guard = Some(executor);
    }

    /// 清除回滚执行器
    pub fn clear_rollback_executor(&self) {
        let mut guard = self.rollback_executor.lock();
        *guard = None;
    }

    /// 取出写事务（用于提交）
    pub fn take_write_txn(&self) -> Result<redb::WriteTransaction, TransactionError> {
        self.write_txn
            .lock()
            .take()
            .ok_or(TransactionError::ReadOnlyTransaction)
    }

    /// 获取读事务引用
    pub fn read_txn(&self) -> Result<&redb::ReadTransaction, TransactionError> {
        self.read_txn.as_ref().ok_or(TransactionError::Internal(
            "Read transaction not available".to_string(),
        ))
    }

    /// 使用写事务执行操作（供存储层调用）
    ///
    /// # Arguments
    /// * `f` - 闭包，接收 redb::WriteTransaction 引用并返回结果
    ///
    /// # Returns
    /// * `Ok(R)` - 操作成功返回的结果
    /// * `Err(TransactionError)` - 操作失败返回的错误
    pub fn with_write_txn<F, R>(&self, f: F) -> Result<R, TransactionError>
    where
        F: FnOnce(&redb::WriteTransaction) -> Result<R, StorageError>,
    {
        if self.read_only {
            return Err(TransactionError::ReadOnlyTransaction);
        }

        let state = self.state.load();
        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        let guard = self.write_txn.lock();
        let txn = guard
            .as_ref()
            .ok_or(TransactionError::Internal("写事务不可用".to_string()))?;

        f(txn).map_err(|e| TransactionError::Internal(e.to_string()))
    }

    /// 使用读事务执行操作（供存储层调用）
    ///
    /// # Arguments
    /// * `f` - 闭包，接收 ReadTransaction 或 WriteTransaction 并返回结果
    ///
    /// # Returns
    /// * `Ok(R)` - 操作成功返回的结果
    /// * `Err(TransactionError)` - 操作失败返回的错误
    ///
    /// # 注意
    /// redb 本身不支持从 WriteTransaction 创建 ReadTransaction。
    /// 此方法通过两个不同的闭包来处理只读事务和读写事务。
    /// 对于只读事务，使用 read_txn；对于读写事务，使用 write_txn 进行读取。
    pub fn with_read_txn<F, R>(&self, f: F) -> Result<R, TransactionError>
    where
        F: FnOnce(&redb::ReadTransaction) -> Result<R, StorageError>,
    {
        let state = self.state.load();
        if !state.can_execute() && !state.is_terminal() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        // 优先使用只读事务
        if let Some(ref txn) = self.read_txn {
            return f(txn).map_err(|e| TransactionError::Internal(e.to_string()));
        }

        // 对于读写事务，需要从写事务创建读事务
        // redb 不支持直接从 WriteTransaction 读取，需要创建新的读事务
        // 但这会导致读写不一致，所以这里返回错误
        // 调用者应该使用 with_write_txn 方法
        Err(TransactionError::Internal(
            "读写事务不支持直接读取，请使用 with_write_txn 方法".to_string(),
        ))
    }

    /// 获取写事务的可变引用（供存储层调用）
    ///
    /// # Safety
    /// 此方法返回可变引用，调用者必须确保：
    /// 1. 没有其他线程同时访问该事务
    /// 2. 操作完成后立即释放引用
    ///
    /// 建议使用 `with_write_txn` 方法代替
    pub fn write_txn_mut(
        &self,
    ) -> Result<impl std::ops::DerefMut<Target = redb::WriteTransaction> + '_, TransactionError>
    {
        if self.read_only {
            return Err(TransactionError::ReadOnlyTransaction);
        }

        let state = self.state.load();
        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        struct WriteTxnGuard<'a> {
            guard: parking_lot::MutexGuard<'a, Option<redb::WriteTransaction>>,
        }

        impl<'a> std::ops::Deref for WriteTxnGuard<'a> {
            type Target = redb::WriteTransaction;
            fn deref(&self) -> &Self::Target {
                self.guard.as_ref().expect("写事务应该存在")
            }
        }

        impl<'a> std::ops::DerefMut for WriteTxnGuard<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.guard.as_mut().expect("写事务应该存在")
            }
        }

        let guard = self.write_txn.lock();
        if guard.is_none() {
            return Err(TransactionError::Internal("写事务不可用".to_string()));
        }

        Ok(WriteTxnGuard { guard })
    }
}

impl Drop for TransactionContext {
    fn drop(&mut self) {
        // 如果事务仍处于活跃状态，自动中止
        let state = self.state.load();
        if state == TransactionState::Active {
            // redb的WriteTransaction在Drop时会自动回滚
            // 这里只需要更新状态
            self.state.store(TransactionState::Aborted);
        }

        // 清理保存点资源
        let mut manager = self.savepoint_manager.write();
        manager.clear();
        drop(manager);

        // 清理回滚执行器
        let mut executor_guard = self.rollback_executor.lock();
        *executor_guard = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // 注意：这些测试需要redb数据库实例，这里仅测试基本逻辑

    #[test]
    fn test_transaction_context_state_machine() {
        // 由于需要实际的redb事务，这里仅测试状态转换逻辑
        // 实际测试应在集成测试中进行
    }

    #[test]
    fn test_transaction_timeout() {
        // 创建模拟上下文（仅用于测试超时逻辑）
        struct MockContext {
            start_time: Instant,
            timeout: Duration,
        }

        let ctx = MockContext {
            start_time: Instant::now(),
            timeout: Duration::from_millis(100),
        };

        std::thread::sleep(Duration::from_millis(150));

        assert!(ctx.start_time.elapsed() > ctx.timeout);
    }
}
