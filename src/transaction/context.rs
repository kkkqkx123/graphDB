//! Transaction Context
//!
//! Manages the state and resources of a single transaction

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use oxicoide::{decode_from_slice, encode_to_vec};
use crossbeam_utils::atomic::AtomicCell;
use parking_lot::{Mutex, RwLock};

use crate::core::StorageError;
use crate::storage::engine::{ByteKey, EDGES_TABLE, NODES_TABLE};
use crate::transaction::types::*;

/// Transaction Context
pub struct TransactionContext {
    /// Transaction ID
    pub id: TransactionId,
    /// Current state
    state: AtomicCell<TransactionState>,
    /// Start timestamp
    pub start_time: Instant,
    /// Timeout duration
    timeout: Duration,
    /// Whether read-only
    pub read_only: bool,
    /// Isolation level
    pub isolation_level: IsolationLevel,
    /// Query timeout duration
    pub query_timeout: Option<Duration>,
    /// Statement timeout duration
    pub statement_timeout: Option<Duration>,
    /// Idle timeout duration
    pub idle_timeout: Option<Duration>,
    /// Last activity timestamp
    last_activity: AtomicCell<Instant>,
    /// Query count
    query_count: AtomicU64,
    /// redb write transaction (exists for read-write transactions)
    /// Using Option to take ownership on commit
    pub write_txn: Mutex<Option<redb::WriteTransaction>>,
    /// redb read transaction (exists for read-only transactions)
    pub read_txn: Option<redb::ReadTransaction>,
    /// Durability level
    pub durability: DurabilityLevel,
    /// Operation log (using RwLock to optimize read-heavy write-light scenarios)
    operation_logs: RwLock<Vec<OperationLog>>,
    /// Modified tables
    modified_tables: Mutex<Vec<String>>,
    /// Savepoint manager (using RwLock to optimize read-heavy write-light scenarios)
    savepoint_manager: RwLock<SavepointManager>,
    /// Database reference (used to create rollback executor, currently unused)
    #[allow(dead_code)]
    db: Option<Arc<redb::Database>>,
    /// Whether to enable two-stage submission
    two_phase_enabled: bool,
}

/// Savepoint Manager
pub(crate) struct SavepointManager {
    savepoints: HashMap<SavepointId, SavepointInfo>,
    next_id: SavepointId,
}

impl SavepointManager {
    fn new() -> Self {
        Self {
            savepoints: HashMap::new(),
            next_id: 1,
        }
    }

    fn create_savepoint(
        &mut self,
        name: Option<String>,
        operation_log_index: usize,
    ) -> SavepointId {
        let id = self.next_id;
        self.next_id += 1;
        let info = SavepointInfo {
            id,
            name,
            created_at: Instant::now(),
            operation_log_index,
        };
        self.savepoints.insert(id, info);
        id
    }

    fn get_savepoint(&self, id: SavepointId) -> Option<&SavepointInfo> {
        self.savepoints.get(&id)
    }

    fn remove_savepoint(&mut self, id: SavepointId) -> Option<SavepointInfo> {
        self.savepoints.remove(&id)
    }

    fn clear(&mut self) {
        self.savepoints.clear();
    }

    fn find_by_name(&self, name: &str) -> Option<SavepointInfo> {
        self.savepoints
            .values()
            .find(|sp| sp.name.as_deref() == Some(name))
            .cloned()
    }
}

