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
fn test_edge_query_cache_basic() {
    let cache = RecordCache::new();

    let key = EdgeQueryKey::new(1, 100, 200);
    let edge = CachedEdge {
        edge_id: 1,
        src_vid: 100,
        dst_vid: 200,
        properties: vec![("weight".to_string(), Value::Double(1.5))],
    };

    cache.insert_edge_query(key, edge);

    let cached = cache.get_edge_by_query(&EdgeQueryKey::new(1, 100, 200));
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().edge_id, 1);
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
        memory_ratio: (40, 0, 40, 20),
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

    cache.insert_edge_query(
        EdgeQueryKey::new(1, 100, 200),
        CachedEdge {
            edge_id: 1,
            src_vid: 100,
            dst_vid: 200,
            properties: vec![],
        },
    );

    cache.insert_id_index(1, "user_001", 100);

    cache.get_vertex(&VertexCacheKey::new(1, 100));
    cache.get_vertex(&VertexCacheKey::new(1, 999));

    cache.get_edge_by_query(&EdgeQueryKey::new(1, 100, 200));
    cache.get_edge_by_query(&EdgeQueryKey::new(1, 999, 200));

    cache.get_id_index(1, "user_001");
    cache.get_id_index(1, "nonexistent");

    let stats = cache.stats();

    assert_eq!(stats.vertex.hits, 1);
    assert_eq!(stats.vertex.misses, 1);

    assert_eq!(stats.edge_query.hits, 1);
    assert_eq!(stats.edge_query.misses, 1);

    assert_eq!(stats.id_index.hits, 1);
    assert_eq!(stats.id_index.misses, 1);

    assert_eq!(stats.total_hits, 3);
    assert_eq!(stats.total_misses, 3);
}

