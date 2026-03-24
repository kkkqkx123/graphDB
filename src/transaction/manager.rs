//! 事务管理器
//!
//! 管理所有事务的生命周期，提供事务的开始、提交、中止等操作

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::Mutex;
use redb::Database;

use crate::storage::operations::rollback::RollbackExecutor;
use crate::transaction::context::TransactionContext;
use crate::transaction::types::*;

/// Rollback executor factory type alias
type RollbackExecutorFactory = Box<dyn Fn() -> Box<dyn RollbackExecutor> + Send + Sync>;

/// 事务管理器
pub struct TransactionManager {
    /// 数据库实例
    db: Arc<Database>,
    /// 配置
    config: TransactionManagerConfig,
    /// 活跃事务表 - 使用 DashMap 替代 RwLock<HashMap> 以获得更好的并发性能
    active_transactions: Arc<DashMap<TransactionId, Arc<TransactionContext>>>,
    /// 事务ID生成器
    id_generator: AtomicU64,
    /// 统计信息
    stats: Arc<TransactionStats>,
    /// 是否已关闭
    shutdown_flag: AtomicU64,
    /// 回滚执行器工厂（用于为每个事务创建回滚执行器）
    rollback_executor_factory: Mutex<Option<RollbackExecutorFactory>>,
}

impl TransactionManager {
    /// 创建新的事务管理器
    pub fn new(db: Arc<Database>, config: TransactionManagerConfig) -> Self {
        Self {
            db,
            config,
            active_transactions: Arc::new(DashMap::new()),
            id_generator: AtomicU64::new(1),
            stats: Arc::new(TransactionStats::new()),
            shutdown_flag: AtomicU64::new(0),
            rollback_executor_factory: Mutex::new(None),
        }
    }

    /// 开始新事务
    pub fn begin_transaction(
        &self,
        options: TransactionOptions,
    ) -> Result<TransactionId, TransactionError> {
        // 检查是否已关闭
        if self.shutdown_flag.load(Ordering::SeqCst) != 0 {
            return Err(TransactionError::Internal("事务管理器已关闭".to_string()));
        }

        // 检查并发事务数限制
        let active_count = self.active_transactions.len();
        if active_count >= self.config.max_concurrent_transactions {
            return Err(TransactionError::TooManyTransactions);
        }

        // 检查是否已经有活跃的写事务
        if !options.read_only {
            for entry in self.active_transactions.iter() {
                let context = entry.value();
                if !context.read_only {
                    return Err(TransactionError::WriteTransactionConflict);
                }
            }
        }

        let txn_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        let timeout = options.timeout.unwrap_or(self.config.default_timeout);

        // 创建事务上下文
        let context = if options.read_only {
            let read_txn = self
                .db
                .begin_read()
                .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;

            Arc::new(TransactionContext::new_readonly(txn_id, timeout, read_txn))
        } else {
            // redb 会自动处理单写者限制，不需要手动管理
            let write_txn = self
                .db
                .begin_write()
                .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;

            Arc::new(TransactionContext::new_writable(
                txn_id,
                timeout,
                options.durability,
                write_txn,
            ))
        };

        // 为读写事务设置回滚执行器
        if !options.read_only {
            let factory_guard = self.rollback_executor_factory.lock();
            if let Some(factory) = factory_guard.as_ref() {
                let executor = (**factory)();
                context.set_rollback_executor(executor);
            }
        }

        self.active_transactions.insert(txn_id, context);
        self.stats.increment_total();
        self.stats.increment_active();

        Ok(txn_id)
    }

    /// 获取事务上下文
    pub fn get_context(
        &self,
        txn_id: TransactionId,
    ) -> Result<Arc<TransactionContext>, TransactionError> {
        self.active_transactions
            .get(&txn_id)
            .map(|entry| entry.value().clone())
            .ok_or(TransactionError::TransactionNotFound(txn_id))
    }

    /// 检查事务是否存在且活跃
    pub fn is_transaction_active(&self, txn_id: TransactionId) -> bool {
        self.active_transactions
            .get(&txn_id)
            .map(|entry| entry.value().state().can_execute())
            .unwrap_or(false)
    }

    /// 提交事务
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        // 从DashMap中移除事务并获取所有权
        let context = {
            let entry = self
                .active_transactions
                .get(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;

            let ctx = entry.value().clone();
            drop(entry);

            // 检查状态
            if !ctx.state().can_commit() {
                return Err(TransactionError::InvalidStateForCommit(ctx.state()));
            }

            // 检查超时
            if ctx.is_expired() {
                // 已经超时，移除并中止
                self.active_transactions.remove(&txn_id);
                self.stats.increment_timeout();
                // 中止事务
                self.abort_transaction_internal(ctx)?;
                return Err(TransactionError::TransactionTimeout);
            }

            // 状态检查通过，移除事务
            self.active_transactions.remove(&txn_id);
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

            // 提交事务
            write_txn
                .commit()
                .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
        }

        context.transition_to(TransactionState::Committed)?;

        // 清理
        self.stats.decrement_active();
        self.stats.increment_committed();

        Ok(())
    }

