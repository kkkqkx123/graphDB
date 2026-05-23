//! Configuration types for Qdrant vector storage
//!
//! This module defines configuration structures for Qdrant client,
//! including connection settings, HNSW parameters, and WAL configuration.

use serde::{Deserialize, Serialize};

// Re-export types from unified config system
pub use crate::config::modules::{CollectionPreset, DistanceMetric, QdrantConfig};

impl DistanceMetric {
    /// Get the string representation for Qdrant API
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cosine => "Cosine",
            Self::Euclid => "Euclid",
            Self::Dot => "Dot",
        }
    }
}

/// HNSW index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    /// Number of neighbors per node (2-128, default: 16)
    pub m: u32,
    /// Search range during index construction (10-1000, default: 128)
    pub ef_construct: u32,
    /// Store HNSW index on disk
    pub on_disk: bool,
    /// Store vector copies directly in HNSW index files (v1.16.0+)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_storage: Option<bool>,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construct: 128,
            on_disk: true,
            inline_storage: None,
        }
    }
}

impl HnswConfig {
    /// Create a new HNSW config
    pub fn new(m: u32, ef_construct: u32, on_disk: bool) -> Self {
        Self {
            m,
            ef_construct,
            on_disk,
            inline_storage: None,
        }
    }

    /// Create tiny preset (no HNSW, full scan)
    pub fn tiny() -> Option<Self> {
        None
    }

    /// Create small preset (m=16, ef_construct=128)
    pub fn small() -> Self {
        Self {
            m: 16,
            ef_construct: 128,
            on_disk: true,
            inline_storage: None,
        }
    }

    /// Create medium preset (m=32, ef_construct=256)
    pub fn medium() -> Self {
        Self {
            m: 32,
            ef_construct: 256,
            on_disk: true,
            inline_storage: None,
        }
    }

    /// Create large preset (m=64, ef_construct=512)
    pub fn large() -> Self {
        Self {
            m: 64,
            ef_construct: 512,
            on_disk: true,
            inline_storage: Some(true), // Enable inline storage for Large preset
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.m < 2 || self.m > 128 {
            return Err(format!("HNSW m must be between 2 and 128, got {}", self.m));
        }
        if self.ef_construct < 10 || self.ef_construct > 1000 {
            return Err(format!(
                "HNSW ef_construct must be between 10 and 1000, got {}",
                self.ef_construct
            ));
        }
        Ok(())
    }
}

/// WAL (Write-Ahead Log) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalConfig {
    /// WAL capacity in MB (default: 32)
    pub capacity_mb: u32,
    /// Number of WAL segments (default: 2)
    pub segments: u32,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            capacity_mb: 32,
            segments: 2,
        }
    }
}

impl WalConfig {
    /// Create a new WAL config
    pub fn new(capacity_mb: u32, segments: u32) -> Self {
        Self {
            capacity_mb,
            segments,
        }
    }

    /// Create tiny/small preset
    pub fn tiny() -> Self {
        Self {
            capacity_mb: 32,
            segments: 2,
        }
    }

    /// Create medium preset
    pub fn medium() -> Self {
        Self {
            capacity_mb: 64,
            segments: 4,
        }
    }

    /// Create large preset
    pub fn large() -> Self {
        Self {
            capacity_mb: 256,
            segments: 8,
        }
    }
}

/// Scalar quantization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalarQuantizationConfig {
    /// Quantization type (int8 or int16)
    #[serde(rename = "type")]
    pub quant_type: String,
    /// Quantile for outlier exclusion (0.0-1.0)
    pub quantile: Option<f32>,
    /// Always keep quantized vectors in RAM
    pub always_ram: Option<bool>,
}

/// Product quantization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductQuantizationConfig {
    /// Compression ratio (x8, x16, x32, x64, x128)
    pub compression: String,
    /// Always keep quantized vectors in RAM
    pub always_ram: Option<bool>,
}

