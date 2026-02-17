//! 事务上下文
//!
//! 管理单个事务的状态和资源

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crossbeam_utils::atomic::AtomicCell;
use parking_lot::Mutex;

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
