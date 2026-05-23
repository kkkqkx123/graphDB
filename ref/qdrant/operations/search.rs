//! Search operations
//!
//! Handles vector similarity search and filtering logic.

use reqwest::Client;

use crate::storage::qdrant::error::QdrantError;
use crate::storage::qdrant::types::{Payload, SearchQuery, SearchResult};

/// Search operations handler
pub struct SearchOperations {
    http_client: Client,
    collection_name: String,
    base_url: String,
}

impl SearchOperations {
    /// Create new search operations handler
    pub fn new(http_client: Client, collection_name: String, base_url: String) -> Self {
        Self {
            http_client,
            collection_name,
            base_url,
        }
    }

    /// Search for similar vectors
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, QdrantError> {
        // Legacy single dense vector search
        let url = self.build_search_url();

        // Build filter
        let filter = self.build_search_filter(query.directory_prefix.as_deref());

        let mut body = serde_json::json!({
            "vector": query.vector,
            "limit": query.limit,
            "with_payload": true
        });

        if let Some(filter) = filter {
            body["filter"] = filter;
        }

        if let Some(min_score) = query.min_score {
            body["score_threshold"] = serde_json::json!(min_score);
        }

        if let Some(hnsw_ef) = query.hnsw_ef {
            body["params"] = serde_json::json!({
                "hnsw_ef": hnsw_ef
            });
        }

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
                "Failed to search: {} - {}",
                status, error_text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| QdrantError::ResponseParse(e.to_string()))?;

        // Parse results
        let results: Vec<SearchResult> = json
            .get("result")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let id = item.get("id")?.as_str()?.to_string();
                        let score = item.get("score")?.as_f64()? as f32;
                        let payload_json = item.get("payload")?;

                        // Parse payload
                        let payload = self.parse_payload(payload_json).ok()?;

                        Some(SearchResult { id, score, payload })
                    })
                    .collect()
            })
            .unwrap_or_default();

        tracing::debug!("Search returned {} results", results.len());
        Ok(results)
    }

    /// Build search filter from directory prefix
    fn build_search_filter(&self, directory_prefix: Option<&str>) -> Option<serde_json::Value> {
        let mut must_conditions: Vec<serde_json::Value> = Vec::new();
        let mut must_not_conditions: Vec<serde_json::Value> = Vec::new();

        // Exclude metadata points
        must_not_conditions.push(serde_json::json!({
            "key": "type",
            "match": { "value": "metadata" }
        }));

        // Add directory prefix filter
        if let Some(prefix) = directory_prefix {
            let normalized = prefix.replace('\\', "/");
            let segments: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();

            for (i, segment) in segments.iter().enumerate().take(5) {
                must_conditions.push(serde_json::json!({
                    "key": format!("pathSegments.{}", i),
                    "match": { "value": segment }
                }));
            }
        }

        if must_conditions.is_empty() && must_not_conditions.is_empty() {
            None
        } else {
            let mut filter = serde_json::json!({});
            if !must_conditions.is_empty() {
                filter["must"] = serde_json::json!(must_conditions);
            }
            if !must_not_conditions.is_empty() {
                filter["must_not"] = serde_json::json!(must_not_conditions);
            }
            Some(filter)
        }
    }

    /// Parse payload from JSON
    fn parse_payload(&self, json: &serde_json::Value) -> Result<Payload, QdrantError> {
        let file_path = json
            .get("file_path")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let entity_id = json.get("entity_id").and_then(|v| v.as_u64());

        let pattern_info = json
            .get("pattern_info")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Payload {
            file_path,
            entity_id,
            pattern_info,
        })
    }

    /// Build URL for search endpoint
    fn build_search_url(&self) -> String {
        format!(
            "{}/collections/{}/points/search",
            self.base_url, self.collection_name
        )
    }
}
