//! Unified Task Buffer
//!
//! Combines task queue and batch processing into a single component.

use crate::coordinator::FulltextCoordinator;
use crate::sync::task::SyncTask;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};

type IndexKey = (u64, String, String);
type Document = (String, String);

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub commit_interval: Duration,
    pub max_wait_time: Duration,
    pub queue_capacity: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            commit_interval: Duration::from_secs(1),
            max_wait_time: Duration::from_secs(5),
            queue_capacity: 10000,
        }
    }
}

pub struct TaskBuffer {
    coordinator: Arc<FulltextCoordinator>,
    config: BatchConfig,
    sender: mpsc::Sender<SyncTask>,
    receiver: Mutex<mpsc::Receiver<SyncTask>>,
    doc_buffers: Mutex<HashMap<IndexKey, Vec<Document>>>,
    delete_buffers: Mutex<HashMap<IndexKey, Vec<String>>>,
    last_commit: Mutex<HashMap<IndexKey, Instant>>,
}

impl TaskBuffer {
    pub fn new(coordinator: Arc<FulltextCoordinator>, config: BatchConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.queue_capacity);
        Self {
            coordinator,
            config,
            sender,
            receiver: Mutex::new(receiver),
            doc_buffers: Mutex::new(HashMap::new()),
            delete_buffers: Mutex::new(HashMap::new()),
            last_commit: Mutex::new(HashMap::new()),
        }
    }

    pub async fn submit(&self, task: SyncTask) -> Result<(), BufferError> {
        match self.sender.try_send(task) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => Err(BufferError::QueueFull),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(BufferError::QueueClosed),
        }
    }

    pub async fn submit_blocking(&self, task: SyncTask) -> Result<(), BufferError> {
        self.sender
            .send(task)
            .await
            .map_err(|_| BufferError::QueueClosed)
    }

    pub async fn next_task(&self) -> Option<SyncTask> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }

    pub async fn try_next_task(&self) -> Option<SyncTask> {
        let mut receiver = self.receiver.lock().await;
        receiver.try_recv().ok()
    }

    pub fn capacity(&self) -> usize {
        self.config.queue_capacity
    }

    pub async fn add_document(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        doc_id: String,
        content: String,
    ) {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        let mut buffers = self.doc_buffers.lock().await;
        buffers
            .entry(key.clone())
            .or_default()
            .push((doc_id, content));

        let mut last_commit = self.last_commit.lock().await;
        last_commit.entry(key).or_insert_with(Instant::now);
    }

    pub async fn add_deletion(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        doc_id: String,
    ) {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        let mut buffers = self.delete_buffers.lock().await;
        buffers.entry(key.clone()).or_default().push(doc_id);

        let mut last_commit = self.last_commit.lock().await;
        last_commit.entry(key).or_insert_with(Instant::now);
    }

    pub async fn should_commit(&self, key: &IndexKey) -> bool {
        {
            let buffers = self.doc_buffers.lock().await;
            if let Some(buffer) = buffers.get(key) {
                if buffer.len() >= self.config.batch_size {
                    return true;
                }
            }
        }

        {
            let buffers = self.delete_buffers.lock().await;
            if let Some(buffer) = buffers.get(key) {
                if buffer.len() >= self.config.batch_size {
                    return true;
                }
            }
        }

        let last_commit = self.last_commit.lock().await;
        if let Some(last) = last_commit.get(key) {
            if last.elapsed() >= self.config.commit_interval {
                return true;
            }
        }

        false
    }

    pub async fn commit_batch(&self, key: IndexKey) -> Result<(), BufferError> {
        let mut buffers = self.doc_buffers.lock().await;
        if let Some(documents) = buffers.remove(&key) {
            if documents.is_empty() {
                return Ok(());
            }

            let (space_id, tag_name, field_name) = key.clone();

            if let Some(engine) = self
                .coordinator
                .get_engine(space_id, &tag_name, &field_name)
            {
                engine
                    .index_batch(documents)
                    .await
                    .map_err(|e| BufferError::IndexError(e.to_string()))?;

                engine
                    .commit()
                    .await
                    .map_err(|e| BufferError::CommitError(e.to_string()))?;
            }

            let mut last_commit = self.last_commit.lock().await;
            last_commit.insert(key, Instant::now());
        }

        Ok(())
    }

    pub async fn commit_deletions(&self, key: IndexKey) -> Result<(), BufferError> {
        let mut buffers = self.delete_buffers.lock().await;
        if let Some(doc_ids) = buffers.remove(&key) {
            if doc_ids.is_empty() {
                return Ok(());
            }

            let (space_id, tag_name, field_name) = key.clone();

            if let Some(engine) = self
                .coordinator
                .get_engine(space_id, &tag_name, &field_name)
            {
                for doc_id in &doc_ids {
                    engine
                        .delete(doc_id)
                        .await
                        .map_err(|e| BufferError::IndexError(e.to_string()))?;
                }

                engine
                    .commit()
                    .await
                    .map_err(|e| BufferError::CommitError(e.to_string()))?;
            }

            let mut last_commit = self.last_commit.lock().await;
            last_commit.insert(key, Instant::now());
        }

        Ok(())
    }

    pub async fn commit_all(&self) -> Vec<(IndexKey, Result<(), BufferError>)> {
        let doc_keys: Vec<_> = {
            let buffers = self.doc_buffers.lock().await;
            buffers.keys().cloned().collect()
        };

        let delete_keys: Vec<_> = {
            let buffers = self.delete_buffers.lock().await;
            buffers.keys().cloned().collect()
        };

        let mut results = Vec::new();

        for key in doc_keys {
            let result = self.commit_batch(key.clone()).await;
            results.push((key, result));
        }

        for key in delete_keys {
            let result = self.commit_deletions(key.clone()).await;
            results.push((key, result));
        }

        results
    }

    pub async fn get_buffer_keys(&self) -> Vec<IndexKey> {
        let doc_buffers = self.doc_buffers.lock().await;
        let delete_buffers = self.delete_buffers.lock().await;
        let mut keys: Vec<_> = doc_buffers.keys().cloned().collect();
        for key in delete_buffers.keys() {
            if !keys.contains(key) {
                keys.push(key.clone());
            }
        }
        keys
    }

    pub fn coordinator(&self) -> &Arc<FulltextCoordinator> {
        &self.coordinator
    }

    pub fn config(&self) -> &BatchConfig {
        &self.config
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BufferError {
    #[error("Queue is full")]
    QueueFull,
    #[error("Queue is closed")]
    QueueClosed,
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("Commit error: {0}")]
    CommitError(String),
}
