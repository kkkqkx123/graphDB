//! TransactionContext 测试
//!
//! 测试事务上下文的功能，包括状态管理、超时检查、操作日志等

use std::sync::Arc;
use std::time::Duration;

use tempfile::TempDir;

use crate::transaction::context::TransactionContext;
use crate::transaction::types::{
    DurabilityLevel, OperationLog, TransactionError, TransactionId, TransactionState,
};

/// 创建测试数据库
fn create_test_db() -> (Arc<redb::Database>, TempDir) {
    let temp_dir = TempDir::new().expect("创建临时目录失败");
    let db = Arc::new(
        redb::Database::create(temp_dir.path().join("test.db")).expect("创建测试数据库失败"),
    );
    (db, temp_dir)
}

#[test]
fn test_transaction_context_writable_creation() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);
    let durability = DurabilityLevel::Immediate;

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(txn_id, timeout, durability, write_txn, None);

    assert_eq!(ctx.id, txn_id);
    assert_eq!(ctx.state(), TransactionState::Active);
    assert!(!ctx.read_only);
    assert_eq!(ctx.durability, durability);
}

#[test]
fn test_transaction_context_readonly_creation() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let read_txn = db.begin_read().expect("创建读事务失败");

    let ctx = TransactionContext::new_readonly(txn_id, timeout, read_txn, None);

    assert_eq!(ctx.id, txn_id);
    assert_eq!(ctx.state(), TransactionState::Active);
    assert!(ctx.read_only);
}

#[test]
fn test_transaction_context_state_transitions() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // Active -> Committing
    assert!(ctx.transition_to(TransactionState::Committing).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committing);

    // Committing -> Committed
    assert!(ctx.transition_to(TransactionState::Committed).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committed);
}

#[test]
fn test_transaction_context_invalid_state_transition() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // Active -> Committed (无效转换)
    let result = ctx.transition_to(TransactionState::Committed);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::InvalidStateTransition { .. }
    ));

    // 正确的状态转换路径: Active -> Committing -> Committed
    assert!(ctx.transition_to(TransactionState::Committing).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committing);

    assert!(ctx.transition_to(TransactionState::Committed).is_ok());
    assert_eq!(ctx.state(), TransactionState::Committed);
}

#[test]
fn test_transaction_context_timeout() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(100);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 初始不应该超时
    assert!(!ctx.is_expired());

    // 等待超时
    std::thread::sleep(Duration::from_millis(150));

    // 现在应该超时
    assert!(ctx.is_expired());
}

#[test]
fn test_transaction_context_remaining_time() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(200);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 初始剩余时间应该接近超时时间
    let remaining = ctx.remaining_time();
    assert!(remaining > Duration::from_millis(150));

    // 等待一段时间
    std::thread::sleep(Duration::from_millis(100));

    // 剩余时间应该减少
    let remaining = ctx.remaining_time();
    assert!(remaining < Duration::from_millis(150));
    assert!(remaining > Duration::from_millis(50));
}

#[test]
fn test_transaction_context_modified_tables() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 记录表修改
    ctx.record_table_modification("vertices");
    ctx.record_table_modification("edges");
    ctx.record_table_modification("vertices"); // 重复记录

    let modified = ctx.get_modified_tables();
    assert_eq!(modified.len(), 2);
    assert!(modified.contains(&"vertices".to_string()));
    assert!(modified.contains(&"edges".to_string()));
}

#[test]
fn test_transaction_context_operation_log() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 初始操作日志为空
    assert_eq!(ctx.operation_log_len(), 0);

    // 添加操作日志
    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_state: None,
    });

    assert_eq!(ctx.operation_log_len(), 1);

    ctx.add_operation_log(OperationLog::UpdateVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_data: vec![4, 5, 6],
    });

    assert_eq!(ctx.operation_log_len(), 2);

    // 截断操作日志
    ctx.truncate_operation_log(1);
    assert_eq!(ctx.operation_log_len(), 1);
}

#[test]
fn test_transaction_context_can_execute() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // Active 状态可以执行
    assert!(ctx.can_execute().is_ok());

    // 转换到 Committing 状态
    ctx.transition_to(TransactionState::Committing)
        .expect("状态转换失败");

    // Committing 状态不能执行
    assert!(ctx.can_execute().is_err());
}

#[test]
fn test_transaction_context_can_execute_expired() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(50);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 等待超时
    std::thread::sleep(Duration::from_millis(100));

    // 超时后不能执行
    let result = ctx.can_execute();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::TransactionExpired
    ));
}

#[test]
fn test_transaction_context_info() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 记录一些修改
    ctx.record_table_modification("vertices");

    let info = ctx.info();
    assert_eq!(info.id, txn_id);
    assert_eq!(info.state, TransactionState::Active);
    assert!(!info.is_read_only);
    assert_eq!(info.modified_tables.len(), 1);
    assert!(info.modified_tables.contains(&"vertices".to_string()));
}

#[test]
fn test_transaction_context_take_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 取出写事务
    let taken_txn = ctx.take_write_txn();
    assert!(taken_txn.is_ok());

    // 再次取出应该失败
    let result = ctx.take_write_txn();
    assert!(result.is_err());
}

