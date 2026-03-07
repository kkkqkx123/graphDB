//! TransactionalStorage 测试
//!
//! 测试事务感知存储的功能，包括事务管理、保存点、两阶段提交等

use std::collections::HashMap;
use std::sync::Arc;

use tempfile::TempDir;

use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, StorageError, Value, Vertex};
use crate::storage::transactional_storage::TransactionalStorage;
use crate::storage::RedbStorage;
use crate::transaction::{
    TransactionManager, TransactionOptions, TransactionState,
};

/// 创建测试存储
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

/// 创建测试顶点
fn create_test_vertex(id: i64) -> Vertex {
    Vertex::new(
        Value::Int(id),
        vec![Tag {
            name: "Test".to_string(),
            properties: HashMap::new(),
        }],
    )
}

/// 创建测试边
fn create_test_edge(src: i64, dst: i64, edge_type: &str) -> Edge {
    Edge::new(
        Value::Int(src),
        Value::Int(dst),
        edge_type.to_string(),
        0,
        HashMap::new(),
    )
}

#[test]
fn test_transactional_storage_creation() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    assert_eq!(
        transactional
            .transaction_manager()
            .stats()
            .total_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_transactional_storage_inner() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let inner = transactional.inner();
    // 验证 inner 存在
    assert!(inner.get_db().as_ref() as *const _ as usize != 0);
}

#[test]
fn test_transactional_storage_transaction_manager() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let manager = transactional.transaction_manager();
    assert_eq!(
        manager.stats().total_transactions.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_begin_transaction() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let txn_id = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    assert!(transactional
        .transaction_manager()
        .is_transaction_active(txn_id));

    let context = transactional
        .transaction_manager()
        .get_context(txn_id)
        .expect("获取事务上下文失败");
    assert_eq!(context.state(), TransactionState::Active);
}

#[test]
fn test_begin_readonly_transaction() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let options = TransactionOptions::new().read_only();
    let txn_id = transactional
        .begin_transaction(options)
        .expect("开始只读事务失败");

    let context = transactional
        .transaction_manager()
        .get_context(txn_id)
        .expect("获取事务上下文失败");
    assert!(context.read_only);
}

#[test]
fn test_commit_transaction() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let txn_id = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    transactional
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    assert!(!transactional
        .transaction_manager()
        .is_transaction_active(txn_id));

    assert_eq!(
        transactional
            .transaction_manager()
            .stats()
            .committed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_abort_transaction() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let txn_id = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    transactional
        .abort_transaction(txn_id)
        .expect("中止事务失败");

    assert!(!transactional
        .transaction_manager()
        .is_transaction_active(txn_id));

    assert_eq!(
        transactional
            .transaction_manager()
            .stats()
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_prepare_transaction() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let txn_id = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    transactional
        .prepare_transaction(txn_id)
        .expect("准备事务失败");

    let context = transactional
        .transaction_manager()
        .get_context(txn_id)
        .expect("获取事务上下文失败");
    assert_eq!(context.state(), TransactionState::Prepared);

    // 清理
    transactional
        .commit_transaction(txn_id)
        .expect("提交事务失败");
}

#[test]
fn test_prepare_invalid_state() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let txn_id = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    // 提交事务
    transactional
        .commit_transaction(txn_id)
        .expect("提交事务失败");

    // 尝试准备已提交的事务应该失败
    let result = transactional.prepare_transaction(txn_id);
    assert!(result.is_err());
}

#[test]
fn test_commit_two_phase() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let txn_id = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    transactional
        .prepare_transaction(txn_id)
        .expect("准备事务失败");

    transactional
        .commit_two_phase(txn_id)
        .expect("提交两阶段事务失败");

    assert!(!transactional
        .transaction_manager()
        .is_transaction_active(txn_id));
}

#[test]
fn test_abort_two_phase() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let txn_id = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始事务失败");

    transactional
        .prepare_transaction(txn_id)
        .expect("准备事务失败");

    transactional
        .abort_two_phase(txn_id)
        .expect("中止两阶段事务失败");

    assert!(!transactional
        .transaction_manager()
        .is_transaction_active(txn_id));
}

#[test]
fn test_execute_in_transaction_success() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let vertex = create_test_vertex(1);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| client.insert_vertex("test_space", vertex),
    );

    assert!(result.is_ok(), "事务执行失败: {:?}", result.err());
}

