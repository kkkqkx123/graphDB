use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::core::Value;

use super::*;

#[test]
fn test_vertex_cache_basic() {
    let cache = RecordCache::new();

    let key = VertexCacheKey::new(1, 100);
    let vertex = CachedVertex {
        internal_id: 100,
        external_id: "test_vertex".to_string(),
        properties: vec![("name".to_string(), Value::String("Alice".to_string()))],
        cached_at_ts: 0,
    };

    cache.insert_vertex(key, vertex);

    let cached = cache.get_vertex(&VertexCacheKey::new(1, 100), 0);
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().external_id, "test_vertex");
}

#[test]
fn test_cache_stats() {
    let cache = RecordCache::new();

    let key = VertexCacheKey::new(1, 100);
    let vertex = CachedVertex {
        internal_id: 100,
        external_id: "test".to_string(),
        properties: vec![],
        cached_at_ts: 0,
    };

    cache.insert_vertex(key, vertex);

    cache.get_vertex(&VertexCacheKey::new(1, 100), 0);
    cache.get_vertex(&VertexCacheKey::new(1, 999), 0);

    let stats = cache.stats();
    assert_eq!(stats.vertex.hits, 1);
    assert_eq!(stats.vertex.misses, 1);
    assert_eq!(stats.total_hits, 1);
    assert_eq!(stats.total_misses, 1);
}

#[test]
fn test_cache_remove() {
    let cache = RecordCache::new();

    let key = VertexCacheKey::new(1, 100);
    let vertex = CachedVertex {
        internal_id: 100,
        external_id: "test".to_string(),
        properties: vec![],
        cached_at_ts: 0,
    };

    cache.insert_vertex(key, vertex);
    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100), 0).is_some());

    cache.remove_vertex(&VertexCacheKey::new(1, 100));
    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100), 0).is_none());
}

#[test]
fn test_cache_clear() {
    let cache = RecordCache::new();

    for i in 0..10u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("v{}", i),
            properties: vec![],
            cached_at_ts: 0,
        };
        cache.insert_vertex(key, vertex);
    }

    cache.clear();

    let stats = cache.stats();
    assert_eq!(stats.vertex.count, 0);
}

#[test]
fn test_memory_weighted_eviction() {
    let config = RecordCacheConfig {
        max_memory: 1024,
        ..Default::default()
    };
    let cache = RecordCache::with_config(config);

    for i in 0..100u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("vertex_{}", i),
            properties: vec![("data".to_string(), Value::String("x".repeat(50)))],
            cached_at_ts: 0,
        };
        cache.insert_vertex(key, vertex);
    }

    let stats = cache.stats();
    assert!(
        stats.vertex.count < 100,
        "Cache should have evicted entries"
    );
}

#[test]
fn test_id_index_cache() {
    let cache = RecordCache::new();

    cache.insert_id_index(1, "user_001", 100, 0);
    cache.insert_id_index(1, "user_002", 200, 0);
    cache.insert_id_index(2, "product_001", 300, 0);

    assert_eq!(cache.get_id_index(1, "user_001", 0), Some(100));
    assert_eq!(cache.get_id_index(1, "user_002", 0), Some(200));
    assert_eq!(cache.get_id_index(2, "product_001", 0), Some(300));
    assert_eq!(cache.get_id_index(1, "nonexistent", 0), None);

    cache.remove_id_index(1, "user_001");
    assert_eq!(cache.get_id_index(1, "user_001", 0), None);
    assert_eq!(cache.get_id_index(1, "user_002", 0), Some(200));
}

#[test]
fn test_cache_config_with_ttl() {
    use std::time::Duration;

    let config = RecordCacheConfig {
        max_memory: 1024 * 1024,
        memory_ratio: (70, 30),
        ttl: Some(Duration::from_secs(60)),
        tti: Some(Duration::from_secs(30)),
        high_priority_ratio: 0.0,
    };
    let cache = RecordCache::with_config(config);

    let key = VertexCacheKey::new(1, 100);
    let vertex = CachedVertex {
        internal_id: 100,
        external_id: "test".to_string(),
        properties: vec![],
        cached_at_ts: 0,
    };
    cache.insert_vertex(key, vertex);

    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100), 0).is_some());
}