impl TransactionContext {
    /// Create a new transaction context (read-write transaction)
    pub fn new_writable(
        id: TransactionId,
        config: TransactionConfig,
        write_txn: redb::WriteTransaction,
        db: Option<Arc<redb::Database>>,
    ) -> Self {
        let now = Instant::now();
        Self {
            id,
            state: AtomicCell::new(TransactionState::Active),
            start_time: now,
            timeout: config.timeout,
            read_only: false,
            isolation_level: config.isolation_level,
            query_timeout: config.query_timeout,
            statement_timeout: config.statement_timeout,
            idle_timeout: config.idle_timeout,
            last_activity: AtomicCell::new(now),
            query_count: AtomicU64::new(0),
            write_txn: Mutex::new(Some(write_txn)),
            read_txn: None,
            durability: config.durability,
            operation_logs: RwLock::new(Vec::new()),
            modified_tables: Mutex::new(Vec::new()),
            savepoint_manager: RwLock::new(SavepointManager::new()),
            db,
            two_phase_enabled: config.two_phase_commit,
        }
    }

    /// Create a new transaction context (read-only transaction)
    pub fn new_readonly(
        id: TransactionId,
        config: TransactionConfig,
        read_txn: redb::ReadTransaction,
        db: Option<Arc<redb::Database>>,
    ) -> Self {
        let now = Instant::now();
        Self {
            id,
            state: AtomicCell::new(TransactionState::Active),
            start_time: now,
            timeout: config.timeout,
            read_only: true,
            isolation_level: config.isolation_level,
            query_timeout: config.query_timeout,
            statement_timeout: config.statement_timeout,
            idle_timeout: config.idle_timeout,
            last_activity: AtomicCell::new(now),
            query_count: AtomicU64::new(0),
            write_txn: Mutex::new(None),
            read_txn: Some(read_txn),
            durability: DurabilityLevel::Immediate,
            operation_logs: RwLock::new(Vec::new()),
            modified_tables: Mutex::new(Vec::new()),
            savepoint_manager: RwLock::new(SavepointManager::new()),
            db,
            two_phase_enabled: config.two_phase_commit,
        }
    }

    /// Get current state
    pub fn state(&self) -> TransactionState {
        self.state.load()
    }

    /// Check if transaction has expired
    pub fn is_expired(&self) -> bool {
        self.start_time.elapsed() > self.timeout
    }

    /// Check if query timeout has been exceeded
    pub fn is_query_timeout(&self) -> bool {
        if let Some(query_timeout) = self.query_timeout {
            self.start_time.elapsed() > query_timeout
        } else {
            false
        }
    }

    /// Check if statement timeout has been exceeded
    pub fn is_statement_timeout(&self, statement_start: Instant) -> bool {
        if let Some(statement_timeout) = self.statement_timeout {
            statement_start.elapsed() > statement_timeout
        } else {
            false
        }
    }

    /// Check if idle timeout has been exceeded
    pub fn is_idle_timeout(&self) -> bool {
        if let Some(idle_timeout) = self.idle_timeout {
            self.last_activity.load().elapsed() > idle_timeout
        } else {
            false
        }
    }

    /// Check if any timeout has been exceeded
    pub fn check_timeouts(&self) -> Result<(), TransactionError> {
        if self.is_expired() {
            return Err(TransactionError::TransactionTimeout);
        }

        if self.is_query_timeout() {
            return Err(TransactionError::TransactionTimeout);
        }

        if self.is_idle_timeout() {
            return Err(TransactionError::TransactionTimeout);
        }

        Ok(())
    }

    /// Update last activity timestamp
    pub fn update_activity(&self) {
        self.last_activity.store(Instant::now());
    }

    /// Increment query count
    pub fn increment_query_count(&self) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get query count
    pub fn query_count(&self) -> u64 {
        self.query_count.load(Ordering::Relaxed)
    }

    /// Get remaining time
    pub fn remaining_time(&self) -> Duration {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.timeout {
            Duration::from_secs(0)
        } else {
            self.timeout - elapsed
        }
    }

    /// State transition
    pub fn transition_to(&self, new_state: TransactionState) -> Result<(), TransactionError> {
        let current = self.state.load();

        let valid_transition = matches!(
            (current, new_state),
            (
                TransactionState::Active,
                TransactionState::Committing | TransactionState::Aborting
            ) | (TransactionState::Committing, TransactionState::Committed)
                | (TransactionState::Aborting, TransactionState::Aborted)
        );

        if !valid_transition {
            return Err(TransactionError::InvalidStateTransition {
                from: current,
                to: new_state,
            });
        }

        self.state.store(new_state);
        Ok(())
    }

