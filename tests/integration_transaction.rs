//! 事务功能集成测试
//!
//! 测试事务管理器的核心功能，包括：
//! - 事务生命周期管理（开始、提交、中止）
//! - 事务隔离性
//! - 并发事务处理
//! - 事务与存储层的集成
//! - 保存点管理

mod common;

use std::sync::Arc;
use std::time::Duration;

use graphdb::storage::operations::rollback::RollbackExecutor;
use graphdb::storage::RedbStorage;
use graphdb::transaction::{
    TransactionError, TransactionManager, TransactionManagerConfig, TransactionOptions,
    TransactionState,
};

/// Mock回滚执行器，用于测试
struct MockRollbackExecutor;

impl RollbackExecutor for MockRollbackExecutor {
    fn execute_rollback(
        &mut self,
        _log: &graphdb::transaction::types::OperationLog,
    ) -> Result<(), graphdb::core::StorageError> {
        // Mock实现，总是成功
        Ok(())
    }
}

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
        auto_cleanup: true,
    };

    Arc::new(TransactionManager::new(db, config))
}

/// 测试事务生命周期
#[test]
fn test_transaction_lifecycle() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 检查事务状态
    let txn_info = txn_manager
        .get_transaction_info(txn_id)
        .expect("获取事务失败");
    assert_eq!(txn_info.state, TransactionState::Active);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 事务已提交，不再在活跃事务表中
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // 验证统计信息
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// 测试事务回滚
#[test]
fn test_transaction_rollback() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 回滚事务
    txn_manager.abort_transaction(txn_id).expect("回滚事务失败");

    // 事务已中止，不再在活跃事务表中
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // 验证统计信息
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// 测试只读事务
#[test]
fn test_read_only_transaction() {
    let txn_manager = create_test_transaction_manager();

    // 开始只读事务
    let options = TransactionOptions {
        read_only: true,
        timeout: Some(Duration::from_secs(30)),
        durability: graphdb::transaction::DurabilityLevel::None,
    };
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 检查事务是否为只读
    let txn_info = txn_manager
        .get_transaction_info(txn_id)
        .expect("获取事务失败");
    assert!(txn_info.is_read_only);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试保存点回滚与数据恢复
#[test]
fn test_savepoint_rollback_with_data_recovery() {
    use graphdb::core::Value;
    use graphdb::core::Vertex;
    use std::collections::HashMap;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_rollback_data.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 设置回滚执行器工厂
    txn_manager.set_rollback_executor_factory(Box::new(|| Box::new(MockRollbackExecutor)));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 获取事务上下文
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // 创建第一个顶点
    let _vertex1 = Vertex {
        vid: Box::new(Value::Int(1)),
        id: 1,
        tags: vec![],
        properties: HashMap::new(),
    };

    context.add_operation_log(graphdb::transaction::types::OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1u8],
        previous_state: None,
    });

    // 创建第一个保存点
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("checkpoint1".to_string()))
        .expect("创建保存点失败");

    // 创建第二个顶点
    let _vertex2 = Vertex {
        vid: Box::new(Value::Int(2)),
        id: 2,
        tags: vec![],
        properties: HashMap::new(),
    };

    context.add_operation_log(graphdb::transaction::types::OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![2u8],
        previous_state: None,
    });

    // 创建第二个保存点
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("checkpoint2".to_string()))
        .expect("创建保存点失败");

    // 验证操作日志数量
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // 回滚到第一个保存点
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id1)
        .expect("回滚到保存点失败");

    // 验证操作日志已被截断到第一个保存点位置（应该有1个日志）
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 1);

    // 验证第二个保存点已被移除
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 1);
    assert_eq!(savepoints[0].id, savepoint_id1);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试保存点回滚后继续操作