#[test]
fn test_fine_grained_stats() {
    let cache = RecordCache::new();

    cache.insert_vertex(
        VertexCacheKey::new(1, 100),
        CachedVertex {
            internal_id: 100,
            external_id: "v1".to_string(),
            properties: vec![],
            cached_at_ts: 0,
        },
    );

    cache.insert_id_index(1, "user_001", 100, 0);

    cache.get_vertex(&VertexCacheKey::new(1, 100), 0);
    cache.get_vertex(&VertexCacheKey::new(1, 999), 0);

    cache.get_id_index(1, "user_001", 0);
    cache.get_id_index(1, "nonexistent", 0);

    let stats = cache.stats();

    assert_eq!(stats.vertex.hits, 1);
    assert_eq!(stats.vertex.misses, 1);

    assert_eq!(stats.id_index.hits, 1);
    assert_eq!(stats.id_index.misses, 1);

    assert_eq!(stats.total_hits, 2);
    assert_eq!(stats.total_misses, 2);
}

#[test]
fn test_high_priority_pool() {
    let config = RecordCacheConfig {
        max_memory: 1024 * 1024,
        memory_ratio: (70, 30),
        high_priority_ratio: 0.1,
        ..Default::default()
    };
    let cache = RecordCache::with_config(config);

    for i in 0..100u32 {
        cache.insert_id_index(1, &format!("id_{}", i), i, 0);
    }

    assert!(cache.get_id_index(1, "id_50", 0).is_some());
}

#[test]
fn test_eviction_callback() {
    let eviction_count = Arc::new(AtomicUsize::new(0));
    let eviction_count_clone = eviction_count.clone();

    let callback = Arc::new(move |_cache_type: &str, _cause: EvictionCause| {
        eviction_count_clone.fetch_add(1, Ordering::Relaxed);
    });

    let config = RecordCacheConfig {
        max_memory: 1024,
        ..Default::default()
    };
    let cache = RecordCache::with_config(config).with_eviction_callback(callback);

    for i in 0..100u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("vertex_{}", i),
            properties: vec![("data".to_string(), Value::String("x".repeat(50)))],
            cached_at_ts: 0,
        };
        cache.insert_vertex(key, vertex);
    }

    cache.clear();

    let _stats = cache.stats();
    assert!(
        eviction_count.load(Ordering::Relaxed) > 0,
        "Eviction callback should have been called"
    );
}

#[test]
fn test_cache_type_stats() {
    let stats = crate::core::stats::CacheStats::new();

    stats.record_hit();
    stats.record_hit();
    stats.record_miss();
    stats.record_eviction();

    assert_eq!(stats.hits(), 2);
    assert_eq!(stats.misses(), 1);
    assert_eq!(stats.evictions(), 1);

    let hit_rate = stats.hit_rate();
    assert!((hit_rate - 0.666).abs() < 0.01);

    stats.reset();
    assert_eq!(stats.hits(), 0);
    assert_eq!(stats.misses(), 0);
    assert_eq!(stats.evictions(), 0);
}

#[test]
fn test_batch_get_vertices() {
    let cache = RecordCache::new();

    for i in 0..10u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("v{}", i),
            properties: vec![],
            cached_at_ts: 0,
        };
        cache.insert_vertex(key, vertex);
    }

    let keys: Vec<VertexCacheKey> = (0..15u32).map(|i| VertexCacheKey::new(1, i)).collect();
    let result = cache.get_vertices_batch(&keys, 0);

    assert_eq!(result.results.len(), 15);
    assert_eq!(result.hits, 10);
    assert_eq!(result.misses, 5);

    for i in 0..10 {
        assert!(result.results[i].is_some());
        assert_eq!(result.results[i].as_ref().unwrap().internal_id, i as u32);
    }
    for i in 10..15 {
        assert!(result.results[i].is_none());
    }
}

