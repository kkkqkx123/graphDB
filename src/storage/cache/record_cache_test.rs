//! Record Cache Tests
//!
//! Comprehensive test suite for RecordCache functionality.

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
    };

    cache.insert_vertex(key, vertex);

    let cached = cache.get_vertex(&VertexCacheKey::new(1, 100));
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
    };

    cache.insert_vertex(key, vertex);

    cache.get_vertex(&VertexCacheKey::new(1, 100));
    cache.get_vertex(&VertexCacheKey::new(1, 999));

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
    };

    cache.insert_vertex(key, vertex);
    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100)).is_some());

    cache.remove_vertex(&VertexCacheKey::new(1, 100));
    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100)).is_none());
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
        };
        cache.insert_vertex(key, vertex);
    }

    let stats = cache.stats();
    assert!(stats.vertex.count < 100, "Cache should have evicted entries");
}

#[test]
fn test_id_index_cache() {
    let cache = RecordCache::new();

    cache.insert_id_index(1, "user_001", 100);
    cache.insert_id_index(1, "user_002", 200);
    cache.insert_id_index(2, "product_001", 300);

    assert_eq!(cache.get_id_index(1, "user_001"), Some(100));
    assert_eq!(cache.get_id_index(1, "user_002"), Some(200));
    assert_eq!(cache.get_id_index(2, "product_001"), Some(300));
    assert_eq!(cache.get_id_index(1, "nonexistent"), None);

    cache.remove_id_index(1, "user_001");
    assert_eq!(cache.get_id_index(1, "user_001"), None);
    assert_eq!(cache.get_id_index(1, "user_002"), Some(200));
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
    };
    cache.insert_vertex(key, vertex);

    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100)).is_some());
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
        },
    );

    cache.insert_id_index(1, "user_001", 100);

    cache.get_vertex(&VertexCacheKey::new(1, 100));
    cache.get_vertex(&VertexCacheKey::new(1, 999));

    cache.get_id_index(1, "user_001");
    cache.get_id_index(1, "nonexistent");

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
        cache.insert_id_index(1, &format!("id_{}", i), i);
    }

    assert!(cache.get_id_index(1, "id_50").is_some());
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
        };
        cache.insert_vertex(key, vertex);
    }

    let keys: Vec<VertexCacheKey> = (0..15u32).map(|i| VertexCacheKey::new(1, i)).collect();
    let result = cache.get_vertices_batch(&keys);

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
                },
            )
        })
        .collect();

    let result = cache.insert_vertices_batch(entries);
    assert!(result.inserted > 0);
    assert!(result.total_size > 0);

    let stats = cache.stats();
    assert!(stats.vertex.count > 0);
}

#[test]
fn test_batch_id_indexes() {
    let cache = RecordCache::new();

    cache.insert_id_index(1, "user_001", 100);
    cache.insert_id_index(1, "user_002", 200);
    cache.insert_id_index(2, "product_001", 300);

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
        match cache.get_id_index(*label, id) {
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
fn test_invalidate_batch() {
    let cache = RecordCache::new();

    cache.insert_vertex(
        VertexCacheKey::new(1, 100),
        CachedVertex {
            internal_id: 100,
            external_id: "v1".to_string(),
            properties: vec![],
        },
    );
    cache.insert_id_index(1, "user_001", 100);

    let keys: Vec<CacheKeyRef<'_>> = vec![
        CacheKeyRef::Vertex(VertexCacheKey::new(1, 100)),
        CacheKeyRef::IdIndex(1, "user_001"),
        CacheKeyRef::Vertex(VertexCacheKey::new(1, 999)),
    ];

    let invalidated = cache.invalidate_batch(&keys);
    assert_eq!(invalidated, 2);

    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100)).is_none());
    assert_eq!(cache.get_id_index(1, "user_001"), None);
}

#[test]
fn test_handle_memory_pressure() {
    let config = RecordCacheConfig {
        max_memory: 1024 * 1024,
        ..Default::default()
    };
    let cache = RecordCache::with_config(config)
        .with_memory_pressure_config(MemoryPressureConfig {
            enabled: true,
            ..Default::default()
        });

    let original = cache.max_memory();
    assert_eq!(original, 1024 * 1024);

    for i in 0..100u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("v{}", i),
            properties: vec![],
        };
        cache.insert_vertex(key, vertex);
    }

    let count_before = cache.stats().vertex.count;
    assert!(count_before > 0);

    cache.handle_memory_pressure(MemoryPressureLevel::Critical);

    let count_after = cache.stats().vertex.count;
    assert!(
        count_after <= count_before,
        "Cache count should decrease or stay same after critical pressure"
    );
}

