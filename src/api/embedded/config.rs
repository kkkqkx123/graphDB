//! 配置管理模块
//!
//! 提供嵌入式数据库的配置管理，从 embedded_api.rs 分离出来

use std::path::{Path, PathBuf};
use std::time::Duration;

/// 数据库配置
///
/// 用于配置嵌入式 GraphDB 数据库的行为
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::DatabaseConfig;
///
/// // 内存数据库配置
/// let config = DatabaseConfig::memory();
///
/// // 文件数据库配置
/// let config = DatabaseConfig::file("/path/to/db");
///
/// // 链式配置
/// let config = DatabaseConfig::memory()
///     .with_cache_size(128)
///     .with_timeout(Duration::from_secs(60));
/// ```
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// 数据库路径，None 表示内存模式
    pub path: Option<PathBuf>,
    /// 缓存大小（MB）
    pub cache_size_mb: usize,
    /// 默认超时
    pub default_timeout: Duration,
    /// 是否启用 WAL（Write-Ahead Logging）
    pub enable_wal: bool,
    /// 同步模式
    pub sync_mode: SyncMode,
}

/// 同步模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// 完全同步，每次写入都同步到磁盘（最安全，最慢）
    Full,
    /// 正常同步，定期同步（平衡）
    Normal,
    /// 异步模式，由操作系统决定何时同步（最快，有风险）
    Off,
}

impl DatabaseConfig {
    /// 创建内存数据库配置
    pub fn memory() -> Self {
        Self {
            path: None,
            cache_size_mb: 64,
            default_timeout: Duration::from_secs(30),
            enable_wal: true,
            sync_mode: SyncMode::Normal,
        }
    }

    /// 创建文件数据库配置
    pub fn file(path: impl AsRef<Path>) -> Self {
        Self {
            path: Some(path.as_ref().to_path_buf()),
            cache_size_mb: 64,
            default_timeout: Duration::from_secs(30),
            enable_wal: true,
            sync_mode: SyncMode::Normal,
        }
    }

    /// 使用路径创建配置（便捷方法）
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self::file(path)
    }

    /// 设置缓存大小
    pub fn with_cache_size(mut self, size_mb: usize) -> Self {
        self.cache_size_mb = size_mb;
        self
    }

    /// 设置默认超时
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// 设置是否启用 WAL
    pub fn with_wal(mut self, enable: bool) -> Self {
        self.enable_wal = enable;
        self
    }

    /// 设置同步模式
    pub fn with_sync_mode(mut self, mode: SyncMode) -> Self {
        self.sync_mode = mode;
        self
    }

    /// 检查是否为内存模式
    pub fn is_memory(&self) -> bool {
        self.path.is_none()
    }

    /// 获取数据库路径
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// 获取缓存大小（字节）
    pub fn cache_size_bytes(&self) -> usize {
        self.cache_size_mb * 1024 * 1024
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::memory()
    }
}

impl Default for SyncMode {
    fn default() -> Self {
        SyncMode::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert!(config.is_memory());
        assert_eq!(config.cache_size_mb, 64);
        assert_eq!(config.default_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_memory_config() {
        let config = DatabaseConfig::memory();
        assert!(config.is_memory());
        assert!(config.path.is_none());
    }

    #[test]
    fn test_file_config() {
        let config = DatabaseConfig::file("/tmp/test.db");
        assert!(!config.is_memory());
        assert_eq!(config.path(), Some(Path::new("/tmp/test.db")));
    }

    #[test]
    fn test_chain_config() {
        let config = DatabaseConfig::memory()
            .with_cache_size(128)
            .with_timeout(Duration::from_secs(60))
            .with_wal(false)
            .with_sync_mode(SyncMode::Full);

        assert_eq!(config.cache_size_mb, 128);
        assert_eq!(config.default_timeout, Duration::from_secs(60));
        assert!(!config.enable_wal);
        assert_eq!(config.sync_mode, SyncMode::Full);
    }

    #[test]
    fn test_cache_size_bytes() {
        let config = DatabaseConfig::memory().with_cache_size(64);
        assert_eq!(config.cache_size_bytes(), 64 * 1024 * 1024);
    }

    #[test]
    fn test_sync_mode_default() {
        let mode = SyncMode::default();
        assert_eq!(mode, SyncMode::Normal);
    }
}
