//! OperationLogRollback 测试
//!
//! 测试操作日志回滚处理器的功能

use std::cell::RefCell;

use crate::storage::operations::operation_log_rollback::{
    OperationLogContext, OperationLogRollback, RollbackExecutor,
};
use crate::transaction::types::OperationLog;

struct TestContext {
    operation_log: RefCell<Vec<OperationLog>>,
}

impl TestContext {
    fn new() -> Self {
        Self {
            operation_log: RefCell::new(Vec::new()),
        }
    }

    fn add_operation_log(&self, operation: OperationLog) {
        self.operation_log.borrow_mut().push(operation);
    }

    fn operation_log_len(&self) -> usize {
        self.operation_log.borrow().len()
    }

    fn truncate_operation_log(&self, index: usize) {
        self.operation_log.borrow_mut().truncate(index);
    }

    fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
        self.operation_log.borrow().get(index).cloned()
    }

    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
        let logs = self.operation_log.borrow();
        if start >= logs.len() {
            return Vec::new();
        }
        let end = end.min(logs.len());
        logs[start..end].to_vec()
    }

    fn clear_operation_log(&self) {
        self.operation_log.borrow_mut().clear();
    }
}

impl OperationLogContext for TestContext {
    fn operation_log_len(&self) -> usize {
        self.operation_log_len()
    }

    fn truncate_operation_log(&self, index: usize) {
        self.truncate_operation_log(index)
    }

    fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
        self.get_operation_log(index)
    }

    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
        self.get_operation_logs(start, end)
    }

    fn clear_operation_log(&self) {
        self.clear_operation_log()
    }
}

struct MockExecutor;

impl RollbackExecutor for MockExecutor {
    fn execute_rollback(&self, _log: &OperationLog) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
}

#[test]
fn test_operation_log_rollback_creation() {
    let ctx = TestContext::new();
    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    assert_eq!(rollback.operation_log_len(), 0);
}

#[test]
fn test_operation_log_rollback_empty() {
    let ctx = TestContext::new();
    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    let result = rollback.rollback_to_index(0);
    assert!(result.is_ok());
}

#[test]
fn test_operation_log_rollback_with_operations() {
    let ctx = TestContext::new();

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::UpdateVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_data: vec![4, 5, 6],
    });

    ctx.add_operation_log(OperationLog::DeleteVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        deleted_data: vec![7, 8, 9],
    });

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    assert_eq!(rollback.operation_log_len(), 3);

    let result = rollback.rollback_to_index(1);
    assert!(result.is_ok());

    assert_eq!(rollback.operation_log_len(), 1);
}

#[test]
fn test_operation_log_rollback_to_zero() {
    let ctx = TestContext::new();

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::UpdateVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_data: vec![4, 5, 6],
    });

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    assert_eq!(rollback.operation_log_len(), 2);

    let result = rollback.rollback_to_index(0);
    assert!(result.is_ok());

    assert_eq!(rollback.operation_log_len(), 0);
}

#[test]
fn test_operation_log_rollback_to_current_length() {
    let ctx = TestContext::new();

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    let len = rollback.operation_log_len();

    let result = rollback.rollback_to_index(len);
    assert!(result.is_ok());
}

#[test]
fn test_operation_log_rollback_invalid_index() {
    let ctx = TestContext::new();
    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    let result = rollback.rollback_to_index(100);
    assert!(result.is_err());
}

#[test]
fn test_operation_log_rollback_large_index() {
    let ctx = TestContext::new();

    for i in 0..10u8 {
        ctx.add_operation_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![i],
            previous_state: None,
        });
    }

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    let result = rollback.rollback_to_index(5);
    assert!(result.is_ok());
    assert_eq!(rollback.operation_log_len(), 5);
}

#[test]
fn test_operation_log_rollback_operation_types() {
    let ctx = TestContext::new();

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::InsertEdge {
        space: "test".to_string(),
        edge_key: vec![1, 2],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::UpdateVertex {
        space: "test".to_string(),
        vertex_id: vec![1],
        previous_data: vec![3, 4],
    });

    ctx.add_operation_log(OperationLog::DeleteVertex {
        space: "test".to_string(),
        vertex_id: vec![1],
        deleted_data: vec![5, 6],
    });

    ctx.add_operation_log(OperationLog::DeleteEdge {
        space: "test".to_string(),
        edge_key: vec![1, 2],
        deleted_data: vec![7, 8],
    });

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    assert_eq!(rollback.operation_log_len(), 5);

    let result = rollback.rollback_to_index(2);
    assert!(result.is_ok());
    assert_eq!(rollback.operation_log_len(), 2);
}