/// Quantization configuration enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QuantizationConfig {
    /// Scalar quantization
    Scalar(ScalarQuantizationConfig),
    /// Product quantization
    Product(ProductQuantizationConfig),
}

impl QuantizationConfig {
    /// Create a scalar quantization config
    pub fn scalar(bits: u8, always_ram: bool) -> Self {
        let quant_type = if bits == 8 { "int8" } else { "int16" }.to_string();
        Self::Scalar(ScalarQuantizationConfig {
            quant_type,
            quantile: Some(0.99),
            always_ram: Some(always_ram),
        })
    }

    /// Create a product quantization config
    pub fn product(compression: &str, always_ram: bool) -> Self {
        Self::Product(ProductQuantizationConfig {
            compression: compression.to_string(),
            always_ram: Some(always_ram),
        })
    }
}

/// Vector storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStorageConfig {
    /// Store vectors on disk
    pub on_disk: bool,
    /// Quantization configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<QuantizationConfig>,
}

impl Default for VectorStorageConfig {
    fn default() -> Self {
        Self {
            on_disk: true,
            quantization: None,
        }
    }
}

impl CollectionPreset {
    /// Determine preset from vector count
    pub fn from_vector_count(count: usize) -> Self {
        if count <= 2000 {
            Self::Tiny
        } else if count <= 10000 {
            Self::Small
        } else if count <= 100000 {
            Self::Medium
        } else {
            Self::Large
        }
    }

    /// Get HNSW config for this preset
    pub fn hnsw_config(&self) -> Option<HnswConfig> {
        match self {
            Self::Tiny => HnswConfig::tiny(),
            Self::Small => Some(HnswConfig::small()),
            Self::Medium => Some(HnswConfig::medium()),
            Self::Large => Some(HnswConfig::large()),
        }
    }

    /// Get WAL config for this preset
    pub fn wal_config(&self) -> WalConfig {
        match self {
            Self::Tiny => WalConfig::tiny(),
            Self::Small => WalConfig::tiny(),
            Self::Medium => WalConfig::medium(),
            Self::Large => WalConfig::large(),
        }
    }
}

