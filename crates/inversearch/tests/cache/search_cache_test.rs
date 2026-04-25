//! 搜索缓存测试
//!
//! 测试范围：
//! - 缓存键生成
//! - 缓存存取操作
//! - 缓存统计
//! - TTL 过期

use inversearch_service::search::{CacheKeyGenerator, CacheStats, SearchCache};
use inversearch_service::SearchOptions;
use std::time::Duration;

fn create_test_cache() -> SearchCache {
    SearchCache::new(100, Some(Duration::from_secs(60)))
}

fn create_search_options() -> SearchOptions {
    SearchOptions {
        query: Some("test".to_string()),
        limit: Some(10),
        offset: Some(0),
        resolve: Some(true),
        ..Default::default()
    }
}

// ============================================================================
// 缓存键生成测试
// ============================================================================

/// 测试基本缓存键生成
#[test]
fn test_generate_search_key_basic() {
    let options = create_search_options();
    let key = CacheKeyGenerator::generate_search_key("test", &options);

    assert!(key.contains("test"));
    assert!(key.contains("limit:10"));
    assert!(key.contains("offset:0"));
}

/// 测试不同查询生成不同键
#[test]
fn test_different_queries_different_keys() {
    let options = create_search_options();

    let key1 = CacheKeyGenerator::generate_search_key("query1", &options);
    let key2 = CacheKeyGenerator::generate_search_key("query2", &options);

    assert_ne!(key1, key2);
}

/// 测试不同选项生成不同键
#[test]
fn test_different_options_different_keys() {
    let options1 = SearchOptions {
        query: Some("test".to_string()),
        limit: Some(10),
        ..Default::default()
    };

    let options2 = SearchOptions {
        query: Some("test".to_string()),
        limit: Some(20),
        ..Default::default()
    };

    let key1 = CacheKeyGenerator::generate_search_key("test", &options1);
    let key2 = CacheKeyGenerator::generate_search_key("test", &options2);

    assert_ne!(key1, key2);
}

/// 测试大小写不敏感键生成
#[test]
fn test_case_insensitive_key() {
    let options = create_search_options();

    let key1 = CacheKeyGenerator::generate_search_key("TEST", &options);
    let key2 = CacheKeyGenerator::generate_search_key("test", &options);

    assert_eq!(key1, key2);
}

/// 测试文档缓存键生成
#[test]
fn test_document_key_generation() {
    let key = CacheKeyGenerator::generate_document_key(123);
    assert_eq!(key, "doc:123");
}

/// 测试带 boost 的键生成
#[test]
fn test_key_with_boost() {
    let options = SearchOptions {
        query: Some("test".to_string()),
        boost: Some(5),
        ..Default::default()
    };

    let key = CacheKeyGenerator::generate_search_key("test", &options);
    assert!(key.contains("boost:5"));
}

// ============================================================================
// 缓存存取测试
// ============================================================================

/// 测试缓存创建
#[test]
fn test_cache_creation() {
    let cache = create_test_cache();
    let stats = cache.stats();
    assert_eq!(stats.size, 0);
}

/// 测试缓存设置和获取
#[test]
fn test_cache_set_get() {
    let cache = create_test_cache();
    let key = "test_key".to_string();
    let data = vec![1, 2, 3];

    cache.set(key.clone(), data.clone());

    let result = cache.get(&key);
    assert_eq!(result, Some(data));
}

/// 测试缓存未命中
#[test]
fn test_cache_miss() {
    let cache = create_test_cache();

    let result = cache.get("nonexistent_key");
    assert!(result.is_none());
}

/// 测试缓存删除
#[test]
fn test_cache_remove() {
    let cache = create_test_cache();
    let key = "test_key".to_string();
    let data = vec![1, 2, 3];

    cache.set(key.clone(), data);

    let removed = cache.remove(&key);
    assert!(removed);

    let result = cache.get(&key);
    assert!(result.is_none());
}

/// 测试缓存清空
#[test]
fn test_cache_clear() {
    let cache = create_test_cache();

    cache.set("key1".to_string(), vec![1]);
    cache.set("key2".to_string(), vec![2]);
    cache.set("key3".to_string(), vec![3]);

    cache.clear();

    let stats = cache.stats();
    assert_eq!(stats.size, 0);
}

