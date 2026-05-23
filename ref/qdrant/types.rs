//! Data types for Qdrant vector storage
//!
//! This module defines data structures for vectors, payloads,
//! search results, and collection information.

use serde::{Deserialize, Serialize};

/// Qdrant fusion configuration for hybrid search
///
/// Controls prefetch limits for dense and sparse paths.
/// Fusion strategy is always RRF (handled server-side by Qdrant).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QdrantFusionConfig {
    /// Multiplier for dense vector prefetch limit (default: 2.5)
    pub dense_prefetch_multiplier: f32,
    /// Multiplier for sparse vector prefetch limit (default: 4.0)
    pub sparse_prefetch_multiplier: f32,
    /// Minimum prefetch limit for both paths (default: 20)
    pub min_prefetch_limit: usize,
    /// RRF k parameter (default: 60)
    pub rrf_k: u32,
}

impl Default for QdrantFusionConfig {
    fn default() -> Self {
        Self {
            dense_prefetch_multiplier: 2.5,
            sparse_prefetch_multiplier: 4.0,
            min_prefetch_limit: 20,
            rrf_k: 60,
        }
    }
}

impl QdrantFusionConfig {
    /// Validate the fusion configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.dense_prefetch_multiplier < 1.0 {
            return Err("dense_prefetch_multiplier must be >= 1.0".to_string());
        }
        if self.sparse_prefetch_multiplier < 1.0 {
            return Err("sparse_prefetch_multiplier must be >= 1.0".to_string());
        }
        if self.min_prefetch_limit == 0 {
            return Err("min_prefetch_limit must be greater than 0".to_string());
        }
        Ok(())
    }
}

/// Sparse vector for Qdrant (indices-values format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparseVector {
    /// Indices (token IDs) - must be sorted in ascending order
    pub indices: Vec<u32>,
    /// Corresponding values (weights)
    pub values: Vec<f32>,
}

impl SparseVector {
    /// Create a new sparse vector from lexical weights (token -> weight)
    /// Note: This requires a tokenizer to convert tokens to IDs.
    /// For now, we store as-is and conversion should happen before calling this.
    pub fn from_lexical_weights(indices: Vec<u32>, values: Vec<f32>) -> Self {
        // Ensure indices are sorted (Qdrant requirement)
        let mut paired: Vec<(u32, f32)> = indices
            .iter()
            .copied()
            .zip(values.iter().copied())
            .collect();
        paired.sort_by_key(|(idx, _)| *idx);

        let (sorted_indices, sorted_values): (Vec<u32>, Vec<f32>) = paired.into_iter().unzip();

        Self {
            indices: sorted_indices,
            values: sorted_values,
        }
    }

    /// Check if the sparse vector is empty
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty() || self.values.is_empty()
    }
}

/// Vector point with payload and optional sparse vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    /// Unique point ID
    pub id: String,
    /// Dense vector data
    pub vector: Vec<f32>,
    /// Optional sparse vector (for BGE M3 lexical weights)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sparse_vector: Option<SparseVector>,
    /// Payload metadata
    pub payload: Payload,
}

impl VectorPoint {
    /// Create a new vector point with dense vector only
    pub fn new(id: impl Into<String>, vector: Vec<f32>, payload: Payload) -> Self {
        Self {
            id: id.into(),
            vector,
            sparse_vector: None,
            payload,
        }
    }

    /// Create a vector point with both dense and sparse vectors
    pub fn with_sparse(
        id: impl Into<String>,
        vector: Vec<f32>,
        sparse_vector: SparseVector,
        payload: Payload,
    ) -> Self {
        Self {
            id: id.into(),
            vector,
            sparse_vector: Some(sparse_vector),
            payload,
        }
    }

    /// Create a vector point with minimal payload
    pub fn with_file_path(
        id: impl Into<String>,
        vector: Vec<f32>,
        file_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            vector,
            sparse_vector: None,
            payload: Payload::new(file_path),
        }
    }
}

/// Compact payload metadata for a vector point (optimized version)
///
/// # Optimization Notes
///
/// This is a memory-optimized version of Payload that stores only essential fields:
/// - `entity_id`: For joining with SQLite metadata
/// - `file_path`: For filtering and display
/// - `start_line`/`end_line`: For locating code in files
///
/// Other fields (entity_type, file_extension, language, content_type, file_name, path_segments)
/// can be computed on-demand or fetched from SQLite via entity_id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactPayload {
    /// Entity ID for relation queries (primary key for metadata lookup)
    pub entity_id: u64,
    /// File path (normalized with forward slashes)
    pub file_path: String,
    /// Start line number
    pub start_line: u32,
    /// End line number
    pub end_line: u32,
}

