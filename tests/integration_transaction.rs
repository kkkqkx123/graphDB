//! 事务功能集成测试
//!
//! 测试事务管理器的核心功能，包括：
//! - 事务生命周期管理（开始、提交、中止）
//! - 事务隔离性
//! - 并发事务处理
//! - 事务与存储层的集成

mod common;

use std::sync::Arc;
use std::time::Duration;

use graphdb::storage::RedbStorage;
use graphdb::transaction::{
    TransactionManager, TransactionManagerConfig, TransactionOptions, TransactionState,
};

/// 创建测试用事务管理器
fn create_test_transaction_manager() -> Arc<TransactionManager> {
    use redb::Database;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_txn.db");
    let db = Arc::new(Database::create(db_path).expect("创建数据库失败"));

    let config = TransactionManagerConfig {
        default_timeout: Duration::from_secs(30),
        max_concurrent_transactions: 1000,
    };

    Arc::new(TransactionManager::new(db, config))
}

/// 测试事务生命周期
#[test]
fn test_transaction_lifecycle() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager.begin_transaction(options).expect("开始事务失败");

    // 检查事务状态
    let txn = txn_manager.get_transaction(txn_id).expect("获取事务失败");
    assert_eq!(txn.state(), TransactionState::Active);

    // 提交事务
    txn_manager.commit_transaction(txn_id).expect("提交事务失败");

    // 检查事务状态
    let txn = txn_manager.get_transaction(txn_id).expect("获取事务失败");
    assert_eq!(txn.state(), TransactionState::Committed);
}

/// 测试事务回滚
#[test]
fn test_transaction_rollback() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager.begin_transaction(options).expect("开始事务失败");

    // 回滚事务
    txn_manager.abort_transaction(txn_id).expect("回滚事务失败");

    // 检查事务状态
    let txn = txn_manager.get_transaction(txn_id).expect("获取事务失败");
    assert_eq!(txn.state(), TransactionState::Aborted);
}

/// 测试只读事务
#[test]
fn test_read_only_transaction() {
    let txn_manager = create_test_transaction_manager();

    // 开始只读事务
    let options = TransactionOptions {
        read_only: true,
        timeout: Duration::from_secs(30),
        durability: graphdb::transaction::DurabilityLevel::None,
    };
    let txn_id = txn_manager.begin_transaction(options).expect("开始事务失败");

    // 检查事务是否为只读
    let txn = txn_manager.get_transaction(txn_id).expect("获取事务失败");
    assert!(txn.read_only());

    // 提交事务
    txn_manager.commit_transaction(txn_id).expect("提交事务失败");
}

/// 测试事务超时
#[test]
fn test_transaction_timeout() {
    let txn_manager = create_test_transaction_manager();

    // 开始一个超时时间很短的事务
    let options = TransactionOptions {
        read_only: false,
        timeout: Duration::from_millis(100),
        durability: graphdb::transaction::DurabilityLevel::None,
    };
    let txn_id = txn_manager.begin_transaction(options).expect("开始事务失败");

    // 等待超时
    std::thread::sleep(Duration::from_millis(150));

    // 检查事务状态（应该被自动中止）
    let txn = txn_manager.get_transaction(txn_id).expect("获取事务失败");
    assert_eq!(txn.state(), TransactionState::Aborted);
}

/// 测试并发事务
#[test]
fn test_concurrent_transactions() {
    let txn_manager = create_test_transaction_manager();

    // 开始多个事务
    let mut txn_ids = Vec::new();
    for _ in 0..10 {
        let options = TransactionOptions::default();
        let txn_id = txn_manager.begin_transaction(options).expect("开始事务失败");
        txn_ids.push(txn_id);
    }

    // 提交所有事务
    for txn_id in txn_ids {
        txn_manager.commit_transaction(txn_id).expect("提交事务失败");
    }

    // 验证所有事务都已提交
    let stats = txn_manager.get_stats();
    assert_eq!(stats.committed_transactions, 10);
}

/// 测试事务与存储层的集成
#[test]
fn test_transaction_with_storage() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_storage.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager.begin_transaction(options).expect("开始事务失败");

    // 提交事务
    txn_manager.commit_transaction(txn_id).expect("提交事务失败");
}

/// 测试事务统计信息
#[test]
fn test_transaction_stats() {
    let txn_manager = create_test_transaction_manager();

    // 开始并提交一个事务
    let options = TransactionOptions::default();
    let txn_id1 = txn_manager.begin_transaction(options).expect("开始事务失败");
    txn_manager.commit_transaction(txn_id1).expect("提交事务失败");

    // 开始并回滚一个事务
    let options = TransactionOptions::default();
    let txn_id2 = txn_manager.begin_transaction(options).expect("开始事务失败");
    txn_manager.abort_transaction(txn_id2).expect("回滚事务失败");

    // 检查统计信息
    let stats = txn_manager.get_stats();
    assert_eq!(stats.committed_transactions, 1);
    assert_eq!(stats.aborted_transactions, 1);
}
