use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    Euclid,
    Dot,
}

impl Default for DistanceMetric {
    fn default() -> Self {
        Self::Cosine
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    pub m: usize,
    pub ef_construct: usize,
    pub full_scan_threshold: Option<usize>,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construct: 100,
            full_scan_threshold: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub vector_size: usize,
    pub distance: DistanceMetric,
    pub hnsw_config: Option<HnswConfig>,
    pub replication_factor: Option<usize>,
    pub write_consistency_factor: Option<usize>,
    pub on_disk_payload: Option<bool>,
}

impl CollectionConfig {
    pub fn new(vector_size: usize, distance: DistanceMetric) -> Self {
        Self {
            vector_size,
            distance,
            hnsw_config: None,
            replication_factor: None,
            write_consistency_factor: None,
            on_disk_payload: None,
        }
    }

    pub fn with_hnsw(mut self, hnsw_config: HnswConfig) -> Self {
        self.hnsw_config = Some(hnsw_config);
        self
    }
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self::new(1536, DistanceMetric::Cosine)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    pub name: String,
    pub vector_count: u64,
    pub indexed_vector_count: u64,
    pub points_count: u64,
    pub segments_count: u64,
    pub config: CollectionConfig,
    pub status: CollectionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionStatus {
    Green,
    Yellow,
    Red,
    Grey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub engine_name: String,
    pub engine_version: String,
    pub message: Option<String>,
}

impl HealthStatus {
    pub fn healthy(engine_name: impl Into<String>, engine_version: impl Into<String>) -> Self {
        Self {
            is_healthy: true,
            engine_name: engine_name.into(),
            engine_version: engine_version.into(),
            message: None,
        }
    }

    pub fn unhealthy(
        engine_name: impl Into<String>,
        engine_version: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            is_healthy: false,
            engine_name: engine_name.into(),
            engine_version: engine_version.into(),
            message: Some(message.into()),
        }
    }
}
