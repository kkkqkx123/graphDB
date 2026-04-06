use crate::config::{
    Bm25Config, FieldWeights, IndexManagerConfig, LogMergePolicyConfig, MergePolicyType,
    ReloadPolicyConfig, SearchConfig,
};
use serde::{Deserialize, Serialize};

/// Builder for IndexManagerConfig
///
/// Provides a fluent API for configuring index manager settings.
///
/// # Examples
///
/// ```rust
/// use bm25_service::config::{IndexManagerConfig, ReloadPolicyConfig, MergePolicyType};
///
/// let config = IndexManagerConfig::builder()
///     .writer_memory_mb(100)
///     .writer_threads(4)
///     .reader_cache(true)
///     .reload_policy(ReloadPolicyConfig::OnCommitWithDelay)
///     .merge_policy(MergePolicyType::Log)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct IndexManagerConfigBuilder {
    writer_memory_budget: usize,
    writer_num_threads: Option<usize>,
    reader_cache_enabled: bool,
    reader_reload_policy: ReloadPolicyConfig,
    merge_policy: MergePolicyType,
    log_merge_policy: LogMergePolicyConfig,
}

impl IndexManagerConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set writer memory budget in megabytes
    ///
    /// # Arguments
    ///
    /// * `mb` - Memory budget in megabytes (will be converted to bytes)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::IndexManagerConfig;
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .writer_memory_mb(100)  // 100MB
    ///     .build();
    /// ```
    pub fn writer_memory_mb(mut self, mb: usize) -> Self {
        self.writer_memory_budget = mb * 1_000_000;
        self
    }

    /// Set writer memory budget in bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - Memory budget in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::IndexManagerConfig;
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .writer_memory_bytes(100_000_000)  // 100MB
    ///     .build();
    /// ```
    pub fn writer_memory_bytes(mut self, bytes: usize) -> Self {
        self.writer_memory_budget = bytes;
        self
    }

    /// Set number of writer threads
    ///
    /// # Arguments
    ///
    /// * `threads` - Number of threads (None for auto-detection)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::IndexManagerConfig;
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .writer_threads(4)
    ///     .build();
    /// ```
    pub fn writer_threads(mut self, threads: usize) -> Self {
        self.writer_num_threads = Some(threads);
        self
    }

    /// Enable or disable reader caching
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable reader caching
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::IndexManagerConfig;
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .reader_cache(true)
    ///     .build();
    /// ```
    pub fn reader_cache(mut self, enabled: bool) -> Self {
        self.reader_cache_enabled = enabled;
        self
    }

    /// Set reader reload policy
    ///
    /// # Arguments
    ///
    /// * `policy` - Reload policy configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::{IndexManagerConfig, ReloadPolicyConfig};
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .reload_policy(ReloadPolicyConfig::OnCommitWithDelay)
    ///     .build();
    /// ```
    pub fn reload_policy(mut self, policy: ReloadPolicyConfig) -> Self {
        self.reader_reload_policy = policy;
        self
    }

    /// Set merge policy type
    ///
    /// # Arguments
    ///
    /// * `policy` - Merge policy type (Log or NoMerge)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::{IndexManagerConfig, MergePolicyType};
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .merge_policy(MergePolicyType::Log)
    ///     .build();
    /// ```
    pub fn merge_policy(mut self, policy: MergePolicyType) -> Self {
        self.merge_policy = policy;
        self
    }

    /// Set log merge policy configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Log merge policy configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::{IndexManagerConfig, LogMergePolicyConfig};
    ///
    /// let log_policy = LogMergePolicyConfig {
    ///     min_num_segments: 10,
    ///     max_docs_before_merge: 5_000_000,
    ///     ..Default::default()
    /// };
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .log_merge_policy(log_policy)
    ///     .build();
    /// ```
    pub fn log_merge_policy(mut self, config: LogMergePolicyConfig) -> Self {
        self.log_merge_policy = config;
        self
    }

    /// Build the final IndexManagerConfig
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::IndexManagerConfig;
    ///
    /// let config = IndexManagerConfig::builder()
    ///     .writer_memory_mb(100)
    ///     .writer_threads(4)
    ///     .reader_cache(true)
    ///     .build();
    /// ```
    pub fn build(self) -> IndexManagerConfig {
        IndexManagerConfig {
            writer_memory_budget: self.writer_memory_budget,
            writer_num_threads: self.writer_num_threads,
            reader_cache_enabled: self.reader_cache_enabled,
            reader_reload_policy: self.reader_reload_policy,
            merge_policy: self.merge_policy,
            log_merge_policy: self.log_merge_policy,
        }
    }
}