impl QdrantConfig {
    /// Create a new config with URL
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Self::default()
        }
    }

    /// Create a new config with vector size
    pub fn with_vector_size(vector_size: usize) -> Self {
        Self {
            vector_size,
            ..Self::default()
        }
    }

    /// Set the API key
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set the timeout
    pub fn timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set the preset
    pub fn preset(mut self, preset: CollectionPreset) -> Self {
        self.preset = preset;
        self
    }

    /// Enable the client
    pub fn enabled(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Disable the client
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Parse and normalize the URL
    pub fn normalized_url(&self) -> String {
        self.parse_url(&self.url)
    }

    /// Parse and normalize a URL string
    fn parse_url(&self, url: &str) -> String {
        let trimmed = url.trim();

        // Handle empty URL
        if trimmed.is_empty() {
            return "http://localhost:6333".to_string();
        }

        // Check if it has a protocol
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return trimmed.to_string();
        }

        // No protocol - treat as hostname
        if trimmed.contains(':') {
            // Has port - add http:// prefix
            format!("http://{}", trimmed)
        } else {
            // No port - add default port
            format!("http://{}:6333", trimmed)
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.vector_size == 0 {
            return Err("Vector size must be greater than 0".to_string());
        }
        if self.timeout_ms == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }
        if self.max_retries > 10 {
            return Err("Max retries must not exceed 10".to_string());
        }

        // Validate URL
        let normalized = self.normalized_url();
        if !normalized.starts_with("http://") && !normalized.starts_with("https://") {
            return Err(format!("Invalid URL: {}", self.url));
        }

        // Validate HNSW config if present
        if let Some(ref hnsw) = self.hnsw {
            if hnsw.m < 2 || hnsw.m > 128 {
                return Err(format!("HNSW m must be between 2 and 128, got {}", hnsw.m));
            }
            if hnsw.ef_construct < 10 || hnsw.ef_construct > 1000 {
                return Err(format!(
                    "HNSW ef_construct must be between 10 and 1000, got {}",
                    hnsw.ef_construct
                ));
            }
        }

        // Validate quantization config if present
        if let Some(ref quant) = self.quantization {
            use crate::config::modules::storage::QuantizationConfig;
            match quant {
                QuantizationConfig::Scalar(scalar) => {
                    if scalar.quant_type != "int8" && scalar.quant_type != "int16" {
                        return Err(format!(
                            "Scalar quantization type must be 'int8' or 'int16', got '{}'",
                            scalar.quant_type
                        ));
                    }
                    if scalar.quantile < 0.0 || scalar.quantile > 1.0 {
                        return Err(format!(
                            "Quantile must be between 0.0 and 1.0, got {}",
                            scalar.quantile
                        ));
                    }
                }
                QuantizationConfig::Product(product) => {
                    let valid_compressions = ["x8", "x16", "x32", "x64", "x128"];
                    if !valid_compressions.contains(&product.compression.as_str()) {
                        return Err(format!(
                            "Product quantization compression must be one of {:?}, got '{}'",
                            valid_compressions, product.compression
                        ));
                    }
                }
                QuantizationConfig::Disabled => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QdrantConfig::default();
        assert_eq!(config.url, "http://localhost:6333");
        assert_eq!(config.vector_size, 1024);
        assert!(config.enabled);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_url_parsing() {
        let config = QdrantConfig::with_url("localhost:6333");
        assert_eq!(config.normalized_url(), "http://localhost:6333");

        let config = QdrantConfig::with_url("http://localhost:6333");
        assert_eq!(config.normalized_url(), "http://localhost:6333");

        let config = QdrantConfig::with_url("https://qdrant.example.com");
        assert_eq!(config.normalized_url(), "https://qdrant.example.com");

        let config = QdrantConfig::with_url("localhost");
        assert_eq!(config.normalized_url(), "http://localhost:6333");

        let config = QdrantConfig::with_url("");
        assert_eq!(config.normalized_url(), "http://localhost:6333");
    }

    #[test]
    fn test_config_builder() {
        let config = QdrantConfig::with_url("localhost:6333")
            .api_key("test-key")
            .timeout(60000)
            .preset(CollectionPreset::Large);

        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout_ms, 60000);
        assert_eq!(config.preset, CollectionPreset::Large);
    }

    #[test]
    fn test_config_validation() {
        let config = QdrantConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = QdrantConfig {
            vector_size: 0,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_hnsw_presets() {
        let small = HnswConfig::small();
        assert_eq!(small.m, 16);
        assert_eq!(small.ef_construct, 128);
        assert!(small.validate().is_ok());

        let medium = HnswConfig::medium();
        assert_eq!(medium.m, 32);
        assert_eq!(medium.ef_construct, 256);

        let large = HnswConfig::large();
        assert_eq!(large.m, 64);
        assert_eq!(large.ef_construct, 512);
    }

    #[test]
    fn test_hnsw_validation() {
        let invalid = HnswConfig::new(1, 128, true);
        assert!(invalid.validate().is_err());

        let invalid = HnswConfig::new(16, 5, true);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_preset_from_count() {
        assert_eq!(
            CollectionPreset::from_vector_count(1000),
            CollectionPreset::Tiny
        );
        assert_eq!(
            CollectionPreset::from_vector_count(5000),
            CollectionPreset::Small
        );
        assert_eq!(
            CollectionPreset::from_vector_count(50000),
            CollectionPreset::Medium
        );
        assert_eq!(
            CollectionPreset::from_vector_count(200000),
            CollectionPreset::Large
        );
    }

    #[test]
    fn test_distance_metric() {
        assert_eq!(DistanceMetric::Cosine.as_str(), "Cosine");
        assert_eq!(DistanceMetric::Euclid.as_str(), "Euclid");
        assert_eq!(DistanceMetric::Dot.as_str(), "Dot");
    }
}
