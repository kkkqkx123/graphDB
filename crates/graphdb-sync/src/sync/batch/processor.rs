use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::Mutex;

use super::buffer::BatchBuffer;
use super::config::BatchConfig;
use super::error::{BatchError, BatchResult};
use super::trait_def::BatchProcessor;
use crate::core::types::TransactionId;
use crate::search::engine::SearchEngine;
use crate::sync::types::{IndexOpKey, IndexOperation};

pub struct FulltextBatchProcessor {
    space_id: u64,
    tag_name: String,
    field_name: String,
    engine: Arc<dyn SearchEngine>,
    config: BatchConfig,
    buffer: Arc<BatchBuffer>,
    background_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    immediate_mode: bool,
}

impl std::fmt::Debug for FulltextBatchProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FulltextBatchProcessor")
            .field("location", &(self.space_id, &self.tag_name, &self.field_name))
            .field("config", &self.config)
            .field("buffer_count", &self.buffer.total_count())
            .finish_non_exhaustive()
    }
}

impl FulltextBatchProcessor {
    pub fn new(
        space_id: u64,
        tag_name: String,
        field_name: String,
        engine: Arc<dyn SearchEngine>,
        config: BatchConfig,
    ) -> Self {
        Self {
            space_id,
            tag_name,
            field_name,
            engine,
            config,
            buffer: Arc::new(BatchBuffer::new()),
            background_task: Mutex::new(None),
            immediate_mode: false,
        }
    }

    pub fn new_immediate(
        space_id: u64,
        tag_name: String,
        field_name: String,
        engine: Arc<dyn SearchEngine>,
    ) -> Self {
        Self {
            space_id,
            tag_name,
            field_name,
            engine,
            config: BatchConfig::default(),
            buffer: Arc::new(BatchBuffer::new()),
            background_task: Mutex::new(None),
            immediate_mode: true,
        }
    }

    fn location(&self) -> (u64, String, String) {
        (self.space_id, self.tag_name.clone(), self.field_name.clone())
    }

    async fn execute_immediate(&self, operation: IndexOperation) -> BatchResult<()> {
        match operation {
            IndexOperation::Insert { id, text, .. } | IndexOperation::Update { id, text, .. } => {
                self.engine
                    .index_batch(vec![(id, text)])
                    .await
                    .map_err(BatchError::from)?;
            }
            IndexOperation::Delete { id, .. } => {
                self.engine
                    .delete_batch(vec![id.as_str()])
                    .await
                    .map_err(BatchError::from)?;
            }
        }
        self.engine.commit().await.map_err(BatchError::from)?;
        Ok(())
    }

    /// Execute a batch of operations immediately, bypassing the buffer.
    ///
    /// This is used for transactional commits where operations should be
    /// applied directly without additional buffering, eliminating the
    /// double-buffering between TransactionBatchBuffer and BatchBuffer.
    pub async fn execute_now(&self, operations: Vec<IndexOperation>) -> BatchResult<()> {
        self.execute_now_without_commit(operations).await?;
        self.engine.commit().await.map_err(BatchError::from)?;
        Ok(())
    }

    /// Like `execute_now`, but does NOT commit.
    ///
    /// This allows multiple batch processors to accumulate changes
    /// before a single final commit across all of them, enabling
    /// atomic multi-index transactional commits.
    pub async fn execute_now_without_commit(
        &self,
        operations: Vec<IndexOperation>,
    ) -> BatchResult<()> {
        let mut deletes = Vec::new();
        let mut items = Vec::new();

        for op in operations {
            match op {
                IndexOperation::Delete { id, .. } => deletes.push(id),
                IndexOperation::Insert { id, text, .. }
                | IndexOperation::Update { id, text, .. } => items.push((id, text)),
            }
        }

        if !deletes.is_empty() {
            let ids: Vec<&str> = deletes.iter().map(|s| s.as_str()).collect();
            self.engine
                .delete_batch(ids)
                .await
                .map_err(BatchError::from)?;
        }

        if !items.is_empty() {
            self.engine
                .index_batch(items)
                .await
                .map_err(BatchError::from)?;
        }

        Ok(())
    }