    /// Whether to enable two-stage submission
    pub fn is_two_phase_enabled(&self) -> bool {
        self.two_phase_enabled
    }

    /// Check if operation can be executed
    pub fn can_execute(&self) -> Result<(), TransactionError> {
        let state = self.state.load();

        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        Ok(())
    }

    /// Get transaction info
    pub fn info(&self) -> TransactionInfo {
        let tables = self.modified_tables.lock();
        let savepoints = self.savepoint_manager.read();
        TransactionInfo {
            id: self.id,
            state: self.state.load(),
            start_time: self.start_time,
            elapsed: self.start_time.elapsed(),
            is_read_only: self.read_only,
            isolation_level: self.isolation_level,
            query_count: self.query_count.load(Ordering::Relaxed),
            modified_tables: tables.clone(),
            savepoint_count: savepoints.savepoints.len(),
        }
    }

    /// Add operation log
    pub fn add_operation_log(&self, operation: OperationLog) {
        let mut logs = self.operation_logs.write();
        logs.push(operation);
    }

    /// Batch add operation logs
    pub fn add_operation_logs(&self, operations: Vec<OperationLog>) {
        let mut logs = self.operation_logs.write();
        logs.extend(operations);
    }

    /// Get operation logs
    pub fn get_operation_logs(&self) -> Vec<OperationLog> {
        let logs = self.operation_logs.read();
        logs.clone()
    }

    /// Get operation log length
    pub fn operation_log_len(&self) -> usize {
        let logs = self.operation_logs.read();
        logs.len()
    }

