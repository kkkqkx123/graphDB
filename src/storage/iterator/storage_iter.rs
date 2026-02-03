//! 存储迭代器 - 提供与存储引擎交互的迭代接口
//!
//! 对应原C++中的StorageIterator
//! 提供：
//! - StorageIterator: 存储引擎迭代器接口
//! - VertexIter: 顶点迭代器
//! - EdgeIter: 边迭代器
//! - PropIter: 属性迭代器

use crate::core::StorageError;
use crate::storage::engine::StorageIterator;

/// 迭代器错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum IterError {
    InvalidState(String),
    IoError(String),
    SchemaMismatch(String),
    NotFound(String),
}

impl std::fmt::Display for IterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IterError::InvalidState(msg) => write!(f, "Invalid iterator state: {}", msg),
            IterError::IoError(msg) => write!(f, "IO error: {}", msg),
            IterError::SchemaMismatch(msg) => write!(f, "Schema mismatch: {}", msg),
            IterError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for IterError {}

impl From<IterError> for StorageError {
    fn from(err: IterError) -> Self {
        StorageError::DbError(err.to_string())
    }
}

/// 迭代器统计信息
#[derive(Debug, Clone, Default)]
pub struct IterStats {
    pub items_scanned: u64,
    pub items_returned: u64,
    pub seek_operations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl IterStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_scan(&mut self) {
        self.items_scanned += 1;
    }

    pub fn record_return(&mut self) {
        self.items_returned += 1;
    }

    pub fn record_seek(&mut self) {
        self.seek_operations += 1;
    }

    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }
}

/// 迭代器配置
#[derive(Debug, Clone)]
pub struct IterConfig {
    pub prefetch_size: usize,
    pub use_cache: bool,
    pub cache_size: usize,
    pub parallel_scan: bool,
}

impl Default for IterConfig {
    fn default() -> Self {
        Self {
            prefetch_size: 100,
            use_cache: true,
            cache_size: 10000,
            parallel_scan: false,
        }
    }
}

#[derive(Debug)]
pub struct VecPairIterator {
    pub keys: Vec<Vec<u8>>,
    pub values: Vec<Vec<u8>>,
    pub index: usize,
    pub current_index: Option<usize>,
}

impl VecPairIterator {
    pub fn new(keys: Vec<Vec<u8>>, values: Vec<Vec<u8>>) -> Self {
        Self { keys, values, index: 0, current_index: None }
    }
}

impl StorageIterator for VecPairIterator {
    fn key(&self) -> Option<&[u8]> {
        self.current_index.and_then(|i| self.keys.get(i).map(|v| v.as_slice()))
    }

    fn value(&self) -> Option<&[u8]> {
        self.current_index.and_then(|i| self.values.get(i).map(|v| v.as_slice()))
    }

    fn next(&mut self) -> bool {
        if self.index < self.keys.len() {
            let current_index = self.index;
            self.index += 1;
            self.current_index = Some(current_index);
            true
        } else {
            self.current_index = None;
            false
        }
    }

    fn estimate_remaining(&self) -> Option<usize> {
        Some(self.keys.len().saturating_sub(self.index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_stats() {
        let mut stats = IterStats::new();
        assert_eq!(stats.items_scanned, 0);
        assert_eq!(stats.items_returned, 0);

        stats.record_scan();
        stats.record_scan();
        assert_eq!(stats.items_scanned, 2);

        stats.record_return();
        assert_eq!(stats.items_returned, 1);
    }

    #[test]
    fn test_iter_config() {
        let config = IterConfig::default();
        assert_eq!(config.prefetch_size, 100);
        assert!(config.use_cache);
        assert_eq!(config.cache_size, 10000);
        assert!(!config.parallel_scan);
    }

    #[test]
    fn test_iter_error_display() {
        let err = IterError::InvalidState("test".to_string());
        assert_eq!(format!("{}", err), "Invalid iterator state: test");

        let err = IterError::NotFound("key".to_string());
        assert_eq!(format!("{}", err), "Not found: key");
    }
}
