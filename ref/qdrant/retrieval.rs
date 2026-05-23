//! Qdrant implementation of the vector retrieval trait
//!
//! This module provides a Qdrant-specific implementation of the VectorRetrievalTrait,
//! using HTTP REST API to interact with Qdrant server.

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;

use crate::storage::qdrant::Payload;
use crate::storage::vector_retrieval::{
    DenseSearchQuery, RetrievalError, ScoredPoint, SearchFilter, SparseSearchQuery,
    VectorRetrievalTrait,
};

/// Qdrant-based vector retrieval implementation
///
/// Uses HTTP REST API to communicate with Qdrant server.
pub struct QdrantRetrieval {
    /// HTTP client for making requests
    http_client: Client,
    /// Qdrant base URL (e.g., "http://localhost:6334")
    base_url: String,
    /// Collection name to search
    collection_name: String,
}

impl QdrantRetrieval {
    /// Create a new Qdrant retrieval instance
    ///
    /// # Arguments
    ///
    /// * `http_client` - HTTP client for making requests
    /// * `base_url` - Qdrant base URL (e.g., "http://localhost:6334")
    /// * `collection_name` - Name of the collection to search
    pub fn new(http_client: Client, base_url: String, collection_name: String) -> Self {
        Self {
            http_client,
            base_url,
            collection_name,
        }
    }

    /// Create a new Qdrant retrieval instance with Arc-wrapped client
    pub fn from_arc(http_client: Arc<Client>, base_url: String, collection_name: String) -> Self {
        Self {
            http_client: Arc::try_unwrap(http_client).unwrap_or_else(|arc| (*arc).clone()),
            base_url,
            collection_name,
        }
    }

    /// Build search filter from filter options
    fn build_filter(&self, filter: Option<&SearchFilter>) -> Option<serde_json::Value> {
        filter.and_then(|f| {
            // If raw_filter is provided, use it directly (takes precedence)
            if let Some(ref raw) = f.raw_filter {
                return Some(raw.clone());
            }

            // Fall back to directory_prefix
            f.directory_prefix.as_ref().map(|prefix| {
                serde_json::json!({
                    "must": [{
                        "key": "file_path",
                        "match": {
                            "value": prefix
                        }
                    }]
                })
            })
        })
    }

    /// Execute a search request and parse results
    async fn execute_search(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<Vec<ScoredPoint>, RetrievalError> {
        let response = self
            .http_client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RetrievalError::Connection(format!("Failed to send request: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(RetrievalError::Query(format!(
                "Search failed with status {}: {}",
                status, error_text
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            RetrievalError::Serialization(format!("Failed to parse response: {}", e))
        })?;

        // Parse results from standard search endpoint
        let results: Vec<ScoredPoint> = json
            .get("result")
            .and_then(|r| r.get("points"))
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let id = item.get("id")?.as_str()?.to_string();
                        let score = item.get("score")?.as_f64()? as f32;
                        let payload_json = item.get("payload")?;

                        // Parse payload - handle both string ID and object ID
                        let payload = match serde_json::from_value::<Payload>(payload_json.clone())
                        {
                            Ok(p) => p,
                            Err(e) => {
                                tracing::warn!("Failed to parse payload: {}", e);
                                return None;
                            }
                        };

                        Some(ScoredPoint { id, score, payload })
                    })
                    .collect()
            })
            .unwrap_or_default();

        tracing::debug!("Search returned {} results", results.len());
        Ok(results)
    }

    /// Execute a query request (for sparse vector search) and parse results
    async fn execute_query(
        &self,
        url: &str,
        body: serde_json::Value,
    ) -> Result<Vec<ScoredPoint>, RetrievalError> {
        let response = self
            .http_client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RetrievalError::Connection(format!("Failed to send request: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(RetrievalError::Query(format!(
                "Query failed with status {}: {}",
                status, error_text
            )));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| {
            RetrievalError::Serialization(format!("Failed to parse response: {}", e))
        })?;

        // Parse results from query endpoint (different structure than search)
        let results: Vec<ScoredPoint> = json
            .get("result")
            .and_then(|r| r.get("points"))
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let id = item.get("id")?.as_str()?.to_string();
                        let score = item.get("score")?.as_f64()? as f32;
                        let payload_json = item.get("payload")?;

                        let payload = match serde_json::from_value::<Payload>(payload_json.clone())
                        {
                            Ok(p) => p,
                            Err(e) => {
                                tracing::warn!("Failed to parse payload: {}", e);
                                return None;
                            }
                        };

                        Some(ScoredPoint { id, score, payload })
                    })
                    .collect()
            })
            .unwrap_or_default();

        tracing::debug!("Query returned {} results", results.len());
        Ok(results)
    }
}

