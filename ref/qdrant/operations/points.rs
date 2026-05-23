//! Point operations
//!
//! Handles all vector point operations including upsert, delete, and batch operations.

use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::storage::qdrant::error::QdrantError;
use crate::storage::qdrant::types::VectorPoint;

/// Point operations handler
pub struct PointOperations {
    http_client: Client,
    collection_name: String,
    base_url: String,
    /// Semaphore to limit concurrent upsert requests
    concurrency_limiter: Arc<Semaphore>,
}

impl PointOperations {
    /// Create new point operations handler
    pub fn new(http_client: Client, collection_name: String, base_url: String) -> Self {
        // Default to allowing 5 concurrent upsert operations
        let concurrency_limiter = Arc::new(Semaphore::new(5));
        Self {
            http_client,
            collection_name,
            base_url,
            concurrency_limiter,
        }
    }

    /// Set the maximum number of concurrent upsert operations
    pub fn with_max_concurrent_upserts(mut self, max: usize) -> Self {
        self.concurrency_limiter = Arc::new(Semaphore::new(max));
        self
    }

    /// Upsert vector points
    pub async fn upsert(&self, points: &[VectorPoint]) -> Result<(), QdrantError> {
        if points.is_empty() {
            return Ok(());
        }

        // Acquire a permit from the semaphore to limit concurrency
        let _permit = self.concurrency_limiter.acquire().await.map_err(|e| {
            QdrantError::api(format!("Failed to acquire concurrency permit: {}", e))
        })?;

        let url = self.build_points_url();

        // Convert points to JSON format with support for sparse vectors
        let points_json: Vec<serde_json::Value> = points
            .iter()
            .map(|p| {
                let mut point_data = serde_json::json!({
                    "id": p.id,
                    "payload": p.payload
                });

                // Add vectors (support both dense and sparse)
                if let Some(ref sparse) = p.sparse_vector {
                    // Named vectors format for hybrid storage
                    point_data["vector"] = serde_json::json!({
                        "dense": p.vector,
                        "sparse": {
                            "indices": sparse.indices,
                            "values": sparse.values
                        }
                    });
                } else {
                    // Legacy single dense vector format
                    point_data["vector"] = serde_json::json!(p.vector);
                }

                point_data
            })
            .collect();

        let body = serde_json::json!({
            "points": points_json
        });

        let response = self
            .http_client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| QdrantError::request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(QdrantError::api(format!(
                "Failed to upsert points: {} - {}",
                status, error_text
            )));
        }

        tracing::debug!("Upserted {} points", points.len());
        Ok(())
    }

    /// Delete points by file path
    pub async fn delete_by_file_path(&self, file_path: &str) -> Result<(), QdrantError> {
        self.delete_by_file_paths(&[file_path]).await
    }

    /// Delete points by multiple file paths
    pub async fn delete_by_file_paths(&self, file_paths: &[&str]) -> Result<(), QdrantError> {
        if file_paths.is_empty() {
            return Ok(());
        }

        let url = self.build_delete_url();

        // Build filters for each file path
        let filters: Vec<serde_json::Value> = file_paths
            .iter()
            .map(|path| {
                let normalized = path.replace('\\', "/");
                let segments: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();

                let must_conditions: Vec<serde_json::Value> = segments
                    .iter()
                    .enumerate()
                    .take(5)
                    .map(|(i, segment)| {
                        serde_json::json!({
                            "key": format!("pathSegments.{}", i),
                            "match": { "value": segment }
                        })
                    })
                    .collect();

                serde_json::json!({ "must": must_conditions })
            })
            .collect();

        let filter = if filters.len() == 1 {
            filters[0].clone()
        } else {
            serde_json::json!({ "should": filters })
        };

        let body = serde_json::json!({ "filter": filter });

        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| QdrantError::request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(QdrantError::api(format!(
                "Failed to delete points: {} - {}",
                status, error_text
            )));
        }

        tracing::debug!("Deleted points for {} file paths", file_paths.len());
        Ok(())
    }

    /// Build URL for points endpoint
    fn build_points_url(&self) -> String {
        format!(
            "{}/collections/{}/points",
            self.base_url, self.collection_name
        )
    }

    /// Build URL for delete endpoint
    fn build_delete_url(&self) -> String {
        format!(
            "{}/collections/{}/points/delete",
            self.base_url, self.collection_name
        )
    }
}
