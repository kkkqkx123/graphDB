use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::search::adapters::{Bm25Config, InversearchConfig};
use crate::search::engine::EngineType;
use crate::sync::SyncMode;

/// 同步失败策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncFailurePolicy {
    /// 失败时记录日志但允许事务提交（默认）
    FailOpen,
    /// 失败时回滚事务
    FailClosed,
}

impl Default for SyncFailurePolicy {
    fn default() -> Self {
        SyncFailurePolicy::FailOpen
    }
}

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
    /// 同步失败时的处理策略
    #[serde(default)]
    pub failure_policy: SyncFailurePolicy,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            mode: SyncMode::Async,
            queue_size: 10000,
            commit_interval_ms: 1000,
            batch_size: 100,
            failure_policy: SyncFailurePolicy::FailOpen,
        }
    }
}