#[async_trait]
impl VectorRetrievalTrait for QdrantRetrieval {
    async fn search_hybrid(
        &self,
        query: crate::storage::vector_retrieval::HybridSearchQuery,
    ) -> Result<Vec<ScoredPoint>, RetrievalError> {
        let url = format!(
            "{}/collections/{}/points/query",
            self.base_url, self.collection_name
        );

        let filter = self.build_filter(query.filter.as_ref());

        let mut body = serde_json::json!({
            "prefetch": [
                {
                    "query": query.dense_vector,
                    "using": "dense",
                    "limit": query.prefetch_limit,
                },
                {
                    "query": {
                        "indices": query.sparse_vector.indices,
                        "values": query.sparse_vector.values,
                    },
                    "using": "sparse",
                    "limit": query.prefetch_limit,
                }
            ],
            "query": { "fusion": "rrf" },
            "limit": query.final_limit,
            "with_payload": true,
        });

        if let Some(filter) = filter {
            body["filter"] = filter;
        }

        tracing::debug!(
            collection = self.collection_name,
            prefetch_limit = query.prefetch_limit,
            final_limit = query.final_limit,
            "Executing hybrid search with Qdrant-level RRF fusion"
        );

        self.execute_query(&url, body).await
    }

    async fn search_dense(
        &self,
        query: DenseSearchQuery,
    ) -> Result<Vec<ScoredPoint>, RetrievalError> {
        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, self.collection_name
        );

        // Build filter
        let filter = self.build_filter(query.filter.as_ref());

        let mut body = serde_json::json!({
            "vector": query.vector,
            "limit": query.limit,
            "with_payload": true
        });

        if let Some(filter) = filter {
            body["filter"] = filter;
        }

        tracing::debug!(
            collection = self.collection_name,
            limit = query.limit,
            "Executing dense vector search"
        );

        self.execute_search(&url, body).await
    }

    async fn search_sparse(
        &self,
        query: SparseSearchQuery,
    ) -> Result<Vec<ScoredPoint>, RetrievalError> {
        let url = format!(
            "{}/collections/{}/points/query",
            self.base_url, self.collection_name
        );

        // Build filter
        let filter = self.build_filter(query.filter.as_ref());

        // Qdrant sparse search requires query API with prefetch
        let mut body = serde_json::json!({
            "prefetch": [{
                "query": {
                    "indices": query.sparse_vector.indices,
                    "values": query.sparse_vector.values,
                },
                "using": "sparse",
                "limit": query.limit,
            }],
            "query": { "fusion": "rrf" },  // Single path also needs fusion wrapper
            "limit": query.limit,
            "with_payload": true
        });

        if let Some(filter) = filter {
            body["filter"] = filter;
        }

        tracing::debug!(
            collection = self.collection_name,
            limit = query.limit,
            sparse_indices_count = query.sparse_vector.indices.len(),
            "Executing sparse vector search"
        );

        self.execute_query(&url, body).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_filter_with_directory_prefix() {
        let client = QdrantRetrieval::new(
            Client::new(),
            "http://localhost:6334".to_string(),
            "test_collection".to_string(),
        );

        let filter = SearchFilter {
            directory_prefix: Some("/src/main".to_string()),
            raw_filter: None,
        };

        let result = client.build_filter(Some(&filter));
        assert!(result.is_some());

        let json = result.unwrap();
        assert_eq!(json["must"][0]["key"], "file_path");
        assert_eq!(json["must"][0]["match"]["value"], "/src/main");
    }

    #[test]
    fn test_build_filter_without_prefix() {
        let client = QdrantRetrieval::new(
            Client::new(),
            "http://localhost:6334".to_string(),
            "test_collection".to_string(),
        );

        let filter = SearchFilter {
            directory_prefix: None,
            raw_filter: None,
        };

        let result = client.build_filter(Some(&filter));
        assert!(result.is_none());
    }
}