impl Default for IndexManagerConfigBuilder {
    fn default() -> Self {
        Self {
            writer_memory_budget: 50_000_000, // 50MB
            writer_num_threads: None,
            reader_cache_enabled: true,
            reader_reload_policy: ReloadPolicyConfig::default(),
            merge_policy: MergePolicyType::default(),
            log_merge_policy: LogMergePolicyConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let config = IndexManagerConfig::builder().build();
        assert_eq!(config.writer_memory_budget, 50_000_000);
        assert_eq!(config.writer_num_threads, None);
        assert!(config.reader_cache_enabled);
    }

    #[test]
    fn test_builder_memory_mb() {
        let config = IndexManagerConfig::builder()
            .writer_memory_mb(100)
            .build();
        assert_eq!(config.writer_memory_budget, 100_000_000);
    }

    #[test]
    fn test_builder_memory_bytes() {
        let config = IndexManagerConfig::builder()
            .writer_memory_bytes(100_000_000)
            .build();
        assert_eq!(config.writer_memory_budget, 100_000_000);
    }

    #[test]
    fn test_builder_threads() {
        let config = IndexManagerConfig::builder()
            .writer_threads(4)
            .build();
        assert_eq!(config.writer_num_threads, Some(4));
    }

    #[test]
    fn test_builder_reader_cache() {
        let config = IndexManagerConfig::builder()
            .reader_cache(false)
            .build();
        assert!(!config.reader_cache_enabled);
    }

    #[test]
    fn test_builder_chain() {
        let config = IndexManagerConfig::builder()
            .writer_memory_mb(100)
            .writer_threads(4)
            .reader_cache(true)
            .merge_policy(MergePolicyType::NoMerge)
            .build();

        assert_eq!(config.writer_memory_budget, 100_000_000);
        assert_eq!(config.writer_num_threads, Some(4));
        assert!(config.reader_cache_enabled);
        assert!(matches!(config.merge_policy, MergePolicyType::NoMerge));
    }
}

// ============================================================================
// Bm25ConfigBuilder
// ============================================================================

/// Builder for Bm25Config
///
/// Provides a fluent API for configuring BM25 algorithm parameters.
///
/// # Examples
///
/// ```rust
/// use bm25_service::config::Bm25Config;
///
/// let config = Bm25Config::builder()
///     .k1(1.5)
///     .b(0.8)
///     .avg_doc_length(150.0)
///     .field_weights(2.5, 1.0)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct Bm25ConfigBuilder {
    k1: f32,
    b: f32,
    avg_doc_length: f32,
    field_weights: FieldWeights,
}

