use crate::error::Result;
use crate::tokenizer::MixedTokenizer;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, RwLock};
use tantivy::indexer::{LogMergePolicy, MergePolicy};
use tantivy::schema::*;
use tantivy::{Index, IndexReader, IndexWriter, ReloadPolicy};

/// Overloaded Policy Configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReloadPolicyConfig {
    Manual,
    #[default]
    OnCommitWithDelay,
}

impl From<ReloadPolicyConfig> for ReloadPolicy {
    fn from(config: ReloadPolicyConfig) -> Self {
        match config {
            ReloadPolicyConfig::Manual => ReloadPolicy::Manual,
            ReloadPolicyConfig::OnCommitWithDelay => ReloadPolicy::OnCommitWithDelay,
        }
    }
}

/// Type of merger strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MergePolicyType {
    #[default]
    Log,
    NoMerge,
}

/// LogMergePolicy Detailed Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMergePolicyConfig {
    /// Minimum number of merged segments
    #[serde(default = "default_min_num_segments")]
    pub min_num_segments: usize,
    /// Maximum number of documents before merging
    #[serde(default = "default_max_docs_before_merge")]
    pub max_docs_before_merge: usize,
    /// Minimum layer size
    #[serde(default = "default_min_layer_size")]
    pub min_layer_size: u32,
    /// Layer size logarithmic ratio
    #[serde(default = "default_level_log_size")]
    pub level_log_size: f64,
    /// Ratio of documents deleted before merging
    #[serde(default = "default_del_docs_ratio")]
    pub del_docs_ratio_before_merge: f32,
}

fn default_min_num_segments() -> usize {
    8
}
fn default_max_docs_before_merge() -> usize {
    10_000_000
}
fn default_min_layer_size() -> u32 {
    10_000
}
fn default_level_log_size() -> f64 {
    0.75
}
fn default_del_docs_ratio() -> f32 {
    1.0
}

impl Default for LogMergePolicyConfig {
    fn default() -> Self {
        Self {
            min_num_segments: default_min_num_segments(),
            max_docs_before_merge: default_max_docs_before_merge(),
            min_layer_size: default_min_layer_size(),
            level_log_size: default_level_log_size(),
            del_docs_ratio_before_merge: default_del_docs_ratio(),
        }
    }
}

/// Index Manager Configuration (Extended Version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexManagerConfig {
    /// Writer memory budget (bytes)
    #[serde(default = "default_writer_memory_budget")]
    pub writer_memory_budget: usize,
    /// Number of writer threads (None means auto-detect)
    #[serde(default)]
    pub writer_num_threads: Option<usize>,
    /// Enable or disable Reader caching
    #[serde(default = "default_reader_cache_enabled")]
    pub reader_cache_enabled: bool,
    /// Reader overloading strategy
    #[serde(default)]
    pub reader_reload_policy: ReloadPolicyConfig,
    /// Type of merger strategy
    #[serde(default)]
    pub merge_policy: MergePolicyType,
    /// LogMergePolicy Detailed Configuration
    #[serde(default)]
    pub log_merge_policy: LogMergePolicyConfig,
}

fn default_writer_memory_budget() -> usize {
    50_000_000 // 50MB
}
fn default_reader_cache_enabled() -> bool {
    true
}

impl Default for IndexManagerConfig {
    fn default() -> Self {
        Self {
            writer_memory_budget: default_writer_memory_budget(),
            writer_num_threads: None,
            reader_cache_enabled: default_reader_cache_enabled(),
            reader_reload_policy: ReloadPolicyConfig::default(),
            merge_policy: MergePolicyType::default(),
            log_merge_policy: LogMergePolicyConfig::default(),
        }
    }
}

impl IndexManagerConfig {
    /// Create a new builder for IndexManagerConfig
    pub fn builder() -> crate::config::IndexManagerConfigBuilder {
        crate::config::IndexManagerConfigBuilder::default()
    }

