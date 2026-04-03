use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    #[cfg(feature = "store-redis")]
    pub redis: Option<RedisConfig>,
    #[cfg(feature = "store-file")]
    pub file: Option<FileStorageConfig>,
    #[cfg(feature = "store-wal")]
    pub wal: Option<WALConfig>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig {
            enabled: false,
            backend: StorageBackend::Memory,
            #[cfg(feature = "store-redis")]
            redis: None,
            #[cfg(feature = "store-file")]
            file: None,
            #[cfg(feature = "store-wal")]
            wal: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Memory,
    #[cfg(feature = "store-file")]
    File,
    #[cfg(feature = "store-redis")]
    Redis,
    #[cfg(feature = "store-wal")]
    Wal,
}

#[cfg(feature = "store-redis")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
}

#[cfg(feature = "store-redis")]
impl Default for RedisConfig {
    fn default() -> Self {
        RedisConfig {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
        }
    }
}

#[cfg(feature = "store-file")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageConfig {
    pub base_path: String,
    pub auto_save: bool,
    pub save_interval_secs: u64,
}

#[cfg(feature = "store-file")]
impl Default for FileStorageConfig {
    fn default() -> Self {
        FileStorageConfig {
            base_path: "./data".to_string(),
            auto_save: true,
            save_interval_secs: 60,
        }
    }
}

#[cfg(feature = "store-wal")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WALConfig {
    pub base_path: String,
    pub max_wal_size: usize,
    pub compression: bool,
    pub snapshot_interval: usize,
}

#[cfg(feature = "store-wal")]
impl Default for WALConfig {
    fn default() -> Self {
        WALConfig {
            base_path: "./wal".to_string(),
            max_wal_size: 100 * 1024 * 1024,
            compression: true,
            snapshot_interval: 1000,
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

        #[cfg(feature = "store-redis")]
        if let Ok(redis_url) = std::env::var("INVERSEARCH_REDIS_URL") {
            if config.storage.redis.is_none() {
                config.storage.redis = Some(RedisConfig::default());
            }
            if let Some(redis_config) = config.storage.redis.as_mut() {
                redis_config.url = redis_url;
            }
        }

        #[cfg(feature = "store-file")]
        if let Ok(file_path) = std::env::var("INVERSEARCH_FILE_PATH") {
            if config.storage.file.is_none() {
                config.storage.file = Some(FileStorageConfig::default());
            }
            if let Some(file_config) = config.storage.file.as_mut() {
                file_config.base_path = file_path;
            }
        }

        #[cfg(feature = "store-wal")]
        if let Ok(wal_path) = std::env::var("INVERSEARCH_WAL_PATH") {
            if config.storage.wal.is_none() {
                config.storage.wal = Some(WALConfig::default());
            }
            if let Some(wal_config) = config.storage.wal.as_mut() {
                wal_config.base_path = wal_path;
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
        assert!(!config.enabled);
        assert_eq!(config.size, 1000);
    }

    #[test]
    fn test_storage_config_default() {
        let config = StorageConfig::default();
        assert!(!config.enabled);
        assert!(matches!(config.backend, StorageBackend::Memory));
    }

    #[cfg(feature = "store-redis")]
    #[test]
    fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert_eq!(config.url, "redis://127.0.0.1:6379");
        assert_eq!(config.pool_size, 10);
    }

    #[cfg(feature = "store-file")]
    #[test]
    fn test_file_storage_config_default() {
        let config = FileStorageConfig::default();
        assert_eq!(config.base_path, "./data");
        assert!(config.auto_save);
        assert_eq!(config.save_interval_secs, 60);
    }

    #[cfg(feature = "store-wal")]
    #[test]
    fn test_wal_config_default() {
        let config = WALConfig::default();
        assert_eq!(config.base_path, "./wal");
        assert_eq!(config.max_wal_size, 100 * 1024 * 1024);
        assert!(config.compression);
        assert_eq!(config.snapshot_interval, 1000);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "json");
    }
}
