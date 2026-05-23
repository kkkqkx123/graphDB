//! Summary collection operations
//!
//! Handles all summary collection operations including creation, upsert, search, and deletion.

use reqwest::Client;

use crate::storage::qdrant::config::QdrantConfig;
use crate::storage::qdrant::error::QdrantError;
use crate::storage::qdrant::types::VectorPoint;

/// Summary search result
#[derive(Debug, Clone)]
pub struct SummarySearchResult {
    /// File path
    pub file_path: String,
    /// Similarity score
    pub score: f32,
    /// Summary text
    pub summary: String,
}

/// Summary collection operations handler
pub struct SummaryOperations {
    config: QdrantConfig,
    http_client: Client,
    collection_name: String,
    base_url: String,
}

impl SummaryOperations {
    /// Create new summary operations handler
    pub fn new(
        config: QdrantConfig,
        http_client: Client,
        collection_name: String,
        base_url: String,
    ) -> Self {
        Self {
            config,
            http_client,
            collection_name,
            base_url,
        }
    }

    /// Ensure summary collection exists
    pub async fn ensure_collection(&self) -> Result<(), QdrantError> {
        if !self.config.enabled {
            return Err(QdrantError::Disabled);
        }

        let url = self.build_url();

        // Check if collection exists
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| QdrantError::request(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            // Create summary collection with smaller HNSW config
            self.create_collection().await?;
        }

        Ok(())
    }

    /// Create summary collection (smaller config for file-level summaries)
    async fn create_collection(&self) -> Result<(), QdrantError> {
        let url = self.build_url();

        // Use smaller HNSW config for summary collection (fewer vectors)
        let body = serde_json::json!({
            "vectors": {
                "size": self.config.vector_size,
                "distance": self.config.distance_metric.as_str(),
                "on_disk": true
            },
            "hnsw_config": {
                "m": 8,
                "ef_construct": 64,
                "on_disk": true
            }
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
                "Failed to create summary collection: {} - {}",
                status, error_text
            )));
        }

        tracing::info!("Created summary collection: {}", self.collection_name);
        Ok(())
    }

    /// Upsert summary points to summary collection
    pub async fn upsert(&self, points: &[VectorPoint]) -> Result<(), QdrantError> {
        if points.is_empty() {
            return Ok(());
        }

        // Ensure summary collection exists
        self.ensure_collection().await?;

        let url = self.build_points_url();

        // Convert points to JSON format
        let points_json: Vec<serde_json::Value> = points
            .iter()
            .map(|p| {
                serde_json::json!({
                    "id": p.id,
                    "vector": p.vector,
                    "payload": p.payload
                })
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
                "Failed to upsert summary points: {} - {}",
                status, error_text
            )));
        }

        tracing::debug!("Upserted {} summary points", points.len());
        Ok(())
    }

    /// Search summary collection
    pub async fn search(
        &self,
        query_vector: Vec<f32>,
        top_k: usize,
        min_score: f32,
    ) -> Result<Vec<SummarySearchResult>, QdrantError> {
        let url = self.build_search_url();

        let body = serde_json::json!({
            "vector": query_vector,
            "limit": top_k,
            "score_threshold": min_score,
            "with_payload": true
        });

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
                "Failed to search summaries: {} - {}",
                status, error_text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| QdrantError::ResponseParse(e.to_string()))?;

        // Parse results
        let results: Vec<SummarySearchResult> = json
            .get("result")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let file_path = item
                            .get("payload")
                            .and_then(|p| p.get("file_path"))
                            .and_then(|v| v.as_str())?
                            .to_string();
                        let score = item.get("score")?.as_f64()? as f32;
                        let summary = item
                            .get("payload")
                            .and_then(|p| p.get("summary"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        Some(SummarySearchResult {
                            file_path,
                            score,
                            summary,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        tracing::debug!("Summary search returned {} results", results.len());
        Ok(results)
    }

    /// Delete summary point by file path
    pub async fn delete_by_file_path(&self, file_path: &str) -> Result<(), QdrantError> {
        let url = self.build_delete_url();

        let body = serde_json::json!({
            "points": [format!("summary:{}", file_path)]
        });

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
                "Failed to delete summary point: {} - {}",
                status, error_text
            )));
        }

        tracing::debug!("Deleted summary point for: {}", file_path);
        Ok(())
    }

    /// Build URL for collection endpoint
    fn build_url(&self) -> String {
        format!("{}/collections/{}", self.base_url, self.collection_name)
    }

    /// Build URL for points endpoint
    fn build_points_url(&self) -> String {
        format!(
            "{}/collections/{}/points",
            self.base_url, self.collection_name
        )
    }

    /// Build URL for search endpoint
    fn build_search_url(&self) -> String {
        format!(
            "{}/collections/{}/points/search",
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
