//! 缓存管理器
//!
//! 提供全局缓存的管理和协调功能
//!
//! 重构后使用工具类：CacheRegistry、CacheFactory、CacheStatsCollector

use super::config::*;
use super::factory::*;
use super::registry::*;
use super::stats_collector::*;
use super::stats_marker::StatsEnabled;
use crate::cache::{
    Cache, ConcurrentAdaptiveCache, ConcurrentFifoCache, ConcurrentLfuCache, ConcurrentLruCache,
    ConcurrentTtlCache, ConcurrentUnboundedCache, StatsCache, StatsCacheWrapper,
};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// 全局缓存管理器
pub struct CacheManager {
    registry: CacheRegistry,
    stats_collector: CacheStatsCollector,
    config: CacheConfig,
}

impl std::fmt::Debug for CacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheManager")
            .field("cache_count", &self.registry.cache_count())
            .field("config", &self.config)
            .field("stats", &self.stats_collector)
            .finish()
    }
}

impl CacheManager {
    /// 创建新的缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        Self {
            registry: CacheRegistry::new(),
            stats_collector: CacheStatsCollector::new(),
            config,
        }
    }

    /// 注册LRU缓存
    pub fn register_lru_cache<K, V>(&self, name: &str, capacity: usize)
    where
        K: 'static + Send + Sync + Eq + Hash + Clone,
        V: 'static + Send + Sync + Clone,
    {
        self.registry
            .register_cache(name, "LRU", capacity, CacheStrategy::LRU)
            .expect("Failed to register LRU cache");
        self.stats_collector
            .record_cache_count(self.registry.cache_count() as u64);
    }

    /// 注册LFU缓存
    pub fn register_lfu_cache<K, V>(&self, name: &str, capacity: usize)
    where
        K: 'static + Send + Sync + Eq + Hash + Clone,
        V: 'static + Send + Sync + Clone,
    {
        self.registry
            .register_cache(name, "LFU", capacity, CacheStrategy::LFU)
            .expect("Failed to register LFU cache");
        self.stats_collector
            .record_cache_count(self.registry.cache_count() as u64);
    }

    /// 注册TTL缓存
    pub fn register_ttl_cache<K, V>(&self, name: &str, capacity: usize, _ttl: Duration)
    where
        K: 'static + Send + Sync + Eq + Hash + Clone,
        V: 'static + Send + Sync + Clone,
    {
        self.registry
            .register_cache(name, "TTL", capacity, CacheStrategy::TTL)
            .expect("Failed to register TTL cache");
        self.stats_collector
            .record_cache_count(self.registry.cache_count() as u64);
    }

    /// 注册FIFO缓存
    pub fn register_fifo_cache<K, V>(&self, name: &str, capacity: usize)
    where
        K: 'static + Send + Sync + Eq + Hash + Clone,
        V: 'static + Send + Sync + Clone,
    {
        self.registry
            .register_cache(name, "FIFO", capacity, CacheStrategy::FIFO)
            .expect("Failed to register FIFO cache");
        self.stats_collector
            .record_cache_count(self.registry.cache_count() as u64);
    }

    /// 注册自适应缓存
    pub fn register_adaptive_cache<K, V>(&self, name: &str, capacity: usize)
    where
        K: 'static + Send + Sync + Eq + Hash + Clone,
        V: 'static + Send + Sync + Clone,
    {
        self.registry
            .register_cache(name, "Adaptive", capacity, CacheStrategy::Adaptive)
            .expect("Failed to register Adaptive cache");
        self.stats_collector
            .record_cache_count(self.registry.cache_count() as u64);
    }

    /// 注册无界缓存
    pub fn register_unbounded_cache<K, V>(&self, name: &str)
    where
        K: 'static + Send + Sync + Eq + Hash + Clone,
        V: 'static + Send + Sync + Clone,
    {
        self.registry
            .register_cache(name, "Unbounded", usize::MAX, CacheStrategy::None)
            .expect("Failed to register Unbounded cache");
        self.stats_collector
            .record_cache_count(self.registry.cache_count() as u64);
    }

    /// 获取缓存注册信息
    pub fn get_cache_info(&self, name: &str) -> Option<CacheRegistryInfo> {
        self.registry.get_cache_info(name)
    }

    /// 获取所有缓存注册信息
    pub fn get_all_cache_info(&self) -> Vec<CacheRegistryInfo> {
        self.registry.get_all_cache_info()
    }

    /// 创建LRU缓存
    pub fn create_lru_cache<K, V>(&self, capacity: usize) -> Arc<ConcurrentLruCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        CacheFactory::create_lru_cache(capacity)
    }

    /// 创建LFU缓存
    pub fn create_lfu_cache<K, V>(&self, capacity: usize) -> Arc<ConcurrentLfuCache<K, V>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
    {
        CacheFactory::create_lfu_cache(capacity)
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
        CacheFactory::create_ttl_cache(capacity, default_ttl)
    }

    /// 创建带统计的缓存
    ///
    /// 返回启用统计的包装器版本
    pub fn create_stats_cache<K, V, C>(
        &self,
        cache: Arc<C>,
    ) -> Arc<StatsCacheWrapper<K, V, C, StatsEnabled>>
    where
        K: 'static + Send + Sync + Hash + Eq + Clone,
        V: 'static + Send + Sync + Clone,
        C: Cache<K, V>,
    {
        CacheFactory::create_stats_wrapper(cache)
    }

    /// 获取配置
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// 获取统计信息
    pub fn stats(&self) -> Arc<RwLock<CacheStats>> {
        self.stats_collector.stats()
    }

    /// 清空所有缓存注册信息
    pub fn clear_all(&self) {
        self.registry.clear_all();
        self.stats_collector.record_cache_count(0);
    }

    /// 获取缓存列表
    pub fn cache_names(&self) -> Vec<String> {
        self.registry.cache_names()
    }

    /// 检查缓存是否存在
    pub fn has_cache(&self, name: &str) -> bool {
        self.registry.has_cache(name)
    }

    /// 移除缓存
    pub fn remove_cache(&self, name: &str) -> bool {
        let removed = self.registry.remove_cache(name);
        if removed {
            self.stats_collector
                .record_cache_count(self.registry.cache_count() as u64);
        }
        removed
    }
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