#[test]
fn test_operation_log_rollback_multiple_times() {
    let ctx = TestContext::new();

    for i in 0..5u8 {
        ctx.add_operation_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![i],
            previous_state: None,
        });
    }

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    assert_eq!(rollback.operation_log_len(), 5);

    rollback.rollback_to_index(3).unwrap();
    assert_eq!(rollback.operation_log_len(), 3);

    rollback.rollback_to_index(1).unwrap();
    assert_eq!(rollback.operation_log_len(), 1);

    rollback.rollback_to_index(0).unwrap();
    assert_eq!(rollback.operation_log_len(), 0);
}

#[test]
fn test_operation_log_rollback_boundary_conditions() {
    let ctx = TestContext::new();
    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    assert!(rollback.rollback_to_index(0).is_ok());

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    let len = rollback.operation_log_len();
    assert!(rollback.rollback_to_index(len).is_ok());
    assert_eq!(rollback.operation_log_len(), len);
}

#[test]
fn test_operation_log_rollback_after_more_operations() {
    let ctx = TestContext::new();

    for i in 0..20u8 {
        ctx.add_operation_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![i],
            previous_state: None,
        });
    }

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    rollback.rollback_to_index(15).unwrap();
    assert_eq!(rollback.operation_log_len(), 15);

    for i in 15..20u8 {
        ctx.add_operation_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![i],
            previous_state: None,
        });
    }

    assert_eq!(rollback.operation_log_len(), 20);
}

#[test]
fn test_operation_log_rollback_with_different_operations() {
    let ctx = TestContext::new();

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "space1".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::InsertEdge {
        space: "space2".to_string(),
        edge_key: vec![2, 3],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::UpdateVertex {
        space: "space3".to_string(),
        vertex_id: vec![4],
        previous_data: vec![5, 6],
    });

    ctx.add_operation_log(OperationLog::DeleteEdge {
        space: "space4".to_string(),
        edge_key: vec![5, 6],
        deleted_data: vec![7, 8],
    });

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    assert_eq!(rollback.operation_log_len(), 4);

    rollback.rollback_to_index(2).unwrap();
    assert_eq!(rollback.operation_log_len(), 2);
}

#[test]
fn test_execute_rollback_with_executor() {
    let ctx = TestContext::new();
    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    let executor = MockExecutor;

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::UpdateVertex {
        space: "test".to_string(),
        vertex_id: vec![1, 2, 3],
        previous_data: vec![4, 5, 6],
    });

    let result = rollback.execute_rollback_to_index(0, &executor);
    assert!(result.is_ok());
    assert_eq!(rollback.operation_log_len(), 0);
}

#[test]
fn test_get_all_logs() {
    let ctx = TestContext::new();
    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    ctx.add_operation_log(OperationLog::DeleteVertex {
        space: "test".to_string(),
        vertex_id: vec![2],
        deleted_data: vec![3],
    });

    let logs = rollback.get_all_logs();
    assert_eq!(logs.len(), 2);
}

#[test]
fn test_clear_logs() {
    let ctx = TestContext::new();
    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);

    ctx.add_operation_log(OperationLog::InsertVertex {
        space: "test".to_string(),
        vertex_id: vec![1],
        previous_state: None,
    });

    assert_eq!(rollback.operation_log_len(), 1);

    rollback.clear_logs();

    assert_eq!(rollback.operation_log_len(), 0);
}

#[test]
fn test_operation_log_with_index_operations() {
    let ctx = TestContext::new();

    ctx.add_operation_log(OperationLog::UpdateIndex {
        space: "test".to_string(),
        index_name: "idx1".to_string(),
        key: vec![1, 2],
        previous_value: Some(vec![3, 4]),
    });

    ctx.add_operation_log(OperationLog::DeleteIndex {
        space: "test".to_string(),
        index_name: "idx1".to_string(),
        key: vec![1, 2],
        deleted_value: vec![5, 6],
    });

    let rollback: OperationLogRollback<TestContext> = OperationLogRollback::new(&ctx);
    assert_eq!(rollback.operation_log_len(), 2);

    let result = rollback.rollback_to_index(1);
    assert!(result.is_ok());
    assert_eq!(rollback.operation_log_len(), 1);
}
