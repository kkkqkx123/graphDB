//! Vector Search Configuration
//!
//! Configuration types for vector search functionality.

use serde::{Deserialize, Serialize};

use vector_client::config::{ConnectionConfig, RetryConfig, TimeoutConfig, VectorClientConfig};
use vector_client::types::{CollectionConfig, DistanceMetric, HnswConfig, QuantizationConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorConfig {
    pub enabled: bool,
    pub engine: VectorEngineType,
    pub qdrant: QdrantConfig,
    pub default_vector_size: usize,
    pub default_distance: VectorDistance,
    pub sync: VectorSyncConfig,
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            engine: VectorEngineType::Qdrant,
            qdrant: QdrantConfig::default(),
            default_vector_size: 1536,
            default_distance: VectorDistance::Cosine,
            sync: VectorSyncConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorEngineType {
    Qdrant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub api_key: Option<String>,
    pub connect_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub search_timeout_secs: u64,
    pub upsert_timeout_secs: u64,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6333,
            use_tls: false,
            api_key: None,
            connect_timeout_secs: 5,
            request_timeout_secs: 30,
            search_timeout_secs: 60,
            upsert_timeout_secs: 30,
        }
    }
}

impl QdrantConfig {
    pub fn to_connection_config(&self) -> ConnectionConfig {
        ConnectionConfig {
            host: self.host.clone(),
            port: self.port,
            use_tls: self.use_tls,
            api_key: self.api_key.clone(),
            connect_timeout_secs: self.connect_timeout_secs,
        }
    }

    pub fn to_timeout_config(&self) -> TimeoutConfig {
        TimeoutConfig::new(
            self.request_timeout_secs,
            self.search_timeout_secs,
            self.upsert_timeout_secs,
        )
    }

    pub fn to_client_config(&self) -> VectorClientConfig {
        VectorClientConfig::qdrant()
            .with_connection(self.to_connection_config())
            .with_timeout(self.to_timeout_config())
            .with_retry(RetryConfig::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorDistance {
    Cosine,
    Euclid,
    Dot,
}

impl From<VectorDistance> for DistanceMetric {
    fn from(dist: VectorDistance) -> Self {
        match dist {
            VectorDistance::Cosine => DistanceMetric::Cosine,
            VectorDistance::Euclid => DistanceMetric::Euclid,
            VectorDistance::Dot => DistanceMetric::Dot,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSyncConfig {
    pub batch_size: usize,
    pub commit_interval_ms: u64,
    pub queue_size: usize,
}

impl Default for VectorSyncConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            commit_interval_ms: 1000,
            queue_size: 10000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexConfig {
    pub vector_size: usize,
    pub distance: VectorDistance,
    pub hnsw: Option<HnswConfigOptions>,
    pub quantization: Option<QuantizationOptions>,
}

impl Default for VectorIndexConfig {
    fn default() -> Self {
        Self {
            vector_size: 1536,
            distance: VectorDistance::Cosine,
            hnsw: None,
            quantization: None,
        }
    }
}

impl VectorIndexConfig {
    pub fn to_collection_config(&self) -> CollectionConfig {
        CollectionConfig::new(self.vector_size, self.distance.into())
            .with_hnsw(
                self.hnsw
                    .as_ref()
                    .map(|h| HnswConfig::new(h.m, h.ef_construct))
                    .unwrap_or_default(),
            )
            .with_quantization(
                self.quantization
                    .as_ref()
                    .map(|q| q.to_quantization_config())
                    .unwrap_or_default(),
            )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfigOptions {
    pub m: usize,
    pub ef_construct: usize,
    pub full_scan_threshold: Option<usize>,
    pub on_disk: Option<bool>,
}

impl Default for HnswConfigOptions {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construct: 100,
            full_scan_threshold: None,
            on_disk: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationOptions {
    pub enabled: bool,
    pub quant_type: QuantizationTypeOption,
}

impl Default for QuantizationOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            quant_type: QuantizationTypeOption::Scalar,
        }
    }
}

impl QuantizationOptions {
    pub fn to_quantization_config(&self) -> QuantizationConfig {
        if !self.enabled {
            return QuantizationConfig::disabled();
        }

        match self.quant_type {
            QuantizationTypeOption::Scalar => QuantizationConfig::scalar(0.99),
            QuantizationTypeOption::Product { compression } => {
                QuantizationConfig::product(compression.into())
            }
            QuantizationTypeOption::Binary => QuantizationConfig::binary(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QuantizationTypeOption {
    Scalar,
    Product { compression: CompressionRatioOption },
    Binary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionRatioOption {
    X4,
    X8,
    X16,
    X32,
    X64,
}

impl From<CompressionRatioOption> for vector_client::types::CompressionRatio {
    fn from(ratio: CompressionRatioOption) -> Self {
        match ratio {
            CompressionRatioOption::X4 => vector_client::types::CompressionRatio::X4,
            CompressionRatioOption::X8 => vector_client::types::CompressionRatio::X8,
            CompressionRatioOption::X16 => vector_client::types::CompressionRatio::X16,
            CompressionRatioOption::X32 => vector_client::types::CompressionRatio::X32,
            CompressionRatioOption::X64 => vector_client::types::CompressionRatio::X64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndexMetadata {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub collection_name: String,
    pub config: VectorIndexConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub vector_count: u64,
}

impl VectorIndexMetadata {
    pub fn index_key(&self) -> String {
        format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
    }

    pub fn collection_name(space_id: u64, tag_name: &str, field_name: &str) -> String {
        format!("space_{}_{}_{}", space_id, tag_name, field_name)
    }
}
