//! TransactionContextGuard 测试
//!
//! 测试事务上下文 RAII 管理器的功能

use std::sync::Arc;

use tempfile::TempDir;

use crate::storage::transaction_context_guard::TransactionContextGuard;
use crate::storage::RedbStorage;
use crate::transaction::{
    TransactionManager, TransactionOptions, TransactionState, TransactionError,
};

/// 创建测试存储和管理器
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
fn test_transaction_context_guard_creation() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false);
    assert!(guard.is_ok());

    let guard = guard.expect("创建守卫失败");
    assert_eq!(guard.txn_id(), txn_id);
    assert!(!guard.is_committed());
}

#[test]
fn test_transaction_context_guard_deref() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 通过 Deref 访问 TransactionContext 的方法
    assert_eq!(guard.id, txn_id);
    assert_eq!(guard.state(), TransactionState::Active);
}

#[test]
fn test_transaction_context_guard_commit() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    assert!(!guard.is_committed());

    guard.commit().expect("提交事务失败");

    // 注意：commit 消耗了 guard，所以这里无法再访问
    // 验证事务已提交
    assert!(!txn_manager.is_transaction_active(txn_id));
}

#[test]
fn test_transaction_context_guard_abort() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    assert!(!guard.is_committed());

    guard.abort().expect("中止事务失败");

    // 验证事务已中止
    assert!(!txn_manager.is_transaction_active(txn_id));
}

#[test]
fn test_transaction_context_guard_context() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    let context = guard.context().expect("获取事务上下文失败");
    assert_eq!(context.id, txn_id);
    assert_eq!(context.state(), TransactionState::Active);
}

#[test]
fn test_transaction_context_guard_drop_auto_abort() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    {
        let _guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
            .expect("创建守卫失败");

        assert!(txn_manager.is_transaction_active(txn_id));

        // guard 在此处离开作用域，应该自动中止
    }

    // 验证事务已中止
    assert!(!txn_manager.is_transaction_active(txn_id));
}

#[test]
fn test_transaction_context_guard_drop_auto_commit() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    {
        let _guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, true)
            .expect("创建守卫失败");

        assert!(txn_manager.is_transaction_active(txn_id));

        // guard 在此处离开作用域，应该自动提交
    }

    // 验证事务已提交
    assert!(!txn_manager.is_transaction_active(txn_id));
    assert_eq!(
        txn_manager
            .stats()
            .committed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_transaction_context_guard_drop_after_commit() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    {
        let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
            .expect("创建守卫失败");

        guard.commit().expect("提交事务失败");

        // guard 在此处离开作用域，但已经提交，不应该再次提交或中止
    }

    // 验证事务已提交
    assert!(!txn_manager.is_transaction_active(txn_id));
    assert_eq!(
        txn_manager
            .stats()
            .committed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_transaction_context_guard_drop_after_abort() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    {
        let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
            .expect("创建守卫失败");

        guard.abort().expect("中止事务失败");

        // guard 在此处离开作用域，但已经中止，不应该再次中止
    }

    // 验证事务已中止
    assert!(!txn_manager.is_transaction_active(txn_id));
    assert_eq!(
        txn_manager
            .stats()
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_transaction_context_guard_readonly() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let options = TransactionOptions::new().read_only();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始只读事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 验证是只读事务
    assert!(guard.read_only);
}

#[test]
fn test_transaction_context_guard_with_timeout() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let options = TransactionOptions::new()
        .with_timeout(std::time::Duration::from_secs(60));

    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 验证剩余时间
    let remaining = guard.remaining_time();
    assert!(remaining > std::time::Duration::from_secs(50));
}

#[test]
fn test_transaction_context_guard_multiple_guards() {
    let (storage, txn_manager, _temp) = create_test_storage();

    // 开始第一个事务
    let txn1 = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第一个事务失败");

    let guard1 = TransactionContextGuard::new(&storage, &txn_manager, txn1, false)
        .expect("创建第一个守卫失败");

    assert!(txn_manager.is_transaction_active(txn1));

    // 提交第一个事务
    guard1.commit().expect("提交第一个事务失败");

    // 现在可以开始第二个事务
    let txn2 = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第二个事务失败");

    let guard2 = TransactionContextGuard::new(&storage, &txn_manager, txn2, false)
        .expect("创建第二个守卫失败");

    assert!(txn_manager.is_transaction_active(txn2));

    // 提交第二个事务
    guard2.commit().expect("提交第二个事务失败");
}

#[test]
fn test_transaction_context_guard_error_on_commit() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 手动中止事务
    txn_manager
        .abort_transaction(txn_id)
        .expect("中止事务失败");

    // 尝试提交应该失败
    let result = guard.commit();
    assert!(result.is_err());
}

#[test]
fn test_transaction_context_guard_error_on_abort() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 手动提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 尝试中止应该失败
    let result = guard.abort();
    assert!(result.is_err());
}

