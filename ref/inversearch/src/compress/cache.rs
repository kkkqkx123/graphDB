use std::sync::Mutex;
use lru::LruCache;
use crate::compress::lcg::lcg;
use crate::compress::radix::to_radix_u64;

pub struct CompressCache {
    cache: Mutex<LruCache<String, String>>,
    max_size: usize,
}

impl CompressCache {
    pub fn new(max_size: usize) -> Self {
        CompressCache {
            cache: Mutex::new(LruCache::unbounded()),
            max_size,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().unwrap();
        cache.peek(key).cloned()
    }

    pub fn insert(&self, key: String, value: String) {
        let mut cache = self.cache.lock().unwrap();
        if cache.len() >= self.max_size {
            cache.clear();
        }
        cache.put(key, value);
    }

    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

pub fn compress_with_cache(input: &str, cache_size: usize) -> String {
    static mut CACHE: Option<CompressCache> = None;
    static mut TIMER_SET: bool = false;

    let cache = unsafe {
        if CACHE.is_none() {
            CACHE = Some(CompressCache::new(cache_size));
        }
        CACHE.as_ref().unwrap()
    };

    if let Some(cached) = cache.get(input) {
        return cached;
    }

    let result = if let Ok(num) = input.parse::<u64>() {
        to_radix_u64(num, 255)
    } else {
        let hash = lcg(input);
        to_radix_u64(hash, 255)
    };

    cache.insert(input.to_string(), result.clone());

    unsafe {
        if !TIMER_SET {
            TIMER_SET = true;
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(1));
                unsafe {
                    if let Some(cache) = &CACHE {
                        cache.clear();
                    }
                    TIMER_SET = false;
                }
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
}