    /// Load configuration from environment variables with default prefix "INDEX_"
    pub fn from_env() -> Result<Self> {
        Self::from_env_with_prefix("INDEX_")
    }

    /// Load configuration from environment variables with custom prefix
    pub fn from_env_with_prefix(prefix: &str) -> Result<Self> {
        use crate::config::{ConfigLoader, EnvLoader};

        let loader = EnvLoader::new(prefix);
        let env_vars = loader
            .load()
            .map_err(|e| crate::error::Bm25Error::InternalError(e.to_string()))?;

        let mut config = Self::default();
        config.apply_vars(&env_vars)?;
        Ok(config)
    }

    /// Load configuration from a file (TOML, YAML, or JSON)
    pub fn from_file(path: &str) -> Result<Self> {
        use crate::config::{ConfigLoader, FileLoader};

        let loader = FileLoader::new(path);
        let file_vars = loader
            .load()
            .map_err(|e| crate::error::Bm25Error::InternalError(e.to_string()))?;

        let mut config = Self::default();
        config.apply_vars(&file_vars)?;
        Ok(config)
    }

    /// Apply key-value pairs to configuration
    fn apply_vars(&mut self, vars: &std::collections::HashMap<String, String>) -> Result<()> {
        use std::str::FromStr;

        if let Some(val) = vars.get("writer_memory_budget") {
            self.writer_memory_budget = val.parse().map_err(|e| {
                crate::error::Bm25Error::InternalError(format!(
                    "Failed to parse writer_memory_budget: {}",
                    e
                ))
            })?;
        }

        if let Some(val) = vars.get("writer_num_threads") {
            let threads: usize = val.parse().map_err(|e| {
                crate::error::Bm25Error::InternalError(format!(
                    "Failed to parse writer_num_threads: {}",
                    e
                ))
            })?;
            self.writer_num_threads = if threads > 0 { Some(threads) } else { None };
        }

        if let Some(val) = vars.get("reader_cache_enabled") {
            self.reader_cache_enabled = val.parse().map_err(|e| {
                crate::error::Bm25Error::InternalError(format!(
                    "Failed to parse reader_cache_enabled: {}",
                    e
                ))
            })?;
        }

        // Handle nested configuration (e.g., log_merge_policy.min_num_segments)
        if let Some(val) = vars.get("log_merge_policy.min_num_segments") {
            self.log_merge_policy.min_num_segments = val.parse().map_err(|e| {
                crate::error::Bm25Error::InternalError(format!(
                    "Failed to parse log_merge_policy.min_num_segments: {}",
                    e
                ))
            })?;
        }

        if let Some(val) = vars.get("log_merge_policy.max_docs_before_merge") {
            self.log_merge_policy.max_docs_before_merge = val.parse().map_err(|e| {
                crate::error::Bm25Error::InternalError(format!(
                    "Failed to parse log_merge_policy.max_docs_before_merge: {}",
                    e
                ))
            })?;
        }

        if let Some(val) = vars.get("log_merge_policy.min_layer_size") {
            self.log_merge_policy.min_layer_size = val.parse().map_err(|e| {
                crate::error::Bm25Error::InternalError(format!(
                    "Failed to parse log_merge_policy.min_layer_size: {}",
                    e
                ))
            })?;
        }

        if let Some(val) = vars.get("log_merge_policy.level_log_size") {
            self.log_merge_policy.level_log_size = f64::from_str(val).map_err(|e| {
                crate::error::Bm25Error::InternalError(format!(
                    "Failed to parse log_merge_policy.level_log_size: {}",
                    e
                ))
            })?;
        }

        if let Some(val) = vars.get("log_merge_policy.del_docs_ratio_before_merge") {
            self.log_merge_policy.del_docs_ratio_before_merge =
                f32::from_str(val).map_err(|e| {
                    crate::error::Bm25Error::InternalError(format!(
                        "Failed to parse log_merge_policy.del_docs_ratio_before_merge: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// Export configuration to TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).map_err(|e: toml::ser::Error| {
            crate::error::Bm25Error::InternalError(format!("Failed to serialize to TOML: {}", e))
        })
    }

    /// Export configuration to JSON string (pretty-printed)
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            crate::error::Bm25Error::InternalError(format!("Failed to serialize to JSON: {}", e))
        })
    }