/// 为 Arc<T> 实现 Cache trait
///
/// 这是一个 blanket implementation（通用实现），允许在 Arc 包装的缓存上直接调用 Cache trait 方法。
///
/// # 性能特性
/// - **零运行时开销**：编译器会将这些方法调用完全内联，等同于直接调用底层实现
/// - **静态分发**：编译期确定具体类型，没有动态分派的开销
///
/// # 使用场景
/// 当缓存需要被多个所有者共享时，通常会使用 Arc 包装。这个实现允许：
/// ```rust
/// let cache: Arc<ConcurrentLruCache<K, V>> = ...;
/// cache.get(&key);  // 直接调用，无需 cache.as_ref().get(&key)
/// ```
///
/// # 设计理由
/// 这种模式是 Rust 社区的标准做法（类似于标准库中 String 对 Deref 的实现），
/// 提供更好的 API 体验，同时保持零成本抽象。
impl<K, V, T> Cache<K, V> for Arc<T>
where
    T: Cache<K, V>,
{
    fn get(&self, key: &K) -> Option<V> {
        self.as_ref().get(key)
    }

    fn put(&self, key: K, value: V) {
        self.as_ref().put(key, value)
    }

    fn contains(&self, key: &K) -> bool {
        self.as_ref().contains(key)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.as_ref().remove(key)
    }

    fn clear(&self) {
        self.as_ref().clear()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }

    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}

/// 为 Arc<T> 实现 StatsCache trait
///
/// 这是一个 blanket implementation（通用实现），允许在 Arc 包装的统计缓存上直接调用 StatsCache trait 方法。
///
/// # 性能特性
/// - **零运行时开销**：编译器会将这些方法调用完全内联，等同于直接调用底层实现
/// - **静态分发**：编译期确定具体类型，没有动态分派的开销
///
/// # 使用场景
/// 当统计缓存需要被多个所有者共享时，通常会使用 Arc 包装。这个实现允许：
/// ```rust
/// let cache: Arc<StatsCacheWrapper<...>> = ...;
/// cache.hits();  // 直接调用，无需 cache.as_ref().hits()
/// ```
///
/// # 设计理由
/// 这种模式是 Rust 社区的标准做法，提供更好的 API 体验，同时保持零成本抽象。
/// 特别是在 `StatsCacheWrapper` 被包装在 `Arc` 中时（如 `parser_cache.rs` 中的使用），
/// 这个实现可以显著提高代码的可读性。
impl<K, V, T> StatsCache<K, V> for Arc<T>
where
    T: StatsCache<K, V>,
{
    fn hits(&self) -> u64 {
        self.as_ref().hits()
    }

    fn misses(&self) -> u64 {
        self.as_ref().misses()
    }

    fn hit_rate(&self) -> f64 {
        self.as_ref().hit_rate()
    }

    fn evictions(&self) -> u64 {
        self.as_ref().evictions()
    }

    fn reset_stats(&self) {
        self.as_ref().reset_stats()
    }
}