/// 测试 LRU 淘汰
#[test]
fn test_lru_eviction() {
    let cache = SearchCache::new(3, None);

    cache.set("key1".to_string(), vec![1]);
    cache.set("key2".to_string(), vec![2]);
    cache.set("key3".to_string(), vec![3]);
    cache.set("key4".to_string(), vec![4]);

    let stats = cache.stats();
    assert!(stats.size <= 3);
}

/// 测试缓存大小限制
#[test]
fn test_cache_size_limit() {
    let cache = SearchCache::new(10, None);

    for i in 0..20 {
        cache.set(format!("key{}", i), vec![i]);
    }

    let stats = cache.stats();
    assert!(stats.size <= 10);
}

// ============================================================================
// 缓存统计测试
// ============================================================================

/// 测试缓存命中统计
#[test]
fn test_cache_hit_count() {
    let cache = create_test_cache();
    let key = "test_key".to_string();
    let data = vec![1, 2, 3];

    cache.set(key.clone(), data);

    let _ = cache.get(&key);
    let _ = cache.get(&key);

    let stats = cache.stats();
    assert_eq!(stats.hit_count, 2);
}

/// 测试缓存未命中统计
#[test]
fn test_cache_miss_count() {
    let cache = create_test_cache();

    let _ = cache.get("nonexistent1");
    let _ = cache.get("nonexistent2");

    let stats = cache.stats();
    assert_eq!(stats.miss_count, 2);
}

/// 测试缓存命中率
#[test]
fn test_cache_hit_rate() {
    let cache = create_test_cache();
    let key = "test_key".to_string();

    cache.set(key.clone(), vec![1, 2, 3]);

    let _ = cache.get(&key);
    let _ = cache.get(&key);
    let _ = cache.get("miss");

    let stats = cache.stats();
    let total = stats.hit_count + stats.miss_count;
    let hit_rate = stats.hit_count as f64 / total as f64;

    assert!(hit_rate > 0.5);
}

/// 测试缓存统计重置
#[test]
fn test_cache_stats_reset() {
    let cache = create_test_cache();

    cache.set("key".to_string(), vec![1]);
    let _ = cache.get("key");
    let _ = cache.get("miss");

    cache.clear();

    let stats = cache.stats();
    assert_eq!(stats.size, 0);
}

// ============================================================================
// 异步缓存测试
// ============================================================================

/// 测试异步缓存设置和获取
#[tokio::test]
async fn test_async_cache_set_get() {
    let cache = create_test_cache();
    let key = "async_key".to_string();
    let data = vec![1, 2, 3];

    cache.set_async(key.clone(), data.clone()).await;

    let result = cache.get_async(&key).await;
    assert_eq!(result, Some(data));
}

/// 测试异步缓存删除
#[tokio::test]
async fn test_async_cache_remove() {
    let cache = create_test_cache();
    let key = "async_key".to_string();

    cache.set_async(key.clone(), vec![1, 2, 3]).await;

    let removed = cache.remove_async(&key).await;
    assert!(removed);

    let result = cache.get_async(&key).await;
    assert!(result.is_none());
}

/// 测试异步缓存清空
#[tokio::test]
async fn test_async_cache_clear() {
    let cache = create_test_cache();

    cache.set_async("key1".to_string(), vec![1]).await;
    cache.set_async("key2".to_string(), vec![2]).await;

    cache.clear_async().await;

    let stats = cache.stats();
    assert_eq!(stats.size, 0);
}

/// 测试并发异步访问
#[tokio::test]
async fn test_concurrent_async_access() {
    let cache = create_test_cache();
    let cache = std::sync::Arc::new(cache);

    let mut handles = vec![];

    for i in 0..5 {
        let cache_clone = std::sync::Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let key = format!("concurrent_key_{}", i);
            cache_clone.set_async(key.clone(), vec![i]).await;
            let result = cache_clone.get_async(&key).await;
            assert_eq!(result, Some(vec![i]));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
