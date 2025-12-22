//! 缓存管理器
//!
//! 提供全局缓存的管理和协调功能

use super::cache_impl::*;
use super::config::*;
use super::traits::*;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// 缓存类型枚举 - 避免动态分发
#[derive(Debug)]
pub enum CacheType<K, V> {
    Lru(Arc<ConcurrentLruCache<K, V>>),
    Lfu(Arc<ConcurrentLfuCache<K, V>>),
    Ttl(Arc<ConcurrentTtlCache<K, V>>),
    Fifo(Arc<ConcurrentFifoCache<K, V>>),
    Adaptive(Arc<AdaptiveCache<K, V>>),
    Unbounded(Arc<ConcurrentUnboundedCache<K, V>>),
}

/// 统计缓存类型枚举 - 避免动态分发
#[derive(Debug)]
pub enum StatsCacheType<K, V> {
    Lru(Arc<StatsCacheWrapper<K, V, ConcurrentLruCache<K, V>>>),
    Lfu(Arc<StatsCacheWrapper<K, V, ConcurrentLfuCache<K, V>>>),
    Ttl(Arc<StatsCacheWrapper<K, V, ConcurrentTtlCache<K, V>>>),
    Fifo(Arc<StatsCacheWrapper<K, V, ConcurrentFifoCache<K, V>>>),
    Adaptive(Arc<StatsCacheWrapper<K, V, AdaptiveCache<K, V>>>),
    Unbounded(Arc<StatsCacheWrapper<K, V, ConcurrentUnboundedCache<K, V>>>),
}

/// 类型擦除的缓存 trait - 仅用于全局缓存管理器
trait AnyCache: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clear(&self);
    fn len(&self) -> usize;
}

impl<K, V> AnyCache for ConcurrentLruCache<K, V>
where
    K: Send + Sync + 'static + Eq + Hash + Clone,
    V: Send + Sync + 'static + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clear(&self) {
        Cache::clear(self)
    }

    fn len(&self) -> usize {
        Cache::len(self)
    }
}

impl<K, V> AnyCache for ConcurrentLfuCache<K, V>
where
    K: Send + Sync + 'static + Eq + Hash + Clone,
    V: Send + Sync + 'static + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clear(&self) {
        Cache::clear(self)
    }

    fn len(&self) -> usize {
        Cache::len(self)
    }
}

impl<K, V> AnyCache for ConcurrentTtlCache<K, V>
where
    K: Send + Sync + 'static + Eq + Hash + Clone,
    V: Send + Sync + 'static + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clear(&self) {
        Cache::clear(self)
    }

    fn len(&self) -> usize {
        Cache::len(self)
    }
}

impl<K, V> AnyCache for ConcurrentFifoCache<K, V>
where
    K: Send + Sync + 'static + Eq + Hash + Clone,
    V: Send + Sync + 'static + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clear(&self) {
        Cache::clear(self)
    }

    fn len(&self) -> usize {
        Cache::len(self)
    }
}

impl<K, V> AnyCache for AdaptiveCache<K, V>
where
    K: Send + Sync + 'static + Eq + Hash + Clone,
    V: Send + Sync + 'static + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clear(&self) {
        Cache::clear(self)
    }

    fn len(&self) -> usize {
        Cache::len(self)
    }
}

impl<K, V> AnyCache for ConcurrentUnboundedCache<K, V>
where
    K: Send + Sync + 'static + Eq + Hash + Clone,
    V: Send + Sync + 'static + Clone,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clear(&self) {
        Cache::clear(self)
    }

    fn len(&self) -> usize {
        Cache::len(self)
    }
}

