//! 事务管理器
//!
//! 管理所有事务的生命周期，提供事务的开始、提交、中止等操作
//!
//! 注意：由于redb使用单写者多读者模型，本管理器采用延迟创建写事务策略，
//! 只在真正需要写入时才获取redb写锁。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crossbeam_utils::atomic::AtomicCell;
use parking_lot::RwLock;
use redb::Database;

use crate::transaction::context::TransactionContext;
use crate::transaction::types::*;

/// 事务管理器
pub struct TransactionManager {
    /// 数据库实例
    db: Arc<Database>,
    /// 配置
    config: TransactionManagerConfig,
    /// 活跃事务表
    active_transactions: Arc<RwLock<HashMap<TransactionId, Arc<TransactionContext>>>>,
    /// 事务ID生成器
    id_generator: AtomicU64,
    /// 统计信息
    stats: Arc<TransactionStats>,
    /// 运行状态
    running: Arc<AtomicCell<bool>>,
    /// 是否有活跃的redb写事务
    has_redb_write_txn: Arc<AtomicCell<bool>>,
}

impl TransactionManager {
    /// 创建新的事务管理器
    pub fn new(db: Arc<Database>, config: TransactionManagerConfig) -> Self {
        let auto_cleanup = config.auto_cleanup;
        let cleanup_interval = config.cleanup_interval;
        let manager = Self {
            db,
            config,
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            id_generator: AtomicU64::new(1),
            stats: Arc::new(TransactionStats::new()),
            running: Arc::new(AtomicCell::new(true)),
            has_redb_write_txn: Arc::new(AtomicCell::new(false)),
        };

        // 启动后台清理任务
        if auto_cleanup {
            manager.start_cleanup_task(cleanup_interval);
        }

        manager
    }

    /// 开始新事务
    pub fn begin_transaction(&self, options: TransactionOptions) -> Result<TransactionId, TransactionError> {
        // 检查管理器是否仍在运行
        if !self.running.load() {
            return Err(TransactionError::Internal("Transaction manager is shutting down".to_string()));
        }

        // 检查并发事务数限制
        let active_count = self.active_transactions.read().len();
        if active_count >= self.config.max_concurrent_transactions {
            return Err(TransactionError::TooManyTransactions);
        }

        let txn_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        let timeout = options.timeout.unwrap_or(self.config.default_timeout);

        // 创建事务上下文
        // 对于读写事务，延迟创建redb写事务，避免阻塞
        let context = if options.read_only {
            let read_txn = self.db.begin_read()
                .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;
            
            Arc::new(TransactionContext::new_readonly(
                txn_id,
                timeout,
                read_txn,
            ))
        } else {
            // 检查是否已有redb写事务
            if self.has_redb_write_txn.load() {
                return Err(TransactionError::WriteTransactionConflict);
            }
            
            // 尝试获取redb写事务
            self.has_redb_write_txn.store(true);
            let write_txn = self.db.begin_write()
                .map_err(|e| {
                    self.has_redb_write_txn.store(false);
                    TransactionError::BeginFailed(e.to_string())
                })?;
            
            Arc::new(TransactionContext::new_writable(
                txn_id,
                timeout,
                options.durability,
                options.two_phase_commit,
                write_txn,
            ))
        };

        self.active_transactions.write().insert(txn_id, context);
        self.stats.increment_total();
        self.stats.increment_active();

        Ok(txn_id)
    }

    /// 获取事务上下文
    pub fn get_context(&self, txn_id: TransactionId) -> Result<Arc<TransactionContext>, TransactionError> {
        self.active_transactions
            .read()
            .get(&txn_id)
            .cloned()
            .ok_or(TransactionError::TransactionNotFound(txn_id))
    }

    /// 检查事务是否存在且活跃
    pub fn is_transaction_active(&self, txn_id: TransactionId) -> bool {
        self.active_transactions
            .read()
            .get(&txn_id)
            .map(|ctx| ctx.state().can_execute())
            .unwrap_or(false)
    }

    /// 提交事务
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        // 从HashMap中移除事务并获取所有权
        let context = {
            let mut write_guard = self.active_transactions.write();
            let ctx = write_guard.remove(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;
            
            // 检查状态
            if !ctx.state().can_commit() {
                // 状态不对，放回去
                write_guard.insert(txn_id, ctx.clone());
                return Err(TransactionError::InvalidStateForCommit(ctx.state()));
            }
            
            // 检查超时
            if ctx.is_expired() {
                // 已经超时，不放回去，直接中止
                drop(write_guard);
                self.stats.increment_timeout();
                // 中止事务（不需要再从HashMap移除，已经移除了）
                self.abort_transaction_internal(ctx)?;
                return Err(TransactionError::TransactionTimeout);
            }
            
            ctx
        };

        // 执行提交
        context.transition_to(TransactionState::Committing)?;

        // 提交redb事务
        if !context.read_only {
            let mut write_txn = context.take_write_txn()?;
            
            // 设置持久性级别
            let durability: redb::Durability = context.durability.into();
            write_txn.set_durability(durability);
            
            // 如果启用2PC，设置两阶段提交
            if context.two_phase_commit {
                write_txn.set_two_phase_commit(true);
            }
            
            // 提交事务
            write_txn.commit()
                .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
            
            // 释放redb写事务标记
            self.has_redb_write_txn.store(false);
        }

        context.transition_to(TransactionState::Committed)?;

        // 清理
        self.stats.decrement_active();
        self.stats.increment_committed();

        Ok(())
    }

