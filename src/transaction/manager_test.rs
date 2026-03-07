//! TransactionManager 测试
//!
//! 测试事务管理器的功能，包括事务生命周期管理、并发控制、超时处理等

use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use crate::transaction::manager::TransactionManager;
use crate::transaction::types::{
    DurabilityLevel, TransactionError, TransactionOptions, TransactionState,
};

/// 创建测试数据库和管理器
fn create_test_manager() -> (TransactionManager, Arc<redb::Database>, TempDir) {
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("创建测试数据库失败"),
    );

    let config = crate::transaction::types::TransactionManagerConfig {
        auto_cleanup: false,
        ..Default::default()
    };

    let manager = TransactionManager::new(db.clone(), config);
    (manager, db, temp_dir)
}

#[test]
fn test_transaction_manager_creation() {
    let (manager, _db, _temp) = create_test_manager();

    // 验证管理器配置
    let config = manager.config();
    assert_eq!(config.max_concurrent_transactions, 1000);
    assert!(!config.auto_cleanup);
}

#[test]
fn test_begin_write_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::default();
    let txn_id = manager
        .begin_transaction(options)
        .expect("开始事务失败");

    assert!(manager.is_transaction_active(txn_id));

    let context = manager
        .get_context(txn_id)
        .expect("获取事务上下文失败");
    assert_eq!(context.id, txn_id);
    assert_eq!(context.state(), TransactionState::Active);
    assert!(!context.read_only);
}

#[test]
fn test_begin_readonly_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new().read_only();
    let txn_id = manager
        .begin_transaction(options)
        .expect("开始只读事务失败");

    assert!(manager.is_transaction_active(txn_id));

    let context = manager
        .get_context(txn_id)
        .expect("获取事务上下文失败");
    assert_eq!(context.id, txn_id);
    assert!(context.read_only);
}

#[test]
fn test_begin_transaction_with_timeout() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new()
        .with_timeout(Duration::from_secs(60))
        .with_durability(DurabilityLevel::None);

    let txn_id = manager
        .begin_transaction(options)
        .expect("开始事务失败");

    let context = manager
        .get_context(txn_id)
        .expect("获取事务上下文失败");
    assert!(context.remaining_time() > Duration::from_secs(50));
}

#[test]
fn test_commit_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    assert!(manager.is_transaction_active(txn_id));

    manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    assert!(!manager.is_transaction_active(txn_id));

    // 验证统计信息
    let stats = manager.stats();
    assert_eq!(
        stats.committed_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_abort_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    assert!(manager.is_transaction_active(txn_id));

    manager
        .abort_transaction(txn_id)
        .expect("中止事务失败");

    assert!(!manager.is_transaction_active(txn_id));

    // 验证统计信息
    let stats = manager.stats();
    assert_eq!(
        stats.aborted_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_commit_readonly_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new().read_only();
    let txn_id = manager
        .begin_transaction(options)
        .expect("开始只读事务失败");

    manager
        .commit_transaction(txn_id)
        .expect("提交只读事务失败");

    assert!(!manager.is_transaction_active(txn_id));
}

#[test]
fn test_get_transaction_not_found() {
    let (manager, _db, _temp) = create_test_manager();

    let result = manager.get_context(9999);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(9999))
    ));
}

#[test]
fn test_commit_transaction_not_found() {
    let (manager, _db, _temp) = create_test_manager();

    let result = manager.commit_transaction(9999);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[test]
fn test_abort_transaction_not_found() {
    let (manager, _db, _temp) = create_test_manager();

    let result = manager.abort_transaction(9999);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[test]
fn test_commit_already_committed_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    manager
        .commit_transaction(txn_id)
        .expect("第一次提交失败");

    // 再次提交应该失败
    let result = manager.commit_transaction(txn_id);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[test]
fn test_abort_already_aborted_transaction() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    manager
        .abort_transaction(txn_id)
        .expect("第一次中止失败");

    // 再次中止应该失败
    let result = manager.abort_transaction(txn_id);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionNotFound(_))
    ));
}

#[test]
fn test_write_transaction_conflict() {
    let (manager, _db, _temp) = create_test_manager();

    // 开始第一个写事务
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第一个事务失败");

    // 尝试开始第二个写事务应该失败
    let result = manager.begin_transaction(TransactionOptions::default());
    assert!(matches!(
        result,
        Err(TransactionError::WriteTransactionConflict)
    ));

    // 提交第一个事务
    manager
        .commit_transaction(txn1)
        .expect("提交第一个事务失败");

    // 现在可以开始新的事务
    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第二个事务失败");

    manager
        .commit_transaction(txn2)
        .expect("提交第二个事务失败");
}

#[test]
fn test_multiple_readonly_transactions() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new().read_only();

    // 可以同时有多个只读事务
    let txn1 = manager
        .begin_transaction(options.clone())
        .expect("开始第一个只读事务失败");
    let txn2 = manager
        .begin_transaction(options.clone())
        .expect("开始第二个只读事务失败");
    let txn3 = manager
        .begin_transaction(options)
        .expect("开始第三个只读事务失败");

    assert!(manager.is_transaction_active(txn1));
    assert!(manager.is_transaction_active(txn2));
    assert!(manager.is_transaction_active(txn3));

    // 提交所有只读事务
    manager
        .commit_transaction(txn1)
        .expect("提交第一个只读事务失败");
    manager
        .commit_transaction(txn2)
        .expect("提交第二个只读事务失败");
    manager
        .commit_transaction(txn3)
        .expect("提交第三个只读事务失败");
}