/// 缓存构建器
///
/// 提供链式API构建缓存实例，支持配置容量、TTL和统计功能
///
/// 设计原则：避免使用动态分发 (dyn)，为每个缓存策略提供独立的构建方法
/// 这样可以在编译时确定类型，避免运行时的动态分发开销
pub struct CacheBuilder<K, V> {
    capacity: usize,
    ttl: Option<Duration>,
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
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// 构建LRU缓存
    pub fn build_lru(self) -> Arc<ConcurrentLruCache<K, V>> {
        CacheFactory::create_lru_cache(self.capacity)
    }

    /// 构建LFU缓存
    pub fn build_lfu(self) -> Arc<ConcurrentLfuCache<K, V>> {
        CacheFactory::create_lfu_cache(self.capacity)
    }

    /// 构建FIFO缓存
    pub fn build_fifo(self) -> Arc<crate::cache::ConcurrentFifoCache<K, V>> {
        CacheFactory::create_fifo_cache(self.capacity)
    }

    /// 构建TTL缓存
    pub fn build_ttl(self) -> Arc<ConcurrentTtlCache<K, V>> {
        let ttl = self.ttl.expect("TTL cache requires TTL duration");
        CacheFactory::create_ttl_cache(self.capacity, ttl)
    }

    /// 构建自适应缓存
    pub fn build_adaptive(self) -> Arc<ConcurrentAdaptiveCache<K, V>> {
        CacheFactory::create_adaptive_cache(self.capacity).into()
    }

    /// 构建无界缓存
    pub fn build_unbounded(self) -> Arc<ConcurrentUnboundedCache<K, V>> {
        CacheFactory::create_unbounded_cache()
    }

    /// 构建带统计的LRU缓存
    pub fn build_lru_with_stats(
        self,
    ) -> Arc<StatsCacheWrapper<K, V, ConcurrentLruCache<K, V>, StatsEnabled>> {
        let cache = self.build_lru();
        CacheFactory::create_stats_wrapper(cache)
    }

    /// 构建带统计的LFU缓存
    pub fn build_lfu_with_stats(
        self,
    ) -> Arc<StatsCacheWrapper<K, V, ConcurrentLfuCache<K, V>, StatsEnabled>> {
        let cache = self.build_lfu();
        CacheFactory::create_stats_wrapper(cache)
    }

    /// 构建带统计的FIFO缓存
    pub fn build_fifo_with_stats(
        self,
    ) -> Arc<StatsCacheWrapper<K, V, ConcurrentFifoCache<K, V>, StatsEnabled>> {
        let cache = self.build_fifo();
        CacheFactory::create_stats_wrapper(cache)
    }

    /// 构建带统计的TTL缓存
    pub fn build_ttl_with_stats(
        self,
    ) -> Arc<StatsCacheWrapper<K, V, ConcurrentTtlCache<K, V>, StatsEnabled>> {
        let cache = self.build_ttl();
        CacheFactory::create_stats_wrapper(cache)
    }

    /// 构建带统计的自适应缓存
    pub fn build_adaptive_with_stats(
        self,
    ) -> Arc<StatsCacheWrapper<K, V, ConcurrentAdaptiveCache<K, V>, StatsEnabled>> {
        let cache = self.build_adaptive();
        CacheFactory::create_stats_wrapper(cache)
    }

