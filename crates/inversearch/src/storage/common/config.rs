//! 存储配置类型
//!
//! 定义各种存储后端的配置选项

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储类型
    pub storage_type: StorageType,
    /// 基础路径（文件存储使用）
    pub base_path: Option<PathBuf>,
    /// Redis 连接字符串（Redis 存储使用）
    pub redis_url: Option<String>,
    /// 是否启用 WAL
    pub enable_wal: bool,
    /// WAL 目录
    pub wal_dir: Option<PathBuf>,
    /// 缓存大小（字节）
    pub cache_size: usize,
    /// 刷新间隔
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

/// 存储类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageType {
    /// 文件存储
    File,
    /// Redis 存储
    Redis,
    /// 内存存储（测试用）
    Memory,
    /// WAL 存储
    WAL,
    /// 冷热缓存存储
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
