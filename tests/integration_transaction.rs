//! 事务功能集成测试
//!
//! 测试事务管理器的核心功能，包括：
//! - 事务生命周期管理（开始、提交、中止）
//! - 保存点功能
//! - 两阶段提交
//! - 事务隔离性
//! - 并发事务处理
//! - 事务与存储层的集成

mod common;

use std::sync::Arc;
use std::time::Duration;

use graphdb::transaction::{
    SavepointManager, TransactionManager, TransactionManagerConfig, TransactionOptions, TwoPhaseCoordinator,
    TransactionState,
};
use graphdb::storage::transactional_storage::TransactionalStorage;

/// 创建测试用事务管理器
fn create_test_transaction_manager() -> Arc<TransactionManager> {
    use tempfile::TempDir;
    use redb::Database;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_txn.db");
    let db = Arc::new(Database::create(db_path).expect("创建数据库失败"));

    let config = TransactionManagerConfig {
        default_timeout: Duration::from_secs(30),
        max_concurrent_transactions: 1000,
        enable_2pc: false,
        deadlock_detection_interval: Duration::from_secs(5),
        auto_cleanup: true,
        cleanup_interval: Duration::from_secs(10),
    };

    Arc::new(TransactionManager::new(db, config))
}

/// 创建测试用保存点管理器
fn create_test_savepoint_manager() -> Arc<SavepointManager> {
    Arc::new(SavepointManager::new())
}

/// 创建测试用2PC协调器
fn create_test_two_phase_coordinator() -> Arc<TwoPhaseCoordinator> {
    Arc::new(TwoPhaseCoordinator::new(Duration::from_secs(30)))
}

// ==================== 事务生命周期测试 ====================

#[test]
fn test_transaction_lifecycle_commit() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    // 验证事务状态
    let info = txn_manager.get_transaction_info(txn_id).expect("获取事务信息失败");
    assert!(matches!(info.state, TransactionState::Active), "事务应该是活跃的");

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 验证事务已结束（不在活跃列表中）
    let info = txn_manager.get_transaction_info(txn_id);
    assert!(info.is_none(), "事务应该已不在活跃列表中");
}

#[test]
fn test_transaction_lifecycle_abort() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    // 中止事务
    txn_manager
        .abort_transaction(txn_id)
        .expect("中止事务失败");

    // 验证事务已结束
    let info = txn_manager.get_transaction_info(txn_id);
    assert!(info.is_none(), "事务应该已不在活跃列表中");
}

#[test]
fn test_readonly_transaction() {
    let txn_manager = create_test_transaction_manager();

    let options = TransactionOptions::default().read_only();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始只读事务失败");

    let info = txn_manager
        .get_transaction_info(txn_id)
        .expect("获取事务信息失败");
    assert!(info.is_read_only, "事务应该是只读的");

    // 只读事务可以直接提交
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交只读事务失败");
}

// ==================== 保存点功能测试 ====================

#[test]
fn test_savepoint_basic_operations() {
    let savepoint_manager = create_test_savepoint_manager();
    let txn_manager = create_test_transaction_manager();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    // 创建保存点
    let sp1 = savepoint_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    let sp2 = savepoint_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // 验证保存点存在（通过活跃保存点列表）
    let active_sps = savepoint_manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 2, "应该有2个活跃保存点");

    // 按名称查找保存点
    let found = savepoint_manager.find_savepoint_by_name(txn_id, "sp1");
    assert_eq!(found, Some(sp1), "应该能找到保存点1");

    // 回滚到保存点1（这会释放sp2）
    savepoint_manager
        .rollback_to_savepoint(sp1)
        .expect("回滚到保存点失败");

    // 验证sp2已被释放
    let active_sps = savepoint_manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 1, "应该只有1个活跃保存点");

    // 验证sp1仍然存在
    let found = savepoint_manager.find_savepoint_by_name(txn_id, "sp1");
    assert_eq!(found, Some(sp1), "保存点1应该仍然存在");
}

#[test]
fn test_savepoint_nested_rollback() {
    let savepoint_manager = create_test_savepoint_manager();
    let txn_manager = create_test_transaction_manager();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    // 创建嵌套保存点
    let _sp1 = savepoint_manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");
    let sp2 = savepoint_manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");
    let _sp3 = savepoint_manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");

    // 回滚到中间保存点
    savepoint_manager
        .rollback_to_savepoint(sp2)
        .expect("回滚到保存点失败");

    // 验证活跃保存点数量
    let active_sps = savepoint_manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 2, "应该有2个活跃保存点");
}

// ==================== 两阶段提交测试 ====================

