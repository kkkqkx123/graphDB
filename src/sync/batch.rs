use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::coordinator::FulltextCoordinator;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BatchConfig {
    pub batch_size: usize,
    pub commit_interval: Duration,
    pub max_wait_time: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            commit_interval: Duration::from_secs(1),
            max_wait_time: Duration::from_secs(5),
        }
    }
}

pub struct BatchProcessor {
    coordinator: Arc<FulltextCoordinator>,
    config: BatchConfig,
    pub buffers: HashMap<(u64, String, String), Vec<(String, String)>>,
    last_commit: HashMap<(u64, String, String), Instant>,
}

impl BatchProcessor {
    pub fn new(coordinator: Arc<FulltextCoordinator>, config: BatchConfig) -> Self {
        Self {
            coordinator,
            config,
            buffers: HashMap::new(),
            last_commit: HashMap::new(),
        }
    }

    pub fn add_document(
        &mut self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        doc_id: String,
        content: String,
    ) {
        let key = (space_id, tag_name.to_string(), field_name.to_string());

        self.buffers
            .entry(key.clone())
            .or_default()
            .push((doc_id, content));

        self.last_commit.entry(key).or_insert_with(Instant::now);
    }

    pub fn should_commit(&self, key: &(u64, String, String)) -> bool {
        if let Some(buffer) = self.buffers.get(key) {
            if buffer.len() >= self.config.batch_size {
                return true;
            }
        }

        if let Some(last) = self.last_commit.get(key) {
            if last.elapsed() >= self.config.commit_interval {
                return true;
            }
        }

        false
    }

    pub async fn commit_batch(
        &mut self,
        key: (u64, String, String),
    ) -> Result<(), BatchError> {
        if let Some(documents) = self.buffers.remove(&key) {
            if documents.is_empty() {
                return Ok(());
            }

            let (space_id, tag_name, field_name) = key.clone();

            if let Some(engine) = self.coordinator.get_engine(space_id, &tag_name, &field_name) {
                engine.index_batch(documents).await
                    .map_err(|e| BatchError::IndexError(e.to_string()))?;

                engine.commit().await
                    .map_err(|e| BatchError::CommitError(e.to_string()))?;
            }

            self.last_commit.insert(key, Instant::now());
        }

        Ok(())
    }

    pub async fn commit_all(&mut self) -> Vec<((u64, String, String), Result<(), BatchError>)> {
        let keys: Vec<_> = self.buffers.keys().cloned().collect();
        let mut results = Vec::new();

        for key in keys {
            let result = self.commit_batch(key.clone()).await;
            results.push((key, result));
        }

        results
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("Commit error: {0}")]
    CommitError(String),
}
