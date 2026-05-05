//! Transaction Rollback Module
//!
//! Provides rollback functionality for transactions using both OperationLog and UndoLog mechanisms.
//! The UndoLog-based rollback is the recommended approach for NeuG architecture.

use crate::core::StorageError;
use crate::transaction::types::OperationLog;
use crate::transaction::undo_log::{UndoLog, UndoLogManager, UndoTarget};
use crate::transaction::wal::types::{LabelId, Timestamp};

pub use crate::transaction::undo_log::{
    CreateVertexTypeUndo, CreateEdgeTypeUndo, InsertVertexUndo, InsertEdgeUndo,
    UpdateVertexPropUndo, UpdateEdgePropUndo, RemoveVertexUndo, RemoveEdgeUndo,
    AddVertexPropUndo, AddEdgePropUndo, DeleteVertexPropUndo, DeleteEdgePropUndo,
    DeleteVertexTypeUndo, DeleteEdgeTypeUndo, RenameVertexPropUndo, RenameEdgePropUndo,
    PropertyValue, RelatedEdgeInfo,
};

/// Operation logging context trait
///
/// Define the basic operations required for operation log rollbacks.
/// This is used for savepoint rollback functionality.
pub trait OperationLogContext {
    fn operation_log_len(&self) -> usize;
    fn truncate_operation_log(&self, index: usize);
    fn get_operation_log(&self, index: usize) -> Option<OperationLog>;
    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog>;
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

/// Undo log context trait
///
/// Defines the basic operations required for undo log rollbacks.
/// This is the primary rollback mechanism for NeuG architecture.
pub trait UndoLogContext {
    fn undo_log_len(&self) -> usize;
    fn add_undo_log(&self, log: Box<dyn UndoLog>);
    fn execute_undo_logs(&self, target: &mut dyn UndoTarget) -> Result<(), StorageError>;
    fn clear_undo_logs(&self);
}

impl UndoLogContext for crate::transaction::context::TransactionContext {
    fn undo_log_len(&self) -> usize {
        self.undo_log_len()
    }

    fn add_undo_log(&self, log: Box<dyn UndoLog>) {
        self.add_undo_log(log);
    }

    fn execute_undo_logs(&self, target: &mut dyn UndoTarget) -> Result<(), StorageError> {
        self.execute_undo_logs(target)
            .map_err(|e| StorageError::DbError(e.to_string()))
    }

    fn clear_undo_logs(&self) {
        self.clear_undo_logs();
    }
}

/// Rollback executor trait (legacy)
///
/// Define how to perform the inverse of a single operation.
/// This is kept for backward compatibility but is deprecated in favor of UndoLog.
#[deprecated(since = "0.2.0", note = "Use UndoLog trait instead")]
pub trait RollbackExecutor: Send {
    fn execute_rollback(&mut self, log: &OperationLog) -> Result<(), StorageError>;

    fn execute_rollback_batch(&mut self, logs: &[OperationLog]) -> Result<(), StorageError> {
        for log in logs.iter().rev() {
            self.execute_rollback(log)?;
        }
        Ok(())
    }
}

/// Operation Log Rollback Processor (legacy)
///
/// Responsible for performing rollback operations based on operation logs.
/// This is kept for backward compatibility but is deprecated in favor of UndoLogRollback.
#[deprecated(since = "0.2.0", note = "Use UndoLogRollback instead")]
pub struct OperationLogRollback<'a, T: OperationLogContext> {
    ctx: &'a T,
}

#[allow(deprecated)]
impl<'a, T: OperationLogContext> OperationLogRollback<'a, T> {
    pub fn new(ctx: &'a T) -> Self {
        Self { ctx }
    }

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

    pub fn operation_log_len(&self) -> usize {
        self.ctx.operation_log_len()
    }

    pub fn get_all_logs(&self) -> Vec<OperationLog> {
        let len = self.ctx.operation_log_len();
        self.ctx.get_operation_logs(0, len)
    }

    pub fn clear_logs(&self) {
        self.ctx.clear_operation_log();
    }
}

/// Undo Log Rollback Processor
///
/// Primary rollback mechanism for NeuG architecture.
/// Uses UndoLog entries to reverse operations during transaction abort.
pub struct UndoLogRollback<'a, T: UndoLogContext> {
    ctx: &'a T,
}

impl<'a, T: UndoLogContext> UndoLogRollback<'a, T> {
    pub fn new(ctx: &'a T) -> Self {
        Self { ctx }
    }

    pub fn execute_rollback(
        &self,
        target: &mut dyn UndoTarget,
        ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.ctx.execute_undo_logs(target)
    }

    pub fn undo_log_len(&self) -> usize {
        self.ctx.undo_log_len()
    }

    pub fn clear_logs(&self) {
        self.ctx.clear_undo_logs();
    }

    pub fn add_log(&self, log: Box<dyn UndoLog>) {
        self.ctx.add_undo_log(log);
    }
}

/// Combined Rollback Processor
///
/// Provides both OperationLog and UndoLog rollback capabilities.
/// Used for transactions that need to support both mechanisms.
pub struct CombinedRollback<'a, T: OperationLogContext + UndoLogContext> {
    ctx: &'a T,
}

impl<'a, T: OperationLogContext + UndoLogContext> CombinedRollback<'a, T> {
    pub fn new(ctx: &'a T) -> Self {
        Self { ctx }
    }