impl Bm25ConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set k1 parameter (term frequency saturation)
    ///
    /// # Arguments
    ///
    /// * `k1` - k1 parameter (non-negative, typically 1.2-2.0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::Bm25Config;
    ///
    /// let config = Bm25Config::builder().k1(1.5).build();
    /// ```
    pub fn k1(mut self, k1: f32) -> Self {
        self.k1 = k1;
        self
    }

    /// Set b parameter (document length normalization)
    ///
    /// # Arguments
    ///
    /// * `b` - b parameter (in range [0.0, 1.0], typically 0.75)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::Bm25Config;
    ///
    /// let config = Bm25Config::builder().b(0.8).build();
    /// ```
    pub fn b(mut self, b: f32) -> Self {
        self.b = b;
        self
    }

    /// Set average document length
    ///
    /// # Arguments
    ///
    /// * `avg_len` - Average document length (positive value)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::Bm25Config;
    ///
    /// let config = Bm25Config::builder().avg_doc_length(150.0).build();
    /// ```
    pub fn avg_doc_length(mut self, avg_len: f32) -> Self {
        self.avg_doc_length = avg_len;
        self
    }

    /// Set field weights
    ///
    /// # Arguments
    ///
    /// * `title` - Weight for title field
    /// * `content` - Weight for content field
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::Bm25Config;
    ///
    /// let config = Bm25Config::builder().field_weights(2.5, 1.0).build();
    /// ```
    pub fn field_weights(mut self, title: f32, content: f32) -> Self {
        self.field_weights = FieldWeights { title, content };
        self
    }

    /// Set field weights using FieldWeights struct
    ///
    /// # Arguments
    ///
    /// * `weights` - Field weights configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::{Bm25Config, FieldWeights};
    ///
    /// let weights = FieldWeights::new(2.5, 1.0);
    /// let config = Bm25Config::builder().field_weights_struct(weights).build();
    /// ```
    pub fn field_weights_struct(mut self, weights: FieldWeights) -> Self {
        self.field_weights = weights;
        self
    }

    /// Build the final Bm25Config
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::Bm25Config;
    ///
    /// let config = Bm25Config::builder()
    ///     .k1(1.5)
    ///     .b(0.8)
    ///     .build();
    /// ```
    pub fn build(self) -> Bm25Config {
        Bm25Config {
            k1: self.k1,
            b: self.b,
            avg_doc_length: self.avg_doc_length,
            field_weights: self.field_weights,
        }
    }
}

impl Default for Bm25ConfigBuilder {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            avg_doc_length: 100.0,
            field_weights: FieldWeights::default(),
        }
    }
}

// ============================================================================
// SearchConfigBuilder
// ============================================================================

/// Builder for SearchConfig
///
/// Provides a fluent API for configuring search behavior.
///
/// # Examples
///
/// ```rust
/// use bm25_service::config::SearchConfig;
///
/// let config = SearchConfig::builder()
///     .default_limit(20)
///     .max_limit(200)
///     .enable_highlight(true)
///     .highlight_fragment_size(250)
///     .fuzzy_matching(true)
///     .fuzzy_distance(2)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SearchConfigBuilder {
    default_limit: usize,
    max_limit: usize,
    enable_highlight: bool,
    highlight_fragment_size: usize,
    enable_spell_check: bool,
    fuzzy_matching: bool,
    fuzzy_distance: u8,
}

impl SearchConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default search result limit
    ///
    /// # Arguments
    ///
    /// * `limit` - Default number of results to return
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder().default_limit(20).build();
    /// ```
    pub fn default_limit(mut self, limit: usize) -> Self {
        self.default_limit = limit;
        self
    }

    /// Set maximum search result limit
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results allowed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder().max_limit(200).build();
    /// ```
    pub fn max_limit(mut self, limit: usize) -> Self {
        self.max_limit = limit;
        self
    }

    /// Enable or disable result highlighting
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable highlighting
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder().enable_highlight(true).build();
    /// ```
    pub fn enable_highlight(mut self, enabled: bool) -> Self {
        self.enable_highlight = enabled;
        self
    }

    /// Set highlight fragment size
    ///
    /// # Arguments
    ///
    /// * `size` - Size of highlight fragments in characters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder().highlight_fragment_size(250).build();
    /// ```
    pub fn highlight_fragment_size(mut self, size: usize) -> Self {
        self.highlight_fragment_size = size;
        self
    }

    /// Enable or disable spell check
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable spell check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder().enable_spell_check(true).build();
    /// ```
    pub fn enable_spell_check(mut self, enabled: bool) -> Self {
        self.enable_spell_check = enabled;
        self
    }

    /// Enable or disable fuzzy matching
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable fuzzy matching
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder().fuzzy_matching(true).build();
    /// ```
    pub fn fuzzy_matching(mut self, enabled: bool) -> Self {
        self.fuzzy_matching = enabled;
        self
    }

    /// Set fuzzy matching distance
    ///
    /// # Arguments
    ///
    /// * `distance` - Fuzzy distance (0-10)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder().fuzzy_distance(2).build();
    /// ```
    pub fn fuzzy_distance(mut self, distance: u8) -> Self {
        self.fuzzy_distance = distance;
        self
    }

    /// Build the final SearchConfig
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::builder()
    ///     .default_limit(20)
    ///     .max_limit(200)
    ///     .build();
    /// ```
    pub fn build(self) -> SearchConfig {
        SearchConfig {
            default_limit: self.default_limit,
            max_limit: self.max_limit,
            enable_highlight: self.enable_highlight,
            highlight_fragment_size: self.highlight_fragment_size,
            enable_spell_check: self.enable_spell_check,
            fuzzy_matching: self.fuzzy_matching,
            fuzzy_distance: self.fuzzy_distance,
        }
    }
}