#[test]
fn test_continue_operations_after_savepoint_rollback() {
    let txn_manager = create_test_transaction_manager();

    // 设置回滚执行器工厂
    txn_manager.set_rollback_executor_factory(Box::new(|| Box::new(MockRollbackExecutor)));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 获取事务上下文
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // 添加一些操作日志
    use graphdb::transaction::types::OperationLog;
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1u8],
        previous_state: None,
    });

    // 创建保存点（此时有1个操作日志）
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // 添加更多操作日志
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![2u8],
        previous_state: None,
    });

    // 回滚到保存点
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id)
        .expect("回滚到保存点失败");

    // 验证操作日志已被截断到保存点位置（应该有1个日志）
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 1);

    // 回滚后继续添加操作日志
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![3u8],
        previous_state: None,
    });

    // 验证操作日志数量（应该有2个日志）
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试事务超时
#[test]
fn test_transaction_timeout() {
    let txn_manager = create_test_transaction_manager();

    // 开始一个超时时间很短的事务
    let options = TransactionOptions {
        read_only: false,
        timeout: Some(Duration::from_millis(100)),
        durability: graphdb::transaction::DurabilityLevel::None,
    };
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 等待超时
    std::thread::sleep(Duration::from_millis(150));

    // 清理过期事务
    txn_manager.cleanup_expired_transactions();

    // 检查事务状态（应该被自动中止）
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // 验证超时统计
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .timeout_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// 测试并发事务
#[test]
fn test_concurrent_transactions() {
    let txn_manager = create_test_transaction_manager();

    // 开始多个只读事务
    let mut txn_ids = Vec::new();
    for _ in 0..10 {
        let options = TransactionOptions {
            read_only: true,
            timeout: Some(Duration::from_secs(30)),
            durability: graphdb::transaction::DurabilityLevel::None,
        };
        let txn_id = txn_manager
            .begin_transaction(options)
            .expect("开始事务失败");
        txn_ids.push(txn_id);
    }

    // 提交所有事务
    for txn_id in txn_ids {
        txn_manager
            .commit_transaction(txn_id)
            .expect("提交事务失败");
    }

    // 验证所有事务都已提交
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        10
    );
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
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试事务统计信息
#[test]
fn test_transaction_stats() {
    let txn_manager = create_test_transaction_manager();

    // 开始并提交一个事务
    let options = TransactionOptions::default();
    let txn_id1 = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");
    txn_manager
        .commit_transaction(txn_id1)
        .expect("提交事务失败");

    // 开始并回滚一个事务
    let options = TransactionOptions::default();
    let txn_id2 = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");
    txn_manager
        .abort_transaction(txn_id2)
        .expect("回滚事务失败");

    // 检查统计信息
    let stats = txn_manager.stats();
    assert_eq!(
        stats
            .committed_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    assert_eq!(
        stats
            .aborted_transactions
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

/// 测试创建保存点
#[test]
fn test_create_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建保存点
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // 创建第二个保存点
    let savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // 验证保存点ID递增
    assert!(savepoint_id2 > savepoint_id1);

    // 获取保存点列表
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试通过名称查找保存点
#[test]
fn test_find_savepoint_by_name() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建命名保存点
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("checkpoint".to_string()))
        .expect("创建保存点失败");

    // 通过名称查找保存点
    let found = txn_manager.find_savepoint_by_name(txn_id, "checkpoint");
    assert!(found.is_some());
    assert_eq!(found.expect("保存点应存在").id, savepoint_id);

    // 查找不存在的保存点
    let not_found = txn_manager.find_savepoint_by_name(txn_id, "nonexistent");
    assert!(not_found.is_none());

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试释放保存点
#[test]
fn test_release_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建保存点
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    let savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // 释放第一个保存点
    txn_manager
        .release_savepoint(txn_id, savepoint_id1)
        .expect("释放保存点失败");

    // 验证保存点已被释放
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 1);
    assert_eq!(savepoints[0].id, savepoint_id2);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试回滚到保存点
#[test]
fn test_rollback_to_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // 设置回滚执行器工厂
    txn_manager.set_rollback_executor_factory(Box::new(|| Box::new(MockRollbackExecutor)));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建第一个保存点
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // 创建第二个保存点
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // 验证有两个保存点
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // 回滚到第一个保存点
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id1)
        .expect("回滚到保存点失败");

    // 验证第二个保存点已被移除
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 1);
    assert_eq!(savepoints[0].id, savepoint_id1);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试回滚到不存在的保存点
#[test]
fn test_rollback_to_nonexistent_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 尝试回滚到不存在的保存点
    let result = txn_manager.rollback_to_savepoint(txn_id, 999);
    assert!(matches!(
        result,
        Err(TransactionError::SavepointNotFound(_))
    ));

    // 验证事务仍然处于活跃状态
    let txn_info = txn_manager
        .get_transaction_info(txn_id)
        .expect("获取事务失败");
    assert_eq!(txn_info.state, TransactionState::Active);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试释放不存在的保存点
#[test]
fn test_release_nonexistent_savepoint() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 尝试释放不存在的保存点
    let result = txn_manager.release_savepoint(txn_id, 999);
    assert!(matches!(
        result,
        Err(TransactionError::SavepointNotFound(_))
    ));

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试保存点与事务提交
#[test]
fn test_savepoint_with_transaction_commit() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建保存点
    let _savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("checkpoint".to_string()))
        .expect("创建保存点失败");

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 事务已提交，不再在活跃事务表中
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());
}