    pub fn engine(&self) -> &Arc<dyn SearchEngine> {
        &self.engine
    }

    pub fn buffer(&self) -> &Arc<BatchBuffer> {
        &self.buffer
    }

    async fn should_commit(&self, key: &(u64, String, String)) -> bool {
        if self.buffer.count(key) >= self.config.batch_size {
            return true;
        }
        self.buffer.is_timeout(key, self.config.flush_interval)
    }

    async fn execute_batch(&self, key: &(u64, String, String)) -> BatchResult<()> {
        let entry = self.buffer.drain_all(key);

        if !entry.deletes.is_empty() {
            let ids: Vec<&str> = entry.deletes.iter().map(|s| s.as_str()).collect();
            self.engine
                .delete_batch(ids)
                .await
                .map_err(BatchError::from)?;
        }

        if !entry.inserts.is_empty() {
            let items: Vec<(String, String)> = entry
                .inserts
                .into_iter()
                .filter_map(|op| match op {
                    IndexOperation::Insert { id, text, .. } => Some((id, text)),
                    IndexOperation::Update { id, text, .. } => Some((id, text)),
                    _ => None,
                })
                .collect();

            if !items.is_empty() {
                self.engine
                    .index_batch(items)
                    .await
                    .map_err(BatchError::from)?;
            }
        }

        self.engine.commit().await.map_err(BatchError::from)?;
        self.buffer.update_commit_time(key);

        Ok(())
    }
}

impl Drop for FulltextBatchProcessor {
    fn drop(&mut self) {
        if let Ok(handle) = self.background_task.try_lock() {
            if let Some(task) = handle.as_ref() {
                task.abort();
            }
        }
    }
}

#[async_trait]
impl BatchProcessor for FulltextBatchProcessor {
    async fn add(&self, operation: IndexOperation) -> BatchResult<()> {
        if self.immediate_mode {
            return self.execute_immediate(operation).await;
        }

        let key = self.location();

        match &operation {
            IndexOperation::Insert { .. } | IndexOperation::Update { .. } => {
                self.buffer.add_insert(&key, operation);
            }
            IndexOperation::Delete { id, .. } => {
                self.buffer.add_delete(&key, id.clone());
            }
        }

        if self.should_commit(&key).await {
            self.execute_batch(&key).await?;
        }

        Ok(())
    }

    async fn add_batch(&self, operations: Vec<IndexOperation>) -> BatchResult<()> {
        if self.immediate_mode {
            for operation in operations {
                self.execute_immediate(operation).await?;
            }
            return Ok(());
        }

        let key = self.location();

        for operation in operations {
            match &operation {
                IndexOperation::Insert { .. } | IndexOperation::Update { .. } => {
                    self.buffer.add_insert(&key, operation);
                }
                IndexOperation::Delete { id, .. } => {
                    self.buffer.add_delete(&key, id.clone());
                }
            }
        }

        if self.should_commit(&key).await {
            self.execute_batch(&key).await?;
        }

        Ok(())
    }

    async fn commit_all(&self) -> BatchResult<()> {
        let keys = self.buffer.keys();
        for key in &keys {
            self.execute_batch(key).await?;
        }
        if keys.is_empty() {
            self.engine.commit().await.map_err(BatchError::from)?;
        }
        Ok(())
    }

    async fn commit_timeout(&self) -> BatchResult<()> {
        let keys = self.buffer.keys();
        for key in keys {
            if self.buffer.is_timeout(&key, self.config.flush_interval) {
                self.execute_batch(&key).await?;
            }
        }
        Ok(())
    }