impl Default for SearchConfigBuilder {
    fn default() -> Self {
        Self {
            default_limit: 10,
            max_limit: 100,
            enable_highlight: true,
            highlight_fragment_size: 200,
            enable_spell_check: false,
            fuzzy_matching: false,
            fuzzy_distance: 2,
        }
    }
}

// ============================================================================
// Tests for Bm25ConfigBuilder and SearchConfigBuilder
// ============================================================================

#[cfg(test)]
mod bm25_builder_tests {
    use super::*;

    #[test]
    fn test_bm25_builder_default() {
        let config = Bm25Config::builder().build();
        assert_eq!(config.k1, 1.2);
        assert_eq!(config.b, 0.75);
        assert_eq!(config.avg_doc_length, 100.0);
        assert_eq!(config.field_weights.title, 2.0);
        assert_eq!(config.field_weights.content, 1.0);
    }

    #[test]
    fn test_bm25_builder_custom() {
        let config = Bm25Config::builder()
            .k1(1.5)
            .b(0.8)
            .avg_doc_length(150.0)
            .field_weights(2.5, 1.0)
            .build();
        assert_eq!(config.k1, 1.5);
        assert_eq!(config.b, 0.8);
        assert_eq!(config.avg_doc_length, 150.0);
        assert_eq!(config.field_weights.title, 2.5);
        assert_eq!(config.field_weights.content, 1.0);
    }

    #[test]
    fn test_bm25_builder_chain() {
        let config = Bm25Config::builder()
            .k1(2.0)
            .b(0.5)
            .build();
        assert_eq!(config.k1, 2.0);
        assert_eq!(config.b, 0.5);
    }
}

#[cfg(test)]
mod search_builder_tests {
    use super::*;

    #[test]
    fn test_search_builder_default() {
        let config = SearchConfig::builder().build();
        assert_eq!(config.default_limit, 10);
        assert_eq!(config.max_limit, 100);
        assert!(config.enable_highlight);
        assert_eq!(config.highlight_fragment_size, 200);
        assert!(!config.enable_spell_check);
        assert!(!config.fuzzy_matching);
        assert_eq!(config.fuzzy_distance, 2);
    }

    #[test]
    fn test_search_builder_custom() {
        let config = SearchConfig::builder()
            .default_limit(20)
            .max_limit(200)
            .enable_highlight(true)
            .highlight_fragment_size(250)
            .enable_spell_check(true)
            .fuzzy_matching(true)
            .fuzzy_distance(3)
            .build();
        assert_eq!(config.default_limit, 20);
        assert_eq!(config.max_limit, 200);
        assert!(config.enable_highlight);
        assert_eq!(config.highlight_fragment_size, 250);
        assert!(config.enable_spell_check);
        assert!(config.fuzzy_matching);
        assert_eq!(config.fuzzy_distance, 3);
    }

    #[test]
    fn test_search_builder_partial() {
        let config = SearchConfig::builder()
            .default_limit(50)
            .max_limit(500)
            .build();
        assert_eq!(config.default_limit, 50);
        assert_eq!(config.max_limit, 500);
        assert!(config.enable_highlight); // default
        assert_eq!(config.highlight_fragment_size, 200); // default
    }
}

// ============================================================================
// StorageConfigBuilder
// ============================================================================

/// Storage type enumeration
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    /// Tantivy local file storage
    #[default]
    Tantivy,
    /// Redis remote storage
    Redis,
}

/// Tantivy storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TantivyStorageConfig {
    pub index_path: String,
    pub writer_memory_mb: usize,
}

impl Default for TantivyStorageConfig {
    fn default() -> Self {
        Self {
            index_path: "./index".to_string(),
            writer_memory_mb: 50,
        }
    }
}

/// Redis storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisStorageConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout_secs: u64,
    pub key_prefix: String,
    pub min_idle: Option<u32>,
    pub max_lifetime_secs: Option<u64>,
}

