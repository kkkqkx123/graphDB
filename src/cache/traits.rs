//! 缓存特征定义
//!
//! 提供统一的缓存接口，支持不同类型的缓存实现

use std::time::Duration;

/// 基础缓存特征
pub trait Cache<K, V> {
    /// 获取缓存值
    fn get(&self, key: &K) -> Option<V>;

    /// 设置缓存值
    fn put(&self, key: K, value: V);

    /// 检查是否包含键
    fn contains(&self, key: &K) -> bool;

    /// 移除缓存项
    fn remove(&self, key: &K) -> Option<V>;

    /// 清空缓存
    fn clear(&self);

    /// 获取缓存大小
    fn len(&self) -> usize;

    /// 检查是否为空
    fn is_empty(&self) -> bool;
}

/// 统计缓存特征
pub trait StatsCache<K, V>: Cache<K, V> {
    /// 获取命中次数
    fn hits(&self) -> u64;

    /// 获取未命中次数
    fn misses(&self) -> u64;

    /// 获取命中率
    fn hit_rate(&self) -> f64;

    /// 获取驱逐次数
    fn evictions(&self) -> u64;

    /// 重置统计信息
    fn reset_stats(&self);
}

/// 缓存条目特征
pub trait CacheEntry<V> {
    /// 获取值
    fn value(&self) -> &V;

    /// 获取创建时间
    fn created_at(&self) -> std::time::Instant;

    /// 获取最后访问时间
    fn last_accessed(&self) -> std::time::Instant;

    /// 获取访问次数
    fn access_count(&self) -> u64;

    /// 检查是否过期
    fn is_expired(&self) -> bool;
}

/// 默认的缓存条目实现
#[derive(Debug, Clone)]
pub struct DefaultCacheEntry<V> {
    value: V,
    created_at: std::time::Instant,
    last_accessed: std::time::Instant,
    access_count: u64,
    ttl: Option<Duration>,
}

impl<V> DefaultCacheEntry<V> {
    pub fn new(value: V, ttl: Option<Duration>) -> Self {
        let now = std::time::Instant::now();
        Self {
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = std::time::Instant::now();
        self.access_count += 1;
    }
}

impl<V> CacheEntry<V> for DefaultCacheEntry<V> {
    fn value(&self) -> &V {
        &self.value
    }

    fn created_at(&self) -> std::time::Instant {
        self.created_at
    }

    fn last_accessed(&self) -> std::time::Instant {
        self.last_accessed
    }

    fn access_count(&self) -> u64 {
        self.access_count
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;

    // 简单的测试缓存实现
    struct TestCache<K, V> {
        data: Arc<Mutex<std::collections::HashMap<K, V>>>,
    }

    impl<K, V> TestCache<K, V>
    where
        K: Clone + Eq + std::hash::Hash,
    {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(std::collections::HashMap::new())),
            }
        }
    }

    impl<K, V> Cache<K, V> for TestCache<K, V>
    where
        K: Clone + Eq + std::hash::Hash,
        V: Clone,
    {
        fn get(&self, key: &K) -> Option<V> {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .get(key)
                .cloned()
        }

        fn put(&self, key: K, value: V) {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .insert(key, value);
        }

        fn contains(&self, key: &K) -> bool {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .contains_key(key)
        }

        fn remove(&self, key: &K) -> Option<V> {
            self.data
                .lock()
                .expect("Data lock was poisoned")
                .remove(key)
        }

        fn clear(&self) {
            self.data.lock().expect("Data lock was poisoned").clear();
        }

        fn len(&self) -> usize {
            self.data.lock().expect("Data lock was poisoned").len()
        }

        fn is_empty(&self) -> bool {
            self.data.lock().expect("Data lock was poisoned").is_empty()
        }
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = TestCache::new();

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.len(), 1);
        assert!(cache.contains(&"key1".to_string()));

        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some("value1".to_string()));

        cache.remove(&"key1".to_string());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_default_cache_entry() {
        let mut entry = DefaultCacheEntry::new("test".to_string(), Some(Duration::from_secs(1)));

        assert_eq!(entry.value(), &"test".to_string());
        assert_eq!(entry.access_count(), 0);
        assert!(!entry.is_expired());

        entry.touch();
        assert_eq!(entry.access_count(), 1);
    }
}