#[test]
fn test_execute_in_transaction_failure() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |_client| -> Result<Value, StorageError> {
            Err(StorageError::DbError("故意失败".to_string()))
        },
    );

    assert!(result.is_err());

    // 验证事务已中止
    assert_eq!(
        transactional
            .transaction_manager()
            .stats()
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_execute_in_transaction_multiple_operations() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            // 插入多个顶点
            let v1 = create_test_vertex(1);
            let v2 = create_test_vertex(2);
            let v3 = create_test_vertex(3);

            client.insert_vertex("test_space", v1)?;
            client.insert_vertex("test_space", v2)?;
            client.insert_vertex("test_space", v3)?;

            Ok(())
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_insert_vertex() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let vertex = create_test_vertex(1);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| client.insert_vertex("test_space", vertex),
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_update_vertex() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let vertex = create_test_vertex(1);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            client.insert_vertex("test_space", vertex.clone())?;
            client.update_vertex("test_space", vertex)
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_delete_vertex() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let vertex = create_test_vertex(1);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            client.insert_vertex("test_space", vertex.clone())?;
            client.delete_vertex("test_space", &vertex.vid)
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_batch_insert_vertices() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let vertices = vec![
        create_test_vertex(1),
        create_test_vertex(2),
        create_test_vertex(3),
    ];

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| client.batch_insert_vertices("test_space", vertices),
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_insert_edge() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let edge = create_test_edge(1, 2, "knows");

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| client.insert_edge("test_space", edge),
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_delete_edge() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let edge = create_test_edge(1, 2, "knows");

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            client.insert_edge("test_space", edge.clone())?;
            client.delete_edge("test_space", &edge.src, &edge.dst, &edge.edge_type)
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_batch_insert_edges() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let edges = vec![
        create_test_edge(1, 2, "knows"),
        create_test_edge(2, 3, "knows"),
        create_test_edge(3, 1, "knows"),
    ];

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| client.batch_insert_edges("test_space", edges),
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_delete_tags() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let vertex = create_test_vertex(1);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            client.insert_vertex("test_space", vertex.clone())?;
            client.delete_tags("test_space", &vertex.vid, &["Test".to_string()])
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_create_savepoint() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            let sp1 = client.create_savepoint(Some("sp1".to_string()))?;
            assert_eq!(sp1.value(), 0);

            let sp2 = client.create_savepoint(Some("sp2".to_string()))?;
            assert_eq!(sp2.value(), 1);

            Ok(())
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_rollback_to_savepoint() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            let sp1 = client.create_savepoint(Some("sp1".to_string()))?;

            // 执行一些操作
            let v1 = create_test_vertex(1);
            client.insert_vertex("test_space", v1)?;

            let _sp2 = client.create_savepoint(Some("sp2".to_string()))?;

            // 执行更多操作
            let v2 = create_test_vertex(2);
            client.insert_vertex("test_space", v2)?;

            // 回滚到 sp1
            client.rollback_to_savepoint(sp1)?;

            Ok(())
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_release_savepoint() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            let sp = client.create_savepoint(Some("sp1".to_string()))?;

            // 执行一些操作
            let v = create_test_vertex(1);
            client.insert_vertex("test_space", v)?;

            // 释放保存点
            client.release_savepoint(sp)?;

            Ok(())
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_client_nested_savepoints() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            let _sp1 = client.create_savepoint(Some("level1".to_string()))?;

            let v1 = create_test_vertex(1);
            client.insert_vertex("test_space", v1)?;

            let sp2 = client.create_savepoint(Some("level2".to_string()))?;

            let v2 = create_test_vertex(2);
            client.insert_vertex("test_space", v2)?;

            let _sp3 = client.create_savepoint(Some("level3".to_string()))?;

            let v3 = create_test_vertex(3);
            client.insert_vertex("test_space", v3)?;

            // 回滚到 level2
            client.rollback_to_savepoint(sp2)?;

            Ok(())
        },
    );

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_readonly_transaction() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    // 首先插入一些数据
    let vertex = create_test_vertex(1);
    transactional
        .execute_in_transaction(TransactionOptions::default(), |client| {
            client.insert_vertex("test_space", vertex)
        })
        .expect("插入数据失败");

    // 然后在只读事务中读取
    let options = TransactionOptions::new().read_only();
    let result = transactional.execute_in_transaction(options, |_client| {
        // 只读事务应该能够执行
        Ok(())
    });

    assert!(result.is_ok());
}

#[test]
fn test_transactional_storage_transaction_isolation() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    // 开始第一个事务
    let txn1 = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始第一个事务失败");

    // 尝试开始第二个写事务应该失败（redb 单写者限制）
    let result = transactional.begin_transaction(TransactionOptions::default());
    assert!(result.is_err());

    // 提交第一个事务
    transactional
        .commit_transaction(txn1)
        .expect("提交事务失败");

    // 现在可以开始新的事务
    let txn2 = transactional
        .begin_transaction(TransactionOptions::default())
        .expect("开始第二个事务失败");

    transactional
        .commit_transaction(txn2)
        .expect("提交事务失败");
}

#[test]
fn test_transactional_storage_error_handling() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result: Result<(), StorageError> = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            // 插入一个顶点
            let v1 = create_test_vertex(1);
            client.insert_vertex("test_space", v1)?;

            // 故意失败
            Err(StorageError::DbError("测试错误".to_string()))
        },
    );

    assert!(result.is_err());

    // 验证事务已中止
    assert_eq!(
        transactional
            .transaction_manager()
            .stats()
            .aborted_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}

#[test]
fn test_transactional_storage_complex_workflow() {
    let (storage, txn_manager, _temp) = create_test_storage();
    let transactional = TransactionalStorage::new(storage, txn_manager);

    let result = transactional.execute_in_transaction(
        TransactionOptions::default(),
        |client| {
            // 创建保存点
            let _sp1 = client.create_savepoint(Some("initial".to_string()))?;

            // 插入顶点
            let v1 = create_test_vertex(1);
            let v2 = create_test_vertex(2);
            client.insert_vertex("test_space", v1)?;
            client.insert_vertex("test_space", v2)?;

            // 创建第二个保存点
            let sp2 = client.create_savepoint(Some("after_vertices".to_string()))?;

            // 插入边
            let e1 = create_test_edge(1, 2, "knows");
            client.insert_edge("test_space", e1)?;

            // 回滚到第二个保存点
            client.rollback_to_savepoint(sp2)?;

            // 再次插入边
            let e2 = create_test_edge(1, 2, "likes");
            client.insert_edge("test_space", e2)?;

            Ok(())
        },
    );

    assert!(result.is_ok());
}