#[test]
fn test_batch_get_vertices_timestamp() {
    let cache = RecordCache::new();

    let key = VertexCacheKey::new(1, 1);
    cache.insert_vertex(
        key,
        CachedVertex {
            internal_id: 1,
            external_id: "v1".to_string(),
            properties: vec![],
            cached_at_ts: 10,
        },
    );

    // query at ts=5 should NOT see entry cached at ts=10
    let result = cache.get_vertices_batch(&[VertexCacheKey::new(1, 1)], 5);
    assert!(result.results[0].is_none());
    assert_eq!(result.misses, 1);

    // query at ts=10 SHOULD see it
    let result = cache.get_vertices_batch(&[VertexCacheKey::new(1, 1)], 10);
    assert!(result.results[0].is_some());
    assert_eq!(result.hits, 1);
}

#[test]
fn test_batch_insert_vertices() {
    let cache = RecordCache::new();

    let entries: Vec<(VertexCacheKey, CachedVertex)> = (0..100u32)
        .map(|i| {
            (
                VertexCacheKey::new(1, i),
                CachedVertex {
                    internal_id: i,
                    external_id: format!("v{}", i),
                    properties: vec![],
                    cached_at_ts: 0,
                },
            )
        })
        .collect();

    let result = cache.insert_vertices_batch(entries);
    assert_eq!(result.inserted, 100);
    assert!(result.total_size > 0);

    let stats = cache.stats();
    assert!(stats.vertex.count > 0);
    assert!(stats.vertex.count <= 100);
}

#[test]
fn test_batch_id_indexes() {
    let cache = RecordCache::new();

    cache.insert_id_index(1, "user_001", 100, 0);
    cache.insert_id_index(1, "user_002", 200, 0);
    cache.insert_id_index(2, "product_001", 300, 0);

    let keys: Vec<(u32, &str)> = vec![
        (1, "user_001"),
        (1, "user_002"),
        (2, "product_001"),
        (1, "nonexistent"),
    ];

    let mut results = Vec::new();
    let mut hits = 0usize;
    let mut misses = 0usize;

    for (label, id) in &keys {
        match cache.get_id_index(*label, id, 0) {
            Some(internal_id) => {
                hits += 1;
                results.push(Some(internal_id));
            }
            None => {
                misses += 1;
                results.push(None);
            }
        }
    }

    assert_eq!(results.len(), 4);
    assert_eq!(hits, 3);
    assert_eq!(misses, 1);
    assert_eq!(results[0], Some(100));
    assert_eq!(results[1], Some(200));
    assert_eq!(results[2], Some(300));
    assert_eq!(results[3], None);
}

#[test]
fn test_invalidate_by_label() {
    let cache = RecordCache::new();

    cache.insert_vertex(
        VertexCacheKey::new(1, 100),
        CachedVertex {
            internal_id: 100,
            external_id: "v1".to_string(),
            properties: vec![],
            cached_at_ts: 0,
        },
    );
    cache.insert_vertex(
        VertexCacheKey::new(2, 200),
        CachedVertex {
            internal_id: 200,
            external_id: "v2".to_string(),
            properties: vec![],
            cached_at_ts: 0,
        },
    );
    cache.insert_id_index(1, "user_001", 100, 0);
    cache.insert_id_index(2, "user_002", 200, 0);

    assert!(
        cache.get_vertex(&VertexCacheKey::new(1, 100), 0).is_some(),
        "Vertex 1,100 should be cached before invalidation"
    );
    assert!(
        cache.get_vertex(&VertexCacheKey::new(2, 200), 0).is_some(),
        "Vertex 2,200 should be cached before invalidation"
    );

    cache.invalidate_vertices_by_label(1);
    cache.invalidate_id_indexes_by_label(1);

    assert!(
        cache.get_vertex(&VertexCacheKey::new(1, 100), 0).is_none(),
        "Vertex 1,100 should be invalidated"
    );
    assert!(
        cache.get_vertex(&VertexCacheKey::new(2, 200), 0).is_some(),
        "Vertex 2,200 should still be cached"
    );
    assert_eq!(
        cache.get_id_index(1, "user_001", 0),
        None,
        "ID index 1,user_001 should be invalidated"
    );
    assert_eq!(
        cache.get_id_index(2, "user_002", 0),
        Some(200),
        "ID index 2,user_002 should still be cached"
    );
}

