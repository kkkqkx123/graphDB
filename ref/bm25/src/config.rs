use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub index: IndexConfig,
    pub cache: CacheConfig,
    pub bm25: Bm25Config,
    pub search: SearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: SocketAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub data_dir: String,
    pub index_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_env() -> anyhow::Result<Self> {
        let server_address = std::env::var("SERVER_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:50051".to_string());
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let data_dir = std::env::var("DATA_DIR")
            .unwrap_or_else(|_| "./data".to_string());
        let index_path = std::env::var("INDEX_PATH")
            .unwrap_or_else(|_| "./index".to_string());

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

        Ok(Config {
            server: ServerConfig {
                address: server_address.parse()?,
            },
            redis: RedisConfig {
                url: redis_url,
                pool_size: 10,
            },
            index: IndexConfig {
                data_dir,
                index_path,
            },
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 3600,
                max_size: 10000,
            },
            bm25: bm25_config,
            search: search_config,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                address: "0.0.0.0:50051".parse()
                    .expect("Failed to parse default server address"),
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
            },
            index: IndexConfig {
                data_dir: "./data".to_string(),
                index_path: "./index".to_string(),
            },
            cache: CacheConfig {
                enabled: true,
                ttl_seconds: 3600,
                max_size: 10000,
            },
            bm25: Bm25Config::default(),
            search: SearchConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Config {
    pub k1: f32,
    pub b: f32,
    pub avg_doc_length: f32,
    pub field_weights: FieldWeights,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldWeights {
    pub title: f32,
    pub content: f32,
}

impl Default for FieldWeights {
    fn default() -> Self {
        FieldWeights {
            title: 2.0,
            content: 1.0,
        }
    }
}

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
