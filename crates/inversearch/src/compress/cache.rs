use crate::compress::lcg::lcg;
use crate::compress::radix::to_radix_u64;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Mutex, OnceLock};

pub struct CompressCache {
    cache: Mutex<LruCache<String, String>>,
    max_size: usize,
}

impl CompressCache {
    pub fn new(max_size: usize) -> Self {
        let cap = NonZeroUsize::new(max_size.max(1))
            .or_else(|| NonZeroUsize::new(1000))
            .expect("Default cache size should be valid");
        CompressCache {
            cache: Mutex::new(LruCache::new(cap)),
            max_size,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().ok()?;
        cache.get(key).cloned()
    }

    pub fn insert(&self, key: String, value: String) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(key, value);
        }
    }

    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    pub fn is_empty(&self) -> bool {
        let cache = self.cache.lock().unwrap();
        cache.is_empty()
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.lock().unwrap();
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
}

static COMPRESS_CACHE: OnceLock<Mutex<LruCache<String, String>>> = OnceLock::new();

fn get_or_init_cache(cache_size: usize) -> &'static Mutex<LruCache<String, String>> {
    COMPRESS_CACHE.get_or_init(|| {
        let cap = NonZeroUsize::new(cache_size.max(1)).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Mutex::new(LruCache::new(cap))
    })
}

pub fn compress_with_cache(input: &str, cache_size: usize) -> String {
    if input.is_empty() {
        return String::new();
    }

    let cache = get_or_init_cache(cache_size);

    if let Ok(mut guard) = cache.lock() {
        if let Some(cached) = guard.get(input) {
            return cached.clone();
        }
    }

    let result = if let Ok(num) = input.parse::<u64>() {
        to_radix_u64(num, 255)
    } else {
        let hash = lcg(input);
        to_radix_u64(hash, 255)
    };

    if let Ok(mut guard) = cache.lock() {
        guard.put(input.to_string(), result.clone());
    }

    result
}

pub fn clear_global_cache() {
    if let Some(cache) = COMPRESS_CACHE.get() {
        if let Ok(mut guard) = cache.lock() {
            guard.clear();
        }
    }
}

pub fn get_cache_stats(cache_size: usize) -> CacheStats {
    let cache = get_or_init_cache(cache_size);
    if let Ok(guard) = cache.lock() {
        CacheStats {
            size: guard.len(),
            max_size: cache_size,
        }
    } else {
        CacheStats {
            size: 0,
            max_size: cache_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache = CompressCache::new(100);
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = CompressCache::new(100);
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = CompressCache::new(3);
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());
        cache.insert("key4".to_string(), "value4".to_string());

        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_compress_with_cache() {
        let result = compress_with_cache("hello", 100);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_compress_deterministic() {
        let result1 = compress_with_cache("hello", 100);
        let result2 = compress_with_cache("hello", 100);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_compress_number() {
        let result = compress_with_cache("123", 100);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_compress_empty() {
        let result = compress_with_cache("", 100);
        assert!(result.is_empty());
    }

    #[test]
    fn test_global_cache_clear() {
        compress_with_cache("test_key", 100);
        clear_global_cache();
        let stats = get_cache_stats(100);
        assert_eq!(stats.size, 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = CompressCache::new(100);
        cache.insert("key1".to_string(), "value1".to_string());
        let stats = cache.stats();
        assert_eq!(stats.size, 1);
        assert_eq!(stats.max_size, 100);
    }
}
