//! Operation Log Rollback Module
//!
//! Provide operation log based rollback function, support transaction save point rollback

use crate::core::StorageError;
use crate::transaction::types::OperationLog;

/// Operation logging context trait
///
/// Define the basic operations required for operation log rollbacks
pub trait OperationLogContext {
    /// Get operation log length
    fn operation_log_len(&self) -> usize;
    /// Truncate operation logs to a specified index
    fn truncate_operation_log(&self, index: usize);
    /// Get the operation log of the specified index
    fn get_operation_log(&self, index: usize) -> Option<OperationLog>;
    /// Get the operation log for a specified range
    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog>;
    /// Empty operation log
    fn clear_operation_log(&self);
}

impl OperationLogContext for crate::transaction::context::TransactionContext {
    fn operation_log_len(&self) -> usize {
        self.operation_log_len()
    }

    fn truncate_operation_log(&self, index: usize) {
        self.truncate_operation_log(index);
    }

    fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
        self.get_operation_log(index)
    }

    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
        self.get_operation_logs_range(start, end)
    }

    fn clear_operation_log(&self) {
        self.clear_operation_log();
    }
}

/// Rollback executor trait
///
/// Define how to perform the inverse of a single operation
pub trait RollbackExecutor: Send {
    /// Perform inverse operation (rollback) of a single operation log
    ///
    /// # Arguments
    /// * :: `log` -- log of operations to be rolled back
    ///
    /// # Returns
    /// * `Ok(())` - Rollback successful
    /// * `Err(StorageError)` - Rollback failed
    fn execute_rollback(&mut self, log: &OperationLog) -> Result<(), StorageError>;

    /// Batch execution of rollback operations
    ///
    /// Performs rollback of operation logs in reverse order
    ///
    /// # Arguments
    /// * `logs` - a list of operation logs to be rolled back
    ///
    /// # Returns
    /// * `Ok(())` - Rollback successful
    /// * `Err(StorageError)` - Rollback failed
    fn execute_rollback_batch(&mut self, logs: &[OperationLog]) -> Result<(), StorageError> {
        for log in logs.iter().rev() {
            self.execute_rollback(log)?;
        }
        Ok(())
    }
}

/// Operation Log Rollback Processor
///
/// Responsible for performing rollback operations based on operation logs
pub struct OperationLogRollback<'a, T: OperationLogContext> {
    ctx: &'a T,
}

impl<'a, T: OperationLogContext> OperationLogRollback<'a, T> {
    /// Creating a new rollback processor
    pub fn new(ctx: &'a T) -> Self {
        Self { ctx }
    }

    /// Rollback to the specified operation log index
    pub fn rollback_to_index(&self, index: usize) -> Result<(), StorageError> {
        let current_len = self.ctx.operation_log_len();

        if index > current_len {
            return Err(StorageError::DbError(format!(
                "Invalid rollback index: {}, operation log length: {}",
                index, current_len
            )));
        }

        self.ctx.truncate_operation_log(index);

        Ok(())
    }

    /// Use the executor to roll back to a specified operation log index
    pub fn execute_rollback_to_index<E: RollbackExecutor>(
        &self,
        index: usize,
        executor: &mut E,
    ) -> Result<(), StorageError> {
        let current_len = self.ctx.operation_log_len();

        if index > current_len {
            return Err(StorageError::DbError(format!(
                "Invalid rollback index: {}, operation log length: {}",
                index, current_len
            )));
        }

        let logs_to_rollback = self.ctx.get_operation_logs(index, current_len);

        executor.execute_rollback_batch(&logs_to_rollback)?;

        self.ctx.truncate_operation_log(index);

        Ok(())
    }

    /// Get operation log length
    pub fn operation_log_len(&self) -> usize {
        self.ctx.operation_log_len()
    }

    /// Get all operation logs
    pub fn get_all_logs(&self) -> Vec<OperationLog> {
        let len = self.ctx.operation_log_len();
        self.ctx.get_operation_logs(0, len)
    }

    /// Empty all operation logs
    pub fn clear_logs(&self) {
        self.ctx.clear_operation_log();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockContext {
        logs: std::cell::RefCell<Vec<OperationLog>>,
    }

    impl MockContext {
        fn new() -> Self {
            Self {
                logs: std::cell::RefCell::new(Vec::new()),
            }
        }

        fn add_log(&self, log: OperationLog) {
            self.logs.borrow_mut().push(log);
        }
    }

    impl OperationLogContext for MockContext {
        fn operation_log_len(&self) -> usize {
            self.logs.borrow().len()
        }

        fn truncate_operation_log(&self, index: usize) {
            self.logs.borrow_mut().truncate(index);
        }

        fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
            self.logs.borrow().get(index).cloned()
        }

        fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
            let logs = self.logs.borrow();
            if start >= logs.len() {
                return Vec::new();
            }
            let end = end.min(logs.len());
            logs[start..end].to_vec()
        }

        fn clear_operation_log(&self) {
            self.logs.borrow_mut().clear();
        }
    }

    #[test]
    fn test_rollback_to_index() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);

        ctx.add_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_state: None,
        });

        ctx.add_log(OperationLog::UpdateVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_data: vec![4, 5, 6],
        });

        assert_eq!(rollback.operation_log_len(), 2);

        let result = rollback.rollback_to_index(1);
        assert!(result.is_ok());
        assert_eq!(rollback.operation_log_len(), 1);
    }
}
