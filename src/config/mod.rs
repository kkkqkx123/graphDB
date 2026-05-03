//! Configuration Management
//!
//! Unified configuration management for different usage patterns.
//!
//! # Module Structure
//!
//! The configuration system is organized into three main modules:
//!
//! - **common**: Configuration shared across all usage patterns (database, storage, logging, etc.)
//! - **server**: Server-specific configuration (gRPC, HTTP, auth, telemetry, etc.) - requires `server` feature
//! - **embedded**: Embedded-specific configuration (runtime settings) - requires `embedded` feature
//!
//! # Usage
//!
//! ## Server Mode
//!
//! ```rust,no_run
//! use graphdb::config::Config;
//!
//! // Load from file
//! let config = Config::load("config.toml").expect("Failed to load config");
//!
//! // Or create default
//! let config = Config::default();
//! ```
//!
//! ## Embedded Mode
//!
//! ```rust
//! use graphdb::config::{Config, EmbeddedConfig};
//!
//! let mut config = Config::default();
//! config.embedded.runtime.cache_size_mb = 128;
//! ```

pub mod common;
pub mod embedded;
pub mod server;

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub use common::*;
pub use embedded::*;
pub use server::*;

// Re-export commonly used types for backward compatibility
pub use common::database::DatabaseConfig;
pub use common::log::LogConfig;
pub use common::monitoring::{MonitoringConfig, SlowQueryLogConfig};
pub use common::optimizer::{OptimizerConfig, OptimizerRulesConfig};
pub use common::storage::{
    CompressionAlgorithm, QueryResourceConfig, StorageConfig, StorageEngine,
};
pub use common::transaction::TransactionConfig;

#[cfg(feature = "server")]
pub use server::auth::AuthConfig;
#[cfg(feature = "server")]
pub use server::bootstrap::BootstrapConfig;
#[cfg(feature = "server")]
pub use server::connection_pool::ConnectionPoolConfig;
#[cfg(feature = "server")]
pub use server::grpc::GrpcConfig;
#[cfg(feature = "server")]
pub use server::http::HttpServerConfig;
#[cfg(feature = "server")]
pub use server::security::{AuditConfig, PasswordPolicyConfig, SecurityConfig, SslConfig};
#[cfg(feature = "server")]
pub use server::telemetry::TelemetryConfig;

use crate::search::config::FulltextConfig;
use vector_client::VectorClientConfig;

/// Global configuration aggregator
///
/// This is the main configuration structure that combines all configuration sections.
/// Use [`Config::default()`] to create a default configuration, or [`Config::load()`] to load from a file.
///
/// # Examples
///
/// ```rust
/// use graphdb::config::Config;
///
/// // Create default configuration
/// let config = Config::default();
///
/// // Access configuration sections
/// println!("Database port: {}", config.common.database.port);
/// #[cfg(feature = "server")]
/// println!("gRPC enabled: {}", config.server.grpc.enabled);
/// ```
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    /// Common configuration (always available)
    #[serde(flatten)]
    pub common: CommonConfig,

    /// Server-specific configuration (only available with `server` feature)
    #[cfg(feature = "server")]
    #[serde(default)]
    pub server: ServerConfig,

    /// Embedded-specific configuration (only available with `embedded` feature)
    #[cfg(feature = "embedded")]
    #[serde(default)]
    pub embedded: EmbeddedConfig,

    /// Vector search configuration
    #[serde(default)]
    pub vector: VectorClientConfig,

    /// Fulltext search configuration
    #[serde(default)]
    pub fulltext: FulltextConfig,
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file (TOML format)
    ///
    /// # Returns
    ///
    /// * `Ok(Config)` - Successfully loaded configuration
    /// * `Err(Box<dyn Error>)` - Error reading or parsing the file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use graphdb::config::Config;
    ///
    /// let config = Config::load("config.toml").expect("Failed to load config");
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.common.database.storage_path =
            Config::resolve_storage_path(&config.common.database.storage_path)?;
        Ok(config)
    }

    /// Save configuration to file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the configuration file (TOML format)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use graphdb::config::Config;
    ///
    /// let config = Config::default();
    /// config.save("config.toml").expect("Failed to save config");
    /// ```
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Resolve storage path (supports relative paths and ~ expansion)
    fn resolve_storage_path(storage_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = PathBuf::from(storage_path);

        if path.is_absolute() {
            return Ok(storage_path.to_string());
        }

        if let Some(relative_path) = storage_path.strip_prefix('~') {
            if let Some(home_dir) = env::home_dir() {
                let absolute_path =
                    if relative_path.starts_with('/') || relative_path.starts_with('\\') {
                        home_dir.join(&relative_path[1..])
                    } else {
                        home_dir.join(relative_path)
                    };
                return Ok(absolute_path.to_string_lossy().into_owned());
            }
            return Err("Failed to get user home directory".into());
        }

        if let Ok(exe_path) = env::current_exe() {
            let exe_dir = exe_path
                .parent()
                .ok_or("Failed to get executable directory")?
                .to_path_buf();
            let absolute_path = exe_dir.join(&path);
            return Ok(absolute_path.to_string_lossy().into_owned());
        }

        Err("Failed to get executable path".into())
    }

    /// Validate all configurations
    pub fn validate(&self) -> Result<(), String> {
        self.common.validate()?;
        #[cfg(feature = "server")]
        self.server.validate()?;
        #[cfg(feature = "embedded")]
        self.embedded.validate()?;
        Ok(())
    }

    // ========== Convenience Methods ==========

    /// Get log level
    pub fn log_level(&self) -> &str {
        &self.common.log.level
    }

    /// Get log directory
    pub fn log_dir(&self) -> &str {
        &self.common.log.dir
    }

    /// Get log file name
    pub fn log_file(&self) -> &str {
        &self.common.log.file
    }

    /// Get host address
    pub fn host(&self) -> &str {
        &self.common.database.host
    }

    /// Get port
    pub fn port(&self) -> u16 {
        self.common.database.port
    }

    /// Get gRPC port (server mode only)
    #[cfg(feature = "server")]
    pub fn grpc_port(&self) -> u16 {
        self.server.grpc.port
    }

    /// Get gRPC configuration (server mode only)
    #[cfg(feature = "server")]
    pub fn grpc(&self) -> &GrpcConfig {
        &self.server.grpc
    }

    /// Check if gRPC is enabled (server mode only)
    #[cfg(feature = "server")]
    pub fn grpc_enabled(&self) -> bool {
        self.server.grpc.enabled
    }

    /// Get storage path
    pub fn storage_path(&self) -> &str {
        &self.common.database.storage_path
    }

    /// Get maximum connections
    pub fn max_connections(&self) -> usize {
        self.common.database.max_connections
    }

    /// Get transaction timeout
    pub fn transaction_timeout(&self) -> u64 {
        self.common.transaction.default_timeout
    }

    /// Get maximum concurrent transactions
    pub fn max_concurrent_transactions(&self) -> usize {
        self.common.transaction.max_concurrent_transactions
    }

    /// Get slow query log configuration
    pub fn slow_query_log(&self) -> &SlowQueryLogConfig {
        &self.common.monitoring.slow_query_log
    }

    /// Get slow query config for StatsManager
    pub fn to_slow_query_config(&self) -> crate::core::stats::SlowQueryConfig {
        self.common.monitoring.slow_query_log.to_slow_query_config()
    }

    /// Get storage configuration
    pub fn storage(&self) -> &StorageConfig {
        &self.common.storage
    }

    /// Get query resource configuration
    pub fn query_resource(&self) -> &QueryResourceConfig {
        &self.common.query_resource
    }
}