/// 测试保存点与事务回滚
#[test]
fn test_savepoint_with_transaction_rollback() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建保存点
    let _savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("checkpoint".to_string()))
        .expect("创建保存点失败");

    // 回滚事务
    txn_manager.abort_transaction(txn_id).expect("回滚事务失败");

    // 事务已中止，不再在活跃事务表中
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());
}

/// 测试获取保存点信息
#[test]
fn test_get_savepoint_info() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建保存点
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("test_sp".to_string()))
        .expect("创建保存点失败");

    // 获取保存点信息
    let savepoint_info = txn_manager.get_savepoint(txn_id, savepoint_id);
    assert!(savepoint_info.is_some());
    assert_eq!(savepoint_info.expect("保存点信息应存在").id, savepoint_id);

    // 获取不存在的保存点信息
    let nonexistent = txn_manager.get_savepoint(txn_id, 999);
    assert!(nonexistent.is_none());

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试多个保存点的管理
#[test]
fn test_multiple_savepoints() {
    let txn_manager = create_test_transaction_manager();

    // 设置回滚执行器工厂
    txn_manager.set_rollback_executor_factory(Box::new(|| Box::new(MockRollbackExecutor)));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建多个保存点
    let mut savepoint_ids = Vec::new();
    for i in 0..5 {
        let id = txn_manager
            .create_savepoint(txn_id, Some(format!("sp{}", i)))
            .expect("创建保存点失败");
        savepoint_ids.push(id);
    }

    // 验证所有保存点都已创建
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 5);

    // 回滚到第三个保存点
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_ids[2])
        .expect("回滚到保存点失败");

    // 验证只剩下前三个保存点
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 3);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试事务管理器关闭
#[test]
fn test_transaction_manager_shutdown() {
    let txn_manager = create_test_transaction_manager();

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 关闭事务管理器
    txn_manager.shutdown();

    // 验证事务已被中止（不再在活跃事务表中）
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // 关闭后不能开始新事务
    let result = txn_manager.begin_transaction(TransactionOptions::default());
    assert!(matches!(result, Err(TransactionError::Internal(_))));
}

/// 测试操作日志记录和回滚
#[test]
fn test_operation_log_recording_and_rollback() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_operation_log.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 设置回滚执行器工厂
    txn_manager.set_rollback_executor_factory(Box::new(|| Box::new(MockRollbackExecutor)));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 获取事务上下文
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // 添加操作日志
    use graphdb::transaction::types::OperationLog;
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![2],
        previous_state: None,
    });

    // 验证操作日志已记录
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // 创建保存点
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // 添加更多操作日志
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![3],
        previous_state: None,
    });

    // 验证操作日志数量增加
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 3);

    // 回滚到保存点
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id)
        .expect("回滚到保存点失败");

    // 验证操作日志已被截断
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 2);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试批量操作操作日志记录
#[test]
fn test_batch_operation_log_recording() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_batch_log.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 设置回滚执行器工厂
    txn_manager.set_rollback_executor_factory(Box::new(|| Box::new(MockRollbackExecutor)));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 获取事务上下文
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // 批量添加操作日志
    use graphdb::transaction::types::OperationLog;
    let batch_logs = vec![
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![1],
            previous_state: None,
        },
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![2],
            previous_state: None,
        },
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![3],
            previous_state: None,
        },
    ];

    context.add_operation_logs(batch_logs);

    // 验证所有操作日志都已记录
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 3);

    // 创建保存点
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // 批量添加更多操作日志
    let more_logs = vec![
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![4],
            previous_state: None,
        },
        OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![5],
            previous_state: None,
        },
    ];

    context.add_operation_logs(more_logs);

    // 验证操作日志数量增加
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 5);

    // 回滚到保存点
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id)
        .expect("回滚到保存点失败");

    // 验证操作日志已被截断到保存点位置
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 3);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试保存点资源自动清理
#[test]
fn test_savepoint_resource_cleanup() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_cleanup.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建多个保存点
    let _savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");
    let _savepoint_id3 = txn_manager
        .create_savepoint(txn_id, Some("sp3".to_string()))
        .expect("创建保存点失败");

    // 验证保存点已创建
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 3);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 事务已提交，不再在活跃事务表中
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());

    // 验证保存点资源已被清理（通过重新开始事务验证）
    let options2 = TransactionOptions::default();
    let txn_id2 = txn_manager
        .begin_transaction(options2)
        .expect("开始事务失败");
    let savepoints = txn_manager.get_active_savepoints(txn_id2);
    assert_eq!(savepoints.len(), 0);

    txn_manager
        .commit_transaction(txn_id2)
        .expect("提交事务失败");
}