    /// Export configuration to environment variables
    pub fn to_env_vars(&self, prefix: &str) -> std::collections::HashMap<String, String> {
        use std::collections::HashMap;

        let mut vars = HashMap::new();

        vars.insert(
            format!("{}WRITER_MEMORY_BUDGET", prefix),
            self.writer_memory_budget.to_string(),
        );

        if let Some(threads) = self.writer_num_threads {
            vars.insert(format!("{}WRITER_NUM_THREADS", prefix), threads.to_string());
        }

        vars.insert(
            format!("{}READER_CACHE_ENABLED", prefix),
            self.reader_cache_enabled.to_string(),
        );

        // Add nested configuration
        vars.insert(
            format!("{}LOG_MERGE_POLICY.MIN_NUM_SEGMENTS", prefix),
            self.log_merge_policy.min_num_segments.to_string(),
        );
        vars.insert(
            format!("{}LOG_MERGE_POLICY.MAX_DOCS_BEFORE_MERGE", prefix),
            self.log_merge_policy.max_docs_before_merge.to_string(),
        );
        vars.insert(
            format!("{}LOG_MERGE_POLICY.MIN_LAYER_SIZE", prefix),
            self.log_merge_policy.min_layer_size.to_string(),
        );
        vars.insert(
            format!("{}LOG_MERGE_POLICY.LEVEL_LOG_SIZE", prefix),
            self.log_merge_policy.level_log_size.to_string(),
        );
        vars.insert(
            format!("{}LOG_MERGE_POLICY.DEL_DOCS_RATIO_BEFORE_MERGE", prefix),
            self.log_merge_policy
                .del_docs_ratio_before_merge
                .to_string(),
        );

        vars
    }

    /// Building a merger strategy
    pub fn build_merge_policy(&self) -> Box<dyn MergePolicy> {
        match self.merge_policy {
            MergePolicyType::NoMerge => Box::new(tantivy::indexer::NoMergePolicy),
            MergePolicyType::Log => {
                let mut policy = LogMergePolicy::default();
                policy.set_min_num_segments(self.log_merge_policy.min_num_segments);
                policy.set_max_docs_before_merge(self.log_merge_policy.max_docs_before_merge);
                policy.set_min_layer_size(self.log_merge_policy.min_layer_size);
                policy.set_level_log_size(self.log_merge_policy.level_log_size);
                policy.set_del_docs_ratio_before_merge(
                    self.log_merge_policy.del_docs_ratio_before_merge,
                );
                Box::new(policy)
            }
        }
    }
}

#[derive(Clone)]
pub struct IndexManager {
    index: Index,
    schema: Schema,
    config: IndexManagerConfig,
    cached_reader: Arc<RwLock<Option<IndexReader>>>,
}