#[test]
fn test_memory_overflow_eviction() {
    let config = RecordCacheConfig {
        max_memory: 512,
        ..Default::default()
    };
    let cache = RecordCache::with_config(config);

    for i in 0..200u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("v{}", i),
            properties: vec![("data".to_string(), Value::String("x".repeat(100)))],
            cached_at_ts: 0,
        };
        cache.insert_vertex(key, vertex);
    }

    let stats = cache.stats();
    assert!(stats.vertex.evictions > 0, "Evictions should have occurred");
}

#[test]
fn test_timestamp_staleness() {
    let cache = RecordCache::new();

    let key = VertexCacheKey::new(1, 42);
    let vertex = CachedVertex {
        internal_id: 42,
        external_id: "fresh".to_string(),
        properties: vec![],
        cached_at_ts: 100,
    };
    cache.insert_vertex(key, vertex);

    // query_ts < cached_at_ts → miss (data from future)
    assert!(cache.get_vertex(&key, 50).is_none());

    // query_ts == cached_at_ts → hit
    assert!(cache.get_vertex(&key, 100).is_some());

    // query_ts > cached_at_ts → hit
    assert!(cache.get_vertex(&key, 200).is_some());
}

#[test]
fn test_id_index_timestamp_staleness() {
    let cache = RecordCache::new();

    cache.insert_id_index(1, "user", 42, 100);

    // query_ts < cached_at_ts → miss
    assert_eq!(cache.get_id_index(1, "user", 50), None);

    // query_ts == cached_at_ts → hit
    assert_eq!(cache.get_id_index(1, "user", 100), Some(42));

    // query_ts > cached_at_ts → hit
    assert_eq!(cache.get_id_index(1, "user", 200), Some(42));
}

#[test]
fn test_concurrent_cache_access() {
    use std::thread;

    let cache = Arc::new(RecordCache::new());
    let mut handles = vec![];

    for t in 0..4 {
        let cache = cache.clone();
        let handle = thread::spawn(move || {
            for i in 0..100u32 {
                let key = VertexCacheKey::new(t, i);
                let vertex = CachedVertex {
                    internal_id: i,
                    external_id: format!("t{}_v{}", t, i),
                    properties: vec![],
                    cached_at_ts: 0,
                };
                cache.insert_vertex(key, vertex);
                let _ = cache.get_vertex(&key, 0);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    let stats = cache.stats();
    assert!(stats.total_hits + stats.total_misses > 0);
}

#[test]
fn test_estimated_size_accuracy() {
    let vertex = CachedVertex {
        internal_id: 1,
        external_id: "test_vertex".to_string(),
        properties: vec![
            ("name".to_string(), Value::String("Alice".to_string())),
            ("age".to_string(), Value::Int(30)),
        ],
        cached_at_ts: 0,
    };

    let estimated = vertex.estimated_size();
    let base_size = std::mem::size_of::<CachedVertex>() as u32;
    let external_cap = vertex.external_id.capacity() as u32;
    let mut property_size = 0u32;
    for (name, value) in &vertex.properties {
        property_size += name.capacity() as u32;
        property_size += value.estimated_size() as u32;
    }

    assert_eq!(estimated, base_size + external_cap + property_size);
    assert!(estimated > 0);
    assert!(
        estimated < 1000,
        "Estimated size should be reasonable for small vertex"
    );
}

#[test]
fn test_config_readonly() {
    let cache = RecordCache::new();
    let cfg = cache.config();
    assert_eq!(cfg.max_memory, 128 * 1024 * 1024);
}
