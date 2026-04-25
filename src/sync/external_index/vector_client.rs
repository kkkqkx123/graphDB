//! Vector Index Client with Remote Index Support
//!
//! This client handles synchronization with remote vector databases (e.g., Qdrant).
//! It includes retry mechanism, timeout control, and dead letter queue support
//! for handling network-related failures.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tracing::{debug, warn};

use super::error::{ExternalIndexError, IndexResult};
use super::trait_def::{ExternalIndexClient, IndexData, IndexStats};
use crate::sync::dead_letter_queue::DeadLetterQueue;
use crate::sync::retry::{with_retry, RetryConfig};

/// Default timeout for vector operations
const DEFAULT_OPERATION_TIMEOUT_MS: u64 = 30_000;

/// Default retry configuration for remote operations
fn default_retry_config() -> RetryConfig {
    RetryConfig::new(3, Duration::from_millis(100), Duration::from_secs(10))
}

/// Configuration for VectorClient
#[derive(Debug, Clone)]
pub struct VectorClientConfig {
    /// Retry configuration for failed operations
    pub retry_config: RetryConfig,
    /// Timeout for individual operations
    pub operation_timeout: Duration,
    /// Whether to use dead letter queue for failed operations
    pub use_dead_letter_queue: bool,
}

impl Default for VectorClientConfig {
    fn default() -> Self {
        Self {
            retry_config: default_retry_config(),
            operation_timeout: Duration::from_millis(DEFAULT_OPERATION_TIMEOUT_MS),
            use_dead_letter_queue: true,
        }
    }
}

impl VectorClientConfig {
    pub fn new(retry_config: RetryConfig, operation_timeout: Duration) -> Self {
        Self {
            retry_config,
            operation_timeout,
            use_dead_letter_queue: true,
        }
    }

    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    pub fn with_operation_timeout(mut self, timeout: Duration) -> Self {
        self.operation_timeout = timeout;
        self
    }

    pub fn with_dead_letter_queue(mut self, use_dlq: bool) -> Self {
        self.use_dead_letter_queue = use_dlq;
        self
    }
}

pub struct VectorClient {
    space_id: u64,
    tag_name: String,
    field_name: String,
    vector_manager: Arc<vector_client::VectorManager>,
    config: VectorClientConfig,
    dead_letter_queue: Option<Arc<DeadLetterQueue>>,
}

impl std::fmt::Debug for VectorClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorClient")
            .field("space_id", &self.space_id)
            .field("tag_name", &self.tag_name)
            .field("field_name", &self.field_name)
            .field("config", &self.config)
            .finish()
    }
}

impl VectorClient {
    pub fn new(
        space_id: u64,
        tag_name: String,
        field_name: String,
        vector_manager: Arc<vector_client::VectorManager>,
    ) -> Self {
        Self {
            space_id,
            tag_name,
            field_name,
            vector_manager,
            config: VectorClientConfig::default(),
            dead_letter_queue: None,
        }
    }

    pub fn with_config(
        space_id: u64,
        tag_name: String,
        field_name: String,
        vector_manager: Arc<vector_client::VectorManager>,
        config: VectorClientConfig,
    ) -> Self {
        Self {
            space_id,
            tag_name,
            field_name,
            vector_manager,
            config,
            dead_letter_queue: None,
        }
    }

    pub fn with_dead_letter_queue(mut self, dlq: Arc<DeadLetterQueue>) -> Self {
        self.dead_letter_queue = Some(dlq);
        self
    }

    fn collection_name(&self) -> String {
        format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
    }

    fn add_to_dlq(&self, operation: &str, error: &str) {
        if let Some(ref dlq) = self.dead_letter_queue {
            if self.config.use_dead_letter_queue {
                let entry = crate::sync::dead_letter_queue::DeadLetterEntry::new(
                    super::trait_def::IndexOperation::Delete {
                        key: super::trait_def::IndexKey::new(
                            self.space_id,
                            self.tag_name.clone(),
                            self.field_name.clone(),
                        ),
                        id: operation.to_string(),
                    },
                    format!("Vector index operation failed: {}", error),
                    self.config.retry_config.max_retries,
                );
                dlq.add(entry);
                debug!("Added failed operation to dead letter queue: {}", operation);
            }
        }
    }
}