#[test]
fn test_memory_pressure_config() {
    let custom_config = MemoryPressureConfig {
        enabled: true,
        high_watermark: 0.8,
        low_watermark: 0.6,
        reduction_factor: 0.3,
    };

    let cache = RecordCache::new().with_memory_pressure_config(custom_config.clone());

    for i in 0..10u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("v{}", i),
            properties: vec![],
        };
        cache.insert_vertex(key, vertex);
    }

    for i in 0..10u32 {
        let key = VertexCacheKey::new(1, i);
        assert!(cache.get_vertex(&key).is_some(), "Vertex {} should be cached", i);
    }

    let stats = cache.stats();
    assert!(stats.vertex.hits >= 10, "Should have at least 10 hits, got {}", stats.vertex.hits);
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
        },
    );
    cache.insert_vertex(
        VertexCacheKey::new(2, 200),
        CachedVertex {
            internal_id: 200,
            external_id: "v2".to_string(),
            properties: vec![],
        },
    );
    cache.insert_id_index(1, "user_001", 100);
    cache.insert_id_index(2, "user_002", 200);

    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100)).is_some(), "Vertex 1,100 should be cached before invalidation");
    assert!(cache.get_vertex(&VertexCacheKey::new(2, 200)).is_some(), "Vertex 2,200 should be cached before invalidation");

    cache.invalidate_vertices_by_label(1);
    cache.invalidate_id_indexes_by_label(1);

    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100)).is_none(), "Vertex 1,100 should be invalidated");
    assert!(cache.get_vertex(&VertexCacheKey::new(2, 200)).is_some(), "Vertex 2,200 should still be cached");
    assert_eq!(cache.get_id_index(1, "user_001"), None, "ID index 1,user_001 should be invalidated");
    assert_eq!(cache.get_id_index(2, "user_002"), Some(200), "ID index 2,user_002 should still be cached");
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
        };
        cache.insert_vertex(key, vertex);
    }

    let stats = cache.stats();
    assert!(
        stats.memory_usage <= stats.max_memory + 1024,
        "Memory usage {} should not significantly exceed max_memory {}",
        stats.memory_usage,
        stats.max_memory
    );
    assert!(stats.vertex.evictions > 0, "Evictions should have occurred");
}

#[test]
fn test_transaction_rollback_vertex() {
    let cache = RecordCache::new();

    let key = VertexCacheKey::new(1, 100);
    let original_vertex = CachedVertex {
        internal_id: 100,
        external_id: "original".to_string(),
        properties: vec![("name".to_string(), Value::String("Alice".to_string()))],
    };
    cache.insert_vertex(key, original_vertex.clone());

    cache.begin_transaction();

    let modified_vertex = CachedVertex {
        internal_id: 100,
        external_id: "modified".to_string(),
        properties: vec![("name".to_string(), Value::String("Bob".to_string()))],
    };
    cache.insert_vertex(key, modified_vertex);

    let cached = cache.get_vertex(&key);
    assert_eq!(cached.unwrap().external_id, "modified");

    cache.rollback_transaction();

    let rolled_back = cache.get_vertex(&key);
    assert!(rolled_back.is_some());
    assert_eq!(rolled_back.unwrap().external_id, "original");
}

#[test]
fn test_transaction_rollback_new_vertex() {
    let cache = RecordCache::new();

    cache.begin_transaction();

    let key = VertexCacheKey::new(1, 999);
    let vertex = CachedVertex {
        internal_id: 999,
        external_id: "new_vertex".to_string(),
        properties: vec![],
    };
    cache.insert_vertex(key, vertex);

    assert!(cache.get_vertex(&key).is_some());

    cache.rollback_transaction();

    assert!(cache.get_vertex(&key).is_none(), "New vertex should be removed after rollback");
}

#[test]
fn test_transaction_rollback_id_index() {
    let cache = RecordCache::new();

    cache.insert_id_index(1, "user_001", 100);

    cache.begin_transaction();
    cache.insert_id_index(1, "user_001", 200);
    assert_eq!(cache.get_id_index(1, "user_001"), Some(200));

    cache.rollback_transaction();
    assert_eq!(cache.get_id_index(1, "user_001"), Some(100));
}

#[test]
fn test_transaction_commit() {
    let cache = RecordCache::new();

    cache.begin_transaction();
    cache.insert_id_index(1, "user_001", 100);
    cache.commit_transaction();

    assert_eq!(cache.get_id_index(1, "user_001"), Some(100));
}

#[test]
fn test_runtime_config_update() {
    let cache = RecordCache::new();

    assert_eq!(cache.max_memory(), 128 * 1024 * 1024);

    cache.set_max_memory(64 * 1024 * 1024);
    assert_eq!(cache.max_memory(), 64 * 1024 * 1024);

    cache.set_memory_ratio(80, 20);
    assert_eq!(cache.config().memory_ratio, (80, 20));
}

#[test]
fn test_cache_stats_with_uptime() {
    let cache = RecordCache::new();

    let stats = cache.stats();
    assert!(stats.memory_fragmentation_estimate >= 0.0);
}

#[test]
fn test_edge_property_cache_read_tracking() {
    let config = EdgePropertyCacheConfig::enabled();
    let cache = EdgePropertyCache::new(config);

    // Simulate reads (not writes) to meet access frequency threshold
    for _ in 0..10 {
        cache.get(1, "frequent_read");
    }

    // Now put should succeed because reads were tracked
    assert!(cache.put(1, "frequent_read", Value::Int(42)));

    // Property that was never read should not be cached
    assert!(!cache.put(1, "never_read", Value::Int(0)));
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
                };
                cache.insert_vertex(key, vertex);
                let _ = cache.get_vertex(&key);
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
    assert!(estimated < 1000, "Estimated size should be reasonable for small vertex");
}