    /// 中止事务（内部版本，不操作HashMap）
    fn abort_transaction_internal(
        &self,
        context: Arc<TransactionContext>,
    ) -> Result<(), TransactionError> {
        if !context.state().can_abort() {
            return Err(TransactionError::InvalidStateForAbort(context.state()));
        }

        context.transition_to(TransactionState::Aborting)?;

        // 取出写事务，Drop时会自动回滚
        if !context.read_only {
            let _ = context.take_write_txn();
        }

        self.stats.decrement_active();
        self.stats.increment_aborted();

        Ok(())
    }

    /// 中止事务
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        // 从DashMap中移除事务并获取所有权
        let context = {
            let entry = self
                .active_transactions
                .get(&txn_id)
                .ok_or(TransactionError::TransactionNotFound(txn_id))?;
            let ctx = entry.value().clone();
            drop(entry);

            if !ctx.state().can_abort() {
                return Err(TransactionError::InvalidStateForAbort(ctx.state()));
            }

            // 状态检查通过，移除事务
            self.active_transactions.remove(&txn_id);
            ctx
        };

        // 执行中止
        self.abort_transaction_internal(context)
    }

    /// 获取活跃事务列表
    pub fn list_active_transactions(&self) -> Vec<TransactionInfo> {
        self.active_transactions
            .iter()
            .map(|entry| entry.value().info())
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
            .get(&txn_id)
            .map(|entry| entry.value().info())
    }

    /// 获取统计信息
    pub fn stats(&self) -> &TransactionStats {
        &self.stats
    }

    /// 清理过期事务
    pub fn cleanup_expired_transactions(&self) {
        // 收集所有过期的事务ID
        let expired: Vec<TransactionId> = {
            self.active_transactions
                .iter()
                .filter(|entry| entry.value().is_expired())
                .map(|entry| *entry.key())
                .collect()
        };

        for txn_id in expired {
            let _ = self.abort_transaction(txn_id);
            self.stats.increment_timeout();
        }
    }

    /// 关闭事务管理器
    pub fn shutdown(&self) {
        // 设置关闭标志
        self.shutdown_flag.store(1, Ordering::SeqCst);

        // 中止所有活跃事务
        let txn_ids: Vec<TransactionId> = {
            self.active_transactions
                .iter()
                .map(|entry| *entry.key())
                .collect()
        };

        for txn_id in txn_ids {
            let _ = self.abort_transaction(txn_id);
        }
    }

    /// 获取配置
    pub fn config(&self) -> TransactionManagerConfig {
        self.config.clone()
    }

    /// 创建保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    /// * `name` - 保存点名称（可选）
    ///
    /// # Returns
    /// * `Ok(SavepointId)` - 保存点ID
    /// * `Err(TransactionError)` - 失败时返回错误
    pub fn create_savepoint(
        &self,
        txn_id: TransactionId,
        name: Option<String>,
    ) -> Result<SavepointId, TransactionError> {
        let context = self.get_context(txn_id)?;
        Ok(context.create_savepoint(name))
    }

    /// 获取保存点信息
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    /// * `id` - 保存点ID
    ///
    /// # Returns
    /// * `Some(SavepointInfo)` - 保存点信息
    /// * `None` - 保存点不存在
    pub fn get_savepoint(&self, txn_id: TransactionId, id: SavepointId) -> Option<SavepointInfo> {
        let context = self.get_context(txn_id).ok()?;
        context.get_savepoint(id)
    }

    /// 释放保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    /// * `id` - 保存点ID
    ///
    /// # Returns
    /// * `Ok(())` - 成功
    /// * `Err(TransactionError)` - 失败时返回错误
    pub fn release_savepoint(
        &self,
        txn_id: TransactionId,
        id: SavepointId,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.release_savepoint(id)
    }

    /// 回滚到保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    /// * `id` - 保存点ID
    ///
    /// # Returns
    /// * `Ok(())` - 成功
    /// * `Err(TransactionError)` - 失败时返回错误
    ///
    /// # 注意
    /// 此方法会移除该保存点之后的所有保存点
    /// 实际的数据回滚需要与存储层配合实现
    pub fn rollback_to_savepoint(
        &self,
        txn_id: TransactionId,
        id: SavepointId,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.rollback_to_savepoint(id)
    }

    /// 获取事务的所有活跃保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    ///
    /// # Returns
    /// * `Vec<SavepointInfo>` - 保存点信息列表
    pub fn get_active_savepoints(&self, txn_id: TransactionId) -> Vec<SavepointInfo> {
        let context = match self.get_context(txn_id) {
            Ok(ctx) => ctx,
            Err(_) => return Vec::new(),
        };
        context.get_all_savepoints()
    }

    /// 通过名称查找保存点
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    /// * `name` - 保存点名称
    ///
    /// # Returns
    /// * `Some(SavepointInfo)` - 保存点信息
    /// * `None` - 保存点不存在
    pub fn find_savepoint_by_name(
        &self,
        txn_id: TransactionId,
        name: &str,
    ) -> Option<SavepointInfo> {
        let context = self.get_context(txn_id).ok()?;
        context.find_savepoint_by_name(name)
    }

    /// 设置回滚执行器工厂
    ///
    /// # Arguments
    /// * `factory` - 回滚执行器工厂，用于为每个事务创建回滚执行器
    ///
    /// # 注意
    /// 此方法用于将存储层的回滚能力集成到事务管理器中，
    /// 使得保存点回滚能够执行实际的数据回滚操作
    pub fn set_rollback_executor_factory(
        &self,
        factory: Box<dyn Fn() -> Box<dyn RollbackExecutor> + Send + Sync>,
    ) {
        let mut guard = self.rollback_executor_factory.lock();
        *guard = Some(factory);
    }

    /// 清除回滚执行器工厂
    pub fn clear_rollback_executor_factory(&self) {
        let mut guard = self.rollback_executor_factory.lock();
        *guard = None;
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
        let db = Arc::new(
            Database::create(temp_dir.path().join("test.db"))
                .expect("Failed to create test database"),
        );
        let config = TransactionManagerConfig::default();
        let manager = TransactionManager::new(db.clone(), config);
        (manager, db, temp_dir)
    }

    #[test]
    fn test_begin_and_commit_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        assert!(manager.is_transaction_active(txn_id));

        manager
            .commit_transaction(txn_id)
            .expect("Failed to commit transaction");

        assert!(!manager.is_transaction_active(txn_id));
        assert_eq!(
            manager
                .stats()
                .committed_transactions
                .load(Ordering::Relaxed),
            1
        );
    }

    #[test]
    fn test_begin_and_abort_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        manager
            .abort_transaction(txn_id)
            .expect("Failed to abort transaction");

        assert!(!manager.is_transaction_active(txn_id));
        assert_eq!(
            manager.stats().aborted_transactions.load(Ordering::Relaxed),
            1
        );
    }

    #[test]
    fn test_readonly_transaction() {
        let (manager, _db, _temp) = create_test_manager();

        let options = TransactionOptions::new().read_only();
        let txn_id = manager
            .begin_transaction(options)
            .expect("Failed to begin readonly transaction");

        let context = manager
            .get_context(txn_id)
            .expect("Failed to get transaction context");
        assert!(context.read_only);

        manager
            .commit_transaction(txn_id)
            .expect("Failed to commit readonly transaction");
    }

    #[test]
    fn test_transaction_not_found() {
        let (manager, _db, _temp) = create_test_manager();

        let result = manager.get_context(9999);
        assert!(matches!(
            result,
            Err(TransactionError::TransactionNotFound(9999))
        ));
    }

    #[test]
    fn test_invalid_state_transition() {
        let (manager, _db, _temp) = create_test_manager();

        let txn_id = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");

        // 提交事务
        manager
            .commit_transaction(txn_id)
            .expect("Failed to commit transaction");

        // 再次提交应该失败
        let result = manager.commit_transaction(txn_id);
        assert!(matches!(
            result,
            Err(TransactionError::TransactionNotFound(_))
        ));
    }

    #[test]
    fn test_concurrent_transactions() {
        let (manager, _db, _temp) = create_test_manager();

        // 由于redb的单写者限制，我们只能顺序执行事务
        // 第一个事务
        let txn1 = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn1));
        manager
            .commit_transaction(txn1)
            .expect("Failed to commit transaction");
        assert!(!manager.is_transaction_active(txn1));

        // 第二个事务
        let txn2 = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn2));
        manager
            .abort_transaction(txn2)
            .expect("Failed to abort transaction");
        assert!(!manager.is_transaction_active(txn2));

        // 第三个事务
        let txn3 = manager
            .begin_transaction(TransactionOptions::default())
            .expect("Failed to begin transaction");
        assert!(manager.is_transaction_active(txn3));
        manager
            .commit_transaction(txn3)
            .expect("Failed to commit transaction");
        assert!(!manager.is_transaction_active(txn3));

        assert_eq!(
            manager
                .stats()
                .committed_transactions
                .load(Ordering::Relaxed),
            2
        );
        assert_eq!(
            manager.stats().aborted_transactions.load(Ordering::Relaxed),
            1
        );
    }

    #[test]
    fn test_multiple_readonly_transactions() {
        let (manager, _db, _temp) = create_test_manager();

        // 只读事务可以并发
        let options = TransactionOptions::new().read_only();
        let txn1 = manager
            .begin_transaction(options.clone())
            .expect("Failed to begin transaction");
        let txn2 = manager
            .begin_transaction(options.clone())
            .expect("Failed to begin transaction");
        let txn3 = manager
            .begin_transaction(options)
            .expect("Failed to begin transaction");

        assert!(manager.is_transaction_active(txn1));
        assert!(manager.is_transaction_active(txn2));
        assert!(manager.is_transaction_active(txn3));

        manager
            .commit_transaction(txn1)
            .expect("Failed to commit transaction");
        manager
            .commit_transaction(txn2)
            .expect("Failed to commit transaction");
        manager
            .commit_transaction(txn3)
            .expect("Failed to commit transaction");
    }
}
