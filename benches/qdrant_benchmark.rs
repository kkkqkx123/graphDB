//! Qdrant Vector Search Benchmarks
//!
//! Benchmark suite for measuring real Qdrant performance with local service.
//! Qdrant must be running on localhost:6333 (HTTP) and localhost:6334 (gRPC).

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use vector_client::types::{CollectionConfig, DistanceMetric, FilterCondition, SearchQuery, VectorFilter, VectorPoint};
use vector_client::{VectorClientConfig, VectorEngine};
use vector_client::engine::QdrantEngine;

fn create_test_vector(size: usize, offset: f32) -> Vec<f32> {
    (0..size)
        .map(|i| (i as f32 + offset) / size as f32)
        .collect()
}

async fn setup_collection(
    engine: &Arc<QdrantEngine>,
    name: &str,
    dim: usize,
    count: usize,
) {
    // 先尝试删除已存在的 collection（如果存在）
    let _ = engine.delete_collection(name).await;
    
    let config = CollectionConfig::new(dim, DistanceMetric::Cosine);
    engine.create_collection(name, config).await.unwrap();

    for i in 0..count {
        let vector = create_test_vector(dim, i as f32);
        // 使用数字 ID 而不是字符串，避免 UUID 解析错误
        let point = VectorPoint::new(i.to_string(), vector);
        engine.upsert(name, point).await.unwrap();
    }
}

async fn cleanup_collection(engine: &Arc<QdrantEngine>, name: &str) {
    let _ = engine.delete_collection(name).await;
}

fn create_engine() -> (tokio::runtime::Runtime, Arc<QdrantEngine>) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let config = VectorClientConfig::qdrant_local("localhost", 6334, 6333);
    let engine = Arc::new(
        rt.block_on(QdrantEngine::new(config))
            .expect("Failed to create QdrantEngine")
    );
    (rt, engine)
}

// ==================== Basic Search Benchmarks ====================