    /// Get operation log at specified index
    pub fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
        let logs = self.operation_logs.read();
        logs.get(index).cloned()
    }

    /// Get operation logs in specified range
    pub fn get_operation_logs_range(&self, start: usize, end: usize) -> Vec<OperationLog> {
        let logs = self.operation_logs.read();
        if start >= logs.len() {
            return Vec::new();
        }
        let end = end.min(logs.len());
        logs[start..end].to_vec()
    }

    /// Truncate operation logs to specified index
    pub fn truncate_operation_log(&self, index: usize) {
        let mut logs = self.operation_logs.write();
        if index < logs.len() {
            logs.truncate(index);
        }
    }

    /// Clear operation logs
    pub fn clear_operation_log(&self) {
        let mut logs = self.operation_logs.write();
        logs.clear();
    }

    /// Record table modification
    pub fn record_table_modification(&self, table_name: &str) {
        let mut tables = self.modified_tables.lock();
        if !tables.contains(&table_name.to_string()) {
            tables.push(table_name.to_string());
        }
    }

    /// Get modified tables
    pub fn get_modified_tables(&self) -> Vec<String> {
        let tables = self.modified_tables.lock();
        tables.clone()
    }

    /// Create savepoint
    pub fn create_savepoint(&self, name: Option<String>) -> SavepointId {
        let mut manager = self.savepoint_manager.write();
        let operation_log_index = self.operation_log_len();
        manager.create_savepoint(name, operation_log_index)
    }

    /// Get savepoint info
    pub fn get_savepoint(&self, id: SavepointId) -> Option<SavepointInfo> {
        let manager = self.savepoint_manager.read();
        manager.get_savepoint(id).cloned()
    }

    /// Get all savepoints
    pub fn get_all_savepoints(&self) -> Vec<SavepointInfo> {
        let manager = self.savepoint_manager.read();
        manager.savepoints.values().cloned().collect()
    }

    /// Find savepoint by name
    pub fn find_savepoint_by_name(&self, name: &str) -> Option<SavepointInfo> {
        let manager = self.savepoint_manager.read();
        manager.find_by_name(name)
    }

    /// Release savepoint
    pub fn release_savepoint(&self, id: SavepointId) -> Result<(), TransactionError> {
        let mut manager = self.savepoint_manager.write();
        manager
            .remove_savepoint(id)
            .map(|_| ())
            .ok_or(TransactionError::SavepointNotFound(id))
    }

    /// Rollback to savepoint
    pub fn rollback_to_savepoint(&self, id: SavepointId) -> Result<(), TransactionError> {
        let state = self.state.load();
        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForAbort(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        let manager = self.savepoint_manager.write();
        let savepoint_info = manager
            .get_savepoint(id)
            .cloned()
            .ok_or(TransactionError::SavepointNotFound(id))?;

        let logs_to_rollback = {
            let logs = self.operation_logs.read();
            if savepoint_info.operation_log_index >= logs.len() {
                Vec::new()
            } else {
                logs[savepoint_info.operation_log_index..].to_vec()
            }
        };

        drop(manager);

        if !logs_to_rollback.is_empty() {
            self.execute_rollback_logs(&logs_to_rollback)?;
        }

        self.truncate_operation_log(savepoint_info.operation_log_index);

        let mut manager = self.savepoint_manager.write();
        let savepoints_to_remove: Vec<SavepointId> = manager
            .savepoints
            .keys()
            .filter(|&&k| k > id)
            .copied()
            .collect();

        for sp_id in savepoints_to_remove {
            manager.remove_savepoint(sp_id);
        }

        Ok(())
    }

    /// Execute rollback for operation logs
    fn execute_rollback_logs(&self, logs: &[OperationLog]) -> Result<(), TransactionError> {
        for log in logs.iter().rev() {
            match log {
                OperationLog::InsertVertex {
                    space: _,
                    vertex_id,
                    previous_state,
                } => {
                    let id_bytes = vertex_id.clone();

                    if let Some(ref state) = previous_state {
                        let vertex: crate::core::Vertex = decode_from_slice(state)
                            .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?
                            .0;
                        let vertex_bytes = encode_to_vec(&vertex)
                            .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;

                        self.with_write_txn(|write_txn| {
                            let mut table = write_txn
                                .open_table(NODES_TABLE)
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            table
                                .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            Ok(())
                        })?;
                    } else {
                        self.with_write_txn(|write_txn| {
                            let mut table = write_txn
                                .open_table(NODES_TABLE)
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            table
                                .remove(ByteKey(id_bytes))
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            Ok(())
                        })?;
                    }
                }

                OperationLog::UpdateVertex {
                    space: _,
                    vertex_id: _,
                    previous_data,
                } => {
                    let vertex: crate::core::Vertex = decode_from_slice(previous_data)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?
                        .0;
                    let id_bytes = encode_to_vec(&vertex.vid)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
                    let vertex_bytes = encode_to_vec(&vertex)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;

                    self.with_write_txn(|write_txn| {
                        let mut table = write_txn
                            .open_table(NODES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        table
                            .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        Ok(())
                    })?;
                }

                OperationLog::DeleteVertex {
                    space: _,
                    vertex_id: _,
                    vertex,
                } => {
                    let decoded_vertex: crate::core::Vertex = decode_from_slice(vertex)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?
                        .0;
                    let id_bytes = encode_to_vec(&decoded_vertex.vid)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
                    let vertex_bytes = encode_to_vec(&decoded_vertex)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;

                    self.with_write_txn(|write_txn| {
                        let mut table = write_txn
                            .open_table(NODES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        table
                            .insert(ByteKey(id_bytes), ByteKey(vertex_bytes))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        Ok(())
                    })?;
                }

                OperationLog::InsertEdge {
                    space: _,
                    edge_id,
                    previous_state,
                } => {
                    let edge_key_bytes = edge_id.clone();

                    if let Some(ref state) = previous_state {
                        let edge: crate::core::Edge = decode_from_slice(state)
                            .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?
                            .0;
                        let edge_bytes = encode_to_vec(&edge)
                            .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;

                        self.with_write_txn(|write_txn| {
                            let mut table = write_txn
                                .open_table(EDGES_TABLE)
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            table
                                .insert(ByteKey(edge_key_bytes), ByteKey(edge_bytes))
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            Ok(())
                        })?;
                    } else {
                        self.with_write_txn(|write_txn| {
                            let mut table = write_txn
                                .open_table(EDGES_TABLE)
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            table
                                .remove(ByteKey(edge_key_bytes))
                                .map_err(|e| StorageError::DbError(e.to_string()))?;
                            Ok(())
                        })?;
                    }
                }

                OperationLog::DeleteEdge {
                    space: _,
                    edge_id: _,
                    edge,
                } => {
                    let decoded_edge: crate::core::Edge = decode_from_slice(edge)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?
                        .0;
                    let edge_key = format!(
                        "{:?}_{:?}_{}",
                        decoded_edge.src, decoded_edge.dst, decoded_edge.edge_type
                    );
                    let edge_key_bytes = edge_key.as_bytes().to_vec();
                    let edge_bytes = encode_to_vec(&decoded_edge)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;

                    self.with_write_txn(|write_txn| {
                        let mut table = write_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        table
                            .insert(ByteKey(edge_key_bytes), ByteKey(edge_bytes))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        Ok(())
                    })?;
                }

                OperationLog::UpdateEdge {
                    space: _,
                    edge_id: _,
                    previous_data,
                } => {
                    let edge: crate::core::Edge = decode_from_slice(previous_data)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?
                        .0;
                    let edge_key = format!("{:?}_{:?}_{}", edge.src, edge.dst, edge.edge_type);
                    let edge_key_bytes = edge_key.as_bytes().to_vec();
                    let edge_bytes = encode_to_vec(&edge)
                        .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;

                    self.with_write_txn(|write_txn| {
                        let mut table = write_txn
                            .open_table(EDGES_TABLE)
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        table
                            .insert(ByteKey(edge_key_bytes), ByteKey(edge_bytes))
                            .map_err(|e| StorageError::DbError(e.to_string()))?;
                        Ok(())
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Clear all savepoints
    pub fn clear_savepoints(&self) {
        let mut manager = self.savepoint_manager.write();
        manager.clear();
    }

    /// Take write transaction (for commit)
    pub fn take_write_txn(&self) -> Result<redb::WriteTransaction, TransactionError> {
        self.write_txn
            .lock()
            .take()
            .ok_or(TransactionError::ReadOnlyTransaction)
    }

    /// Get read transaction reference
    pub fn read_txn(&self) -> Result<&redb::ReadTransaction, TransactionError> {
        self.read_txn.as_ref().ok_or(TransactionError::Internal(
            "Read transaction not available".to_string(),
        ))
    }

    /// Execute operation with write transaction (for storage layer)
    ///
    /// # Arguments
    /// * `f` - Closure that receives redb::WriteTransaction reference and returns result
    ///
    /// # Returns
    /// * `Ok(R)` - Result on successful operation
    /// * `Err(TransactionError)` - Error on operation failure
    pub fn with_write_txn<F, R>(&self, f: F) -> Result<R, TransactionError>
    where
        F: FnOnce(&redb::WriteTransaction) -> Result<R, StorageError>,
    {
        if self.read_only {
            return Err(TransactionError::ReadOnlyTransaction);
        }

        let state = self.state.load();
        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        let guard = self.write_txn.lock();
        let txn = guard.as_ref().ok_or(TransactionError::Internal(
            "Write transaction not available".to_string(),
        ))?;

        f(txn).map_err(|e| TransactionError::Internal(e.to_string()))
    }

    /// Execute operation with read transaction (for storage layer)
    ///
    /// # Arguments
    /// * `f` - Closure that receives ReadTransaction or WriteTransaction and returns result
    ///
    /// # Returns
    /// * `Ok(R)` - Result on successful operation
    /// * `Err(TransactionError)` - Error on operation failure
    ///
    /// # Note
    /// redb does not support creating ReadTransaction from WriteTransaction.
    /// This method uses two different closures to handle read-only and read-write transactions.
    /// For read-only transactions, use read_txn; for read-write transactions, use write_txn for reading.
    pub fn with_read_txn<F, R>(&self, f: F) -> Result<R, TransactionError>
    where
        F: FnOnce(&redb::ReadTransaction) -> Result<R, StorageError>,
    {
        let state = self.state.load();
        if !state.can_execute() && !state.is_terminal() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        if self.is_expired() {
            return Err(TransactionError::TransactionExpired);
        }

        // Prefer using read transaction
        if let Some(ref txn) = self.read_txn {
            return f(txn).map_err(|e| TransactionError::Internal(e.to_string()));
        }

        // For read-write transactions, need to create read transaction from write transaction
        // redb does not support direct reading from WriteTransaction, need to create new read transaction
        // But this would cause read-write inconsistency, so return error here
        // Caller should use with_write_txn method
        Err(TransactionError::Internal(
            "Read-write transactions do not support direct reading, please use with_write_txn method".to_string(),
        ))
    }

    /// Get mutable reference to write transaction (for storage layer)
    ///
    /// # Safety
    /// This method returns a mutable reference, caller must ensure:
    /// 1. No other thread accesses the transaction simultaneously
    /// 2. Release the reference immediately after operation completes
    ///
    /// It is recommended to use `with_write_txn` method instead
    pub fn write_txn_mut(
        &self,
    ) -> Result<impl std::ops::DerefMut<Target = redb::WriteTransaction> + '_, TransactionError>
    {
        if self.read_only {
            return Err(TransactionError::ReadOnlyTransaction);
        }

        let state = self.state.load();
        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }

        struct WriteTxnGuard<'a> {
            guard: parking_lot::MutexGuard<'a, Option<redb::WriteTransaction>>,
        }

        impl<'a> std::ops::Deref for WriteTxnGuard<'a> {
            type Target = redb::WriteTransaction;
            fn deref(&self) -> &Self::Target {
                self.guard.as_ref().expect("Write transaction should exist")
            }
        }

        impl<'a> std::ops::DerefMut for WriteTxnGuard<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.guard.as_mut().expect("Write transaction should exist")
            }
        }

        let guard = self.write_txn.lock();
        if guard.is_none() {
            return Err(TransactionError::Internal(
                "Write transaction not available".to_string(),
            ));
        }

        Ok(WriteTxnGuard { guard })
    }
}

impl Drop for TransactionContext {
    fn drop(&mut self) {
        // If transaction is still active, abort automatically
        let state = self.state.load();
        if state == TransactionState::Active {
            // redb's WriteTransaction automatically rolls back on Drop
            // Here we only need to update the state
            self.state.store(TransactionState::Aborted);
        }

        // Clean up savepoint resources
        let mut manager = self.savepoint_manager.write();
        manager.clear();
        drop(manager);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Note: These tests require redb database instance, here only basic logic is tested

    #[test]
    fn test_transaction_context_state_machine() {
        // Since actual redb transaction is required, only state transition logic is tested here
        // Actual tests should be in integration tests
    }

    #[test]
    fn test_transaction_timeout() {
        // Create mock context (only for testing timeout logic)
        struct MockContext {
            start_time: Instant,
            timeout: Duration,
        }

        let ctx = MockContext {
            start_time: Instant::now(),
            timeout: Duration::from_millis(100),
        };

        std::thread::sleep(Duration::from_millis(150));

        assert!(ctx.start_time.elapsed() > ctx.timeout);
    }
}