    async fn start_background_task(self: Arc<Self>) {
        if self.immediate_mode {
            return;
        }

        let mut handle = self.background_task.lock().await;
        if handle.is_some() {
            return;
        }

        let processor = self.clone();
        let interval = processor.config.flush_interval;

        let task = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                if let Err(e) = processor.commit_timeout().await {
                    tracing::error!("Background batch commit failed: {:?}", e);
                }
            }
        });

        *handle = Some(task);
    }

    async fn stop_background_task(&self) {
        let mut handle = self.background_task.lock().await;
        if let Some(task) = handle.take() {
            task.abort();
        }
    }
}

/// Entry for buffering operations within a transaction
#[derive(Debug, Default)]
pub struct TransactionBufferEntry {
    pub operations: Vec<IndexOperation>,
}

/// Transaction-aware buffer for index operations
///
/// This buffer provides temporary storage for index operations during a transaction.
/// It supports two-phase commit pattern:
/// - Phase 1 (prepare): Buffer operations as they are generated
/// - Phase 2 (commit/rollback): Either clear the buffer (commit) or discard (rollback)
///
/// Note: This buffer only stores operations. The actual execution of operations
/// must be performed by the caller using `take_operations()` to retrieve and execute them.
pub struct TransactionBatchBuffer {
    pending: DashMap<TransactionId, DashMap<IndexOpKey, TransactionBufferEntry>>,
}

impl std::fmt::Debug for TransactionBatchBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionBatchBuffer")
            .field("pending_count", &self.pending.len())
            .finish_non_exhaustive()
    }
}

impl Default for TransactionBatchBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionBatchBuffer {
    /// Create a new transaction batch buffer
    pub fn new() -> Self {
        Self {
            pending: DashMap::new(),
        }
    }

    /// Take all buffered operations for a transaction
    ///
    /// This removes the operations from the buffer and returns them.
    /// The caller is responsible for executing these operations.
    pub fn take_operations(
        &self,
        txn_id: TransactionId,
    ) -> BatchResult<Vec<(IndexOpKey, Vec<IndexOperation>)>> {
        if let Some((_, txn_buffer)) = self.pending.remove(&txn_id) {
            let mut result = Vec::new();
            for entry in txn_buffer.iter() {
                let key = entry.key().clone();
                let ops = entry.value().operations.clone();
                if !ops.is_empty() {
                    result.push((key, ops));
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Buffer an operation for the given transaction
    pub fn prepare(&self, txn_id: TransactionId, operation: IndexOperation) -> BatchResult<()> {
        let txn_buffer = self.pending.entry(txn_id).or_default();

        let key = match &operation {
            IndexOperation::Insert { key, .. }
            | IndexOperation::Update { key, .. }
            | IndexOperation::Delete { key, .. } => key.clone(),
        };

        let mut entry = txn_buffer.entry(key).or_default();
        entry.operations.push(operation);
        Ok(())
    }

    /// Peek at operations without removing them (non-destructive)
    ///
    /// Returns a clone of all grouped operations for validation during prepare phase.
    pub fn peek_operations(
        &self,
        txn_id: TransactionId,
    ) -> BatchResult<Vec<(IndexOpKey, Vec<IndexOperation>)>> {
        if let Some(txn_buffer) = self.pending.get(&txn_id) {
            let mut result = Vec::new();
            for entry in txn_buffer.iter() {
                let key = entry.key().clone();
                let ops = entry.value().operations.clone();
                if !ops.is_empty() {
                    result.push((key, ops));
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    /// Rollback the transaction by discarding all buffered operations
    pub fn rollback(&self, txn_id: TransactionId) -> BatchResult<()> {
        self.pending.remove(&txn_id);
        Ok(())
    }

    /// Get the number of pending operations for a transaction
    pub fn pending_count(&self, txn_id: TransactionId) -> usize {
        self.pending
            .get(&txn_id)
            .map(|txn_buffer| txn_buffer.iter().map(|e| e.value().operations.len()).sum())
            .unwrap_or(0)
    }
}