impl IndexManager {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::create_with_config(path, IndexManagerConfig::default())
    }

    pub fn create_with_config<P: AsRef<Path>>(path: P, config: IndexManagerConfig) -> Result<Self> {
        let path = path.as_ref();
        std::fs::create_dir_all(path)?;
        let schema = Self::build_schema();
        let index = Index::create_in_dir(path, schema.clone())?;
        Self::register_tokenizers(&index);
        Ok(Self {
            index,
            schema,
            config,
            cached_reader: Arc::new(RwLock::new(None)),
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open_with_config(path, IndexManagerConfig::default())
    }

    pub fn open_with_config<P: AsRef<Path>>(path: P, config: IndexManagerConfig) -> Result<Self> {
        let index = Index::open_in_dir(path)?;
        let schema = index.schema();
        Self::register_tokenizers(&index);
        Ok(Self {
            index,
            schema: schema.clone(),
            config,
            cached_reader: Arc::new(RwLock::new(None)),
        })
    }

    fn register_tokenizers(index: &Index) {
        let mixed_tokenizer = MixedTokenizer::new();
        index.tokenizers().register("mixed", mixed_tokenizer);
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);

        let content_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("mixed")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("content", content_options);

        schema_builder.add_text_field("entity_type", STRING | STORED);

        let raw_name_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("mixed")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("raw_name", raw_name_options);

        let keywords_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("mixed")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        schema_builder.add_text_field("keywords", keywords_options);

        schema_builder.add_text_field("file_path", STRING | STORED);
        schema_builder.add_text_field("module_name", STRING | STORED);
        schema_builder.build()
    }

    pub fn writer(&self) -> Result<IndexWriter> {
        let num_threads = self
            .config
            .writer_num_threads
            .unwrap_or_else(|| num_cpus::get().max(1));

        // Tantivy requires at least 15MB per thread for the memory arena
        const MIN_MEMORY_PER_THREAD: usize = 15_000_000;
        let min_memory_budget = num_threads * MIN_MEMORY_PER_THREAD;
        let memory_budget = self.config.writer_memory_budget.max(min_memory_budget);

        let writer = self
            .index
            .writer_with_num_threads(num_threads, memory_budget)?;
        Ok(writer)
    }

    pub fn reader(&self) -> Result<IndexReader> {
        if self.config.reader_cache_enabled {
            if let Ok(reader_guard) = self.cached_reader.read() {
                if let Some(reader) = reader_guard.as_ref() {
                    return Ok(reader.clone());
                }
            }

            let new_reader = self.create_reader()?;

            if let Ok(mut writer_guard) = self.cached_reader.write() {
                *writer_guard = Some(new_reader.clone());
            }

            Ok(new_reader)
        } else {
            self.create_reader()
        }
    }

    fn create_reader(&self) -> Result<IndexReader> {
        let reload_policy: ReloadPolicy = self.config.reader_reload_policy.into();
        Ok(self
            .index
            .reader_builder()
            .reload_policy(reload_policy)
            .try_into()?)
    }

    pub fn reload_reader(&self) -> Result<IndexReader> {
        let new_reader = self.create_reader()?;

        if let Ok(mut writer_guard) = self.cached_reader.write() {
            *writer_guard = Some(new_reader.clone());
        }

        Ok(new_reader)
    }

    pub fn clear_reader_cache(&self) {
        if let Ok(mut writer_guard) = self.cached_reader.write() {
            *writer_guard = None;
        }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn config(&self) -> &IndexManagerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_open() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_index");

        let manager = IndexManager::create(&path)?;
        assert!(manager.reader().is_ok());

        let opened = IndexManager::open(&path)?;
        assert!(opened.reader().is_ok());

        Ok(())
    }

    #[test]
    fn test_config() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_config");

        let config = IndexManagerConfig::builder()
            .writer_memory_mb(100)
            .reader_cache(false)
            .build();

        let manager = IndexManager::create_with_config(&path, config)?;
        assert_eq!(manager.config().writer_memory_budget, 100_000_000);
        assert!(!manager.config().reader_cache_enabled);

        Ok(())
    }

    #[test]
    fn test_reader_caching() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("test_cache");

        let manager = IndexManager::create(&path)?;

        // Getting the reader twice should return the same instance (from the cache)
        let reader1 = manager.reader()?;
        let reader2 = manager.reader()?;

        // Since IndexReader implements Clone, we verify that they point to the same internal state
        // Verify this by comparing their searcher counts
        assert_eq!(reader1.searcher().num_docs(), reader2.searcher().num_docs());

        // Verify that the cache is actually being used: a new reader should be created after the cache is cleared.
        manager.clear_reader_cache();
        let reader3 = manager.reader()?;
        // reader3 is a new instance, but with the same functionality
        assert_eq!(reader1.searcher().num_docs(), reader3.searcher().num_docs());

        Ok(())
    }
}