    /// 构建带统计的无界缓存
    pub fn build_unbounded_with_stats(
        self,
    ) -> Arc<StatsCacheWrapper<K, V, ConcurrentUnboundedCache<K, V>, StatsEnabled>> {
        let cache = self.build_unbounded();
        CacheFactory::create_stats_wrapper(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .build_ttl();

        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_builder_with_stats() {
        let cache = CacheBuilder::new(100)
            .with_ttl(Duration::from_secs(60))
            .build_ttl_with_stats();

        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
        assert_eq!(cache.hits(), 1);
    }

    #[test]
    fn test_cache_builder_lru() {
        let cache = CacheBuilder::new(100).build_lru();
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_builder_lfu() {
        let cache = CacheBuilder::new(100).build_lfu();
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_builder_fifo() {
        let cache = CacheBuilder::new(100).build_fifo();
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_builder_adaptive() {
        let cache = CacheBuilder::new(100).build_adaptive();
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_builder_unbounded() {
        let cache = CacheBuilder::new(100).build_unbounded();
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
    }

    #[test]
    fn test_cache_builder_lru_with_stats() {
        let cache = CacheBuilder::new(100).build_lru_with_stats();
        cache.put("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
        assert_eq!(cache.hits(), 1);
    }

    #[test]
    fn test_cache_builder_lfu_with_stats() {
        let cache = CacheBuilder::new(100).build_lfu_with_stats();
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
    fn test_cache_manager_config() {
        let manager = CacheManager::new(CacheConfig::default());
        assert!(manager.config().enabled);
    }

    #[test]
    fn test_cache_registration() {
        let manager = CacheManager::new(CacheConfig::default());

        manager.register_lru_cache::<String, String>("test", 100);
        assert!(manager.has_cache("test"));

        let info = manager
            .get_cache_info("test")
            .expect("Cache info should exist");
        assert_eq!(info.name, "test");
        assert_eq!(info.cache_type, "LRU");
        assert_eq!(info.capacity, 100);
    }

    #[test]
    fn test_cache_registry_operations() {
        let manager = CacheManager::new(CacheConfig::default());

        // 注册不同类型的缓存
        manager.register_lru_cache::<String, String>("lru_cache", 100);
        manager.register_lfu_cache::<i32, String>("lfu_cache", 200);
        manager.register_ttl_cache::<u64, Vec<i32>>("ttl_cache", 300, Duration::from_secs(60));
        manager.register_fifo_cache::<char, f64>("fifo_cache", 400);
        manager.register_adaptive_cache::<bool, String>("adaptive_cache", 500);
        manager.register_unbounded_cache::<String, i32>("unbounded_cache");

        // 验证缓存数量
        assert_eq!(manager.cache_names().len(), 6);

        // 验证缓存信息
        let lru_info = manager
            .get_cache_info("lru_cache")
            .expect("LRU cache info should exist");
        assert_eq!(lru_info.cache_type, "LRU");
        assert_eq!(lru_info.capacity, 100);

        let lfu_info = manager
            .get_cache_info("lfu_cache")
            .expect("LFU cache info should exist");
        assert_eq!(lfu_info.cache_type, "LFU");
        assert_eq!(lfu_info.capacity, 200);

        let unbounded_info = manager
            .get_cache_info("unbounded_cache")
            .expect("Unbounded cache info should exist");
        assert_eq!(unbounded_info.cache_type, "Unbounded");
        assert_eq!(unbounded_info.capacity, usize::MAX);

        // 测试移除缓存
        assert!(manager.remove_cache("lru_cache"));
        assert!(!manager.has_cache("lru_cache"));
        assert_eq!(manager.cache_names().len(), 5);

        // 测试清空所有缓存
        manager.clear_all();
        assert_eq!(manager.cache_names().len(), 0);
    }

    #[test]
    fn test_cache_stats_update() {
        let manager = CacheManager::new(CacheConfig::default());

        // 初始状态
        let stats_arc = manager.stats();
        let stats = stats_arc.read().expect("Stats lock should be acquired");
        assert_eq!(stats.cache_count, 0);
        drop(stats);

        // 注册缓存
        manager.register_lru_cache::<String, String>("test1", 100);
        manager.register_lfu_cache::<i32, String>("test2", 200);

        // 验证统计信息更新
        let stats_arc = manager.stats();
        let stats = stats_arc.read().expect("Stats lock should be acquired");
        assert_eq!(stats.cache_count, 2);
        drop(stats);

        // 移除缓存
        manager.remove_cache("test1");

        // 验证统计信息更新
        let stats_arc = manager.stats();
        let stats = stats_arc.read().expect("Stats lock should be acquired");
        assert_eq!(stats.cache_count, 1);
        drop(stats);

        // 清空所有缓存
        manager.clear_all();

        // 验证统计信息更新
        let stats_arc = manager.stats();
        let stats = stats_arc.read().expect("Stats lock should be acquired");
        assert_eq!(stats.cache_count, 0);
    }
}
