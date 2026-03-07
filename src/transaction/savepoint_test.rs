//! SavepointManager 测试
//!
//! 测试保存点管理器的功能，包括创建、回滚、释放保存点等

use crate::transaction::savepoint::{
    Savepoint, SavepointId, SavepointManager, SavepointState,
};
use crate::transaction::types::{TransactionError, TransactionId};

#[test]
fn test_savepoint_id_creation() {
    let id = SavepointId::new(1);
    assert_eq!(id.value(), 1);
}

#[test]
fn test_savepoint_id_display() {
    let id = SavepointId::new(42);
    assert_eq!(format!("{}", id), "42");
}

#[test]
fn test_savepoint_id_default() {
    let id = SavepointId::default();
    assert_eq!(id.value(), 0);
}

#[test]
fn test_savepoint_creation() {
    let id = SavepointId::new(1);
    let txn_id: TransactionId = 1;
    let name = Some("test_savepoint".to_string());
    let sequence = 1;

    let savepoint = Savepoint::new(id, txn_id, name, sequence);

    assert_eq!(savepoint.id, id);
    assert_eq!(savepoint.txn_id, txn_id);
    assert_eq!(savepoint.name, Some("test_savepoint".to_string()));
    assert_eq!(savepoint.sequence, sequence);
    assert_eq!(savepoint.state, SavepointState::Active);
    assert!(savepoint.is_active());
}

#[test]
fn test_savepoint_manager_creation() {
    let manager = SavepointManager::new();

    assert_eq!(
        manager.stats().total_created.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_create_savepoint() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp_id = manager
        .create_savepoint(txn_id, Some("test".to_string()))
        .expect("创建保存点失败");

    assert_eq!(sp_id.value(), 1);
    assert_eq!(
        manager.stats().total_created.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_create_savepoint_without_name() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp_id = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");

    assert_eq!(sp_id.value(), 1);
}

#[test]
fn test_create_multiple_savepoints() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");
    let sp3 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点3失败");

    assert_eq!(sp1.value(), 1);
    assert_eq!(sp2.value(), 2);
    assert_eq!(sp3.value(), 3);

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 3);
}

#[test]
fn test_savepoint_sequence_independent_per_transaction() {
    let manager = SavepointManager::new();
    let txn1: TransactionId = 1;
    let txn2: TransactionId = 2;

    let _sp1 = manager
        .create_savepoint(txn1, None)
        .expect("创建保存点1失败");
    let _sp2 = manager
        .create_savepoint(txn2, None)
        .expect("创建保存点2失败");
    let _sp3 = manager
        .create_savepoint(txn1, None)
        .expect("创建保存点3失败");

    // 验证序列号
    let active_sps1 = manager.get_active_savepoints(txn1);
    let active_sps2 = manager.get_active_savepoints(txn2);

    assert_eq!(active_sps1.len(), 2);
    assert_eq!(active_sps2.len(), 1);

    // txn1 的保存点序列号应该是 1 和 2
    assert_eq!(active_sps1[0].sequence, 1);
    assert_eq!(active_sps1[1].sequence, 2);

    // txn2 的保存点序列号应该是 1
    assert_eq!(active_sps2[0].sequence, 1);
}

#[test]
fn test_rollback_to_savepoint() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");
    let sp3 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点3失败");

    // 回滚到 sp1，sp2 和 sp3 应该被标记为已回滚
    manager
        .rollback_to_savepoint(sp1)
        .expect("回滚失败");

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 1);
    assert_eq!(active_sps[0].id, sp1);

    // sp2 应该不能再回滚
    let result = manager.rollback_to_savepoint(sp2);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::SavepointNotActive(_)
    ));

    // sp3 也应该不能再回滚
    let result = manager.rollback_to_savepoint(sp3);
    assert!(result.is_err());

    // 验证统计信息
    assert_eq!(
        manager.stats().rollback_count.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_rollback_to_middle_savepoint() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");
    let _sp3 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点3失败");

    // 回滚到 sp2，sp3 应该被标记为已回滚
    manager
        .rollback_to_savepoint(sp2)
        .expect("回滚失败");

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 2);
    assert_eq!(active_sps[0].id, sp1);
    assert_eq!(active_sps[1].id, sp2);
}