fn bench_search_100_vectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("qdrant_search");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(30);
    group.throughput(Throughput::Elements(1));

    let (rt, engine) = create_engine();
    let collection_name = "bench_100";

    rt.block_on(async {
        setup_collection(&engine, collection_name, 128, 100).await;
        
        for _ in 0..3 {
            let query_vector = create_test_vector(128, 0.5);
            let query = SearchQuery::new(query_vector, 10);
            let _ = engine.search(collection_name, query).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    group.bench_function("search_100_vectors_128d", |b| {
        b.to_async(&rt).iter(|| async {
            let query_vector = create_test_vector(128, 0.5);
            let query = SearchQuery::new(query_vector, 10);
            let results = engine.search(collection_name, query).await.unwrap();
            black_box(results)
        })
    });

    rt.block_on(cleanup_collection(&engine, collection_name));
    group.finish();
}

fn bench_search_1000_vectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("qdrant_search");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(30);
    group.throughput(Throughput::Elements(1));

    let (rt, engine) = create_engine();
    let collection_name = "bench_1000";

    rt.block_on(async {
        setup_collection(&engine, collection_name, 128, 1000).await;
        
        for _ in 0..3 {
            let query_vector = create_test_vector(128, 0.5);
            let query = SearchQuery::new(query_vector, 10);
            let _ = engine.search(collection_name, query).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    group.bench_function("search_1000_vectors_128d", |b| {
        b.to_async(&rt).iter(|| async {
            let query_vector = create_test_vector(128, 0.5);
            let query = SearchQuery::new(query_vector, 10);
            let results = engine.search(collection_name, query).await.unwrap();
            black_box(results)
        })
    });

    rt.block_on(cleanup_collection(&engine, collection_name));
    group.finish();
}

// ==================== Different Dimensions Benchmarks ====================

fn bench_search_different_dimensions(c: &mut Criterion) {
    let mut group = c.benchmark_group("qdrant_dimensions");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(20);

    let (rt, engine) = create_engine();

    for dim in [128, 256, 512] {
        let collection_name = format!("bench_dim_{}", dim);
        
        rt.block_on(async {
            setup_collection(&engine, &collection_name, dim, 1000).await;
            
            for _ in 0..3 {
                let query_vector = create_test_vector(dim, 0.5);
                let query = SearchQuery::new(query_vector, 10);
                let _ = engine.search(&collection_name, query).await;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(dim),
            &dim,
            |b, &dim| {
                b.to_async(&rt).iter(|| async {
                    let query_vector = create_test_vector(dim, 0.5);
                    let query = SearchQuery::new(query_vector, 10);
                    let results = engine.search(&collection_name, query).await.unwrap();
                    black_box(results)
                })
            },
        );

        rt.block_on(cleanup_collection(&engine, &collection_name));
    }

    group.finish();
}

// ==================== Distance Metrics Benchmarks ====================

fn bench_search_different_distance_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("qdrant_distance_metrics");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(20);

    let (rt, engine) = create_engine();

    for metric in [DistanceMetric::Cosine, DistanceMetric::Euclid, DistanceMetric::Dot] {
        let collection_name = format!("bench_metric_{:?}", metric).to_lowercase();
        
        rt.block_on(async {
            let config = CollectionConfig::new(128, metric);
            engine.create_collection(&collection_name, config).await.unwrap();
            
            for i in 0..1000 {
                let vector = create_test_vector(128, i as f32);
                // 使用数字 ID
                let point = VectorPoint::new(i.to_string(), vector);
                engine.upsert(&collection_name, point).await.unwrap();
            }
            
            for _ in 0..3 {
                let query_vector = create_test_vector(128, 0.5);
                let query = SearchQuery::new(query_vector, 10);
                let _ = engine.search(&collection_name, query).await;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", metric)),
            &metric,
            |b, &_metric| {
                b.to_async(&rt).iter(|| async {
                    let query_vector = create_test_vector(128, 0.5);
                    let query = SearchQuery::new(query_vector, 10);
                    let results = engine.search(&collection_name, query).await.unwrap();
                    black_box(results)
                })
            },
        );

        rt.block_on(cleanup_collection(&engine, &collection_name));
    }

    group.finish();
}

// ==================== Batch Operations Benchmarks ====================

fn bench_batch_upsert(c: &mut Criterion) {
    let mut group = c.benchmark_group("qdrant_batch_upsert");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(20);

    let (rt, engine) = create_engine();
    let collection_name = "bench_batch";

    rt.block_on(async {
        let config = CollectionConfig::new(128, DistanceMetric::Cosine);
        engine.create_collection(collection_name, config).await.unwrap();
    });

    for batch_size in [10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &batch_size| {
                b.to_async(&rt).iter(|| async {
                    let points: Vec<VectorPoint> = (0..batch_size)
                        .map(|i| {
                            let vector = create_test_vector(128, i as f32);
                            // 使用数字 ID
                            VectorPoint::new(i.to_string(), vector)
                        })
                        .collect();
                    
                    let result = engine.upsert_batch(collection_name, points).await.unwrap();
                    black_box(result)
                })
            },
        );
    }

    rt.block_on(cleanup_collection(&engine, collection_name));
    group.finish();
}

// ==================== Filter Search Benchmarks ====================

fn bench_filter_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("qdrant_filter_search");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(30);
    group.throughput(Throughput::Elements(1));

    let (rt, engine) = create_engine();
    let collection_name = "bench_filter";

    rt.block_on(async {
        let config = CollectionConfig::new(128, DistanceMetric::Cosine);
        engine.create_collection(collection_name, config).await.unwrap();
        
        for i in 0..1000 {
            let vector = create_test_vector(128, i as f32);
            let mut payload = HashMap::new();
            payload.insert(
                "category".to_string(),
                serde_json::json!(if i % 2 == 0 { "A" } else { "B" }),
            );
            // 使用数字 ID
            let point = VectorPoint::new(i.to_string(), vector).with_payload(payload);
            engine.upsert(collection_name, point).await.unwrap();
        }
        
        for _ in 0..3 {
            let query_vector = create_test_vector(128, 0.5);
            let filter = VectorFilter::new().must(FilterCondition::match_value("category", "A"));
            let query = SearchQuery::new(query_vector, 10).with_filter(filter);
            let _ = engine.search(collection_name, query).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    group.bench_function("search_with_filter", |b| {
        b.to_async(&rt).iter(|| async {
            let query_vector = create_test_vector(128, 0.5);
            let filter = VectorFilter::new().must(FilterCondition::match_value("category", "A"));
            let query = SearchQuery::new(query_vector, 10).with_filter(filter);
            let results = engine.search(collection_name, query).await.unwrap();
            black_box(results)
        })
    });

    rt.block_on(cleanup_collection(&engine, collection_name));
    group.finish();
}

// ==================== Concurrent Operations Benchmarks ====================

fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("qdrant_concurrent");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(15);

    let (rt, engine) = create_engine();
    let collection_name = "bench_concurrent";

    rt.block_on(async {
        setup_collection(&engine, collection_name, 128, 1000).await;
    });

    for concurrency in [1, 4, 8] {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |b, &concurrency| {
                let engine = Arc::clone(&engine);
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for _ in 0..concurrency {
                        let query_vector = create_test_vector(128, 0.5);
                        let query = SearchQuery::new(query_vector, 10);
                        let engine = Arc::clone(&engine);
                        
                        let handle = tokio::spawn(async move {
                            let results = engine.search(collection_name, query).await.unwrap();
                            black_box(results)
                        });
                        
                        handles.push(handle);
                    }
                    
                    let results = futures::future::join_all(handles).await;
                    black_box(results)
                })
            },
        );
    }

    rt.block_on(cleanup_collection(&engine, collection_name));
    group.finish();
}

criterion_group!(
    benches,
    bench_search_100_vectors,
    bench_search_1000_vectors,
    bench_search_different_dimensions,
    bench_search_different_distance_metrics,
    bench_batch_upsert,
    bench_filter_search,
    bench_concurrent_operations,
);

criterion_main!(benches);
