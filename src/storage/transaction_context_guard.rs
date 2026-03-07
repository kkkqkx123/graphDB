//! 事务上下文 RAII 管理器
//!
//! 提供自动管理事务上下文生命周期的 RAII 包装器
//!
//! # 注意
//! redb 本身支持事务的自动回滚（通过 Drop trait）。
//! 此模块提供更高级的 RAII 管理，包括自动清理和错误处理。

use std::ops::Deref;
use std::sync::Arc;

use crate::storage::RedbStorage;
use crate::transaction::{TransactionContext, TransactionError, TransactionId, TransactionManager};

/// 事务上下文 RAII 守卫
///
/// 自动管理事务上下文的生命周期，确保在作用域结束时正确清理
///
/// # 注意
/// redb 的 WriteTransaction 在 Drop 时会自动回滚。
/// 此 RAII 守卫确保：
/// 1. 事务上下文在作用域结束时自动清除
/// 2. 错误时自动中止事务
/// 3. 防止事务上下文泄露
pub struct TransactionContextGuard<'a> {
    storage: &'a RedbStorage,
    txn_manager: &'a TransactionManager,
    txn_id: TransactionId,
    auto_commit: bool,
    committed: bool,
    context: Arc<TransactionContext>,
}

impl<'a> TransactionContextGuard<'a> {
    /// 创建新的事务上下文守卫
    ///
    /// # Arguments
    /// * `storage` - 存储实例
    /// * `txn_manager` - 事务管理器
    /// * `txn_id` - 事务ID
    /// * `auto_commit` - 是否在 Drop 时自动提交（如果未显式提交）
    ///
    /// # 注意
    /// 如果 auto_commit 为 false，则在 Drop 时自动中止事务。
    /// 如果 auto_commit 为 true，则在 Drop 时尝试提交事务（如果未显式提交）。
    pub fn new(
        storage: &'a RedbStorage,
        txn_manager: &'a TransactionManager,
        txn_id: TransactionId,
        auto_commit: bool,
    ) -> Result<Self, TransactionError> {
        let ctx = txn_manager.get_context(txn_id)?;
        storage.set_transaction_context(Some(ctx.clone()));

        Ok(Self {
            storage,
            txn_manager,
            txn_id,
            auto_commit,
            committed: false,
            context: ctx,
        })
    }

    /// 显式提交事务
    ///
    /// # Returns
    /// * `Ok(())` - 提交成功
    /// * `Err(TransactionError)` - 提交失败
    ///
    /// # 注意
    /// 调用此方法后，committed 标志被设置，Drop 时不会再次提交或中止。
    pub fn commit(mut self) -> Result<(), TransactionError> {
        self.txn_manager
            .commit_transaction(self.txn_id)
            .map_err(|e| {
                // 清除事务上下文
                self.storage.set_transaction_context(None);
                e
            })?;

        self.committed = true;
        Ok(())
    }

    /// 显式中止事务
    ///
    /// # Returns
    /// * `Ok(())` - 中止成功
    /// * `Err(TransactionError)` - 中止失败
    ///
    /// # 注意
    /// 调用此方法后，committed 标志被设置，Drop 时不会再次提交或中止。
    pub fn abort(mut self) -> Result<(), TransactionError> {
        self.txn_manager
            .abort_transaction(self.txn_id)
            .map_err(|e| {
                // 清除事务上下文
                self.storage.set_transaction_context(None);
                e
            })?;

        self.committed = true;
        Ok(())
    }

    /// 获取事务ID
    pub fn txn_id(&self) -> TransactionId {
        self.txn_id
    }

    /// 检查事务是否已提交
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// 获取事务上下文
    pub fn context(&self) -> Result<Arc<TransactionContext>, TransactionError> {
        self.txn_manager.get_context(self.txn_id)
    }
}

impl<'a> Deref for TransactionContextGuard<'a> {
    type Target = TransactionContext;

    fn deref(&self) -> &Self::Target {
        self.context.as_ref()
    }
}

impl<'a> Drop for TransactionContextGuard<'a> {
    fn drop(&mut self) {
        // 如果已经提交或中止，直接返回
        if self.committed {
            return;
        }

        // 清除事务上下文
        self.storage.set_transaction_context(None);

        // 根据 auto_commit 标志决定是提交还是中止
        if self.auto_commit {
            // 尝试自动提交
            if let Err(e) = self.txn_manager.commit_transaction(self.txn_id) {
                log::error!("自动提交事务失败: {}", e);
                // 提交失败，尝试中止
                let _ = self.txn_manager.abort_transaction(self.txn_id);
            }
        } else {
            // 自动中止
            if let Err(e) = self.txn_manager.abort_transaction(self.txn_id) {
                log::error!("自动中止事务失败: {}", e);
            }
        }
    }
}

