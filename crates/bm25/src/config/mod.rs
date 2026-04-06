//! Configuration module for BM25 service
//!
//! This module provides configuration structures and builders for the BM25 service.
//!
//! # Examples
//!
//! ```rust
//! use bm25_service::config::IndexManagerConfig;
//!
//! // Use builder pattern for fluent configuration
//! let config = IndexManagerConfig::builder()
//!     .writer_memory_mb(100)
//!     .writer_threads(4)
//!     .reader_cache(true)
//!     .build();
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod builder;
mod loader;
mod validator;

pub use builder::{Bm25ConfigBuilder, IndexManagerConfigBuilder, SearchConfigBuilder, StorageConfigBuilder};
pub use builder::{StorageConfig, StorageType, TantivyStorageConfig, RedisStorageConfig};
pub use loader::{ConfigFormat, ConfigLoader, EnvLoader, FileLoader, LoaderError, LoaderResult};
pub use validator::{ConfigValidator, ValidationError, ValidationResult};

// Re-export types from api::core
pub use crate::api::core::{
    IndexManagerConfig, LogMergePolicyConfig, MergePolicyType, ReloadPolicyConfig,
};

/// BM25 algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Config {
    pub k1: f32,
    pub b: f32,
    pub avg_doc_length: f32,
    pub field_weights: FieldWeights,
}

impl Bm25Config {
    /// Create a new Bm25ConfigBuilder
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
    pub fn builder() -> Bm25ConfigBuilder {
        Bm25ConfigBuilder::new()
    }

    /// Load configuration from environment variables
    ///
    /// # Arguments
    ///
    /// * `prefix` - Environment variable prefix (e.g., "BM25_")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::Bm25Config;
    ///
    /// // Set environment variables:
    /// // BM25_K1=1.5
    /// // BM25_B=0.8
    /// let config = Bm25Config::from_env("BM25_").unwrap();
    /// ```
    pub fn from_env(prefix: &str) -> Result<Self, crate::config::loader::LoaderError> {
        let loader = EnvLoader::new(prefix);
        let vars = loader.load()?;

        let mut config = Self::default();
        config.apply_vars(&vars)?;
        Ok(config)
    }

    /// Load configuration from file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to configuration file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bm25_service::config::Bm25Config;
    ///
    /// let config = Bm25Config::from_file("config.toml").unwrap();
    /// ```
    pub fn from_file(path: &str) -> Result<Self, crate::config::loader::LoaderError> {
        let loader = FileLoader::new(path);
        let vars = loader.load()?;

        let mut config = Self::default();
        config.apply_vars(&vars)?;
        Ok(config)
    }

    /// Apply configuration from key-value pairs
    fn apply_vars(&mut self, vars: &HashMap<String, String>) -> Result<(), crate::config::loader::LoaderError> {
        if let Some(val) = vars.get("k1") {
            self.k1 = val.parse::<f32>()
                .map_err(|e| LoaderError::ParseError(format!("k1: {}", e)))?;
        }

        if let Some(val) = vars.get("b") {
            self.b = val.parse::<f32>()
                .map_err(|e| LoaderError::ParseError(format!("b: {}", e)))?;
        }

        if let Some(val) = vars.get("avg_doc_length") {
            self.avg_doc_length = val.parse::<f32>()
                .map_err(|e| LoaderError::ParseError(format!("avg_doc_length: {}", e)))?;
        }

        if let Some(val) = vars.get("field_weights") {
            // Parse as "title,content" format
            let parts: Vec<&str> = val.split(',').collect();
            if parts.len() == 2 {
                self.field_weights.title = parts[0].parse::<f32>()
                    .map_err(|e| LoaderError::ParseError(format!("field_weights.title: {}", e)))?;
                self.field_weights.content = parts[1].parse::<f32>()
                    .map_err(|e| LoaderError::ParseError(format!("field_weights.content: {}", e)))?;
            }
        }

        Ok(())
    }
}

impl Default for Bm25Config {
    fn default() -> Self {
        Bm25Config {
            k1: 1.2,
            b: 0.75,
            avg_doc_length: 100.0,
            field_weights: FieldWeights::default(),
        }
    }
}

/// Field weights for search scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldWeights {
    pub title: f32,
    pub content: f32,
}

