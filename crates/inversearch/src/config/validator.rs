//! Configuration validation module
//!
//! Provides traits and implementations for validating configuration values.
//!
//! # Examples
//!
//! ```rust
//! use inversearch_service::config::validator::{ConfigValidator, ValidationResult};
//! use inversearch_service::config::Config;
//!
//! let config = Config::default();
//! config.validate().expect("Configuration validation failed");
//! ```

use thiserror::Error;

/// Validation result type
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validation error types
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Invalid value for a configuration field
    #[error("Invalid value for {field}: {value} ({reason})")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
    },
    
    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Configuration dependency error
    #[error("Configuration dependency error: {dependency}")]
    DependencyError { dependency: String },
}

/// Trait for configuration validation
pub trait ConfigValidator {
    /// Validate the configuration
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(ValidationError)` if validation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use inversearch_service::config::validator::ConfigValidator;
    ///
    /// struct MyConfig {
    ///     value: u32,
    /// }
    ///
    /// impl ConfigValidator for MyConfig {
    ///     fn validate(&self) -> ValidationResult<()> {
    ///         if self.value == 0 {
    ///             return Err(ValidationError::InvalidValue {
    ///                 field: "value".to_string(),
    ///                 value: self.value.to_string(),
    ///                 reason: "must be positive".to_string(),
    ///             });
    ///         }
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn validate(&self) -> ValidationResult<()>;
}

// ============================================================================
// Validation Implementations
// ============================================================================

use crate::config::{Config, ServerConfig, IndexConfig, CacheConfig, StorageConfig, LoggingConfig};

impl ConfigValidator for Config {
    fn validate(&self) -> ValidationResult<()> {
        self.server.validate()?;
        self.index.validate()?;
        self.cache.validate()?;
        self.storage.validate()?;
        self.logging.validate()?;
        Ok(())
    }
}

impl ConfigValidator for ServerConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate port range
        if self.port == 0 {
            return Err(ValidationError::InvalidValue {
                field: "server.port".to_string(),
                value: self.port.to_string(),
                reason: "port cannot be 0".to_string(),
            });
        }
        
        // Validate host is not empty
        if self.host.is_empty() {
            return Err(ValidationError::InvalidValue {
                field: "server.host".to_string(),
                value: "empty".to_string(),
                reason: "host cannot be empty".to_string(),
            });
        }
        
        // Validate workers range
        if self.workers == 0 {
            return Err(ValidationError::InvalidValue {
                field: "server.workers".to_string(),
                value: self.workers.to_string(),
                reason: "workers must be at least 1".to_string(),
            });
        }
        
        Ok(())
    }
}

impl ConfigValidator for IndexConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate resolution range (1-12)
        if self.resolution < 1 || self.resolution > 12 {
            return Err(ValidationError::InvalidValue {
                field: "index.resolution".to_string(),
                value: self.resolution.to_string(),
                reason: "must be between 1 and 12".to_string(),
            });
        }
        
        // Validate tokenize mode
        let valid_modes = ["strict", "forward", "reverse", "full", "bidirectional"];
        if !valid_modes.contains(&self.tokenize.as_str()) {
            return Err(ValidationError::InvalidValue {
                field: "index.tokenize".to_string(),
                value: self.tokenize.clone(),
                reason: format!("must be one of: {:?}", valid_modes),
            });
        }
        
        // Validate depth range
        if self.depth > 10 {
            return Err(ValidationError::InvalidValue {
                field: "index.depth".to_string(),
                value: self.depth.to_string(),
                reason: "depth should not exceed 10".to_string(),
            });
        }
        
        Ok(())
    }
}

impl ConfigValidator for CacheConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate cache size when enabled
        if self.enabled && self.size == 0 {
            return Err(ValidationError::InvalidValue {
                field: "cache.size".to_string(),
                value: self.size.to_string(),
                reason: "cache size must be positive when enabled".to_string(),
            });
        }
        
        // Validate cache size upper bound
        if self.size > 1_000_000 {
            return Err(ValidationError::InvalidValue {
                field: "cache.size".to_string(),
                value: self.size.to_string(),
                reason: "cache size should not exceed 1,000,000".to_string(),
            });
        }
        
        // Validate TTL range
        if let Some(ttl) = self.ttl {
            if ttl > 86400 {
                return Err(ValidationError::InvalidValue {
                    field: "cache.ttl".to_string(),
                    value: ttl.to_string(),
                    reason: "TTL should not exceed 86400 seconds (24 hours)".to_string(),
                });
            }
        }
        
        Ok(())
    }
}

impl ConfigValidator for StorageConfig {
    fn validate(&self) -> ValidationResult<()> {
        if self.enabled {
            // Validate storage-specific configurations
            #[cfg(feature = "store-redis")]
            if let Some(redis_config) = &self.redis {
                if redis_config.url.is_empty() {
                    return Err(ValidationError::InvalidValue {
                        field: "storage.redis.url".to_string(),
                        value: "empty".to_string(),
                        reason: "Redis URL cannot be empty".to_string(),
                    });
                }
                
                if redis_config.pool_size == 0 {
                    return Err(ValidationError::InvalidValue {
                        field: "storage.redis.pool_size".to_string(),
                        value: redis_config.pool_size.to_string(),
                        reason: "pool size must be positive".to_string(),
                    });
                }
            }
            
            #[cfg(feature = "store-file")]
            if let Some(file_config) = &self.file {
                if file_config.base_path.is_empty() {
                    return Err(ValidationError::InvalidValue {
                        field: "storage.file.base_path".to_string(),
                        value: "empty".to_string(),
                        reason: "base path cannot be empty".to_string(),
                    });
                }
                
                if file_config.save_interval_secs == 0 {
                    return Err(ValidationError::InvalidValue {
                        field: "storage.file.save_interval_secs".to_string(),
                        value: file_config.save_interval_secs.to_string(),
                        reason: "save interval must be positive".to_string(),
                    });
                }
            }
            
            #[cfg(feature = "store-wal")]
            if let Some(wal_config) = &self.wal {
                if wal_config.base_path.is_empty() {
                    return Err(ValidationError::InvalidValue {
                        field: "storage.wal.base_path".to_string(),
                        value: "empty".to_string(),
                        reason: "base path cannot be empty".to_string(),
                    });
                }
                
                if wal_config.max_wal_size == 0 {
                    return Err(ValidationError::InvalidValue {
                        field: "storage.wal.max_wal_size".to_string(),
                        value: wal_config.max_wal_size.to_string(),
                        reason: "max WAL size must be positive".to_string(),
                    });
                }
            }
        }
        
        Ok(())
    }
}

impl ConfigValidator for LoggingConfig {
    fn validate(&self) -> ValidationResult<()> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.level.as_str()) {
            return Err(ValidationError::InvalidValue {
                field: "logging.level".to_string(),
                value: self.level.clone(),
                reason: format!("must be one of: {:?}", valid_levels),
            });
        }
        
        // Validate log format
        let valid_formats = ["json", "text"];
        if !valid_formats.contains(&self.format.as_str()) {
            return Err(ValidationError::InvalidValue {
                field: "logging.format".to_string(),
                value: self.format.clone(),
                reason: format!("must be one of: {:?}", valid_formats),
            });
        }
        
        Ok(())
    }
}
