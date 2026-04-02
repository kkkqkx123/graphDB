use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub index: IndexConfig,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 50051,
            workers: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub resolution: usize,
    pub tokenize: String,
    pub depth: usize,
    pub bidirectional: bool,
    pub fastupdate: bool,
    pub keystore: Option<usize>,
}

impl Default for IndexConfig {
    fn default() -> Self {
        IndexConfig {
            resolution: 9,
            tokenize: "strict".to_string(),
            depth: 0,
            bidirectional: true,
            fastupdate: false,
            keystore: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub size: usize,
    pub ttl: Option<u64>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            enabled: false,
            size: 1000,
            ttl: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub enabled: bool,
    pub backend: StorageBackend,
    pub redis: Option<RedisConfig>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            enabled: false,
            backend: StorageBackend::Memory,
            redis: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Memory,
    Redis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
}

impl Default for RedisConfig {
    fn default() -> Self {
        RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig::default(),
            index: IndexConfig::default(),
            cache: CacheConfig::default(),
            storage: StorageConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let mut config = Config::default();

        if let Ok(host) = std::env::var("INVERSEARCH_HOST") {
            config.server.host = host;
        }

        if let Ok(port) = std::env::var("INVERSEARCH_PORT") {
            config.server.port = port.parse()?;
        }

        if let Ok(redis_url) = std::env::var("INVERSEARCH_REDIS_URL") {
            if config.storage.redis.is_none() {
                config.storage.redis = Some(RedisConfig::default());
            }
            if let Some(redis_config) = config.storage.redis.as_mut() {
                redis_config.url = redis_url;
            }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 50051);
        assert_eq!(config.index.resolution, 9);
    }

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 50051);
        assert_eq!(config.workers, 4);
    }

    #[test]
    fn test_index_config_default() {
        let config = IndexConfig::default();
        assert_eq!(config.resolution, 9);
        assert_eq!(config.tokenize, "strict");
        assert_eq!(config.depth, 0);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.enabled, false);
        assert_eq!(config.size, 1000);
    }

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert_eq!(config.enabled, false);
        assert!(matches!(config.backend, StorageBackend::Memory));
    }

    #[test]
    fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert_eq!(config.url, "redis://127.0.0.1:6379");
        assert_eq!(config.pool_size, 10);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "json");
    }
}
