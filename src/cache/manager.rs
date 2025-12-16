//! 缓存管理器
//!
//! 提供全局缓存的管理和协调功能

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use super::traits::*;
use super::config::*;
use super::implementations::*;

/// 全局缓存管理器
#[derive(Debug)]
pub struct CacheManager {
    caches: RwLock<HashMap<String, Box<dyn CacheEraser>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        Self {
            caches: RwLock::new(HashMap::new()),
            stats: Arc::new(RwLock::new(CacheStats::new())),
            config,
        }
    }
    
    /// 注册缓存实例
    pub fn register_cache<K, V>(&self, name: &str, cache: Box<dyn Cache<K, V>>)
    where
        K: 'static + Send + Sync,
        V: 'static + Send + Sync,
    {
        let mut caches = self.caches.write().unwrap();
        caches.insert(name.to_string(), cache);
    }
    
    /// 获取缓存实例
    pub fn get_cache<K, V>(&self, name: &str) -> Option<Arc<dyn Cache<K, V>>>
    where
        K: 'static + Send + Sync,
        V: 'static + Send + Sync + Clone,
    {
        // 简化实现，直接返回None
        // 实际实现需要更复杂的类型擦除机制
        None
    }
    
    /// 创建LRU缓存
    pub fn create_lru_cache<K, V>(&self, capacity: usize) -> Arc<dyn Cache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(LruCache::new(capacity))
    }
    
    /// 创建LFU缓存
    pub fn create_lfu_cache<K, V>(&self, capacity: usize) -> Arc<dyn Cache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(LfuCache::new(capacity))
    }
    
    /// 创建TTL缓存
    pub fn create_ttl_cache<K, V>(&self, capacity: usize, default_ttl: Duration) -> Arc<dyn Cache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(TtlCache::new(capacity, default_ttl))
    }
    
    /// 创建带统计的缓存
    pub fn create_stats_cache<K, V>(&self, cache: Arc<dyn Cache<K, V>>) -> Arc<dyn StatsCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(StatsCacheWrapper::new(cache))
    }
    
    /// 获取配置
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
    
    /// 获取统计信息
    pub fn stats(&self) -> Arc<RwLock<CacheStats>> {
        self.stats.clone()
    }
    
    /// 清空所有缓存
    pub fn clear_all(&self) {
        let caches = self.caches.read().unwrap();
        // 简化实现，实际需要类型擦除的清理方法
        let _ = caches;
    }
    
    /// 获取缓存列表
    pub fn cache_names(&self) -> Vec<String> {
        let caches = self.caches.read().unwrap();
        caches.keys().cloned().collect()
    }
    
    /// 检查缓存是否存在
    pub fn has_cache(&self, name: &str) -> bool {
        let caches = self.caches.read().unwrap();
        caches.contains_key(name)
    }
    
    /// 移除缓存
    pub fn remove_cache(&self, name: &str) -> bool {
        let mut caches = self.caches.write().unwrap();
        caches.remove(name).is_some()
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub total_operations: u64,
    pub memory_usage: usize,
    pub cache_count: usize,
}

impl CacheStats {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn hit_rate(&self) -> f64 {
        if self.total_hits + self.total_misses == 0 {
            0.0
        } else {
            self.total_hits as f64 / (self.total_hits + self.total_misses) as f64
        }
    }
    
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    pub fn merge(&mut self, other: &CacheStats) {
        self.total_hits += other.total_hits;
        self.total_misses += other.total_misses;
        self.total_evictions += other.total_evictions;
        self.total_operations += other.total_operations;
        self.memory_usage += other.memory_usage;
        self.cache_count += other.cache_count;
    }
}

/// 全局缓存管理器实例
static GLOBAL_CACHE_MANAGER: once_cell::sync::Lazy<Arc<CacheManager>> = 
    once_cell::sync::Lazy::new(|| {
        Arc::new(CacheManager::new(CacheConfig::default()))
    });

/// 获取全局缓存管理器
pub fn global_cache_manager() -> Arc<CacheManager> {
    GLOBAL_CACHE_MANAGER.clone()
}

/// 初始化全局缓存管理器
pub fn init_global_cache_manager(config: CacheConfig) -> Result<(), String> {
    config.validate()?;
    
    // 注意：这里需要重新初始化全局实例
    // 由于once_cell的限制，这里只是验证配置
    // 实际应用中可能需要使用其他方法
    
    Ok(())
}

/// 缓存构建器
pub struct CacheBuilder<K, V> {
    capacity: usize,
    ttl: Option<Duration>,
    policy: CachePolicy,
    collect_stats: bool,
    _phantom: std::marker::PhantomData<(K, V)>,
}

impl<K, V> CacheBuilder<K, V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            ttl: None,
            policy: CachePolicy::LRU,
            collect_stats: false,
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self.policy = CachePolicy::TTL(ttl);
        self
    }
    
    pub fn with_policy(mut self, policy: CachePolicy) -> Self {
        self.policy = policy;
        self
    }
    
    pub fn with_stats(mut self, collect_stats: bool) -> Self {
        self.collect_stats = collect_stats;
        self
    }
    
    pub fn build(self) -> Arc<dyn Cache<K, V>> {
        let cache: Arc<dyn Cache<K, V>> = match self.policy {
            CachePolicy::LRU => Arc::new(LruCache::new(self.capacity)),
            CachePolicy::LFU => Arc::new(LfuCache::new(self.capacity)),
            CachePolicy::TTL(ttl) => Arc::new(TtlCache::new(self.capacity, ttl)),
            CachePolicy::FIFO => Arc::new(FifoCache::new(self.capacity)),
            CachePolicy::Adaptive => Arc::new(AdaptiveCache::new(self.capacity)),
            CachePolicy::None => Arc::new(UnboundedCache::new()),
        };
        
        if self.collect_stats {
            Arc::new(StatsCacheWrapper::new(cache))
        } else {
            cache
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_cache_manager_creation() {
        let config = CacheConfig::default();
        let manager = CacheManager::new(config);
        
        assert_eq!(manager.cache_names().len(), 0);
        assert!(!manager.has_cache("test"));
    }

    #[test]
    fn test_cache_builder() {
        let cache: Arc<dyn Cache<String, String>> = CacheBuilder::new(100)
            .with_ttl(Duration::from_secs(60))
            .with_stats(true)
            .build();
        
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        assert_eq!(stats.hit_rate(), 0.0);
        
        stats.total_hits = 80;
        stats.total_misses = 20;
        assert_eq!(stats.hit_rate(), 0.8);
        
        stats.reset();
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_global_cache_manager() {
        let manager = global_cache_manager();
        assert!(manager.config().enabled);
    }

    #[test]
    fn test_cache_registration() {
        let manager = CacheManager::new(CacheConfig::default());
        let cache: Arc<dyn Cache<String, String>> = CacheBuilder::new(100).build();
        
        // 注意：这里需要解决类型擦除的问题
        // 暂时跳过具体实现
    }
}