#[async_trait]
impl ExternalIndexClient for VectorClient {
    fn client_type(&self) -> &'static str {
        "vector"
    }

    fn index_key(&self) -> (u64, String, String) {
        (
            self.space_id,
            self.tag_name.clone(),
            self.field_name.clone(),
        )
    }

    async fn insert(&self, id: &str, data: &IndexData) -> IndexResult<()> {
        if let IndexData::Vector(vector) = data {
            let point = vector_client::types::VectorPoint::new(id.to_string(), vector.clone());
            let collection_name = self.collection_name();
            let vector_manager = self.vector_manager.clone();

            let result = with_retry(
                || {
                    let point_clone = point.clone();
                    let collection_name_clone = collection_name.clone();
                    let vm = vector_manager.clone();
                    async move { vm.upsert(&collection_name_clone, point_clone).await }
                },
                &self.config.retry_config,
            )
            .await;

            match result {
                Ok(_) => Ok(()),
                Err(e) => {
                    let error_msg = e.to_string();
                    warn!(
                        "Vector insert failed after {} retries for id {}: {}",
                        self.config.retry_config.max_retries, id, error_msg
                    );
                    self.add_to_dlq(id, &error_msg);
                    Err(ExternalIndexError::InsertError(error_msg))
                }
            }
        } else {
            Err(ExternalIndexError::InvalidData(
                "Expected vector data".to_string(),
            ))
        }
    }

    async fn insert_batch(&self, items: Vec<(String, IndexData)>) -> IndexResult<()> {
        let points: Vec<vector_client::types::VectorPoint> = items
            .iter()
            .filter_map(|(id, data)| {
                if let IndexData::Vector(vector) = data {
                    Some(vector_client::types::VectorPoint::new(
                        id.clone(),
                        vector.clone(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        if points.is_empty() {
            return Ok(());
        }

        let collection_name = self.collection_name();
        let vector_manager = self.vector_manager.clone();
        let points_clone = points.clone();

        let result = with_retry(
            || {
                let pts = points_clone.clone();
                let cn = collection_name.clone();
                let vm = vector_manager.clone();
                async move { vm.upsert_batch(&cn, pts).await }
            },
            &self.config.retry_config,
        )
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = e.to_string();
                warn!(
                    "Vector batch insert failed after {} retries: {}",
                    self.config.retry_config.max_retries, error_msg
                );
                for (id, _) in items {
                    self.add_to_dlq(&id, &error_msg);
                }
                Err(ExternalIndexError::InsertError(error_msg))
            }
        }
    }

    async fn delete(&self, id: &str) -> IndexResult<()> {
        let collection_name = self.collection_name();
        let vector_manager = self.vector_manager.clone();
        let id_owned = id.to_string();

        let result = with_retry(
            || {
                let cn = collection_name.clone();
                let vm = vector_manager.clone();
                let id = id_owned.clone();
                async move { vm.delete(&cn, &id).await }
            },
            &self.config.retry_config,
        )
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = e.to_string();
                warn!(
                    "Vector delete failed after {} retries for id {}: {}",
                    self.config.retry_config.max_retries, id, error_msg
                );
                self.add_to_dlq(id, &error_msg);
                Err(ExternalIndexError::DeleteError(error_msg))
            }
        }
    }

    async fn delete_batch(&self, ids: &[&str]) -> IndexResult<()> {
        let collection_name = self.collection_name();
        let vector_manager = self.vector_manager.clone();
        let ids_vec: Vec<String> = ids.iter().map(|s| s.to_string()).collect();

        let result = with_retry(
            || {
                let cn = collection_name.clone();
                let vm = vector_manager.clone();
                let ids_ref: Vec<&str> = ids_vec.iter().map(|s| s.as_str()).collect();
                async move { vm.delete_batch(&cn, ids_ref).await }
            },
            &self.config.retry_config,
        )
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = e.to_string();
                warn!(
                    "Vector batch delete failed after {} retries: {}",
                    self.config.retry_config.max_retries, error_msg
                );
                for id in ids {
                    self.add_to_dlq(id, &error_msg);
                }
                Err(ExternalIndexError::DeleteError(error_msg))
            }
        }
    }

    async fn commit(&self) -> IndexResult<()> {
        debug!("VectorClient commit: no-op for remote vector store");
        Ok(())
    }

    async fn rollback(&self) -> IndexResult<()> {
        debug!(
            "VectorClient rollback: no-op for remote vector store (no real transaction support)"
        );
        Ok(())
    }

    async fn stats(&self) -> IndexResult<IndexStats> {
        let collection_name = self.collection_name();
        let vector_manager = self.vector_manager.clone();

        let result = with_retry(
            || {
                let cn = collection_name.clone();
                let vm = vector_manager.clone();
                async move { vm.count(&cn).await }
            },
            &self.config.retry_config,
        )
        .await;

        match result {
            Ok(count) => Ok(IndexStats {
                doc_count: count as usize,
                index_size_bytes: 0,
                last_commit_time: None,
            }),
            Err(e) => {
                let error_msg = e.to_string();
                warn!("Vector stats failed: {}", error_msg);
                Err(ExternalIndexError::StatsError(error_msg))
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = VectorClientConfig::default();
        assert_eq!(config.retry_config.max_retries, 3);
        assert_eq!(config.operation_timeout, Duration::from_millis(30_000));
        assert!(config.use_dead_letter_queue);
    }

    #[test]
    fn test_config_builder() {
        let config = VectorClientConfig::default()
            .with_retry_config(RetryConfig::new(
                5,
                Duration::from_millis(200),
                Duration::from_secs(20),
            ))
            .with_operation_timeout(Duration::from_secs(60))
            .with_dead_letter_queue(false);

        assert_eq!(config.retry_config.max_retries, 5);
        assert_eq!(config.operation_timeout, Duration::from_secs(60));
        assert!(!config.use_dead_letter_queue);
    }
}