impl FieldWeights {
    /// Create a new FieldWeights with the specified weights
    ///
    /// # Arguments
    ///
    /// * `title` - Weight for title field
    /// * `content` - Weight for content field
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::FieldWeights;
    ///
    /// let weights = FieldWeights::new(2.5, 1.0);
    /// ```
    pub fn new(title: f32, content: f32) -> Self {
        Self { title, content }
    }
}

impl Default for FieldWeights {
    fn default() -> Self {
        FieldWeights {
            title: 2.0,
            content: 1.0,
        }
    }
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub enable_highlight: bool,
    pub highlight_fragment_size: usize,
    pub enable_spell_check: bool,
    pub fuzzy_matching: bool,
    pub fuzzy_distance: u8,
}

impl SearchConfig {
    /// Create a new SearchConfigBuilder
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
    ///     .build();
    /// ```
    pub fn builder() -> SearchConfigBuilder {
        SearchConfigBuilder::new()
    }

    /// Load configuration from environment variables
    ///
    /// # Arguments
    ///
    /// * `prefix` - Environment variable prefix (e.g., "SEARCH_")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bm25_service::config::SearchConfig;
    ///
    /// // Set environment variables:
    /// // SEARCH_DEFAULT_LIMIT=20
    /// // SEARCH_MAX_LIMIT=200
    /// let config = SearchConfig::from_env("SEARCH_").unwrap();
    /// ```
    pub fn from_env(prefix: &str) -> Result<Self, crate::config::loader::LoaderError> {
        let loader = EnvLoader::new(prefix);
        let vars = loader.load()?;

        let mut config = Self::default();
        config.apply_vars(&vars)?;
        Ok(config)
    }

    /// Load configuration from file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to configuration file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use bm25_service::config::SearchConfig;
    ///
    /// let config = SearchConfig::from_file("config.toml").unwrap();
    /// ```
    pub fn from_file(path: &str) -> Result<Self, crate::config::loader::LoaderError> {
        let loader = FileLoader::new(path);
        let vars = loader.load()?;

        let mut config = Self::default();
        config.apply_vars(&vars)?;
        Ok(config)
    }

    /// Apply configuration from key-value pairs
    fn apply_vars(&mut self, vars: &HashMap<String, String>) -> Result<(), crate::config::loader::LoaderError> {
        if let Some(val) = vars.get("default_limit") {
            self.default_limit = val.parse::<usize>()
                .map_err(|e| LoaderError::ParseError(format!("default_limit: {}", e)))?;
        }

        if let Some(val) = vars.get("max_limit") {
            self.max_limit = val.parse::<usize>()
                .map_err(|e| LoaderError::ParseError(format!("max_limit: {}", e)))?;
        }

        if let Some(val) = vars.get("enable_highlight") {
            self.enable_highlight = val.parse::<bool>()
                .map_err(|e| LoaderError::ParseError(format!("enable_highlight: {}", e)))?;
        }

        if let Some(val) = vars.get("highlight_fragment_size") {
            self.highlight_fragment_size = val.parse::<usize>()
                .map_err(|e| LoaderError::ParseError(format!("highlight_fragment_size: {}", e)))?;
        }

        if let Some(val) = vars.get("enable_spell_check") {
            self.enable_spell_check = val.parse::<bool>()
                .map_err(|e| LoaderError::ParseError(format!("enable_spell_check: {}", e)))?;
        }

        if let Some(val) = vars.get("fuzzy_matching") {
            self.fuzzy_matching = val.parse::<bool>()
                .map_err(|e| LoaderError::ParseError(format!("fuzzy_matching: {}", e)))?;
        }

        if let Some(val) = vars.get("fuzzy_distance") {
            self.fuzzy_distance = val.parse::<u8>()
                .map_err(|e| LoaderError::ParseError(format!("fuzzy_distance: {}", e)))?;
        }

        Ok(())
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
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
// Validation Implementations
// ============================================================================

impl ConfigValidator for IndexManagerConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate writer memory budget (minimum 1MB)
        if self.writer_memory_budget < 1_000_000 {
            return Err(ValidationError::InvalidValue {
                field: "writer_memory_budget".to_string(),
                value: self.writer_memory_budget.to_string(),
                reason: "must be at least 1MB (1_000_000 bytes)".to_string(),
            });
        }

        // Validate writer num threads
        if let Some(threads) = self.writer_num_threads {
            if threads == 0 {
                return Err(ValidationError::InvalidValue {
                    field: "writer_num_threads".to_string(),
                    value: threads.to_string(),
                    reason: "must be greater than 0".to_string(),
                });
            }
        }

        // Validate log merge policy
        self.log_merge_policy.validate()?;

        Ok(())
    }
}