impl CompactPayload {
    /// Create a new compact payload
    pub fn new(
        entity_id: u64,
        file_path: impl Into<String>,
        start_line: u32,
        end_line: u32,
    ) -> Self {
        Self {
            entity_id,
            file_path: file_path.into().replace('\\', "/"),
            start_line,
            end_line,
        }
    }

    /// Get file extension (computed on-demand)
    pub fn file_extension(&self) -> Option<String> {
        std::path::Path::new(&self.file_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
    }

    /// Get file name (computed on-demand)
    pub fn file_name(&self) -> Option<String> {
        std::path::Path::new(&self.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
    }

    /// Get programming language (computed on-demand)
    pub fn language(&self) -> Option<String> {
        Self::infer_language(self.file_extension().as_deref())
    }

    /// Infer programming language from file extension
    fn infer_language(file_extension: Option<&str>) -> Option<String> {
        let ext = file_extension?;
        match ext {
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "java" => Some("java".to_string()),
            "go" => Some("go".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "c" => Some("c".to_string()),
            "h" | "hpp" => Some("header".to_string()),
            "vue" => Some("vue".to_string()),
            "svelte" => Some("svelte".to_string()),
            _ => None,
        }
    }
}

/// Payload metadata for a vector point
///
/// Minimal payload design: only essential fields for filtering and relation queries.
/// All other metadata (lines, entity_type, etc.) is stored in SQLite and fetched on-demand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    /// File path (normalized with forward slashes) - used for filtering
    pub file_path: String,
    /// Entity ID for relation queries - primary key for metadata lookup
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<u64>,
    /// Serialized pattern detection information (JSON) for the entity group
    /// Populated during indexing from EntityGroup.pattern_info.
    /// Consumers can deserialize into PatternInfo for pattern-aware processing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_info: Option<String>,
}

impl Payload {
    /// Create a new payload
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into().replace('\\', "/"),
            entity_id: None,
            pattern_info: None,
        }
    }

    /// Set the entity ID
    pub fn with_entity_id(mut self, entity_id: u64) -> Self {
        self.entity_id = Some(entity_id);
        self
    }

    /// Set the pattern info
    pub fn with_pattern_info(mut self, pattern_info: Option<String>) -> Self {
        self.pattern_info = pattern_info;
        self
    }
}

/// Search query parameters
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Query vector (dense)
    pub vector: Vec<f32>,
    /// Optional sparse query vector
    pub sparse_vector: Option<SparseVector>,
    /// Maximum number of results
    pub limit: usize,
    /// Minimum score threshold
    pub min_score: Option<f32>,
    /// Directory prefix filter
    pub directory_prefix: Option<String>,
    /// HNSW ef parameter for search
    pub hnsw_ef: Option<u32>,
    /// Fusion configuration (optional, uses default if None)
    pub fusion_config: Option<QdrantFusionConfig>,
}

impl SearchQuery {
    /// Create a new search query with dense vector only
    pub fn new(vector: Vec<f32>, limit: usize) -> Self {
        Self {
            vector,
            sparse_vector: None,
            limit,
            min_score: None,
            directory_prefix: None,
            hnsw_ef: None,
            fusion_config: None,
        }
    }

    /// Create a hybrid search query with both dense and sparse vectors
    pub fn new_hybrid(vector: Vec<f32>, sparse_vector: SparseVector, limit: usize) -> Self {
        Self {
            vector,
            sparse_vector: Some(sparse_vector),
            limit,
            min_score: None,
            directory_prefix: None,
            hnsw_ef: None,
            fusion_config: None,
        }
    }

    /// Create a hybrid search query with custom fusion configuration
    pub fn new_hybrid_with_config(
        vector: Vec<f32>,
        sparse_vector: SparseVector,
        limit: usize,
        fusion_config: QdrantFusionConfig,
    ) -> Self {
        Self {
            vector,
            sparse_vector: Some(sparse_vector),
            limit,
            min_score: None,
            directory_prefix: None,
            hnsw_ef: None,
            fusion_config: Some(fusion_config),
        }
    }