#[test]
fn test_high_priority_pool() {
    let config = RecordCacheConfig {
        max_memory: 1024 * 1024,
        memory_ratio: (40, 0, 40, 20),
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
    let stats = CacheTypeStats::new();

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
fn test_batch_get_edges() {
    let cache = RecordCache::new();

    for i in 0..5u64 {
        let key = EdgeQueryKey::new(1, i, i + 1);
        let edge = CachedEdge {
            edge_id: i,
            src_vid: i,
            dst_vid: i + 1,
            properties: vec![],
        };
        cache.insert_edge_query(key, edge);
    }

    let keys: Vec<EdgeQueryKey> = (0..8u64).map(|i| EdgeQueryKey::new(1, i, i + 1)).collect();
    let result = cache.get_edge_queries_batch(&keys);

    assert_eq!(result.results.len(), 8);
    assert_eq!(result.hits, 5);
    assert_eq!(result.misses, 3);
}

#[test]
fn test_batch_id_indexes() {
    let cache = RecordCache::new();

    cache.insert_id_index(1, "user_001", 100);
    cache.insert_id_index(1, "user_002", 200);
    cache.insert_id_index(2, "product_001", 300);

    let keys: Vec<(u16, &str)> = vec![
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
    cache.insert_edge_query(
        EdgeQueryKey::new(1, 100, 200),
        CachedEdge {
            edge_id: 1,
            src_vid: 100,
            dst_vid: 200,
            properties: vec![],
        },
    );
    cache.insert_id_index(1, "user_001", 100);

    let keys: Vec<CacheKeyRef<'_>> = vec![
        CacheKeyRef::Vertex(VertexCacheKey::new(1, 100)),
        CacheKeyRef::EdgeQuery(EdgeQueryKey::new(1, 100, 200)),
        CacheKeyRef::IdIndex(1, "user_001"),
        CacheKeyRef::Vertex(VertexCacheKey::new(1, 999)),
    ];

    let invalidated = cache.invalidate_batch(&keys);
    assert_eq!(invalidated, 3);

    assert!(cache.get_vertex(&VertexCacheKey::new(1, 100)).is_none());
    assert!(cache.get_edge_by_query(&EdgeQueryKey::new(1, 100, 200)).is_none());
    assert_eq!(cache.get_id_index(1, "user_001"), None);
}

#[test]
fn test_memory_pressure_level() {
    let config = RecordCacheConfig {
        max_memory: 1024,
        ..Default::default()
    };
    let cache = RecordCache::with_config(config);

    assert_eq!(
        cache.memory_pressure_level(),
        MemoryPressureLevel::Normal
    );

    for i in 0..50u32 {
        let key = VertexCacheKey::new(1, i);
        let vertex = CachedVertex {
            internal_id: i,
            external_id: format!("vertex_{}", i),
            properties: vec![("data".to_string(), Value::String("x".repeat(20)))],
        };
        cache.insert_vertex(key, vertex);
    }

    let utilization = cache.utilization();
    let pressure = cache.memory_pressure_level();

    if utilization >= 0.9 {
        assert_eq!(pressure, MemoryPressureLevel::Critical);
    } else if utilization >= 0.7 {
        assert_eq!(pressure, MemoryPressureLevel::Warning);
    } else {
        assert_eq!(pressure, MemoryPressureLevel::Normal);
    }
}

#[test]
fn test_handle_memory_pressure() {
    let config = RecordCacheConfig {
        max_memory: 1024 * 1024,
        ..Default::default()
    };
    let mut cache = RecordCache::with_config(config)
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

    let pressure_level = cache.memory_pressure_level();
    assert_eq!(pressure_level, MemoryPressureLevel::Normal);
}

#[test]
fn test_hit_rate_predictor_basic() {
    let mut predictor = HitRatePredictor::new(1000, 1024 * 1024);

    for i in 0..100 {
        let access = CacheAccess {
            cache_type: CacheAccessType::Vertex,
            key_hash: i as u64,
            size: 1024,
            timestamp: std::time::Instant::now(),
        };
        predictor.record_access(access);
    }

    assert_eq!(predictor.access_count(), 100);
}

#[test]
fn test_hit_rate_predictor_prediction() {
    let mut predictor = HitRatePredictor::new(1000, 1024 * 1024);

    for i in 0..10 {
        for _ in 0..10 {
            let access = CacheAccess {
                cache_type: CacheAccessType::Vertex,
                key_hash: i as u64,
                size: 1024,
                timestamp: std::time::Instant::now(),
            };
            predictor.record_access(access);
        }
    }

    let result = predictor.predict_for_capacity(10 * 1024);
    assert!(result.predicted_hit_rate > 0.0);
    assert_eq!(result.recommended_capacity, 10 * 1024);

    let result = predictor.predict_for_capacity(5 * 1024);
    assert!(result.predicted_hit_rate >= 0.0);
}

#[test]
fn test_hit_rate_predictor_optimal_capacity() {
    let mut predictor = HitRatePredictor::new(1000, 1024 * 1024);

    for i in 0..20 {
        for _ in 0..5 {
            let access = CacheAccess {
                cache_type: CacheAccessType::Vertex,
                key_hash: i as u64,
                size: 1024,
                timestamp: std::time::Instant::now(),
            };
            predictor.record_access(access);
        }
    }

    let result = predictor.find_optimal_capacity(0.5);
    assert!(result.is_some());

    let result = result.unwrap();
    assert!(result.predicted_hit_rate >= 0.5);
}

#[test]
fn test_hit_rate_predictor_clear() {
    let mut predictor = HitRatePredictor::new(100, 1024 * 1024);

    for i in 0..50 {
        let access = CacheAccess {
            cache_type: CacheAccessType::Vertex,
            key_hash: i as u64,
            size: 1024,
            timestamp: std::time::Instant::now(),
        };
        predictor.record_access(access);
    }

    assert_eq!(predictor.access_count(), 50);

    predictor.clear_history();
    assert_eq!(predictor.access_count(), 0);
}

#[test]
fn test_hit_rate_predictor_history_limit() {
    let mut predictor = HitRatePredictor::new(10, 1024 * 1024);

    for i in 0..20 {
        let access = CacheAccess {
            cache_type: CacheAccessType::Vertex,
            key_hash: i as u64,
            size: 1024,
            timestamp: std::time::Instant::now(),
        };
        predictor.record_access(access);
    }

    assert_eq!(predictor.access_count(), 10);
}
