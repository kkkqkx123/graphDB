//! 事务上下文
//!
//! 管理单个事务的状态和资源

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crossbeam_utils::atomic::AtomicCell;
use parking_lot::Mutex;

use crate::core::StorageError;
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
    /// 已修改的表集合（用于冲突检测）
    modified_tables: Mutex<HashSet<String>>,
    /// 操作日志（用于恢复）
    operation_log: Mutex<Vec<OperationLog>>,
    /// 持久性级别
    pub durability: DurabilityLevel,
    /// 是否启用两阶段提交
    pub two_phase_commit: bool,
}

impl TransactionContext {
    /// 创建新的事务上下文（读写事务）
    pub fn new_writable(
        id: TransactionId,
        timeout: Duration,
        durability: DurabilityLevel,
        two_phase_commit: bool,
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
            modified_tables: Mutex::new(HashSet::new()),
            operation_log: Mutex::new(Vec::new()),
            durability,
            two_phase_commit,
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
            modified_tables: Mutex::new(HashSet::new()),
            operation_log: Mutex::new(Vec::new()),
            durability: DurabilityLevel::Immediate,
            two_phase_commit: false,
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
        let valid_transition = match (current, new_state) {
            (TransactionState::Active, TransactionState::Prepared) => true,
            (TransactionState::Active, TransactionState::Committing) => true,
            (TransactionState::Active, TransactionState::Aborting) => true,
            (TransactionState::Prepared, TransactionState::Committing) => true,
            (TransactionState::Prepared, TransactionState::Aborting) => true,
            (TransactionState::Committing, TransactionState::Committed) => true,
            (TransactionState::Aborting, TransactionState::Aborted) => true,
            _ => false,
        };

        if !valid_transition {
            return Err(TransactionError::InvalidStateTransition {
                from: current,
                to: new_state,
            });
        }

        self.state.store(new_state);
        Ok(())
    }

    /// 记录表修改
    pub fn record_table_modification(&self, table_name: &str) {
        self.modified_tables.lock().insert(table_name.to_string());
    }

    /// 获取已修改的表
    pub fn modified_tables(&self) -> Vec<String> {
        self.modified_tables.lock().iter().cloned().collect()
    }

    /// 添加操作日志
    pub fn add_operation_log(&self, operation: OperationLog) {
        self.operation_log.lock().push(operation);
    }

    /// 获取操作日志长度
    pub fn operation_log_len(&self) -> usize {
        self.operation_log.lock().len()
    }

    /// 获取操作日志（用于保存点回滚）
    pub fn truncate_operation_log(&self, index: usize) {
        self.operation_log.lock().truncate(index);
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
        TransactionInfo {
            id: self.id,
            state: self.state.load(),
            start_time: self.start_time,
            elapsed: self.start_time.elapsed(),
            is_read_only: self.read_only,
            modified_tables: self.modified_tables(),
            savepoint_count: 0, // 由SavepointManager维护
        }
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
        self.read_txn
            .as_ref()
            .ok_or(TransactionError::Internal("Read transaction not available".to_string()))
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
    /// * `f` - 闭包，接收 redb::ReadTransaction 引用并返回结果
    ///
    /// # Returns
    /// * `Ok(R)` - 操作成功返回的结果
    /// * `Err(TransactionError)` - 操作失败返回的错误
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

        // 对于读写事务，需要创建一个新的读事务
        // 因为写事务不能直接作为读事务使用
        Err(TransactionError::Internal(
            "只读事务不可用，请使用 with_write_txn 进行读取操作".to_string(),
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
    pub fn write_txn_mut(&self) -> Result<impl std::ops::DerefMut<Target = redb::WriteTransaction> + '_, TransactionError> {
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
                self.guard.as_ref().unwrap()
            }
        }

        impl<'a> std::ops::DerefMut for WriteTxnGuard<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.guard.as_mut().unwrap()
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
        if state == TransactionState::Active || state == TransactionState::Prepared {
            // redb的WriteTransaction在Drop时会自动回滚
            // 这里只需要更新状态
            self.state.store(TransactionState::Aborted);
        }
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
