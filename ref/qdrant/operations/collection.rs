//! Collection operations
//!
//! Handles all collection-related database operations including lifecycle management,
// indexing, and information retrieval.

use reqwest::Client;

use crate::config::project::{
    HnswConfigOverride, QuantizationConfigOverride, SparseVectorQuantizationConfig,
};
use crate::storage::qdrant::{
    config::QdrantConfig,
    error::QdrantError,
    types::{CollectionInfo, CollectionStatus, HnswConfigInfo},
};

/// Collection operations handler
pub struct CollectionOperations {
    config: QdrantConfig,
    http_client: Client,
    collection_name: String,
    base_url: String,
}

impl CollectionOperations {
    /// Create new collection operations handler
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

    /// Initialize the collection
    ///
    /// Returns true if a new collection was created, false if it already existed
    pub async fn initialize(&self) -> Result<bool, QdrantError> {
        if !self.config.enabled {
            return Err(QdrantError::Disabled);
        }

        // Check if collection exists
        match self.get_info().await {
            Ok(info) => {
                // Collection exists, check vector size
                if info.vector_size != self.config.vector_size {
                    // Need to recreate with new dimension
                    tracing::warn!(
                        "Collection {} exists with vector size {}, but expected {}. Recreating.",
                        self.collection_name,
                        info.vector_size,
                        self.config.vector_size
                    );
                    self.delete().await?;
                    return self
                        .create_with_config(None, None, None, None)
                        .await
                        .map(|_| true);
                }
                Ok(false)
            }
            Err(QdrantError::CollectionNotFound(_)) => {
                // Collection doesn't exist, create it
                self.create_with_config(None, None, None, None)
                    .await
                    .map(|_| true)
            }
            Err(e) => Err(e),
        }
    }

