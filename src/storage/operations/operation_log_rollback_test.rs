//! OperationLogRollback Test
//!
//! Testing the functionality of the operation log rollback processor

use std::cell::RefCell;

use crate::storage::operations::rollback::{
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
    fn execute_rollback(&mut self, _log: &OperationLog) -> Result<(), crate::core::StorageError> {
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
