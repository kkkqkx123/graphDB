//! Storage configuration type
//!
//! Define configuration options for various storage backends

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// storage type
    pub storage_type: StorageType,
    /// Basic path (file storage usage)
    pub base_path: Option<PathBuf>,
    /// Redis connection string (Redis storage used)
    pub redis_url: Option<String>,
    /// Whether to enable WAL
    pub enable_wal: bool,
    /// WAL Directory
    pub wal_dir: Option<PathBuf>,
    /// Cache size (bytes)
    pub cache_size: usize,
    /// refresh interval
    pub flush_interval: Duration,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_type: StorageType::File,
            base_path: None,
            redis_url: None,
            enable_wal: false,
            wal_dir: None,
            cache_size: 1024 * 1024 * 100, // 100MB
            flush_interval: Duration::from_secs(60),
        }
    }
}

/// Storage type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    /// file storage
    File,
    /// Redis storage
    Redis,
    /// Memory storage (for testing)
    Memory,
    /// WAL Storage
    WAL,
    /// hot and cold cache storage
    ColdWarmCache,
}

impl std::fmt::Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageType::File => write!(f, "file"),
            StorageType::Redis => write!(f, "redis"),
            StorageType::Memory => write!(f, "memory"),
            StorageType::WAL => write!(f, "wal"),
            StorageType::ColdWarmCache => write!(f, "cold_warm_cache"),
        }
    }
}