#[test]
fn test_sequential_write_transactions() {
    let (manager, _db, _temp) = create_test_manager();

    // 第一个事务
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第一个事务失败");
    manager
        .commit_transaction(txn1)
        .expect("提交第一个事务失败");

    // 第二个事务
    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第二个事务失败");
    manager
        .abort_transaction(txn2)
        .expect("中止第二个事务失败");

    // 第三个事务
    let txn3 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第三个事务失败");
    manager
        .commit_transaction(txn3)
        .expect("提交第三个事务失败");

    // 验证统计信息
    let stats = manager.stats();
    assert_eq!(
        stats.committed_transactions.load(std::sync::atomic::Ordering::Relaxed),
        2
    );
    assert_eq!(
        stats.aborted_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_transaction_timeout() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new()
        .with_timeout(Duration::from_millis(50));

    let txn_id = manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 等待事务超时
    std::thread::sleep(Duration::from_millis(100));

    // 提交超时的事务应该失败
    let result = manager.commit_transaction(txn_id);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionTimeout)
    ));

    // 验证统计信息
    let stats = manager.stats();
    assert_eq!(
        stats.timeout_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_list_active_transactions() {
    let (manager, _db, _temp) = create_test_manager();

    // 开始几个事务
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第一个事务失败");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("开始第二个事务失败");

    // 列出活跃事务
    let active_txns = manager.list_active_transactions();
    assert_eq!(active_txns.len(), 2);

    // 提交一个事务
    manager
        .commit_transaction(txn1)
        .expect("提交事务失败");

    // 再次列出活跃事务
    let active_txns = manager.list_active_transactions();
    assert_eq!(active_txns.len(), 1);

    // 清理
    manager
        .commit_transaction(txn2)
        .expect("提交事务失败");
}

#[test]
fn test_get_transaction_info() {
    let (manager, _db, _temp) = create_test_manager();

    let txn_id = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let info = manager
        .get_transaction_info(txn_id)
        .expect("获取事务信息失败");

    assert_eq!(info.id, txn_id);
    assert_eq!(info.state, TransactionState::Active);
    assert!(!info.is_read_only);

    manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 提交后事务信息应该不存在
    let info = manager.get_transaction_info(txn_id);
    assert!(info.is_none());
}

#[test]
fn test_max_concurrent_transactions() {
    let config = crate::transaction::types::TransactionManagerConfig {
        max_concurrent_transactions: 2,
        auto_cleanup: false,
        ..Default::default()
    };

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("创建测试数据库失败"),
    );

    let manager = TransactionManager::new(db, config);

    // 开始第一个只读事务
    let txn1 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("开始第一个事务失败");

    // 开始第二个只读事务
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("开始第二个事务失败");

    // 尝试开始第三个事务应该失败（超过最大并发数）
    let result = manager.begin_transaction(TransactionOptions::new().read_only());
    assert!(matches!(
        result,
        Err(TransactionError::TooManyTransactions)
    ));

    // 清理
    manager
        .commit_transaction(txn1)
        .expect("提交第一个事务失败");
    manager
        .commit_transaction(txn2)
        .expect("提交第二个事务失败");
}

#[test]
fn test_transaction_stats() {
    let (manager, _db, _temp) = create_test_manager();

    let stats = manager.stats();

    // 初始统计
    assert_eq!(
        stats.total_transactions.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats.active_transactions.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats.committed_transactions.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats.aborted_transactions.load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // 开始一个事务
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    assert_eq!(
        stats.total_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        stats.active_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // 提交事务
    manager
        .commit_transaction(txn1)
        .expect("提交事务失败");

    assert_eq!(
        stats.active_transactions.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        stats.committed_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // 开始并中止另一个事务
    let txn2 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    manager
        .abort_transaction(txn2)
        .expect("中止事务失败");

    assert_eq!(
        stats.aborted_transactions.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_cleanup_expired_transactions() {
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db"))
            .expect("创建测试数据库失败"),
    );

    let config = crate::transaction::types::TransactionManagerConfig {
        auto_cleanup: false,
        ..Default::default()
    };

    let manager = TransactionManager::new(db.clone(), config);

    // 开始一个短超时的事务
    let txn1 = manager
        .begin_transaction(
            TransactionOptions::new()
                .with_timeout(Duration::from_millis(50)),
        )
        .expect("开始事务失败");

    // 等待第一个事务超时
    std::thread::sleep(Duration::from_millis(100));

    // 清理过期事务
    manager.cleanup_expired_transactions();

    // 第一个事务应该被清理
    assert!(!manager.is_transaction_active(txn1));
}

#[test]
fn test_transaction_with_two_phase_commit() {
    let (manager, _db, _temp) = create_test_manager();

    let options = TransactionOptions::new()
        .with_two_phase_commit()
        .with_durability(DurabilityLevel::Immediate);

    let txn_id = manager
        .begin_transaction(options)
        .expect("开始两阶段提交事务失败");

    let context = manager
        .get_context(txn_id)
        .expect("获取事务上下文失败");

    assert!(context.two_phase_commit);
    assert_eq!(context.durability, DurabilityLevel::Immediate);

    manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

#[test]
fn test_shutdown_manager() {
    let (manager, _db, _temp) = create_test_manager();

    // 开始几个事务
    let txn1 = manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始第一个事务失败");
    let txn2 = manager
        .begin_transaction(TransactionOptions::new().read_only())
        .expect("开始第二个事务失败");

    // 关闭管理器
    manager.shutdown();

    // 所有事务应该被中止
    assert!(!manager.is_transaction_active(txn1));
    assert!(!manager.is_transaction_active(txn2));

    // 关闭后不能开始新事务
    let result = manager.begin_transaction(TransactionOptions::default());
    assert!(matches!(
        result,
        Err(TransactionError::Internal(_))
    ));
}