/// 事务作用域执行器
///
/// 提供类似 Rust 作用块的自动事务管理
///
/// # 示例
/// ```rust
/// let result = TransactionScope::execute(storage, txn_manager, options, |ctx| {
///     // 在事务中执行操作
///     let id = client.insert_vertex("space", vertex)?;
///     client.insert_edge("space", edge)?;
///     Ok(id)
/// });
/// ```
pub struct TransactionScope;

impl TransactionScope {
    /// 在事务作用域中执行操作
    ///
    /// # Arguments
    /// * `storage` - 存储实例
    /// * `txn_manager` - 事务管理器
    /// * `options` - 事务选项
    /// * `f` - 闭包，接收事务上下文守卫并执行操作
    ///
    /// # Returns
    /// * `Ok(R)` - 操作成功返回的结果
    /// * `Err(TransactionError)` - 操作失败返回的错误
    ///
    /// # 注意
    /// 如果闭包返回 Ok，则自动提交事务。
    /// 如果闭包返回 Err，则自动中止事务。
    pub fn execute<F, R>(
        storage: &RedbStorage,
        txn_manager: &TransactionManager,
        options: crate::transaction::TransactionOptions,
        f: F,
    ) -> Result<R, TransactionError>
    where
        F: FnOnce(&mut TransactionContextGuard) -> Result<R, TransactionError>,
    {
        let txn_id = txn_manager.begin_transaction(options)?;

        let mut guard = TransactionContextGuard::new(storage, txn_manager, txn_id, false)?;

        match f(&mut guard) {
            Ok(result) => {
                // 操作成功，提交事务
                guard.commit()?;
                Ok(result)
            }
            Err(e) => {
                // 操作失败，中止事务
                let _ = guard.abort();
                Err(e)
            }
        }
    }

    /// 在事务作用域中执行操作（自动提交）
    ///
    /// # Arguments
    /// * `storage` - 存储实例
    /// * `txn_manager` - 事务管理器
    /// * `options` - 事务选项
    /// * `f` - 闭包，接收事务上下文守卫并执行操作
    ///
    /// # Returns
    /// * `Ok(R)` - 操作成功返回的结果
    /// * `Err(TransactionError)` - 操作失败返回的错误
    ///
    /// # 注意
    /// 无论闭包返回 Ok 还是 Err，都会尝试提交事务。
    /// 这适用于需要保留部分结果的场景。
    pub fn execute_always_commit<F, R>(
        storage: &RedbStorage,
        txn_manager: &TransactionManager,
        options: crate::transaction::TransactionOptions,
        f: F,
    ) -> Result<R, TransactionError>
    where
        F: FnOnce(&mut TransactionContextGuard) -> Result<R, TransactionError>,
    {
        let txn_id = txn_manager.begin_transaction(options)?;

        let mut guard = TransactionContextGuard::new(storage, txn_manager, txn_id, true)?;

        let result = f(&mut guard);

        // 尝试提交事务（即使操作失败）
        let _ = guard.commit();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_storage() -> (RedbStorage, Arc<TransactionManager>, TempDir) {
        use std::sync::atomic::{AtomicU64, Ordering};

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);

        let temp_dir = TempDir::new().expect("创建临时目录失败");
        let db_path = temp_dir.path().join(format!("test_{}.db", counter));

        let storage = RedbStorage::new_with_path(db_path.clone()).expect("创建存储失败");
        let db = Arc::clone(storage.get_db());

        let txn_manager = Arc::new(TransactionManager::new(db, Default::default()));
        (storage, txn_manager, temp_dir)
    }

    #[test]
    fn test_transaction_scope_commit_on_success() {
        let (storage, txn_manager, _temp) = create_test_storage();

        let options = crate::transaction::TransactionOptions::default();
        let result = TransactionScope::execute(&storage, &txn_manager, options, |_guard| {
            Ok::<i32, TransactionError>(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_transaction_scope_abort_on_error() {
        let (storage, txn_manager, _temp) = create_test_storage();

        let options = crate::transaction::TransactionOptions::default();
        let result = TransactionScope::execute(&storage, &txn_manager, options, |_guard| {
            Err::<i32, TransactionError>(TransactionError::Internal("测试错误".to_string()))
        });

        assert!(result.is_err());
    }
}
