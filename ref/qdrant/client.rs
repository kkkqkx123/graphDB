//! Qdrant HTTP client implementation
//!
//! This module provides the main client for interacting with Qdrant vector database
//! via HTTP REST API. The client acts as a facade that coordinates various operations.

use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::config::project::{
    HnswConfigOverride, QuantizationConfigOverride, SparseVectorQuantizationConfig,
    WalConfigOverride,
};
use crate::metrics::domain::QdrantMetrics;
use crate::storage::qdrant::{
    config::QdrantConfig,
    error::QdrantError,
    operations::{CollectionOperations, PointOperations, SearchOperations, SummaryOperations},
    types::{Payload, SearchQuery, SearchResult, SizeEstimation, VectorPoint},
};
use crate::utils::hash::calculate_hash;

/// Indexing metadata point ID
const INDEXING_METADATA_ID: &str = "__indexing_metadata__";

/// Sanitize directory name for collection name usage
/// - Replace special characters with underscore
/// - Limit length to 30 chars per component
fn sanitize_collection_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .take(30)
        .collect();

    if sanitized.is_empty() {
        "unnamed".to_string()
    } else {
        sanitized.to_lowercase()
    }
}

/// Qdrant vector storage client
pub struct QdrantClient {
    config: QdrantConfig,
    http_client: Client,
    collection_name: String,
    summary_collection_name: String,
    base_url: String,

    // Operation handlers wrapped in Arc for efficient cloning
    collection_ops: Arc<CollectionOperations>,
    point_ops: Arc<PointOperations>,
    search_ops: Arc<SearchOperations>,
    summary_ops: Arc<SummaryOperations>,

    // Metrics collector
    metrics: Option<Arc<QdrantMetrics>>,
}

