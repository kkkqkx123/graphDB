use crate::search::adapters::InversearchConfig;
use crate::search::engine::EngineType;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextConfig {
    pub enabled: bool,
    pub default_engine: EngineType,
    pub index_path: PathBuf,
    pub sync: SyncConfig,
    pub bm25: Bm25Config,
    pub inversearch: InversearchConfig,
    pub cache_size: usize,
    pub max_result_cache: usize,
    pub result_cache_ttl_secs: u64,
}

impl Default for FulltextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_engine: EngineType::Bm25,
            index_path: PathBuf::from("data/fulltext"),
            sync: SyncConfig::default(),
            bm25: Bm25Config::default(),
            inversearch: InversearchConfig::default(),
            cache_size: 100,
            max_result_cache: 1000,
            result_cache_ttl_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub mode: SyncMode,
    pub queue_size: usize,
    pub commit_interval_ms: u64,
    pub batch_size: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: SyncMode::Async,
            queue_size: 10000,
            commit_interval_ms: 1000,
            batch_size: 100,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    Sync,
    Async,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25Config {
    pub memory_limit_mb: usize,
    pub auto_commit: bool,
}

impl Default for Bm25Config {
    fn default() -> Self {
        Self {
            memory_limit_mb: 50,
            auto_commit: true,
        }
    }
}