impl ConfigValidator for LogMergePolicyConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate min_num_segments
        if self.min_num_segments < 2 {
            return Err(ValidationError::InvalidValue {
                field: "min_num_segments".to_string(),
                value: self.min_num_segments.to_string(),
                reason: "must be at least 2".to_string(),
            });
        }

        // Validate max_docs_before_merge
        if self.max_docs_before_merge == 0 {
            return Err(ValidationError::InvalidValue {
                field: "max_docs_before_merge".to_string(),
                value: self.max_docs_before_merge.to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        // Validate min_layer_size
        if self.min_layer_size == 0 {
            return Err(ValidationError::InvalidValue {
                field: "min_layer_size".to_string(),
                value: self.min_layer_size.to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        // Validate level_log_size (should be in range (0.0, 1.0])
        if self.level_log_size <= 0.0 || self.level_log_size > 1.0 {
            return Err(ValidationError::InvalidValue {
                field: "level_log_size".to_string(),
                value: self.level_log_size.to_string(),
                reason: "must be in range (0.0, 1.0]".to_string(),
            });
        }

        // Validate del_docs_ratio_before_merge
        if self.del_docs_ratio_before_merge < 0.0 || self.del_docs_ratio_before_merge > 1.0 {
            return Err(ValidationError::InvalidValue {
                field: "del_docs_ratio_before_merge".to_string(),
                value: self.del_docs_ratio_before_merge.to_string(),
                reason: "must be in range [0.0, 1.0]".to_string(),
            });
        }

        Ok(())
    }
}

impl ConfigValidator for Bm25Config {
    fn validate(&self) -> ValidationResult<()> {
        // Validate k1 (must be non-negative)
        if self.k1 < 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "k1".to_string(),
                value: self.k1.to_string(),
                reason: "must be non-negative".to_string(),
            });
        }

        // Validate b (must be in range [0.0, 1.0])
        if self.b < 0.0 || self.b > 1.0 {
            return Err(ValidationError::InvalidValue {
                field: "b".to_string(),
                value: self.b.to_string(),
                reason: "must be in range [0.0, 1.0]".to_string(),
            });
        }

        // Validate avg_doc_length (must be positive)
        if self.avg_doc_length <= 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "avg_doc_length".to_string(),
                value: self.avg_doc_length.to_string(),
                reason: "must be positive".to_string(),
            });
        }

        // Validate field weights
        self.field_weights.validate()?;

        Ok(())
    }
}

impl ConfigValidator for FieldWeights {
    fn validate(&self) -> ValidationResult<()> {
        // Validate title weight (must be non-negative)
        if self.title < 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "title".to_string(),
                value: self.title.to_string(),
                reason: "must be non-negative".to_string(),
            });
        }

        // Validate content weight (must be non-negative)
        if self.content < 0.0 {
            return Err(ValidationError::InvalidValue {
                field: "content".to_string(),
                value: self.content.to_string(),
                reason: "must be non-negative".to_string(),
            });
        }

        Ok(())
    }
}

impl ConfigValidator for SearchConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate default_limit
        if self.default_limit == 0 {
            return Err(ValidationError::InvalidValue {
                field: "default_limit".to_string(),
                value: self.default_limit.to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        // Validate max_limit
        if self.max_limit == 0 {
            return Err(ValidationError::InvalidValue {
                field: "max_limit".to_string(),
                value: self.max_limit.to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        // Validate default_limit <= max_limit
        if self.default_limit > self.max_limit {
            return Err(ValidationError::DependencyError {
                field: "default_limit".to_string(),
                dependency: format!(
                    "default_limit ({}) must not exceed max_limit ({})",
                    self.default_limit, self.max_limit
                ),
            });
        }

        // Validate highlight_fragment_size
        if self.highlight_fragment_size == 0 {
            return Err(ValidationError::InvalidValue {
                field: "highlight_fragment_size".to_string(),
                value: self.highlight_fragment_size.to_string(),
                reason: "must be greater than 0".to_string(),
            });
        }

        // Validate fuzzy_distance
        if self.fuzzy_distance > 10 {
            return Err(ValidationError::InvalidValue {
                field: "fuzzy_distance".to_string(),
                value: self.fuzzy_distance.to_string(),
                reason: "must not exceed 10".to_string(),
            });
        }

        Ok(())
    }
}
