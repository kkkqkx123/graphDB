use crate::config::{Bm25Config, SearchConfig, StorageConfig};
use crate::api::core::IndexManagerConfig;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub index: IndexConfig,
    pub bm25: Bm25Config,
    pub search: SearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: SocketAddr,
}

/// Index configuration with Tantivy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub data_dir: String,
    pub index_path: String,
    /// Tantivy index manager configuration
    #[serde(default)]
    pub manager: IndexManagerConfig,
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let server_address =
            std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "0.0.0.0:50051".to_string());
        let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
        let index_path = std::env::var("INDEX_PATH").unwrap_or_else(|_| "./index".to_string());

        // Index manager config from env using new loader
        let manager_config = IndexManagerConfig::from_env_with_prefix("INDEX_")
            .unwrap_or_else(|_| IndexManagerConfig::default());

        // BM25 config from env
        let mut bm25_config = Bm25Config::default();
        if let Ok(k1) = std::env::var("BM25_K1") {
            bm25_config.k1 = k1.parse().unwrap_or(1.2);
        }
        if let Ok(b) = std::env::var("BM25_B") {
            bm25_config.b = b.parse().unwrap_or(0.75);
        }
        if let Ok(avg_doc_length) = std::env::var("BM25_AVG_DOC_LENGTH") {
            bm25_config.avg_doc_length = avg_doc_length.parse().unwrap_or(100.0);
        }
        if let Ok(title_weight) = std::env::var("BM25_TITLE_WEIGHT") {
            bm25_config.field_weights.title = title_weight.parse().unwrap_or(2.0);
        }
        if let Ok(content_weight) = std::env::var("BM25_CONTENT_WEIGHT") {
            bm25_config.field_weights.content = content_weight.parse().unwrap_or(1.0);
        }

        // Search config from env
        let mut search_config = SearchConfig::default();
        if let Ok(default_limit) = std::env::var("SEARCH_DEFAULT_LIMIT") {
            search_config.default_limit = default_limit.parse().unwrap_or(10);
        }
        if let Ok(max_limit) = std::env::var("SEARCH_MAX_LIMIT") {
            search_config.max_limit = max_limit.parse().unwrap_or(100);
        }
        if let Ok(enable_highlight) = std::env::var("SEARCH_ENABLE_HIGHLIGHT") {
            search_config.enable_highlight = enable_highlight.parse().unwrap_or(true);
        }
        if let Ok(highlight_fragment_size) = std::env::var("SEARCH_HIGHLIGHT_FRAGMENT_SIZE") {
            search_config.highlight_fragment_size = highlight_fragment_size.parse().unwrap_or(200);
        }

        let config = Config {
            server: ServerConfig {
                address: server_address.parse()?,
            },
            storage: StorageConfig::default(),
            index: IndexConfig {
                data_dir,
                index_path,
                manager: manager_config,
            },
            bm25: bm25_config,
            search: search_config,
        };

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.bm25.k1 <= 0.0 {
            anyhow::bail!("BM25 k1 parameter must be positive");
        }
        if self.bm25.b < 0.0 || self.bm25.b > 1.0 {
            anyhow::bail!("BM25 b parameter must be between 0 and 1");
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                address: "0.0.0.0:50051".parse().unwrap(),
            },
            storage: StorageConfig::default(),
            index: IndexConfig {
                data_dir: "./data".to_string(),
                index_path: "./index".to_string(),
                manager: IndexManagerConfig::default(),
            },
            bm25: Bm25Config::default(),
            search: SearchConfig::default(),
        }
    }
}