#[test]
fn test_transaction_context_guard_context_cleanup() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    {
        let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
            .expect("创建守卫失败");

        // 验证事务上下文已设置
        let ctx = guard.context().expect("获取事务上下文失败");
        assert_eq!(ctx.id, txn_id);

        // guard 在此处离开作用域，应该清除事务上下文
    }

    // 验证事务上下文已被清除
    let context = storage.get_transaction_context();
    assert!(context.is_none());
}

#[test]
fn test_transaction_context_guard_with_operations() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 记录一些操作
    guard.record_table_modification("vertices");
    guard.record_table_modification("edges");

    let modified = guard.modified_tables();
    assert_eq!(modified.len(), 2);
    assert!(modified.contains(&"vertices".to_string()));
    assert!(modified.contains(&"edges".to_string()));

    // 提交事务
    guard.commit().expect("提交事务失败");
}

#[test]
fn test_transaction_context_guard_operation_log() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 添加操作日志
    guard.add_operation_log(crate::transaction::OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_state: None,
    });

    assert_eq!(guard.operation_log_len(), 1);

    // 提交事务
    guard.commit().expect("提交事务失败");
}

#[test]
fn test_transaction_context_guard_state_transitions() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 初始状态
    assert_eq!(guard.state(), TransactionState::Active);

    // 转换到 Prepared 状态
    guard.transition_to(TransactionState::Prepared)
        .expect("状态转换失败");
    assert_eq!(guard.state(), TransactionState::Prepared);

    // 转换到 Committing 状态
    guard.transition_to(TransactionState::Committing)
        .expect("状态转换失败");
    assert_eq!(guard.state(), TransactionState::Committing);

    // 转换到 Committed 状态
    guard.transition_to(TransactionState::Committed)
        .expect("状态转换失败");
    assert_eq!(guard.state(), TransactionState::Committed);
}

#[test]
fn test_transaction_context_guard_invalid_state_transition() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 尝试无效的状态转换
    let result = guard.transition_to(TransactionState::Committed);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::InvalidStateTransition { .. }
    ));
}

#[test]
fn test_transaction_context_guard_can_execute() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // Active 状态可以执行
    assert!(guard.can_execute().is_ok());

    // 转换到 Prepared 状态
    guard.transition_to(TransactionState::Prepared)
        .expect("状态转换失败");

    // Prepared 状态不能执行
    assert!(guard.can_execute().is_err());
}

#[test]
fn test_transaction_context_guard_info() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 记录一些修改
    guard.record_table_modification("vertices");

    let info = guard.info();
    assert_eq!(info.id, txn_id);
    assert_eq!(info.state, TransactionState::Active);
    assert!(!info.is_read_only);
    assert_eq!(info.modified_tables.len(), 1);
    assert!(info.modified_tables.contains(&"vertices".to_string()));
}

#[test]
fn test_transaction_context_guard_with_write_txn() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 使用写事务执行操作
    let result = guard.with_write_txn(|_txn| {
        Ok::<(), crate::core::StorageError>(())
    });

    assert!(result.is_ok());

    // 提交事务
    guard.commit().expect("提交事务失败");
}

#[test]
fn test_transaction_context_guard_readonly_with_write_txn() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let options = TransactionOptions::new().read_only();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始只读事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 只读事务不能使用 with_write_txn
    let result = guard.with_write_txn(|_txn| {
        Ok::<(), crate::core::StorageError>(())
    });

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::ReadOnlyTransaction
    ));
}

#[test]
fn test_transaction_context_guard_readonly_with_read_txn() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let options = TransactionOptions::new().read_only();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始只读事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    // 只读事务可以使用 with_read_txn
    let result = guard.with_read_txn(|_txn| {
        Ok::<(), crate::core::StorageError>(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_transaction_context_guard_auto_commit_on_error() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    {
        let _guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, true)
            .expect("创建守卫失败");

        assert!(txn_manager.is_transaction_active(txn_id));

        // guard 在此处离开作用域，应该尝试自动提交
        // 即使有错误，也会尝试提交（但可能会失败）
    }

    // 验证事务已提交（或中止，取决于提交是否成功）
    assert!(!txn_manager.is_transaction_active(txn_id));
}

#[test]
fn test_transaction_context_guard_committed_flag() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    assert!(!guard.is_committed());

    guard.commit().expect("提交事务失败");

    // 注意：commit 消耗了 guard，所以这里无法再访问 is_committed
}

#[test]
fn test_transaction_context_guard_aborted_flag() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    assert!(!guard.is_committed());

    guard.abort().expect("中止事务失败");

    // 注意：abort 消耗了 guard，所以这里无法再访问 is_committed
}

#[test]
fn test_transaction_context_guard_with_two_phase_commit() {
    let (storage, txn_manager, _temp) = create_test_storage();

    let options = TransactionOptions::new()
        .with_two_phase_commit()
        .with_durability(crate::transaction::types::DurabilityLevel::Immediate);

    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    let guard = TransactionContextGuard::new(&storage, &txn_manager, txn_id, false)
        .expect("创建守卫失败");

    assert!(guard.two_phase_commit);
    assert_eq!(guard.durability, crate::transaction::types::DurabilityLevel::Immediate);

    guard.commit().expect("提交事务失败");
}