// ========== Backward Compatibility Field Access ==========
//
// These implementations provide backward compatibility by allowing
// direct field access like `config.database` instead of `config.common.database`

impl std::ops::Deref for Config {
    type Target = CommonConfig;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl std::ops::DerefMut for Config {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.common.database.host, "127.0.0.1");
        assert_eq!(config.common.database.port, 9758);
        assert_eq!(config.common.log.level, "info");
        assert_eq!(config.common.optimizer.max_iteration_rounds, 5);
        #[cfg(feature = "server")]
        assert_eq!(config.server.grpc.port, 9669);
        #[cfg(feature = "server")]
        assert!(config.server.grpc.enabled);
    }

    #[test]
    fn test_config_load_save() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");

        let config = Config::default();
        let toml_content =
            toml::to_string_pretty(&config).expect("Failed to serialize config to TOML");
        temp_file
            .write_all(toml_content.as_bytes())
            .expect("Failed to write TOML content to temporary file");

        let loaded_config =
            Config::load(temp_file.path()).expect("Failed to load config from temporary file");
        assert_eq!(
            config.common.database.host,
            loaded_config.common.database.host
        );
        assert_eq!(
            config.common.database.port,
            loaded_config.common.database.port
        );
        assert_eq!(config.common.log.level, loaded_config.common.log.level);
    }

    #[test]
    fn test_nested_config_load() {
        let config_content = r#"
[database]
host = "0.0.0.0"
port = 8080
storage_path = "/tmp/graphdb"
max_connections = 100

[transaction]
default_timeout = 60
max_concurrent_transactions = 500

[log]
level = "debug"
dir = "/var/log/graphdb"
file = "graphdb"
max_file_size = 104857600
max_files = 10

[storage]
engine = "propertygraph"
compression = "lz4"
compression_level = 5

[query_resource]
max_concurrent_queries = 50
max_memory_per_query = 1073741824
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
        temp_file
            .write_all(config_content.as_bytes())
            .expect("Failed to write config file");

        let config = Config::load(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.common.database.host, "0.0.0.0");
        assert_eq!(config.common.database.port, 8080);
        assert_eq!(config.common.transaction.default_timeout, 60);
        assert_eq!(config.common.transaction.max_concurrent_transactions, 500);
        assert_eq!(config.common.log.level, "debug");
        assert_eq!(config.common.storage.compression, CompressionAlgorithm::Lz4);
        assert_eq!(config.common.storage.compression_level, 5);
        assert_eq!(config.common.query_resource.max_concurrent_queries, 50);
    }

    #[test]
    fn test_config_validate() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_backward_compatibility() {
        let config = Config::default();
        // Test Deref implementation
        assert_eq!(config.database.host, "127.0.0.1");
        assert_eq!(config.port(), 9758);
        assert_eq!(config.storage_path(), "data/graphdb");
    }

    #[cfg(feature = "server")]
    #[test]
    fn test_server_config() {
        let config = Config::default();
        assert!(config.server.grpc.enabled);
        assert!(config.server.http.enabled);
        assert!(config.server.auth.enable_authorize);
        assert_eq!(config.server.grpc.port, 9669);
        assert_eq!(config.server.http.port, 9758);
    }

    #[cfg(feature = "embedded")]
    #[test]
    fn test_embedded_config() {
        let config = Config::default();
        assert!(config.embedded.runtime.is_memory());
        assert_eq!(config.embedded.runtime.cache_size_mb, 64);
    }
}