#[test]
fn test_rollback_to_last_savepoint() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");

    // 回滚到 sp2（最后一个保存点）
    manager
        .rollback_to_savepoint(sp2)
        .expect("回滚失败");

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 2);
    assert_eq!(active_sps[0].id, sp1);
    assert_eq!(active_sps[1].id, sp2);
}

#[test]
fn test_rollback_nonexistent_savepoint() {
    let manager = SavepointManager::new();
    let _txn_id: TransactionId = 1;

    let sp_id = SavepointId::new(999);

    let result = manager.rollback_to_savepoint(sp_id);
    assert!(result.is_err());
}

#[test]
fn test_release_savepoint() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");

    manager
        .release_savepoint(sp)
        .expect("释放保存点失败");

    // 释放后不能再回滚
    let result = manager.rollback_to_savepoint(sp);
    assert!(result.is_err());

    // 验证统计信息
    assert_eq!(
        manager.stats().released_count.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_release_nonexistent_savepoint() {
    let manager = SavepointManager::new();
    let sp_id = SavepointId::new(999);

    let result = manager.release_savepoint(sp_id);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::SavepointNotFound(_)
    ));
}

#[test]
fn test_release_already_released_savepoint() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");

    manager
        .release_savepoint(sp)
        .expect("第一次释放失败");

    // 再次释放应该失败
    let result = manager.release_savepoint(sp);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TransactionError::SavepointNotActive(_)
    ));
}

#[test]
fn test_release_after_rollback() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let _sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");

    // 回滚到 sp1
    manager
        .rollback_to_savepoint(sp1)
        .expect("回滚失败");

    // 释放 sp1
    manager
        .release_savepoint(sp1)
        .expect("释放保存点失败");

    // 验证活跃保存点
    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 0);
}

#[test]
fn test_find_savepoint_by_name() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let _sp1 = manager
        .create_savepoint(txn_id, Some("first".to_string()))
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn_id, Some("second".to_string()))
        .expect("创建保存点2失败");

    let found = manager.find_savepoint_by_name(txn_id, "second");
    assert_eq!(found, Some(sp2));

    let not_found = manager.find_savepoint_by_name(txn_id, "third");
    assert_eq!(not_found, None);
}

#[test]
fn test_find_savepoint_by_name_nonexistent() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let found = manager.find_savepoint_by_name(txn_id, "nonexistent");
    assert_eq!(found, None);
}

#[test]
fn test_get_active_savepoints() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, Some("sp1".to_string()))
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn_id, Some("sp2".to_string()))
        .expect("创建保存点2失败");

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 2);
    assert_eq!(active_sps[0].id, sp1);
    assert_eq!(active_sps[0].name, Some("sp1".to_string()));
    assert_eq!(active_sps[1].id, sp2);
    assert_eq!(active_sps[1].name, Some("sp2".to_string()));
}

#[test]
fn test_get_active_savepoints_after_rollback() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let _sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");
    let _sp3 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点3失败");

    // 回滚到 sp1
    manager
        .rollback_to_savepoint(sp1)
        .expect("回滚失败");

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 1);
    assert_eq!(active_sps[0].id, sp1);
}

#[test]
fn test_get_active_savepoints_after_release() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");

    // 释放 sp1
    manager
        .release_savepoint(sp1)
        .expect("释放保存点失败");

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 1);
    assert_eq!(active_sps[0].id, sp2);
}

#[test]
fn test_get_active_savepoints_nonexistent_transaction() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 999;

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 0);
}

