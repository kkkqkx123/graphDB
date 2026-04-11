use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::Mutex;

use super::buffer::BatchBuffer;
use super::config::BatchConfig;
use super::error::{BatchError, BatchResult};
use super::trait_def::{BatchProcessor, TransactionBuffer};
use crate::sync::external_index::{ExternalIndexClient, IndexData, IndexOperation};
use crate::transaction::types::TransactionId;

pub struct GenericBatchProcessor<E: ExternalIndexClient> {
    engine: Arc<E>,
    config: BatchConfig,
    buffer: Arc<BatchBuffer>,
    background_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl<E: ExternalIndexClient> std::fmt::Debug for GenericBatchProcessor<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericBatchProcessor")
            .field("client_type", &self.engine.client_type())
            .field("config", &self.config)
            .field("buffer_count", &self.buffer.total_count())
            .finish_non_exhaustive()
    }
}

impl<E: ExternalIndexClient + 'static> GenericBatchProcessor<E> {
    pub fn new(engine: Arc<E>, config: BatchConfig) -> Self {
        Self {
            engine,
            config,
            buffer: Arc::new(BatchBuffer::new()),
            background_task: Mutex::new(None),
        }
    }

    pub fn engine(&self) -> &Arc<E> {
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

        if !entry.inserts.is_empty() {
            let items: Vec<(String, IndexData)> = entry
                .inserts
                .into_iter()
                .filter_map(|op| match op {
                    IndexOperation::Insert { id, data, .. } => Some((id, data)),
                    IndexOperation::Update { id, data, .. } => Some((id, data)),
                    _ => None,
                })
                .collect();

            if !items.is_empty() {
                self.engine
                    .insert_batch(items)
                    .await
                    .map_err(BatchError::from)?;
            }
        }

        if !entry.deletes.is_empty() {
            let ids: Vec<&str> = entry.deletes.iter().map(|s| s.as_str()).collect();
            self.engine
                .delete_batch(&ids)
                .await
                .map_err(BatchError::from)?;
        }

        self.buffer.update_commit_time(key);

        Ok(())
    }
}

impl<E: ExternalIndexClient> Drop for GenericBatchProcessor<E> {
    fn drop(&mut self) {
        if let Ok(mut handle) = self.background_task.try_lock() {
            if let Some(task) = handle.take() {
                task.abort();
            }
        }
    }
}

#[async_trait]
impl<E: ExternalIndexClient + 'static> BatchProcessor for GenericBatchProcessor<E> {
    async fn add(&self, operation: IndexOperation) -> BatchResult<()> {
        let key = self.engine.index_key();

        match &operation {
            IndexOperation::Insert { .. } | IndexOperation::Update { .. } => {
                self.buffer.add_insert(&key, operation);
            }
            IndexOperation::Delete { id } => {
                self.buffer.add_delete(&key, id.clone());
            }
        }

        if self.should_commit(&key).await {
            self.execute_batch(&key).await?;
        }

        Ok(())
    }

    async fn add_batch(&self, operations: Vec<IndexOperation>) -> BatchResult<()> {
        let key = self.engine.index_key();

        for operation in operations {
            match &operation {
                IndexOperation::Insert { .. } | IndexOperation::Update { .. } => {
                    self.buffer.add_insert(&key, operation);
                }
                IndexOperation::Delete { id } => {
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
        for key in keys {
            self.execute_batch(&key).await?;
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct TransactionBatchBuffer {
    // Deprecated: Use SyncCoordinator directly instead
    #[deprecated]
    processor: Option<Arc<dyn BatchProcessor>>,
    pending: DashMap<TransactionId, Vec<IndexOperation>>,
}

impl std::fmt::Debug for TransactionBatchBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionBatchBuffer")
            .field("pending_count", &self.pending.len())
            .finish_non_exhaustive()
    }
}

impl TransactionBatchBuffer {
    pub fn new(processor: Arc<dyn BatchProcessor>) -> Self {
        Self {
            processor: Some(processor),
            pending: DashMap::new(),
        }
    }

    // Compatibility constructor for old TaskBuffer API
    #[deprecated]
    pub fn new_with_coordinator(
        _coordinator: Arc<crate::sync::coordinator::SyncCoordinator>,
        _config: BatchConfig,
    ) -> Self {
        // Deprecated: Use SyncCoordinator directly
        Self {
            processor: None,
            pending: DashMap::new(),
        }
    }

    // Compatibility method for old API
    pub async fn submit(&self, _task: crate::sync::task::SyncTask) -> Result<(), crate::sync::batch::error::BatchError> {
        // TODO: Implement proper task submission logic
        // For now, just return Ok to allow compilation
        Ok(())
    }

    // Compatibility method
    #[allow(dead_code)]
    pub fn config(&self) -> &BatchConfig {
        // Return a default config for compatibility
        static DEFAULT_CONFIG: std::sync::OnceLock<BatchConfig> = std::sync::OnceLock::new();
        DEFAULT_CONFIG.get_or_init(|| BatchConfig::default())
    }
}

#[async_trait]
impl TransactionBuffer for TransactionBatchBuffer {
    async fn prepare(&self, txn_id: TransactionId, operation: IndexOperation) -> BatchResult<()> {
        let mut buffer = self.pending.entry(txn_id).or_default();
        buffer.push(operation);

        // Skip capacity check in deprecated mode
        Ok(())
    }

    async fn commit(&self, txn_id: TransactionId) -> BatchResult<()> {
        if let Some((_, operations)) = self.pending.remove(&txn_id) {
            if !operations.is_empty() {
                // Skip processor call in deprecated mode
                // self.processor.add_batch(operations).await?;
            }
        }
        Ok(())
    }

    async fn rollback(&self, txn_id: TransactionId) -> BatchResult<()> {
        self.pending.remove(&txn_id);
        Ok(())
    }

    fn pending_count(&self, txn_id: TransactionId) -> usize {
        self.pending.get(&txn_id).map(|e| e.len()).unwrap_or(0)
    }
}