#[test]
fn test_two_phase_commit_success() {
    let coordinator = create_test_two_phase_coordinator();
    let txn_manager = create_test_transaction_manager();

    // 开始一个事务
    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    // 开始2PC
    let participant_ids = vec!["resource1".to_string(), "resource2".to_string()];
    let two_phase_id = coordinator
        .begin_two_phase(txn_id, participant_ids, None)
        .expect("开始2PC失败");

    // 所有参与者投票准备
    coordinator
        .record_vote(two_phase_id, "resource1", graphdb::transaction::ParticipantVote::Ready)
        .expect("记录投票失败");
    coordinator
        .record_vote(two_phase_id, "resource2", graphdb::transaction::ParticipantVote::Ready)
        .expect("记录投票失败");

    // 验证可以提交
    assert!(coordinator.can_commit(two_phase_id), "应该可以提交");

    // 标记提交
    coordinator
        .mark_committing(two_phase_id)
        .expect("标记提交中失败");
    coordinator
        .mark_committed(two_phase_id)
        .expect("标记已提交失败");

    // 验证状态
    let txn = coordinator
        .get_transaction(two_phase_id)
        .expect("获取事务失败");
    assert!(
        matches!(txn.state, graphdb::transaction::TwoPhaseState::Committed),
        "事务应该已提交"
    );
}

#[test]
fn test_two_phase_commit_abort() {
    let coordinator = create_test_two_phase_coordinator();
    let txn_manager = create_test_transaction_manager();

    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    let participant_ids = vec!["resource1".to_string(), "resource2".to_string()];
    let two_phase_id = coordinator
        .begin_two_phase(txn_id, participant_ids, None)
        .expect("开始2PC失败");

    // 一个参与者投票准备，另一个投票中止
    coordinator
        .record_vote(two_phase_id, "resource1", graphdb::transaction::ParticipantVote::Ready)
        .expect("记录投票失败");
    coordinator
        .record_vote(two_phase_id, "resource2", graphdb::transaction::ParticipantVote::Abort)
        .expect("记录投票失败");

    // 验证不能提交，但可以中止
    assert!(!coordinator.can_commit(two_phase_id), "不应该可以提交");
    assert!(coordinator.can_abort(two_phase_id), "应该可以中止");

    // 中止事务
    coordinator
        .mark_aborting(two_phase_id)
        .expect("标记中止中失败");
    coordinator
        .mark_aborted(two_phase_id)
        .expect("标记已中止失败");

    // 验证状态
    let txn = coordinator
        .get_transaction(two_phase_id)
        .expect("获取事务失败");
    assert!(
        matches!(txn.state, graphdb::transaction::TwoPhaseState::Aborted),
        "事务应该已中止"
    );
}

// ==================== 并发事务测试 ====================