    /// Create the collection with optional custom configurations
    pub async fn create_with_config(
        &self,
        hnsw_override: Option<HnswConfigOverride>,
        quantization_override: Option<QuantizationConfigOverride>,
        _wal_override: Option<crate::config::project::WalConfigOverride>,
        sparse_quant_override: Option<SparseVectorQuantizationConfig>,
    ) -> Result<(), QdrantError> {
        let url = self.build_url();

        // Get HNSW config from QdrantConfig (already merged with project overrides)
        let hnsw_config = if let Some(override_cfg) = hnsw_override {
            // Manual override takes highest precedence
            Some(crate::storage::qdrant::config::HnswConfig {
                m: override_cfg.m,
                ef_construct: override_cfg.ef_construct,
                on_disk: override_cfg.on_disk,
                inline_storage: override_cfg.inline_storage,
            })
        } else {
            // Use config from QdrantConfig (which includes preset or manual override)
            self.config
                .get_hnsw_config()
                .map(|h| crate::storage::qdrant::config::HnswConfig {
                    m: h.m,
                    ef_construct: h.ef_construct,
                    on_disk: h.on_disk,
                    inline_storage: h.inline_storage,
                })
        };

        // Get vector storage config
        let vector_storage = self.config.get_vector_storage_config();

        // Build request body
        let mut body = serde_json::json!({
            "vectors": {
                "size": self.config.vector_size,
                "distance": self.config.distance_metric.as_str(),
                "on_disk": vector_storage.on_disk
            }
        });

        // Add HNSW config
        if let Some(hnsw) = hnsw_config {
            body["hnsw_config"] = serde_json::json!({
                "m": hnsw.m,
                "ef_construct": hnsw.ef_construct,
                "on_disk": hnsw.on_disk,
                "inline_storage": hnsw.inline_storage
            });
        }

        // Add quantization config (manual override > QdrantConfig > preset)
        if let Some(quant) = quantization_override {
            // Manual override has highest precedence
            match quant {
                QuantizationConfigOverride::Scalar {
                    quant_type,
                    quantile,
                    always_ram,
                } => {
                    body["quantization_config"] = serde_json::json!({
                        "scalar": {
                            "type": quant_type,
                            "quantile": quantile,
                            "always_ram": always_ram
                        }
                    });
                }
                QuantizationConfigOverride::Product {
                    compression,
                    always_ram,
                } => {
                    body["quantization_config"] = serde_json::json!({
                        "product": {
                            "compression": compression,
                            "always_ram": always_ram
                        }
                    });
                }
                QuantizationConfigOverride::Disabled => {
                    // No quantization
                }
            }
        } else if let Some(ref quant_config) = self.config.quantization {
            // Use quantization from QdrantConfig
            use crate::config::modules::storage::QuantizationConfig;
            match quant_config {
                QuantizationConfig::Scalar(scalar) => {
                    body["quantization_config"] = serde_json::json!({
                        "scalar": {
                            "type": scalar.quant_type,
                            "quantile": scalar.quantile,
                            "always_ram": scalar.always_ram
                        }
                    });
                }
                QuantizationConfig::Product(product) => {
                    body["quantization_config"] = serde_json::json!({
                        "product": {
                            "compression": product.compression,
                            "always_ram": product.always_ram
                        }
                    });
                }
                QuantizationConfig::Disabled => {
                    // No quantization
                }
            }
        } else if let crate::storage::qdrant::config::CollectionPreset::Large = self.config.preset {
            // Fallback to preset-based quantization for backward compatibility
            body["quantization_config"] = serde_json::json!({
                "scalar": {
                    "type": "int8",
                    "quantile": 0.99,
                    "always_ram": false
                }
            });
        }

        // Add sparse vector quantization
        if let Some(sparse_quant) = sparse_quant_override {
            if sparse_quant.enabled {
                body["sparse_vectors_config"] = serde_json::json!({
                    "sparse": {
                        "index": {
                            "on_disk": false
                        },
                        "modifier": "idf",
                        "quantization": {
                            "scalar": {
                                "type": sparse_quant.quant_type,
                                "always_ram": sparse_quant.always_ram
                            }
                        }
                    }
                });
            }
        } else {
            // Default sparse vector config without quantization
            body["sparse_vectors"] = serde_json::json!({
                "sparse": {
                    "index": {
                        "on_disk": false
                    },
                    "modifier": "idf"
                }
            });
        }

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
                "Failed to create collection: {} - {}",
                status, error_text
            )));
        }

        // Create payload indexes
        self.create_payload_indexes().await?;

        tracing::info!("Created collection: {}", self.collection_name);
        Ok(())
    }

    /// Create the collection (legacy method, delegates to create_with_config)
    pub async fn create(&self) -> Result<(), QdrantError> {
        self.create_with_config(None, None, None, None).await
    }

    /// Delete the collection
    pub async fn delete(&self) -> Result<(), QdrantError> {
        let url = self.build_url();

        let response = self
            .http_client
            .delete(&url)
            .send()
            .await
            .map_err(|e| QdrantError::request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() && status != reqwest::StatusCode::NOT_FOUND {
            let error_text = response.text().await.unwrap_or_default();
            return Err(QdrantError::api(format!(
                "Failed to delete collection: {} - {}",
                status, error_text
            )));
        }

        tracing::info!("Deleted collection: {}", self.collection_name);
        Ok(())
    }

    /// Clear all points from the collection
    pub async fn clear(&self) -> Result<(), QdrantError> {
        let url = format!(
            "{}/collections/{}/points/delete",
            self.base_url, self.collection_name
        );

        let body = serde_json::json!({
            "filter": {
                "must": []
            }
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
                "Failed to clear collection: {} - {}",
                status, error_text
            )));
        }

        tracing::info!("Cleared collection: {}", self.collection_name);
        Ok(())
    }

    /// Check if collection exists
    pub async fn exists(&self) -> Result<bool, QdrantError> {
        match self.get_info().await {
            Ok(_) => Ok(true),
            Err(QdrantError::CollectionNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Get collection information
    pub async fn get_info(&self) -> Result<CollectionInfo, QdrantError> {
        let url = self.build_url();

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| QdrantError::request(e.to_string()))?;

        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(QdrantError::CollectionNotFound(
                crate::types::error::common::NotFoundError::new(self.collection_name.clone()),
            ));
        }

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(QdrantError::api(format!(
                "Failed to get collection info: {} - {}",
                status, error_text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| QdrantError::ResponseParse(e.to_string()))?;

        // Parse response
        let result = json
            .get("result")
            .ok_or_else(|| QdrantError::ResponseParse("Missing result field".to_string()))?;

        let points_count = result
            .get("points_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let indexed_vectors_count = result
            .get("indexed_vectors_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let segments_count = result
            .get("segments_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let status_str = result
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("grey");

        let status = match status_str {
            "green" => CollectionStatus::Green,
            "yellow" => CollectionStatus::Yellow,
            "red" => CollectionStatus::Red,
            _ => CollectionStatus::Grey,
        };

        // Parse vector config
        let vectors_config = result
            .get("config")
            .and_then(|c| c.get("params"))
            .and_then(|p| p.get("vectors"));

        let vector_size = vectors_config
            .and_then(|v| v.get("size"))
            .and_then(|s| s.as_u64())
            .unwrap_or(self.config.vector_size as u64) as usize;

        let distance_metric = vectors_config
            .and_then(|v| v.get("distance"))
            .and_then(|d| d.as_str())
            .unwrap_or("Cosine")
            .to_string();

        let vectors_on_disk = vectors_config
            .and_then(|v| v.get("on_disk"))
            .and_then(|d| d.as_bool())
            .unwrap_or(false);

        // Parse HNSW config
        let hnsw_config = result
            .get("config")
            .and_then(|c| c.get("hnsw_config"))
            .map(|h| HnswConfigInfo {
                m: h.get("m").and_then(|v| v.as_u64()).unwrap_or(16) as u32,
                ef_construct: h
                    .get("ef_construct")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(128) as u32,
                on_disk: h.get("on_disk").and_then(|v| v.as_bool()).unwrap_or(false),
            });

        Ok(CollectionInfo {
            name: self.collection_name.clone(),
            vector_size,
            distance_metric,
            points_count,
            indexed_vectors_count,
            segments_count,
            status,
            hnsw_config,
            vectors_on_disk,
        })
    }

    /// Create payload indexes for filtering
    async fn create_payload_indexes(&self) -> Result<(), QdrantError> {
        let url = self.build_index_url();

        // Create index for 'type' field
        self.create_index(&url, "type").await?;

        // Create indexes for pathSegments.0-4
        for i in 0..5 {
            self.create_index(&url, &format!("pathSegments.{}", i))
                .await?;
        }

        // Create index for 'file_extension' field
        self.create_index(&url, "file_extension").await?;

        // Create index for 'entity_type' field
        self.create_index(&url, "entity_type").await?;

        // Create index for 'language' field
        self.create_index(&url, "language").await?;

        // Create index for 'content_type' field
        self.create_index(&url, "content_type").await?;

        Ok(())
    }

    /// Create a single payload index
    async fn create_index(&self, url: &str, field_name: &str) -> Result<(), QdrantError> {
        let body = serde_json::json!({
            "field_name": field_name,
            "field_schema": "keyword"
        });

        let response = self
            .http_client
            .put(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| QdrantError::request(e.to_string()))?;

        if !response.status().is_success() {
            tracing::debug!("Payload index '{}' may already exist", field_name);
        }

        Ok(())
    }

    /// Build URL for collection endpoint
    fn build_url(&self) -> String {
        format!("{}/collections/{}", self.base_url, self.collection_name)
    }

    /// Build URL for index endpoint
    fn build_index_url(&self) -> String {
        format!(
            "{}/collections/{}/index",
            self.base_url, self.collection_name
        )
    }
}
