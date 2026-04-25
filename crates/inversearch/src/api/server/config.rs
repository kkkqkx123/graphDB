// Server configuration - only compiled when "service" feature is enabled
#![cfg(feature = "service")]

use crate::config::{
    CacheConfig, Config, IndexConfig, LoggingConfig, ServerConfig as AppConfig, StorageConfig,
};

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub server: AppConfig,
    pub index: IndexConfig,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            server: AppConfig::default(),
            index: IndexConfig::default(),
            cache: CacheConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl ServiceConfig {
    /// Create a new service configuration
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            server: AppConfig {
                host: host.into(),
                port,
                workers: 4,
            },
            index: IndexConfig::default(),
            cache: CacheConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
        }
    }

    /// Load configuration from file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let config = Config::from_file(path)?;
        Ok(Self {
            server: config.server,
            index: config.index,
            cache: config.cache,
            storage: config.storage,
            logging: config.logging,
        })
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();

        // Override with environment variables
        if let Ok(host) = std::env::var("INVSEARCH_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("INVSEARCH_PORT") {
            config.server.port = port.parse()?;
        }
        if let Ok(workers) = std::env::var("INVSEARCH_WORKERS") {
            config.server.workers = workers.parse()?;
        }

        Ok(Self {
            server: config.server,
            index: config.index,
            cache: config.cache,
            storage: config.storage,
            logging: config.logging,
        })
    }

    /// Load configuration from file with environment variable overrides
    pub fn from_file_with_env_override(path: &str) -> anyhow::Result<Self> {
        let mut config = Self::from_file(path)?;

        // Override with environment variables
        if let Ok(host) = std::env::var("INVSEARCH_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("INVSEARCH_PORT") {
            config.server.port = port.parse()?;
        }
        if let Ok(workers) = std::env::var("INVSEARCH_WORKERS") {
            config.server.workers = workers.parse()?;
        }

        Ok(config)
    }
}

/// Server configuration (alias for ServiceConfig)
pub type ServerConfig = ServiceConfig;