    pub fn execute_undo_rollback(
        &self,
        target: &mut dyn UndoTarget,
        ts: Timestamp,
    ) -> Result<(), StorageError> {
        self.ctx.execute_undo_logs(target)
    }

    pub fn rollback_operation_log_to_index(&self, index: usize) -> Result<(), StorageError> {
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

    pub fn operation_log_len(&self) -> usize {
        self.ctx.operation_log_len()
    }

    pub fn undo_log_len(&self) -> usize {
        self.ctx.undo_log_len()
    }

    pub fn clear_all_logs(&self) {
        self.ctx.clear_operation_log();
        self.ctx.clear_undo_logs();
    }
}

/// Rollback helper functions
pub struct RollbackHelper;

impl RollbackHelper {
    pub fn create_insert_vertex_undo(label: LabelId, vid: u64) -> Box<dyn UndoLog> {
        Box::new(InsertVertexUndo {
            v_label: label,
            vid,
        })
    }

    pub fn create_insert_edge_undo(
        src_label: LabelId,
        dst_label: LabelId,
        edge_label: LabelId,
        src_vid: u64,
        dst_vid: u64,
        oe_offset: i32,
        ie_offset: i32,
    ) -> Box<dyn UndoLog> {
        Box::new(InsertEdgeUndo {
            src_label,
            dst_label,
            edge_label,
            src_vid,
            dst_vid,
            oe_offset,
            ie_offset,
        })
    }

    pub fn create_update_vertex_prop_undo(
        label: LabelId,
        vid: u64,
        col_id: i32,
        old_value: PropertyValue,
    ) -> Box<dyn UndoLog> {
        Box::new(UpdateVertexPropUndo {
            v_label: label,
            vid,
            col_id,
            old_value,
        })
    }

    pub fn create_update_edge_prop_undo(
        src_label: LabelId,
        src_vid: u64,
        dst_label: LabelId,
        dst_vid: u64,
        edge_label: LabelId,
        oe_offset: i32,
        ie_offset: i32,
        col_id: i32,
        old_value: PropertyValue,
    ) -> Box<dyn UndoLog> {
        Box::new(UpdateEdgePropUndo {
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            oe_offset,
            ie_offset,
            col_id,
            old_value,
        })
    }

    pub fn create_remove_vertex_undo(
        label: LabelId,
        vid: u64,
        related_edges: Vec<(LabelId, LabelId, LabelId, Vec<RelatedEdgeInfo>)>,
    ) -> Box<dyn UndoLog> {
        Box::new(RemoveVertexUndo {
            v_label: label,
            vid,
            related_edges,
        })
    }

    pub fn create_remove_edge_undo(
        src_label: LabelId,
        src_vid: u64,
        dst_label: LabelId,
        dst_vid: u64,
        edge_label: LabelId,
        oe_offset: i32,
        ie_offset: i32,
    ) -> Box<dyn UndoLog> {
        Box::new(RemoveEdgeUndo {
            src_label,
            src_vid,
            dst_label,
            dst_vid,
            edge_label,
            oe_offset,
            ie_offset,
        })
    }

    pub fn create_create_vertex_type_undo(label: LabelId) -> Box<dyn UndoLog> {
        Box::new(CreateVertexTypeUndo { vertex_type: label })
    }

    pub fn create_create_edge_type_undo(
        src_type: LabelId,
        dst_type: LabelId,
        edge_type: LabelId,
    ) -> Box<dyn UndoLog> {
        Box::new(CreateEdgeTypeUndo {
            src_type,
            dst_type,
            edge_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUndoContext {
        logs: std::cell::RefCell<UndoLogManager>,
    }

    impl MockUndoContext {
        fn new() -> Self {
            Self {
                logs: std::cell::RefCell::new(UndoLogManager::new()),
            }
        }
    }

    impl UndoLogContext for MockUndoContext {
        fn undo_log_len(&self) -> usize {
            self.logs.borrow().len()
        }

        fn add_undo_log(&self, log: Box<dyn UndoLog>) {
            self.logs.borrow_mut().add(log);
        }

        fn execute_undo_logs(&self, _target: &mut dyn UndoTarget) -> Result<(), StorageError> {
            self.logs.borrow_mut().clear();
            Ok(())
        }

        fn clear_undo_logs(&self) {
            self.logs.borrow_mut().clear();
        }
    }

    #[test]
    fn test_undo_log_rollback() {
        let ctx = MockUndoContext::new();
        let rollback = UndoLogRollback::new(&ctx);

        assert_eq!(rollback.undo_log_len(), 0);

        rollback.add_log(RollbackHelper::create_insert_vertex_undo(1, 100));
        assert_eq!(rollback.undo_log_len(), 1);

        rollback.clear_logs();
        assert_eq!(rollback.undo_log_len(), 0);
    }

    #[test]
    fn test_rollback_helper() {
        let undo = RollbackHelper::create_insert_vertex_undo(1, 100);
        assert!(undo.description().contains("InsertVertexUndo"));

        let undo = RollbackHelper::create_insert_edge_undo(1, 2, 3, 100, 200, 0, 0);
        assert!(undo.description().contains("InsertEdgeUndo"));

        let undo = RollbackHelper::create_update_vertex_prop_undo(
            1, 100, 0, PropertyValue::Int(42)
        );
        assert!(undo.description().contains("UpdateVertexPropUndo"));
    }
}