    /// Set minimum score threshold
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Set directory prefix filter
    pub fn with_directory_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.directory_prefix = Some(prefix.into());
        self
    }

    /// Set HNSW ef parameter
    pub fn with_hnsw_ef(mut self, ef: u32) -> Self {
        self.hnsw_ef = Some(ef);
        self
    }

    /// Set fusion configuration
    pub fn with_fusion_config(mut self, config: QdrantFusionConfig) -> Self {
        self.fusion_config = Some(config);
        self
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Point ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Payload
    pub payload: Payload,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(id: impl Into<String>, score: f32, payload: Payload) -> Self {
        Self {
            id: id.into(),
            score,
            payload,
        }
    }
}

/// Collection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInfo {
    /// Collection name
    pub name: String,
    /// Vector size
    pub vector_size: usize,
    /// Distance metric
    pub distance_metric: String,
    /// Total number of points
    pub points_count: u64,
    /// Number of indexed vectors
    pub indexed_vectors_count: u64,
    /// Number of segments
    pub segments_count: u64,
    /// Collection status
    pub status: CollectionStatus,
    /// HNSW config
    pub hnsw_config: Option<HnswConfigInfo>,
    /// Whether vectors are stored on disk
    pub vectors_on_disk: bool,
}

/// Collection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollectionStatus {
    /// Green - healthy
    Green,
    /// Yellow - optimization in progress
    Yellow,
    /// Red - error
    Red,
    /// Grey - initializing
    Grey,
}

/// HNSW configuration info from collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfigInfo {
    /// M parameter
    pub m: u32,
    /// Ef construct parameter
    pub ef_construct: u32,
    /// Whether index is on disk
    pub on_disk: bool,
}

/// Indexing metadata marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingMetadata {
    /// Whether indexing is complete
    pub indexing_complete: bool,
    /// Timestamp when indexing started/completed
    pub timestamp: u64,
    /// Metadata type marker
    #[serde(rename = "type")]
    pub metadata_type: String,
}

impl IndexingMetadata {
    /// Create a completion marker
    pub fn complete() -> Self {
        Self {
            indexing_complete: true,
            timestamp: crate::utils::current_timestamp_ms(),
            metadata_type: "metadata".to_string(),
        }
    }

    /// Create an in-progress marker
    pub fn in_progress() -> Self {
        Self {
            indexing_complete: false,
            timestamp: crate::utils::current_timestamp_ms(),
            metadata_type: "metadata".to_string(),
        }
    }
}

/// Size estimation result
#[derive(Debug, Clone)]
pub struct SizeEstimation {
    /// Estimated vector count
    pub estimated_vector_count: usize,
    /// File count used for estimation
    pub file_count: usize,
    /// Average vectors per file
    pub avg_vectors_per_file: f32,
}

impl SizeEstimation {
    /// Estimate from file count
    pub fn from_file_count(file_count: usize, avg_vectors_per_file: f32) -> Self {
        Self {
            estimated_vector_count: (file_count as f32 * avg_vectors_per_file) as usize,
            file_count,
            avg_vectors_per_file,
        }
    }