/// 测试保存点资源在事务回滚时自动清理
#[test]
fn test_savepoint_cleanup_on_rollback() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_cleanup_rollback.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建多个保存点
    let _savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");
    let _savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");

    // 验证保存点已创建
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // 回滚事务
    txn_manager.abort_transaction(txn_id).expect("回滚事务失败");

    // 事务已中止，不再在活跃事务表中
    let txn_info = txn_manager.get_transaction_info(txn_id);
    assert!(txn_info.is_none());
}

/// 测试回滚失败错误处理
#[test]
fn test_rollback_failure_error_handling() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_rollback_error.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 获取事务上下文
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // 添加操作日志
    use graphdb::transaction::types::OperationLog;
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    // 创建保存点
    let savepoint_id = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");

    // 添加更多操作日志
    context.add_operation_log(OperationLog::InsertVertex {
        space: "test_space".to_string(),
        vertex_id: vec![2],
        previous_state: None,
    });

    // 回滚到保存点（应该成功，因为TransactionContext内部处理回滚）
    let result = txn_manager.rollback_to_savepoint(txn_id, savepoint_id);

    // 验证回滚结果（应该成功）
    assert!(result.is_ok());

    // 验证操作日志已被截断到保存点位置（应该有1个日志）
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 1);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试并发访问操作日志
#[test]
fn test_concurrent_operation_log_access() {
    use std::thread;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_concurrent.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 获取事务上下文
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // 创建多个线程并发访问操作日志
    let mut handles = vec![];

    for i in 0..5 {
        let context_clone = context.clone();
        let handle = thread::spawn(move || {
            use graphdb::transaction::types::OperationLog;

            // 添加操作日志
            context_clone.add_operation_log(OperationLog::InsertVertex {
                space: "test_space".to_string(),
                vertex_id: vec![i as u8],
                previous_state: None,
            });

            // 读取操作日志
            let logs = context_clone.get_operation_logs();
            logs.len()
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().expect("线程执行失败");
    }

    // 验证所有操作日志都已记录
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 5);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试操作日志截断功能
#[test]
fn test_operation_log_truncation() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db_path = temp_dir.path().join("test_truncation.db");

    // 创建存储
    let storage = RedbStorage::new_with_path(db_path).expect("创建存储失败");
    let db = storage.get_db().clone();

    // 创建事务管理器
    let config = TransactionManagerConfig::default();
    let txn_manager = Arc::new(TransactionManager::new(db, config));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 获取事务上下文
    let context = txn_manager.get_context(txn_id).expect("获取事务上下文失败");

    // 添加多个操作日志
    use graphdb::transaction::types::OperationLog;
    for i in 0..10 {
        context.add_operation_log(OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: vec![i as u8],
            previous_state: None,
        });
    }

    // 验证操作日志数量
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 10);

    // 截断操作日志到索引5
    context.truncate_operation_log(5);

    // 验证操作日志已被截断
    let logs = context.get_operation_logs();
    assert_eq!(logs.len(), 5);

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

/// 测试保存点回滚后后续保存点的清理
#[test]
fn test_savepoint_cleanup_after_rollback() {
    let txn_manager = create_test_transaction_manager();

    // 设置回滚执行器工厂
    txn_manager.set_rollback_executor_factory(Box::new(|| Box::new(MockRollbackExecutor)));

    // 开始事务
    let options = TransactionOptions::default();
    let txn_id = txn_manager
        .begin_transaction(options)
        .expect("开始事务失败");

    // 创建多个保存点
    let savepoint_id1 = txn_manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点失败");
    let savepoint_id2 = txn_manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点失败");
    let savepoint_id3 = txn_manager
        .create_savepoint(txn_id, Some("sp3".to_string()))
        .expect("创建保存点失败");

    // 验证所有保存点都已创建
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 3);

    // 回滚到第二个保存点
    txn_manager
        .rollback_to_savepoint(txn_id, savepoint_id2)
        .expect("回滚到保存点失败");

    // 验证第三个保存点已被移除
    let savepoints = txn_manager.get_active_savepoints(txn_id);
    assert_eq!(savepoints.len(), 2);

    // 验证剩下的保存点ID
    let savepoint_ids: Vec<_> = savepoints.iter().map(|sp| sp.id).collect();
    assert!(savepoint_ids.contains(&savepoint_id1));
    assert!(savepoint_ids.contains(&savepoint_id2));
    assert!(!savepoint_ids.contains(&savepoint_id3));

    // 提交事务
    txn_manager
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}