    /// 中止事务（内部版本，不操作HashMap）
    fn abort_transaction_internal(&self, context: Arc<TransactionContext>) -> Result<(), TransactionError> {
        if !context.state().can_abort() {
            return Err(TransactionError::InvalidStateForAbort(context.state()));
        }

        context.transition_to(TransactionState::Aborting)?;

        // 取出写事务，Drop时会自动回滚
        if !context.read_only {
            let _ = context.take_write_txn();
            // 释放redb写事务标记
            self.has_redb_write_txn.store(false);
        }

        self.stats.decrement_active();
        self.stats.increment_aborted();

        Ok(())
    }

    /// 中止事务
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        // 从HashMap中移除事务并获取所有权
        let context = {
            let mut write_guard = self.active_transactions.write();
            let ctx = write_guard.remove(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;
            
            if !ctx.state().can_abort() {
                // 状态不对，放回去
                write_guard.insert(txn_id, ctx.clone());
                return Err(TransactionError::InvalidStateForAbort(ctx.state()));
            }
            
            ctx
        };

        // 执行中止
        self.abort_transaction_internal(context)
    }

    /// 获取活跃事务列表
    pub fn list_active_transactions(&self) -> Vec<TransactionInfo> {
        self.active_transactions
            .read()
            .values()
            .map(|ctx| ctx.info())
            .collect()
    }

    /// 获取指定事务的信息
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    ///
    /// # Returns
    /// * `Some(TransactionInfo)` - 如果事务存在
    /// * `None` - 如果事务不存在
    pub fn get_transaction_info(&self, txn_id: TransactionId) -> Option<TransactionInfo> {
        self.active_transactions
            .read()
            .get(&txn_id)
            .map(|ctx| ctx.info())
    }

    /// 获取统计信息
    pub fn stats(&self) -> &TransactionStats {
        &self.stats
    }

    /// 清理过期事务
    pub fn cleanup_expired_transactions(&self) {
        // 收集所有过期的事务ID
        let expired: Vec<TransactionId> = {
            let read_guard = self.active_transactions.read();
            read_guard
                .iter()
                .filter(|(_, ctx)| ctx.is_expired())
                .map(|(id, _)| *id)
                .collect()
        };

        for txn_id in expired {
            let _ = self.abort_transaction(txn_id);
            self.stats.increment_timeout();
        }
    }

    /// 启动后台清理任务
    fn start_cleanup_task(&self, interval: std::time::Duration) {
        let active_transactions = self.active_transactions.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();
        let has_redb_write_txn = self.has_redb_write_txn.clone();

        std::thread::spawn(move || {
            while running.load() {
                std::thread::sleep(interval);

                if !running.load() {
                    break;
                }

                // 先收集所有过期的事务ID
                let expired: Vec<(TransactionId, bool, bool)> = {
                    let read_guard = active_transactions.read();
                    read_guard
                        .iter()
                        .filter(|(_, ctx)| ctx.is_expired())
                        .map(|(id, ctx)| (*id, ctx.state().can_abort(), ctx.read_only))
                        .collect()
                };

                // 然后逐个处理，避免在持有读锁时获取写锁
                for (txn_id, can_abort, is_readonly) in expired {
                    if can_abort {
                        let mut write_guard = active_transactions.write();
                        if let Some(ctx) = write_guard.remove(&txn_id) {
                            let _ = ctx.transition_to(TransactionState::Aborting);
                            if !is_readonly {
                                let _ = ctx.take_write_txn();
                                has_redb_write_txn.store(false);
                            }
                            stats.decrement_active();
                            stats.increment_timeout();
                        }
                    }
                }
            }
        });
    }

    /// 关闭事务管理器
    pub fn shutdown(&self) {
        self.running.store(false);

        // 中止所有活跃事务
        let txn_ids: Vec<TransactionId> = {
            let read_guard = self.active_transactions.read();
            read_guard.keys().cloned().collect()
        };

        for txn_id in txn_ids {
            let _ = self.abort_transaction(txn_id);
        }
    }