impl Default for RedisStorageConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
            connection_timeout_secs: 5,
            key_prefix: "bm25".to_string(),
            min_idle: Some(2),
            max_lifetime_secs: Some(60),
        }
    }
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub storage_type: StorageType,
    #[serde(default)]
    pub tantivy: TantivyStorageConfig,
    #[serde(default)]
    pub redis: RedisStorageConfig,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: StorageType::Tantivy,
            tantivy: TantivyStorageConfig::default(),
            redis: RedisStorageConfig::default(),
        }
    }
}

impl StorageConfig {
    /// Create a new StorageConfigBuilder
    pub fn builder() -> StorageConfigBuilder {
        StorageConfigBuilder::new()
    }
}

/// Builder for StorageConfig
#[derive(Debug, Clone)]
pub struct StorageConfigBuilder {
    storage_type: StorageType,
    tantivy: TantivyStorageConfig,
    redis: RedisStorageConfig,
}

impl StorageConfigBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set storage type
    pub fn storage_type(mut self, storage_type: StorageType) -> Self {
        self.storage_type = storage_type;
        self
    }

    /// Set Tantivy storage configuration
    pub fn tantivy_config(mut self, config: TantivyStorageConfig) -> Self {
        self.tantivy = config;
        self
    }

    /// Set Tantivy index path
    pub fn tantivy_index_path(mut self, path: String) -> Self {
        self.tantivy.index_path = path;
        self
    }

    /// Set Tantivy writer memory in MB
    pub fn tantivy_writer_memory_mb(mut self, mb: usize) -> Self {
        self.tantivy.writer_memory_mb = mb;
        self
    }

    /// Set Redis storage configuration
    pub fn redis_config(mut self, config: RedisStorageConfig) -> Self {
        self.redis = config;
        self
    }

    /// Set Redis URL
    pub fn redis_url(mut self, url: String) -> Self {
        self.redis.url = url;
        self
    }

    /// Set Redis pool size
    pub fn redis_pool_size(mut self, size: u32) -> Self {
        self.redis.pool_size = size;
        self
    }

    /// Set Redis key prefix
    pub fn redis_key_prefix(mut self, prefix: String) -> Self {
        self.redis.key_prefix = prefix;
        self
    }

    /// Build the final StorageConfig
    pub fn build(self) -> StorageConfig {
        StorageConfig {
            storage_type: self.storage_type,
            tantivy: self.tantivy,
            redis: self.redis,
        }
    }
}

impl Default for StorageConfigBuilder {
    fn default() -> Self {
        Self {
            storage_type: StorageType::Tantivy,
            tantivy: TantivyStorageConfig::default(),
            redis: RedisStorageConfig::default(),
        }
    }
}

#[cfg(test)]
mod storage_builder_tests {
    use super::*;

    #[test]
    fn test_storage_builder_default() {
        let config = StorageConfig::builder().build();
        assert_eq!(config.storage_type, StorageType::Tantivy);
        assert_eq!(config.tantivy.index_path, "./index");
        assert_eq!(config.tantivy.writer_memory_mb, 50);
        assert_eq!(config.redis.pool_size, 10);
    }

    #[test]
    fn test_storage_builder_tantivy() {
        let config = StorageConfig::builder()
            .storage_type(StorageType::Tantivy)
            .tantivy_index_path("/data/index".to_string())
            .tantivy_writer_memory_mb(100)
            .build();
        assert_eq!(config.storage_type, StorageType::Tantivy);
        assert_eq!(config.tantivy.index_path, "/data/index");
        assert_eq!(config.tantivy.writer_memory_mb, 100);
    }

    #[test]
    fn test_storage_builder_redis() {
        let config = StorageConfig::builder()
            .storage_type(StorageType::Redis)
            .redis_url("redis://localhost:6379".to_string())
            .redis_pool_size(20)
            .redis_key_prefix("mybm25".to_string())
            .build();
        assert_eq!(config.storage_type, StorageType::Redis);
        assert_eq!(config.redis.url, "redis://localhost:6379");
        assert_eq!(config.redis.pool_size, 20);
        assert_eq!(config.redis.key_prefix, "mybm25");
    }
}