impl QdrantClient {
    /// Create a new Qdrant client
    pub fn new(config: QdrantConfig, workspace_path: &str) -> Result<Self, QdrantError> {
        debug!("Creating Qdrant client");

        // Validate config
        config.validate().map_err(|e| {
            error!(error = %e, "Config validation failed");
            QdrantError::config(e)
        })?;

        // Create HTTP client
        let mut builder = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .user_agent("CodeContextEngine")
            .pool_max_idle_per_host(10)
            .tcp_keepalive(Duration::from_secs(60));

        // Add API key if provided
        if let Some(ref api_key) = config.api_key {
            builder = builder.default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "api-key",
                    reqwest::header::HeaderValue::from_str(api_key).map_err(|e| {
                        error!(error = %e, "Invalid API key format");
                        QdrantError::config(format!("Invalid API key: {}", e))
                    })?,
                );
                headers
            });
        }

        let http_client = builder.build().map_err(|e| {
            error!(error = %e, "Failed to build HTTP client");
            QdrantError::connection(e.to_string())
        })?;

        // Generate collection name from workspace path
        let collection_name = Self::generate_collection_name(workspace_path);
        let summary_collection_name = format!("{}-summary", collection_name);

        // Normalize URL
        let base_url = config.normalized_url();

        info!(
            collection_name = %collection_name,
            summary_collection_name = %summary_collection_name,
            base_url = %base_url,
            "Qdrant client initialized"
        );

        // Create operation handlers
        let collection_ops = Arc::new(CollectionOperations::new(
            config.clone(),
            http_client.clone(),
            collection_name.clone(),
            base_url.clone(),
        ));

        let point_ops = Arc::new(PointOperations::new(
            http_client.clone(),
            collection_name.clone(),
            base_url.clone(),
        ));

        let search_ops = Arc::new(SearchOperations::new(
            http_client.clone(),
            collection_name.clone(),
            base_url.clone(),
        ));

        let summary_ops = Arc::new(SummaryOperations::new(
            config.clone(),
            http_client.clone(),
            summary_collection_name.clone(),
            base_url.clone(),
        ));

        Ok(Self {
            config,
            http_client,
            collection_name,
            summary_collection_name,
            base_url,
            collection_ops,
            point_ops,
            search_ops,
            summary_ops,
            metrics: None,
        })
    }

    /// Attach metrics collector to the client
    pub fn with_metrics(mut self, metrics: Arc<QdrantMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Get a reference to the metrics collector
    pub fn metrics(&self) -> Option<&Arc<QdrantMetrics>> {
        self.metrics.as_ref()
    }

    /// Create a new Qdrant client with default config
    pub fn with_default_config(workspace_path: &str) -> Result<Self, QdrantError> {
        Self::new(QdrantConfig::default(), workspace_path)
    }

    /// Generate collection name from workspace path
    ///
    /// Format: cce_<last_dir>_<hash> or cce_<parent>_<last_dir>_<hash>
    /// Examples:
    ///   - /home/user/myproject → cce_user_myproject_a1b2c3d4
    ///   - D:\\work\\project → cce_work_project_e5f6789a
    fn generate_collection_name(workspace_path: &str) -> String {
        use std::path::Path;

        let hash = calculate_hash(workspace_path.as_bytes());
        let hash_suffix = &hash[..8]; // Use first 8 hex chars

        // Extract last 2 directory components
        let path = Path::new(workspace_path);
        let components: Vec<_> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .filter(|s| !s.is_empty() && *s != "/" && *s != "\\")
            .collect();

        // Build readable name from last 1-2 components
        let name_part = match components.len() {
            0 => "unknown".to_string(),
            1 => sanitize_collection_name(components[0]),
            _ => {
                let parent = sanitize_collection_name(components[components.len() - 2]);
                let current = sanitize_collection_name(components[components.len() - 1]);
                format!("{}_{}", parent, current)
            }
        };

        format!("cce_{}_{}", name_part, hash_suffix)
    }

    /// Get the collection name
    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    /// Get the summary collection name
    pub fn summary_collection_name(&self) -> &str {
        &self.summary_collection_name
    }

    /// Get the config
    pub fn config(&self) -> &QdrantConfig {
        &self.config
    }

    /// Check if the client is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the HTTP client for retrieval operations
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Get the base URL for retrieval operations
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Initialize the collection
    ///
    /// Returns true if a new collection was created, false if it already existed
    pub async fn initialize(&self) -> Result<bool, QdrantError> {
        debug!(collection_name = %self.collection_name, "Starting collection initialization");
        self.initialize_with_config(None, None, None, None).await
    }

    /// Initialize the collection with custom configuration overrides
    ///
    /// Returns true if a new collection was created, false if it already existed
    pub async fn initialize_with_config(
        &self,
        hnsw_override: Option<HnswConfigOverride>,
        quantization_override: Option<QuantizationConfigOverride>,
        wal_override: Option<WalConfigOverride>,
        sparse_quant_override: Option<SparseVectorQuantizationConfig>,
    ) -> Result<bool, QdrantError> {
        debug!(collection = %self.collection_name, "Initializing collection");
        let start = Instant::now();

        // Check if collection exists first
        match self.collection_ops.get_info().await {
            Ok(_) => {
                // Collection exists
                info!(
                    collection = %self.collection_name,
                    latency_ms = start.elapsed().as_millis(),
                    "Collection already exists"
                );
                Ok(false)
            }
            Err(QdrantError::CollectionNotFound(_)) => {
                // Collection doesn't exist, create it
                self.collection_ops
                    .create_with_config(
                        hnsw_override,
                        quantization_override,
                        wal_override,
                        sparse_quant_override,
                    )
                    .await?;

                info!(
                    collection = %self.collection_name,
                    latency_ms = start.elapsed().as_millis(),
                    "Collection created"
                );
                Ok(true)
            }
            Err(e) => {
                error!(
                    collection = %self.collection_name,
                    error = %e,
                    latency_ms = start.elapsed().as_millis(),
                    "Failed to check collection existence"
                );
                Err(e)
            }
        }
    }

    /// Get collection information
    pub async fn get_collection_info(
        &self,
    ) -> Result<crate::storage::qdrant::types::CollectionInfo, QdrantError> {
        self.collection_ops.get_info().await
    }

    /// Check if collection exists
    pub async fn collection_exists(&self) -> Result<bool, QdrantError> {
        self.collection_ops.exists().await
    }

    /// Delete the collection
    pub async fn delete_collection(&self) -> Result<(), QdrantError> {
        self.collection_ops.delete().await
    }

    /// Clear all points from the collection
    pub async fn clear_collection(&self) -> Result<(), QdrantError> {
        debug!(collection = %self.collection_name, "Clearing collection");

        let start = Instant::now();
        let result = self.collection_ops.clear().await;
        let latency_ms = start.elapsed().as_millis() as u64;

        match &result {
            Ok(_) => {
                info!(
                    collection = %self.collection_name,
                    latency_ms = latency_ms,
                    "Collection cleared successfully"
                );
            }
            Err(e) => {
                error!(
                    collection = %self.collection_name,
                    error = %e,
                    latency_ms = latency_ms,
                    "Collection clear failed"
                );
            }
        }

        result
    }

    /// Upsert vector points
    pub async fn upsert_points(&self, points: &[VectorPoint]) -> Result<(), QdrantError> {
        let point_count = points.len();
        debug!(collection = %self.collection_name, point_count = point_count, "Upserting points");

        let start = Instant::now();
        let result = self.point_ops.upsert(points).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_upsert(latency_ms, point_count, result.is_ok());
        }

        match &result {
            Ok(_) => {
                info!(
                    collection = %self.collection_name,
                    point_count = point_count,
                    latency_ms = latency_ms,
                    "Points upserted successfully"
                );
            }
            Err(e) => {
                error!(
                    collection = %self.collection_name,
                    point_count = point_count,
                    error = %e,
                    latency_ms = latency_ms,
                    "Points upsert failed"
                );
            }
        }

        result
    }

    /// Delete points by file path
    pub async fn delete_by_file_path(&self, file_path: &str) -> Result<(), QdrantError> {
        debug!(collection = %self.collection_name, file_path = %file_path, "Deleting points by file path");

        let start = Instant::now();
        let result = self.point_ops.delete_by_file_path(file_path).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_delete(latency_ms, 1, result.is_ok());
        }

        match &result {
            Ok(_) => {
                info!(
                    collection = %self.collection_name,
                    file_path = %file_path,
                    latency_ms = latency_ms,
                    "Points deleted successfully"
                );
            }
            Err(e) => {
                warn!(
                    collection = %self.collection_name,
                    file_path = %file_path,
                    error = %e,
                    latency_ms = latency_ms,
                    "Points deletion failed"
                );
            }
        }

        result
    }

    /// Delete points by multiple file paths
    pub async fn delete_by_file_paths(&self, file_paths: &[&str]) -> Result<(), QdrantError> {
        debug!(
            collection = %self.collection_name,
            file_count = file_paths.len(),
            "Deleting points by multiple file paths"
        );

        let start = Instant::now();
        let result = self.point_ops.delete_by_file_paths(file_paths).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_delete(latency_ms, file_paths.len(), result.is_ok());
        }

        match &result {
            Ok(_) => {
                info!(
                    collection = %self.collection_name,
                    file_count = file_paths.len(),
                    latency_ms = latency_ms,
                    "Points deleted successfully"
                );
            }
            Err(e) => {
                warn!(
                    collection = %self.collection_name,
                    file_count = file_paths.len(),
                    error = %e,
                    latency_ms = latency_ms,
                    "Points deletion failed"
                );
            }
        }

        result
    }

    /// Search for similar vectors
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, QdrantError> {
        debug!(collection = %self.collection_name, top_k = query.limit, "Searching vectors");

        let start = Instant::now();
        let result = self.search_ops.search(query).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record metrics
        if let Some(metrics) = &self.metrics {
            metrics.record_search(latency_ms, result.is_ok());
        }

        match &result {
            Ok(results) => {
                debug!(
                    collection = %self.collection_name,
                    result_count = results.len(),
                    latency_ms = latency_ms,
                    "Search completed"
                );
            }
            Err(e) => {
                error!(
                    collection = %self.collection_name,
                    error = %e,
                    latency_ms = latency_ms,
                    "Search failed"
                );
            }
        }

        result
    }

    /// Check if collection has indexed data
    pub async fn has_indexed_data(&self) -> Result<bool, QdrantError> {
        let info = self.collection_ops.get_info().await?;

        if info.points_count == 0 {
            return Ok(false);
        }

        // Check for indexing metadata marker
        let url = format!(
            "{}/collections/{}/points/{}",
            self.base_url, self.collection_name, INDEXING_METADATA_ID
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| QdrantError::request(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            // No marker, assume indexed if points exist (backward compatibility)
            tracing::debug!("No indexing metadata marker found, assuming indexed");
            return Ok(info.points_count > 0);
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| QdrantError::ResponseParse(e.to_string()))?;

        let complete = json
            .get("result")
            .and_then(|r| r.get("payload"))
            .and_then(|p| p.get("indexing_complete"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(complete)
    }

    /// Mark indexing as complete
    pub async fn mark_indexing_complete(&self) -> Result<(), QdrantError> {
        let payload = Payload::new("__metadata__".to_string());

        let point = VectorPoint::new(
            INDEXING_METADATA_ID,
            vec![0.0; self.config.vector_size],
            payload,
        );

        self.point_ops.upsert(&[point]).await?;
        tracing::info!("Marked indexing as complete");
        Ok(())
    }

    /// Mark indexing as in progress
    pub async fn mark_indexing_in_progress(&self) -> Result<(), QdrantError> {
        let payload = Payload::new("__metadata__".to_string());

        let point = VectorPoint::new(
            INDEXING_METADATA_ID,
            vec![0.0; self.config.vector_size],
            payload,
        );

        self.point_ops.upsert(&[point]).await?;
        tracing::info!("Marked indexing as in progress");
        Ok(())
    }

    /// Estimate collection size from file count
    pub fn estimate_size(&self, file_count: usize, avg_vectors_per_file: f32) -> SizeEstimation {
        SizeEstimation::from_file_count(file_count, avg_vectors_per_file)
    }

    /// Get recommended preset for estimated size
    pub fn get_recommended_preset(
        &self,
        vector_count: usize,
    ) -> crate::storage::qdrant::config::CollectionPreset {
        crate::storage::qdrant::config::CollectionPreset::from_vector_count(vector_count)
    }

    /// Ensure summary collection exists
    pub async fn ensure_summary_collection(&self) -> Result<(), QdrantError> {
        self.summary_ops.ensure_collection().await
    }

    /// Upsert summary points to summary collection
    pub async fn upsert_summary_points(&self, points: &[VectorPoint]) -> Result<(), QdrantError> {
        let point_count = points.len();
        let start = Instant::now();
        let result = self.summary_ops.upsert(points).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record metrics for summary operations (reuse main metrics)
        if let Some(metrics) = &self.metrics {
            metrics.record_upsert(latency_ms, point_count, result.is_ok());
        }

        result
    }

    /// Search summary collection
    pub async fn search_summaries(
        &self,
        query_vector: Vec<f32>,
        top_k: usize,
        min_score: f32,
    ) -> Result<Vec<crate::storage::qdrant::operations::SummarySearchResult>, QdrantError> {
        let start = Instant::now();
        let result = self
            .summary_ops
            .search(query_vector, top_k, min_score)
            .await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record metrics for summary search
        if let Some(metrics) = &self.metrics {
            metrics.record_search(latency_ms, result.is_ok());
        }

        result
    }

    /// Delete summary point by file path
    pub async fn delete_summary_by_file_path(&self, file_path: &str) -> Result<(), QdrantError> {
        let start = Instant::now();
        let result = self.summary_ops.delete_by_file_path(file_path).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1000.0;

        // Record metrics for summary delete
        if let Some(metrics) = &self.metrics {
            metrics.record_delete(latency_ms, 1, result.is_ok());
        }

        result
    }

    /// Search summary collection with file path filtering
    ///
    /// This method searches the summary collection and optionally filters results
    /// to only include specified file paths for better performance.
    pub async fn search_summaries_with_paths(
        &self,
        query_vector: Vec<f32>,
        top_k: usize,
        min_score: f32,
        allowed_paths: &std::collections::HashSet<String>,
    ) -> Result<Vec<crate::storage::qdrant::operations::summary::SummarySearchResult>, QdrantError>
    {
        // Perform search
        let all_results = self
            .summary_ops
            .search(query_vector, top_k, min_score)
            .await?;

        // Filter by allowed paths if provided
        if !allowed_paths.is_empty() {
            let filtered = all_results
                .into_iter()
                .filter(|r| allowed_paths.contains(&r.file_path))
                .collect();
            Ok(filtered)
        } else {
            Ok(all_results)
        }
    }
}

impl Clone for QdrantClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            http_client: self.http_client.clone(),
            collection_name: self.collection_name.clone(),
            summary_collection_name: self.summary_collection_name.clone(),
            base_url: self.base_url.clone(),
            collection_ops: self.collection_ops.clone(),
            point_ops: self.point_ops.clone(),
            search_ops: self.search_ops.clone(),
            summary_ops: self.summary_ops.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = QdrantConfig::default();
        let client = QdrantClient::new(config, "/test/workspace").expect("Failed to create client");
        assert!(client.is_enabled());
        assert!(client.collection_name().starts_with("cce_"));
    }

    #[test]
    fn test_collection_name_format() {
        // Test with 2-level path
        let client = QdrantClient::with_default_config("/home/user/myproject")
            .expect("Failed to create client");
        let name = client.collection_name();
        assert!(name.starts_with("cce_"));
        assert!(name.contains("user_myproject") || name.contains("myproject"));
        assert_eq!(name.len(), "cce_user_myproject_".len() + 8); // prefix + 8 char hash

        // Test with 1-level path
        let client2 =
            QdrantClient::with_default_config("/workspace").expect("Failed to create client");
        let name2 = client2.collection_name();
        assert!(name2.starts_with("cce_workspace_"));
    }

    #[test]
    fn test_collection_name_special_chars() {
        // Test special characters are sanitized
        let client = QdrantClient::with_default_config("/path/My Project@v1.0/src")
            .expect("Failed to create client");
        let name = client.collection_name();
        assert!(name.starts_with("cce_"));
        // Special chars should be replaced with underscore
        assert!(!name.contains(" "));
        assert!(!name.contains("@"));
        assert!(!name.contains("."));
    }

    #[test]
    fn test_collection_name_generation() {
        let client1 =
            QdrantClient::with_default_config("/workspace1").expect("Failed to create client");
        let client2 =
            QdrantClient::with_default_config("/workspace2").expect("Failed to create client");

        // Different workspaces should have different collection names
        assert_ne!(client1.collection_name(), client2.collection_name());
    }

    #[test]
    fn test_same_workspace_same_collection() {
        let client1 =
            QdrantClient::with_default_config("/same/workspace").expect("Failed to create client");
        let client2 =
            QdrantClient::with_default_config("/same/workspace").expect("Failed to create client");

        // Same workspace should have same collection name
        assert_eq!(client1.collection_name(), client2.collection_name());
    }

    #[test]
    fn test_disabled_client() {
        let config = QdrantConfig::default().disabled();
        let client = QdrantClient::new(config, "/test").expect("Failed to create client");
        assert!(!client.is_enabled());
    }

    #[test]
    fn test_size_estimation() {
        let client = QdrantClient::with_default_config("/test").expect("Failed to create client");
        let estimation = client.estimate_size(100, 10.0);
        assert_eq!(estimation.estimated_vector_count, 1000);
    }
}