    /// Get recommended preset for this size
    pub fn recommended_preset(&self) -> crate::storage::qdrant::config::CollectionPreset {
        crate::storage::qdrant::config::CollectionPreset::from_vector_count(
            self.estimated_vector_count,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_creation() {
        let payload = Payload::new("src/lib.rs");
        assert_eq!(payload.file_path, "src/lib.rs");
        assert!(payload.entity_id.is_none());
    }

    #[test]
    fn test_payload_path_normalization() {
        let payload = Payload::new("src\\lib\\test.rs");
        assert_eq!(payload.file_path, "src/lib/test.rs");
    }

    #[test]
    fn test_payload_validation() {
        // Test empty file path
        let invalid = Payload::new("");
        assert_eq!(invalid.file_path, "");

        // Test valid payload
        let valid = Payload::new("test.rs").with_entity_id(1);
        assert_eq!(valid.file_path, "test.rs");
        assert_eq!(valid.entity_id, Some(1));
    }

    #[test]
    fn test_vector_point() {
        let point = VectorPoint::with_file_path("point-1", vec![0.1, 0.2, 0.3], "src/main.rs");
        assert_eq!(point.id, "point-1");
        assert_eq!(point.vector.len(), 3);
        assert_eq!(point.payload.file_path, "src/main.rs");
    }

    #[test]
    fn test_search_query() {
        let query = SearchQuery::new(vec![0.1, 0.2], 10)
            .with_min_score(0.5)
            .with_directory_prefix("src/lib");

        assert_eq!(query.limit, 10);
        assert_eq!(query.min_score, Some(0.5));
        assert_eq!(query.directory_prefix, Some("src/lib".to_string()));
    }

    #[test]
    fn test_size_estimation() {
        let estimation = SizeEstimation::from_file_count(100, 10.0);
        assert_eq!(estimation.estimated_vector_count, 1000);
        assert_eq!(estimation.file_count, 100);
    }

    #[test]
    fn test_payload_with_entity_id() {
        // Test Rust code file
        let payload = Payload::new("src/main.rs").with_entity_id(1);
        assert_eq!(payload.file_path, "src/main.rs");
        assert_eq!(payload.entity_id, Some(1));

        // Test Python code file
        let payload = Payload::new("app.py").with_entity_id(2);
        assert_eq!(payload.file_path, "app.py");
        assert_eq!(payload.entity_id, Some(2));

        // Test TypeScript file
        let payload = Payload::new("components/App.tsx").with_entity_id(3);
        assert_eq!(payload.file_path, "components/App.tsx");
        assert_eq!(payload.entity_id, Some(3));

        // Test Markdown document
        let payload = Payload::new("README.md");
        assert_eq!(payload.file_path, "README.md");
        assert!(payload.entity_id.is_none());

        // Test TOML config
        let payload = Payload::new("Cargo.toml");
        assert_eq!(payload.file_path, "Cargo.toml");
        assert!(payload.entity_id.is_none());

        // Test .env file (no extension)
        let payload = Payload::new(".env");
        assert_eq!(payload.file_path, ".env");
        assert!(payload.entity_id.is_none());

        // Test C++ header file
        let payload = Payload::new("include/utils.hpp").with_entity_id(4);
        assert_eq!(payload.file_path, "include/utils.hpp");
        assert_eq!(payload.entity_id, Some(4));

        // Test JavaScript file
        let payload = Payload::new("script.js").with_entity_id(5);
        assert_eq!(payload.file_path, "script.js");
        assert_eq!(payload.entity_id, Some(5));

        // Test unknown file type
        let payload = Payload::new("data.bin");
        assert_eq!(payload.file_path, "data.bin");
        assert!(payload.entity_id.is_none());
    }

    #[test]
    fn test_indexing_metadata() {
        let complete = IndexingMetadata::complete();
        assert!(complete.indexing_complete);
        assert_eq!(complete.metadata_type, "metadata");

        let in_progress = IndexingMetadata::in_progress();
        assert!(!in_progress.indexing_complete);
    }

    #[test]
    fn test_qdrant_fusion_config_default() {
        let config = QdrantFusionConfig::default();
        assert!((config.dense_prefetch_multiplier - 2.5).abs() < f32::EPSILON);
        assert!((config.sparse_prefetch_multiplier - 4.0).abs() < f32::EPSILON);
        assert_eq!(config.min_prefetch_limit, 20);
        assert_eq!(config.rrf_k, 60);
    }

    #[test]
    fn test_qdrant_fusion_config_validation() {
        let config = QdrantFusionConfig::default();
        assert!(config.validate().is_ok());

        let invalid_mult = QdrantFusionConfig {
            dense_prefetch_multiplier: 0.5,
            ..Default::default()
        };
        assert!(invalid_mult.validate().is_err());

        let invalid_min = QdrantFusionConfig {
            min_prefetch_limit: 0,
            ..Default::default()
        };
        assert!(invalid_min.validate().is_err());
    }

    #[test]
    fn test_search_query_with_fusion_config() {
        let fusion_config = QdrantFusionConfig {
            dense_prefetch_multiplier: 3.0,
            sparse_prefetch_multiplier: 5.0,
            min_prefetch_limit: 30,
            ..Default::default()
        };

        let query = SearchQuery::new(vec![0.1, 0.2], 10).with_fusion_config(fusion_config.clone());
        assert!(query.fusion_config.is_some());
        let config = query.fusion_config.unwrap();
        assert!((config.dense_prefetch_multiplier - 3.0).abs() < f32::EPSILON);
        assert!((config.sparse_prefetch_multiplier - 5.0).abs() < f32::EPSILON);
    }
}
