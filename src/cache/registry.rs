//! 缓存注册表
//!
//! 负责管理缓存注册信息，提供统一的注册和查询接口

use crate::cache::CacheStrategy;
use std::collections::HashMap;
use std::sync::RwLock;

/// 缓存注册信息
#[derive(Debug, Clone)]
pub struct CacheRegistryInfo {
    pub name: String,
    pub cache_type: String,
    pub capacity: usize,
    pub created_at: std::time::Instant,
    pub policy: CacheStrategy,
}

/// 缓存注册表
///
/// 负责管理所有缓存的注册信息，不存储实际缓存实例
#[derive(Clone)]
pub struct CacheRegistry {
    registry: std::sync::Arc<RwLock<HashMap<String, CacheRegistryInfo>>>,
}

impl CacheRegistry {
    /// 创建新的缓存注册表
    pub fn new() -> Self {
        Self {
            registry: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册缓存信息
    pub fn register_cache(
        &self,
        name: &str,
        cache_type: &str,
        capacity: usize,
        policy: CacheStrategy,
    ) -> Result<(), String> {
        if name.is_empty() {
            return Err("缓存名称不能为空".to_string());
        }

        let info = CacheRegistryInfo {
            name: name.to_string(),
            cache_type: cache_type.to_string(),
            capacity,
            created_at: std::time::Instant::now(),
            policy,
        };

        let mut registry = self
            .registry
            .write()
            .expect("Cache registry write lock was poisoned");
        registry.insert(name.to_string(), info);

        Ok(())
    }

    /// 获取缓存注册信息
    pub fn get_cache_info(&self, name: &str) -> Option<CacheRegistryInfo> {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        registry.get(name).cloned()
    }

    /// 获取所有缓存注册信息
    pub fn get_all_cache_info(&self) -> Vec<CacheRegistryInfo> {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        registry.values().cloned().collect()
    }

    /// 获取缓存名称列表
    pub fn cache_names(&self) -> Vec<String> {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        registry.keys().cloned().collect()
    }

    /// 检查缓存是否存在
    pub fn has_cache(&self, name: &str) -> bool {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        registry.contains_key(name)
    }

    /// 移除缓存
    pub fn remove_cache(&self, name: &str) -> bool {
        let mut registry = self
            .registry
            .write()
            .expect("Cache registry write lock was poisoned");
        registry.remove(name).is_some()
    }

    /// 清空所有缓存注册信息
    pub fn clear_all(&self) {
        let mut registry = self
            .registry
            .write()
            .expect("Cache registry write lock was poisoned");
        registry.clear();
    }

    /// 获取缓存数量
    pub fn cache_count(&self) -> usize {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        registry.len()
    }

    /// 根据策略类型获取缓存列表
    pub fn get_caches_by_policy(&self, policy: CacheStrategy) -> Vec<CacheRegistryInfo> {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        registry
            .values()
            .filter(|info| info.policy == policy)
            .cloned()
            .collect()
    }

    /// 获取在指定时间后创建的缓存
    pub fn get_caches_created_after(&self, after: std::time::Instant) -> Vec<CacheRegistryInfo> {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        registry
            .values()
            .filter(|info| info.created_at > after)
            .cloned()
            .collect()
    }
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CacheRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let registry = self
            .registry
            .read()
            .expect("Cache registry read lock was poisoned");
        f.debug_struct("CacheRegistry")
            .field("cache_count", &registry.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_registry_basic_operations() {
        let registry = CacheRegistry::new();

        // 初始状态
        assert_eq!(registry.cache_count(), 0);
        assert!(!registry.has_cache("test"));

        // 注册缓存
        registry
            .register_cache("test", "LRU", 100, CacheStrategy::LRU)
            .expect("Registration should succeed");
        assert_eq!(registry.cache_count(), 1);
        assert!(registry.has_cache("test"));

        // 获取缓存信息
        let info = registry
            .get_cache_info("test")
            .expect("Cache info should exist");
        assert_eq!(info.name, "test");
        assert_eq!(info.cache_type, "LRU");
        assert_eq!(info.capacity, 100);
        assert_eq!(info.policy, CacheStrategy::LRU);

        // 移除缓存
        assert!(registry.remove_cache("test"));
        assert!(!registry.has_cache("test"));
        assert_eq!(registry.cache_count(), 0);
    }

    #[test]
    fn test_cache_registry_multiple_caches() {
        let registry = CacheRegistry::new();

        // 注册多个缓存
        registry
            .register_cache("lru_cache", "LRU", 100, CacheStrategy::LRU)
            .expect("LRU registration should succeed");
        registry
            .register_cache("lfu_cache", "LFU", 200, CacheStrategy::LFU)
            .expect("LFU registration should succeed");
        registry
            .register_cache("fifo_cache", "FIFO", 300, CacheStrategy::FIFO)
            .expect("FIFO registration should succeed");

        assert_eq!(registry.cache_count(), 3);

        // 获取所有缓存名称
        let names = registry.cache_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"lru_cache".to_string()));
        assert!(names.contains(&"lfu_cache".to_string()));
        assert!(names.contains(&"fifo_cache".to_string()));

        // 获取所有缓存信息
        let all_info = registry.get_all_cache_info();
        assert_eq!(all_info.len(), 3);

        // 按策略筛选
        let lru_caches = registry.get_caches_by_policy(CacheStrategy::LRU);
        assert_eq!(lru_caches.len(), 1);
        assert_eq!(lru_caches[0].name, "lru_cache");

        // 清空所有缓存
        registry.clear_all();
        assert_eq!(registry.cache_count(), 0);
    }

    #[test]
    fn test_cache_registry_validation() {
        let registry = CacheRegistry::new();

        // 空名称应该失败
        let result = registry.register_cache("", "LRU", 100, CacheStrategy::LRU);
        assert!(result.is_err());
        assert_eq!(
            result.expect_err("Should return an error"),
            "缓存名称不能为空"
        );
    }

    #[test]
    fn test_cache_registry_created_after() {
        let registry = CacheRegistry::new();
        let now = std::time::Instant::now();

        // 注册缓存
        registry
            .register_cache("cache1", "LRU", 100, CacheStrategy::LRU)
            .expect("Cache1 registration should succeed");

        // 稍等一下再注册第二个缓存
        std::thread::sleep(std::time::Duration::from_millis(1));
        let later = std::time::Instant::now();
        registry
            .register_cache("cache2", "LFU", 200, CacheStrategy::LFU)
            .expect("Cache2 registration should succeed");

        // 获取在指定时间后创建的缓存
        let caches_after = registry.get_caches_created_after(now);
        assert_eq!(caches_after.len(), 2);

        let caches_after_later = registry.get_caches_created_after(later);
        assert_eq!(caches_after_later.len(), 1);
        assert_eq!(caches_after_later[0].name, "cache2");
    }
}