#[test]
fn test_concurrent_transactions() {
    use std::thread;

    let txn_manager = create_test_transaction_manager();
    let mut handles = vec![];

    // 启动多个并发只读事务（避免写冲突）
    for i in 0..5 {
        let manager = Arc::clone(&txn_manager);
        let handle = thread::spawn(move || {
            // 使用只读事务避免写冲突
            let options = TransactionOptions::default().read_only();
            let txn_id = manager
                .begin_transaction(options)
                .expect(&format!("线程{}开始事务失败", i));

            // 模拟一些工作
            std::thread::sleep(Duration::from_millis(10));

            manager
                .commit_transaction(txn_id)
                .expect(&format!("线程{}提交事务失败", i));

            txn_id
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    let mut txn_ids = vec![];
    for handle in handles {
        txn_ids.push(handle.join().expect("线程执行失败"));
    }

    // 验证所有事务都已提交（不在活跃列表中）
    for txn_id in txn_ids {
        let info = txn_manager.get_transaction_info(txn_id);
        assert!(info.is_none(), "事务应该已不在活跃列表中");
    }
}

// ==================== 事务超时测试 ====================

#[test]
fn test_transaction_timeout() {
    use tempfile::TempDir;
    use redb::Database;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_timeout.db");
    let db = Arc::new(Database::create(db_path).expect("创建数据库失败"));

    let config = TransactionManagerConfig {
        default_timeout: Duration::from_millis(100),
        max_concurrent_transactions: 1000,
        enable_2pc: false,
        deadlock_detection_interval: Duration::from_secs(5),
        auto_cleanup: false, // 禁用自动清理以便测试
        cleanup_interval: Duration::from_secs(10),
    };
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 等待超时
    std::thread::sleep(Duration::from_millis(200));

    // 尝试提交应该失败（事务已过期）
    let result = txn_manager.commit_transaction(txn_id);
    assert!(result.is_err(), "过期事务不应该能提交");
}

// ==================== 事务统计测试 ====================

#[test]
fn test_transaction_stats() {
    let txn_manager = create_test_transaction_manager();

    // 记录初始统计
    let initial_stats = txn_manager.stats();
    let initial_total = initial_stats.total_transactions.load(std::sync::atomic::Ordering::Relaxed);

    // 开始并提交一些事务
    for _ in 0..3 {
        let txn_id = txn_manager
            .begin_transaction(TransactionOptions::default())
            .expect("开始事务失败");
        txn_manager
            .commit_transaction(txn_id)
            .expect("提交事务失败");
    }

    // 开始并中止一些事务
    for _ in 0..2 {
        let txn_id = txn_manager
            .begin_transaction(TransactionOptions::default())
            .expect("开始事务失败");
        txn_manager
            .abort_transaction(txn_id)
            .expect("中止事务失败");
    }

    // 验证统计
    let stats = txn_manager.stats();
    let total = stats.total_transactions.load(std::sync::atomic::Ordering::Relaxed);
    let committed = stats.committed_transactions.load(std::sync::atomic::Ordering::Relaxed);
    let aborted = stats.aborted_transactions.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(total, initial_total + 5, "总事务数应该增加5");
    assert_eq!(committed, 3, "已提交事务数应该是3");
    assert_eq!(aborted, 2, "已中止事务数应该是2");
}

// ==================== 事务与存储层集成测试 ====================

#[test]
fn test_transactional_storage_integration() {
    use tempfile::TempDir;
    use graphdb::storage::redb_storage::RedbStorage;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test.db");

    // 创建存储和事务管理器
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let txn_manager = create_test_transaction_manager();

    // 创建事务感知存储
    let transactional_storage =
        TransactionalStorage::new(storage, Arc::clone(&txn_manager));

    // 在事务中执行操作
    let result = transactional_storage.execute_in_transaction(
        TransactionOptions::default(),
        |_client| {
            // 这里可以执行存储操作
            // 例如：client.insert_vertex(...)
            Ok(42) // 返回一个测试值
        },
    );

    assert_eq!(result.expect("执行事务失败"), 42, "应该返回正确的值");

    // 验证事务已提交
    let stats = txn_manager.stats();
    let committed = stats.committed_transactions.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(committed, 1, "应该有1个已提交的事务");
}

#[test]
fn test_transactional_storage_rollback() {
    use tempfile::TempDir;
    use graphdb::storage::redb_storage::RedbStorage;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_rollback.db");

    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let txn_manager = create_test_transaction_manager();

    let transactional_storage =
        TransactionalStorage::new(storage, Arc::clone(&txn_manager));

    // 在事务中执行会失败的操作
    let result: Result<i32, _> = transactional_storage.execute_in_transaction(
        TransactionOptions::default(),
        |_client| {
            // 模拟操作失败
            Err(graphdb::storage::StorageError::DbError("测试错误".to_string()))
        },
    );

    assert!(result.is_err(), "操作应该失败");

    // 验证事务已中止
    let stats = txn_manager.stats();
    let aborted = stats.aborted_transactions.load(std::sync::atomic::Ordering::Relaxed);
    let committed = stats.committed_transactions.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(aborted, 1, "应该有1个已中止的事务");
    assert_eq!(committed, 0, "不应该有已提交的事务");
}

// ==================== 综合场景测试 ====================

#[test]
fn test_complex_transaction_scenario() {
    let txn_manager = create_test_transaction_manager();
    let savepoint_manager = create_test_savepoint_manager();

    // 开始事务
    let txn_id = txn_manager
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    // 创建多个保存点
    let sp1 = savepoint_manager
        .create_savepoint(txn_id, Some("checkpoint1".to_string()))
        .expect("创建保存点失败");
    let _sp2 = savepoint_manager
        .create_savepoint(txn_id, Some("checkpoint2".to_string()))
        .expect("创建保存点失败");

    // 获取保存点统计
    let stats = savepoint_manager.stats();
    let total_created = stats.total_created.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(total_created, 2, "应该创建了2个保存点");

    // 回滚到第一个保存点
    savepoint_manager
        .rollback_to_savepoint(sp1)
        .expect("回滚失败");

    // 验证统计更新
    let stats = savepoint_manager.stats();
    let rollback_count = stats.rollback_count.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(rollback_count, 1, "应该有1次回滚");

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 清理保存点
    savepoint_manager.cleanup_transaction(txn_id);

    // 验证保存点已被清理（通过获取活跃保存点列表）
    let active_savepoints = savepoint_manager.get_active_savepoints(txn_id);
    assert!(
        active_savepoints.is_empty(),
        "保存点应该已被清理"
    );
}