#[test]
fn test_cleanup_transaction() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    let _sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点1失败");
    let _sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点2失败");

    manager.cleanup_transaction(txn_id);

    let active_sps = manager.get_active_savepoints(txn_id);
    assert!(active_sps.is_empty());
}

#[test]
fn test_cleanup_nonexistent_transaction() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 999;

    // 清理不存在的事务不应该报错
    manager.cleanup_transaction(txn_id);

    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 0);
}

#[test]
fn test_multiple_transactions_independent() {
    let manager = SavepointManager::new();
    let txn1: TransactionId = 1;
    let txn2: TransactionId = 2;

    let sp1 = manager
        .create_savepoint(txn1, None)
        .expect("创建保存点1失败");
    let sp2 = manager
        .create_savepoint(txn2, None)
        .expect("创建保存点2失败");

    // 验证每个事务的保存点是独立的
    let active_sps1 = manager.get_active_savepoints(txn1);
    let active_sps2 = manager.get_active_savepoints(txn2);

    assert_eq!(active_sps1.len(), 1);
    assert_eq!(active_sps2.len(), 1);
    assert_eq!(active_sps1[0].id, sp1);
    assert_eq!(active_sps2[0].id, sp2);

    // 回滚 txn1 的保存点
    manager
        .rollback_to_savepoint(sp1)
        .expect("回滚失败");

    // txn2 的保存点应该仍然活跃
    let active_sps2 = manager.get_active_savepoints(txn2);
    assert_eq!(active_sps2.len(), 1);
    assert_eq!(active_sps2[0].id, sp2);

    // txn1 的保存点应该仍然存在（虽然已回滚）
    let active_sps1 = manager.get_active_savepoints(txn1);
    assert_eq!(active_sps1.len(), 1);
}

#[test]
fn test_savepoint_stats() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    // 初始统计
    assert_eq!(
        manager.stats().total_created.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        manager.stats().released_count.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        manager.stats().rollback_count.load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // 创建保存点
    let sp1 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");
    let _sp2 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");
    let _sp3 = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");

    assert_eq!(
        manager.stats().total_created.load(std::sync::atomic::Ordering::Relaxed),
        3
    );

    // 回滚保存点 sp1 - 这会将 sp2 和 sp3 都标记为已回滚
    manager
        .rollback_to_savepoint(sp1)
        .expect("回滚失败");

    assert_eq!(
        manager.stats().rollback_count.load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // 释放 sp1 (sp1 是目标保存点，释放它应该成功)
    manager
        .release_savepoint(sp1)
        .expect("释放保存点失败");

    assert_eq!(
        manager.stats().released_count.load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    // 注意：sp2 和 sp3 现在都是已回滚状态，不能再次释放或回滚
}

#[test]
fn test_nested_savepoints() {
    let manager = SavepointManager::new();
    let txn_id: TransactionId = 1;

    // 创建嵌套保存点
    let sp1 = manager
        .create_savepoint(txn_id, Some("level1".to_string()))
        .expect("创建level1失败");
    let sp2 = manager
        .create_savepoint(txn_id, Some("level2".to_string()))
        .expect("创建level2失败");
    let _sp3 = manager
        .create_savepoint(txn_id, Some("level3".to_string()))
        .expect("创建level3失败");

    // 验证所有保存点都活跃
    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 3);

    // 回滚到 level2
    manager
        .rollback_to_savepoint(sp2)
        .expect("回滚失败");

    // 只有 level1 和 level2 应该活跃
    let active_sps = manager.get_active_savepoints(txn_id);
    assert_eq!(active_sps.len(), 2);
    assert_eq!(active_sps[0].id, sp1);
    assert_eq!(active_sps[1].id, sp2);
}

#[test]
fn test_savepoint_manager_default() {
    let manager = SavepointManager::default();

    let txn_id: TransactionId = 1;
    let sp = manager
        .create_savepoint(txn_id, None)
        .expect("创建保存点失败");

    assert_eq!(sp.value(), 1);
}