#[test]
fn test_transaction_context_readonly_take_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let read_txn = db.begin_read().expect("创建读事务失败");

    let ctx = TransactionContext::new_readonly(txn_id, timeout, read_txn, None);

    // 只读事务不能取出写事务
    let result = ctx.take_write_txn();
    assert!(result.is_err());
}

#[test]
fn test_transaction_context_with_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 使用写事务执行操作
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_ok());
}

#[test]
fn test_transaction_context_readonly_with_write_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let read_txn = db.begin_read().expect("创建读事务失败");

    let ctx = TransactionContext::new_readonly(txn_id, timeout, read_txn, None);

    // 只读事务不能使用 with_write_txn
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::ReadOnlyTransaction
    ));
}

#[test]
fn test_transaction_context_with_read_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let read_txn = db.begin_read().expect("创建读事务失败");

    let ctx = TransactionContext::new_readonly(txn_id, timeout, read_txn, None);

    // 使用读事务执行操作
    let result = ctx.with_read_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_ok());
}

#[test]
fn test_transaction_context_writable_with_read_txn() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 读写事务不能直接使用 with_read_txn
    let result = ctx.with_read_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TransactionError::Internal(_)));
}

#[test]
fn test_transaction_context_with_write_txn_expired() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_millis(50);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 等待超时
    std::thread::sleep(Duration::from_millis(100));

    // 超时后不能执行
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::TransactionExpired
    ));
}

#[test]
fn test_transaction_context_with_write_txn_invalid_state() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 正确的状态转换路径: Active -> Committing -> Committed
    ctx.transition_to(TransactionState::Committing)
        .expect("状态转换失败");
    ctx.transition_to(TransactionState::Committed)
        .expect("状态转换失败");

    // Committed 状态不能执行
    let result = ctx.with_write_txn(|_txn| Ok::<(), crate::core::StorageError>(()));

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::InvalidStateForCommit(_)
    ));
}

#[test]
fn test_savepoint_creation() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 创建保存点
    let savepoint_id = ctx.create_savepoint(Some("sp1".to_string()));
    assert_eq!(savepoint_id, 1);

    // 获取保存点信息
    let savepoint_info = ctx.get_savepoint(savepoint_id);
    assert!(savepoint_info.is_some());
    let info = savepoint_info.unwrap();
    assert_eq!(info.name, Some("sp1".to_string()));
    assert_eq!(info.operation_log_index, 0);
}

#[test]
fn test_multiple_savepoints() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 创建多个保存点
    let sp1 = ctx.create_savepoint(Some("sp1".to_string()));
    let sp2 = ctx.create_savepoint(Some("sp2".to_string()));
    let sp3 = ctx.create_savepoint(Some("sp3".to_string()));

    assert_eq!(sp1, 1);
    assert_eq!(sp2, 2);
    assert_eq!(sp3, 3);

    // 验证保存点信息
    assert!(ctx.get_savepoint(sp1).is_some());
    assert!(ctx.get_savepoint(sp2).is_some());
    assert!(ctx.get_savepoint(sp3).is_some());
}

#[test]
fn test_rollback_to_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 创建保存点
    let savepoint_id = ctx.create_savepoint(Some("sp1".to_string()));

    // 回滚到保存点
    let result = ctx.rollback_to_savepoint(savepoint_id);
    assert!(result.is_ok());

    // 验证操作日志已被截断
    assert_eq!(ctx.operation_log_len(), 0);
}

#[test]
fn test_rollback_to_nonexistent_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 尝试回滚到不存在的保存点
    let result = ctx.rollback_to_savepoint(999);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::SavepointNotFound(_)
    ));
}

#[test]
fn test_release_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 创建保存点
    let savepoint_id = ctx.create_savepoint(Some("sp1".to_string()));

    // 释放保存点
    let result = ctx.release_savepoint(savepoint_id);
    assert!(result.is_ok());

    // 验证保存点已被释放
    let savepoint_info = ctx.get_savepoint(savepoint_id);
    assert!(savepoint_info.is_none());
}

#[test]
fn test_release_nonexistent_savepoint() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 尝试释放不存在的保存点
    let result = ctx.release_savepoint(999);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::SavepointNotFound(_)
    ));
}

#[test]
fn test_savepoint_with_operations() {
    let (db, _temp) = create_test_db();
    let txn_id: TransactionId = 1;
    let timeout = Duration::from_secs(30);

    let write_txn = db.begin_write().expect("创建写事务失败");

    let ctx = TransactionContext::new_writable(
        txn_id,
        timeout,
        DurabilityLevel::Immediate,
        write_txn,
        None,
    );

    // 创建第一个保存点
    let sp1 = ctx.create_savepoint(Some("sp1".to_string()));

    // 添加一些操作日志
    let log1 = OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1u8, 2u8, 3u8],
        previous_state: None,
    };
    ctx.add_operation_log(log1);

    let log2 = OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![4u8, 5u8, 6u8],
        previous_state: None,
    };
    ctx.add_operation_log(log2);

    // 创建第二个保存点
    let _sp2 = ctx.create_savepoint(Some("sp2".to_string()));

    // 验证操作日志数量
    assert_eq!(ctx.operation_log_len(), 2);

    // 回滚到第一个保存点
    let result = ctx.rollback_to_savepoint(sp1);
    assert!(result.is_ok());

    // 验证操作日志已被截断到第一个保存点
    assert_eq!(ctx.operation_log_len(), 0);
}