    /// 获取配置
    pub fn config(&self) -> &TransactionManagerConfig {
        &self.config
    }
}

impl Drop for TransactionManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_manager() -> (TransactionManager, Arc<Database>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db = Arc::new(Database::create(temp_dir.path().join("test.db")).expect("Failed to create test database"));
        let config = TransactionManagerConfig {
            auto_cleanup: false, // 禁用后台清理，避免测试中的死锁
            ..TransactionManagerConfig::default()
        };
        let manager = TransactionManager::new(db.clone(), config);
        (manager, db, temp_dir)
    }

    #[test]
    fn test_begin_and_commit_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager.begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        assert!(manager.is_transaction_active(txn_id));

        manager.commit_transaction(txn_id)
            .expect("Failed to commit transaction");

        assert!(!manager.is_transaction_active(txn_id));
        assert_eq!(manager.stats().committed_transactions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_begin_and_abort_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager.begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager.abort_transaction(txn_id)
            .expect("Failed to abort transaction");

        assert!(!manager.is_transaction_active(txn_id));
        assert_eq!(manager.stats().aborted_transactions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_readonly_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let options = TransactionOptions::new().read_only();
        let txn_id = manager.begin_transaction(options)
            .expect("Failed to begin readonly transaction");

        let context = manager.get_context(txn_id).expect("Failed to get transaction context");
        assert!(context.read_only);

        manager.commit_transaction(txn_id)
            .expect("Failed to commit readonly transaction");
    }

    #[test]
    fn test_transaction_not_found() {
        let (manager, _db, _temp) = create_test_manager();

        let result = manager.get_context(9999);
        assert!(matches!(result, Err(TransactionError::TransactionNotFound(9999))));
    }

    #[test]
    fn test_invalid_state_transition() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager.begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        // 提交事务
        manager.commit_transaction(txn_id).expect("Failed to commit transaction");

        // 再次提交应该失败
        let result = manager.commit_transaction(txn_id);
        assert!(matches!(result, Err(TransactionError::TransactionNotFound(_))));
    }

    #[test]
    fn test_concurrent_transactions() {
        let (manager, _db, _temp) = create_test_manager();

        // 由于redb的单写者限制，我们只能顺序执行事务
        // 第一个事务
        let txn1 = manager.begin_transaction(TransactionOptions::default()).expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn1));
        manager.commit_transaction(txn1).expect("Failed to commit transaction");
        assert!(!manager.is_transaction_active(txn1));

        // 第二个事务
        let txn2 = manager.begin_transaction(TransactionOptions::default()).expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn2));
        manager.abort_transaction(txn2).expect("Failed to abort transaction");
        assert!(!manager.is_transaction_active(txn2));

        // 第三个事务
        let txn3 = manager.begin_transaction(TransactionOptions::default()).expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn3));
        manager.commit_transaction(txn3).expect("Failed to commit transaction");
        assert!(!manager.is_transaction_active(txn3));

        assert_eq!(manager.stats().committed_transactions.load(Ordering::Relaxed), 2);
        assert_eq!(manager.stats().aborted_transactions.load(Ordering::Relaxed), 1);
    }
    
    #[test]
    fn test_write_transaction_conflict() {
        let (manager, _db, _temp) = create_test_manager();
        
        // 开始第一个写事务
        let txn1 = manager.begin_transaction(TransactionOptions::default()).expect("Failed to begin transaction");
        
        // 尝试开始第二个写事务应该失败（因为redb只支持单写者）
        let result = manager.begin_transaction(TransactionOptions::default());
        assert!(matches!(result, Err(TransactionError::WriteTransactionConflict)));
        
        // 提交第一个事务
        manager.commit_transaction(txn1).expect("Failed to commit transaction");
        
        // 现在可以开始新的事务了
        let txn2 = manager.begin_transaction(TransactionOptions::default()).expect("Failed to begin transaction");
        manager.commit_transaction(txn2).expect("Failed to commit transaction");
    }
    
    #[test]
    fn test_multiple_readonly_transactions() {
        let (manager, _db, _temp) = create_test_manager();
        
        // 只读事务可以并发
        let options = TransactionOptions::new().read_only();
        let txn1 = manager.begin_transaction(options.clone()).expect("Failed to begin transaction");
        let txn2 = manager.begin_transaction(options.clone()).expect("Failed to begin transaction");
        let txn3 = manager.begin_transaction(options).expect("Failed to begin transaction");
        
        assert!(manager.is_transaction_active(txn1));
        assert!(manager.is_transaction_active(txn2));
        assert!(manager.is_transaction_active(txn3));
        
        manager.commit_transaction(txn1).expect("Failed to commit transaction");
        manager.commit_transaction(txn2).expect("Failed to commit transaction");
        manager.commit_transaction(txn3).expect("Failed to commit transaction");
    }
}
