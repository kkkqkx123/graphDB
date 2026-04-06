//! 冷热缓存配置

use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ColdWarmCacheConfig {
    pub hot_cache_max_size: usize,
    pub warm_cache_max_size: usize,
    pub warm_cache_mmap_enabled: bool,
    pub cold_storage_path: PathBuf,
    pub cold_storage_compression: bool,
    pub cold_storage_compression_level: i32,
    pub wal_enabled: bool,
    pub wal_path: PathBuf,
    pub wal_max_size: usize,
    pub wal_max_files: usize,
    pub wal_flush_interval: Duration,
    pub wal_auto_rotate: bool,
    pub flush_interval: Duration,
    pub merge_interval: Duration,
    pub cleanup_interval: Duration,
    pub checkpoint_interval: Duration,
    pub write_buffer_size: usize,
    pub read_ahead_enabled: bool,
    pub pre_fetch_enabled: bool,
}

impl Default for ColdWarmCacheConfig {
    fn default() -> Self {
        Self {
            hot_cache_max_size: 500 * 1024 * 1024,
            warm_cache_max_size: 2 * 1024 * 1024 * 1024,
            warm_cache_mmap_enabled: true,
            cold_storage_path: PathBuf::from("./data/cold"),
            cold_storage_compression: true,
            cold_storage_compression_level: 3,
            wal_enabled: true,
            wal_path: PathBuf::from("./data/wal"),
            wal_max_size: 100 * 1024 * 1024,
            wal_max_files: 10,
            wal_flush_interval: Duration::from_millis(100),
            wal_auto_rotate: true,
            flush_interval: Duration::from_secs(10),
            merge_interval: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(3600),
            checkpoint_interval: Duration::from_secs(300),
            write_buffer_size: 4 * 1024 * 1024,
            read_ahead_enabled: true,
            pre_fetch_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WALConfig {
    pub base_path: PathBuf,
    pub max_wal_size: usize,
    pub max_wal_files: usize,
    pub flush_interval: Duration,
    pub auto_rotate: bool,
    pub compression: bool,
    pub compression_level: i32,
}

impl Default for WALConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./data/wal"),
            max_wal_size: 100 * 1024 * 1024,
            max_wal_files: 10,
            flush_interval: Duration::from_millis(100),
            auto_rotate: true,
            compression: true,
            compression_level: 3,
        }
    }
}