/// 全局缓存管理器
pub struct CacheManager {
    // 仅在全局缓存管理器中保留必要的 dyn 用于异构类型存储
    caches: RwLock<HashMap<String, Box<dyn AnyCache>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

impl std::fmt::Debug for CacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheManager")
            .field("cache_count", &self.caches.read().unwrap().len())
            .field("config", &self.config)
            .field("stats", &self.stats)
            .finish()
    }
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
    pub fn register_lru_cache<K, V>(&self, name: &str, cache: Arc<ConcurrentLruCache<K, V>>)
    where
        K: 'static + Send + Sync + Eq + Hash + Clone,
        V: 'static + Send + Sync + Clone,
    {
        let mut caches = self.caches.write().expect("Caches write lock was poisoned");
        // 由于 ConcurrentLruCache 没有实现 Clone，我们需要使用 Arc 的克隆
        // 这里需要重新设计注册机制，暂时使用 Arc::clone
        // 注意：这实际上只是增加了引用计数，不是真正的克隆
        let _cache_clone = Arc::clone(&cache);

        // 由于类型擦除的限制，我们需要使用不同的方法
        // 暂时注释掉这行，需要重新设计
        // caches.insert(name.to_string(), Box::new(cache_clone));

        // 暂时使用一个占位符实现
        // TODO: 重新设计缓存注册机制
        println!("缓存注册功能需要重新设计");
    }

    /// 创建LRU缓存
    pub fn create_lru_cache<K, V>(&self, capacity: usize) -> Arc<ConcurrentLruCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentLruCache::new(capacity))
    }

    /// 创建LFU缓存
    pub fn create_lfu_cache<K, V>(&self, capacity: usize) -> Arc<ConcurrentLfuCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentLfuCache::new(capacity))
    }

    /// 创建TTL缓存
    pub fn create_ttl_cache<K, V>(
        &self,
        capacity: usize,
        default_ttl: Duration,
    ) -> Arc<ConcurrentTtlCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        Arc::new(ConcurrentTtlCache::new(capacity, default_ttl))
    }

    /// 创建带统计的缓存
    pub fn create_stats_cache<K, V, C>(&self, cache: Arc<C>) -> Arc<StatsCacheWrapper<K, V, C>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
        C: Cache<K, V>,
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
        let caches = self.caches.read().expect("Caches read lock was poisoned");
        // 简化实现，实际需要类型擦除的清理方法
        drop(caches);
    }

    /// 获取缓存列表
    pub fn cache_names(&self) -> Vec<String> {
        let caches = self.caches.read().expect("Caches read lock was poisoned");
        caches.keys().cloned().collect()
    }

    /// 检查缓存是否存在
    pub fn has_cache(&self, name: &str) -> bool {
        let caches = self.caches.read().expect("Caches read lock was poisoned");
        caches.contains_key(name)
    }

    /// 移除缓存
    pub fn remove_cache(&self, name: &str) -> bool {
        let mut caches = self.caches.write().expect("Caches write lock was poisoned");
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
    once_cell::sync::Lazy::new(|| Arc::new(CacheManager::new(CacheConfig::default())));

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

/// 缓存策略枚举
#[derive(Debug, Clone)]
pub enum CachePolicy {
    LRU,
    LFU,
    FIFO,
    TTL(Duration),
    Adaptive,
    None,
}

/// 缓存策略枚举 - 用于统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CacheStrategy {
    LRU,
    LFU,
    FIFO,
    TTL,
    Adaptive,
    None,
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

    pub fn build(self) -> CacheType<K, V> {
        match self.policy {
            CachePolicy::LRU => CacheType::Lru(Arc::new(ConcurrentLruCache::new(self.capacity))),
            CachePolicy::LFU => CacheType::Lfu(Arc::new(ConcurrentLfuCache::new(self.capacity))),
            CachePolicy::TTL(ttl) => {
                CacheType::Ttl(Arc::new(ConcurrentTtlCache::new(self.capacity, ttl)))
            }
            CachePolicy::FIFO => CacheType::Fifo(Arc::new(ConcurrentFifoCache::new(self.capacity))),
            CachePolicy::Adaptive => {
                CacheType::Adaptive(Arc::new(AdaptiveCache::new(self.capacity)))
            }
            CachePolicy::None => CacheType::Unbounded(Arc::new(ConcurrentUnboundedCache::new())),
        }
    }

    pub fn build_with_stats(self) -> StatsCacheType<K, V> {
        match self.policy {
            CachePolicy::LRU => {
                let cache = Arc::new(ConcurrentLruCache::new(self.capacity));
                StatsCacheType::Lru(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::LFU => {
                let cache = Arc::new(ConcurrentLfuCache::new(self.capacity));
                StatsCacheType::Lfu(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::TTL(ttl) => {
                let cache = Arc::new(ConcurrentTtlCache::new(self.capacity, ttl));
                StatsCacheType::Ttl(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::FIFO => {
                let cache = Arc::new(ConcurrentFifoCache::new(self.capacity));
                StatsCacheType::Fifo(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::Adaptive => {
                let cache = Arc::new(AdaptiveCache::new(self.capacity));
                StatsCacheType::Adaptive(Arc::new(StatsCacheWrapper::new(cache)))
            }
            CachePolicy::None => {
                let cache = Arc::new(ConcurrentUnboundedCache::new());
                StatsCacheType::Unbounded(Arc::new(StatsCacheWrapper::new(cache)))
            }
        }
    }
}

// 为 CacheType 实现 Cache trait
impl<K, V> Cache<K, V> for CacheType<K, V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync + Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        match self {
            CacheType::Lru(cache) => cache.get(key),
            CacheType::Lfu(cache) => cache.get(key),
            CacheType::Ttl(cache) => cache.get(key),
            CacheType::Fifo(cache) => cache.get(key),
            CacheType::Adaptive(cache) => cache.get(key),
            CacheType::Unbounded(cache) => cache.get(key),
        }
    }

    fn put(&self, key: K, value: V) {
        match self {
            CacheType::Lru(cache) => cache.put(key, value),
            CacheType::Lfu(cache) => cache.put(key, value),
            CacheType::Ttl(cache) => cache.put(key, value),
            CacheType::Fifo(cache) => cache.put(key, value),
            CacheType::Adaptive(cache) => cache.put(key, value),
            CacheType::Unbounded(cache) => cache.put(key, value),
        }
    }

    fn contains(&self, key: &K) -> bool {
        match self {
            CacheType::Lru(cache) => cache.contains(key),
            CacheType::Lfu(cache) => cache.contains(key),
            CacheType::Ttl(cache) => cache.contains(key),
            CacheType::Fifo(cache) => cache.contains(key),
            CacheType::Adaptive(cache) => cache.contains(key),
            CacheType::Unbounded(cache) => cache.contains(key),
        }
    }

    fn remove(&self, key: &K) -> Option<V> {
        match self {
            CacheType::Lru(cache) => cache.remove(key),
            CacheType::Lfu(cache) => cache.remove(key),
            CacheType::Ttl(cache) => cache.remove(key),
            CacheType::Fifo(cache) => cache.remove(key),
            CacheType::Adaptive(cache) => cache.remove(key),
            CacheType::Unbounded(cache) => cache.remove(key),
        }
    }

    fn clear(&self) {
        match self {
            CacheType::Lru(cache) => Cache::clear(cache.as_ref()),
            CacheType::Lfu(cache) => Cache::clear(cache.as_ref()),
            CacheType::Ttl(cache) => Cache::clear(cache.as_ref()),
            CacheType::Fifo(cache) => Cache::clear(cache.as_ref()),
            CacheType::Adaptive(cache) => Cache::clear(cache.as_ref()),
            CacheType::Unbounded(cache) => Cache::clear(cache.as_ref()),
        }
    }

    fn len(&self) -> usize {
        match self {
            CacheType::Lru(cache) => Cache::len(cache.as_ref()),
            CacheType::Lfu(cache) => Cache::len(cache.as_ref()),
            CacheType::Ttl(cache) => Cache::len(cache.as_ref()),
            CacheType::Fifo(cache) => Cache::len(cache.as_ref()),
            CacheType::Adaptive(cache) => Cache::len(cache.as_ref()),
            CacheType::Unbounded(cache) => Cache::len(cache.as_ref()),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            CacheType::Lru(cache) => cache.is_empty(),
            CacheType::Lfu(cache) => cache.is_empty(),
            CacheType::Ttl(cache) => cache.is_empty(),
            CacheType::Fifo(cache) => cache.is_empty(),
            CacheType::Adaptive(cache) => cache.is_empty(),
            CacheType::Unbounded(cache) => cache.is_empty(),
        }
    }
}

// 为 StatsCacheType 实现 Cache 和 StatsCache trait
impl<K, V> Cache<K, V> for StatsCacheType<K, V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync + Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        match self {
            StatsCacheType::Lru(cache) => cache.get(key),
            StatsCacheType::Lfu(cache) => cache.get(key),
            StatsCacheType::Ttl(cache) => cache.get(key),
            StatsCacheType::Fifo(cache) => cache.get(key),
            StatsCacheType::Adaptive(cache) => cache.get(key),
            StatsCacheType::Unbounded(cache) => cache.get(key),
        }
    }

    fn put(&self, key: K, value: V) {
        match self {
            StatsCacheType::Lru(cache) => cache.put(key, value),
            StatsCacheType::Lfu(cache) => cache.put(key, value),
            StatsCacheType::Ttl(cache) => cache.put(key, value),
            StatsCacheType::Fifo(cache) => cache.put(key, value),
            StatsCacheType::Adaptive(cache) => cache.put(key, value),
            StatsCacheType::Unbounded(cache) => cache.put(key, value),
        }
    }

    fn contains(&self, key: &K) -> bool {
        match self {
            StatsCacheType::Lru(cache) => cache.contains(key),
            StatsCacheType::Lfu(cache) => cache.contains(key),
            StatsCacheType::Ttl(cache) => cache.contains(key),
            StatsCacheType::Fifo(cache) => cache.contains(key),
            StatsCacheType::Adaptive(cache) => cache.contains(key),
            StatsCacheType::Unbounded(cache) => cache.contains(key),
        }
    }

    fn remove(&self, key: &K) -> Option<V> {
        match self {
            StatsCacheType::Lru(cache) => cache.remove(key),
            StatsCacheType::Lfu(cache) => cache.remove(key),
            StatsCacheType::Ttl(cache) => cache.remove(key),
            StatsCacheType::Fifo(cache) => cache.remove(key),
            StatsCacheType::Adaptive(cache) => cache.remove(key),
            StatsCacheType::Unbounded(cache) => cache.remove(key),
        }
    }

    fn clear(&self) {
        match self {
            StatsCacheType::Lru(cache) => cache.clear(),
            StatsCacheType::Lfu(cache) => cache.clear(),
            StatsCacheType::Ttl(cache) => cache.clear(),
            StatsCacheType::Fifo(cache) => cache.clear(),
            StatsCacheType::Adaptive(cache) => cache.clear(),
            StatsCacheType::Unbounded(cache) => cache.clear(),
        }
    }

    fn len(&self) -> usize {
        match self {
            StatsCacheType::Lru(cache) => cache.len(),
            StatsCacheType::Lfu(cache) => cache.len(),
            StatsCacheType::Ttl(cache) => cache.len(),
            StatsCacheType::Fifo(cache) => cache.len(),
            StatsCacheType::Adaptive(cache) => cache.len(),
            StatsCacheType::Unbounded(cache) => cache.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            StatsCacheType::Lru(cache) => cache.is_empty(),
            StatsCacheType::Lfu(cache) => cache.is_empty(),
            StatsCacheType::Ttl(cache) => cache.is_empty(),
            StatsCacheType::Fifo(cache) => cache.is_empty(),
            StatsCacheType::Adaptive(cache) => cache.is_empty(),
            StatsCacheType::Unbounded(cache) => cache.is_empty(),
        }
    }
}

impl<K, V> StatsCache<K, V> for StatsCacheType<K, V>
where
    K: 'static + Send + Sync + Hash + Eq + Clone,
    V: 'static + Send + Sync + Clone,
{
    fn hits(&self) -> u64 {
        match self {
            StatsCacheType::Lru(cache) => cache.hits(),
            StatsCacheType::Lfu(cache) => cache.hits(),
            StatsCacheType::Ttl(cache) => cache.hits(),
            StatsCacheType::Fifo(cache) => cache.hits(),
            StatsCacheType::Adaptive(cache) => cache.hits(),
            StatsCacheType::Unbounded(cache) => cache.hits(),
        }
    }

    fn misses(&self) -> u64 {
        match self {
            StatsCacheType::Lru(cache) => cache.misses(),
            StatsCacheType::Lfu(cache) => cache.misses(),
            StatsCacheType::Ttl(cache) => cache.misses(),
            StatsCacheType::Fifo(cache) => cache.misses(),
            StatsCacheType::Adaptive(cache) => cache.misses(),
            StatsCacheType::Unbounded(cache) => cache.misses(),
        }
    }

    fn hit_rate(&self) -> f64 {
        match self {
            StatsCacheType::Lru(cache) => cache.hit_rate(),
            StatsCacheType::Lfu(cache) => cache.hit_rate(),
            StatsCacheType::Ttl(cache) => cache.hit_rate(),
            StatsCacheType::Fifo(cache) => cache.hit_rate(),
            StatsCacheType::Adaptive(cache) => cache.hit_rate(),
            StatsCacheType::Unbounded(cache) => cache.hit_rate(),
        }
    }

    fn evictions(&self) -> u64 {
        match self {
            StatsCacheType::Lru(cache) => cache.evictions(),
            StatsCacheType::Lfu(cache) => cache.evictions(),
            StatsCacheType::Ttl(cache) => cache.evictions(),
            StatsCacheType::Fifo(cache) => cache.evictions(),
            StatsCacheType::Adaptive(cache) => cache.evictions(),
            StatsCacheType::Unbounded(cache) => cache.evictions(),
        }
    }

    fn reset_stats(&self) {
        match self {
            StatsCacheType::Lru(cache) => cache.reset_stats(),
            StatsCacheType::Lfu(cache) => cache.reset_stats(),
            StatsCacheType::Ttl(cache) => cache.reset_stats(),
            StatsCacheType::Fifo(cache) => cache.reset_stats(),
            StatsCacheType::Adaptive(cache) => cache.reset_stats(),
            StatsCacheType::Unbounded(cache) => cache.reset_stats(),
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
        let cache = CacheBuilder::new(100)
            .with_ttl(Duration::from_secs(60))
            .build();

        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_builder_with_stats() {
        let cache = CacheBuilder::new(100)
            .with_ttl(Duration::from_secs(60))
            .build_with_stats();

        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
        assert_eq!(cache.hits(), 1);
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
        let cache = manager.create_lru_cache(100);

        manager.register_lru_cache("test", cache);
        assert!(manager.has_cache("test"));
    }
}
