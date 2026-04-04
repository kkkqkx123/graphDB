use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use graphdb::coordinator::FulltextCoordinator;
use graphdb::core::vertex_edge_path::Tag;
use graphdb::core::{Value, Vertex};
use graphdb::search::config::FulltextConfig;
use graphdb::search::engine::EngineType;
use graphdb::search::manager::FulltextIndexManager;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn create_test_vertex(vid: i64, tag_name: &str, properties: Vec<(&str, &str)>) -> Vertex {
    let mut props = HashMap::new();
    for (key, value) in properties {
        props.insert(key.to_string(), Value::String(value.to_string()));
    }
    let tag = Tag {
        name: tag_name.to_string(),
        properties: props,
    };
    Vertex::new(Value::Int(vid), vec![tag])
}

async fn setup_coordinator() -> (FulltextCoordinator, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = FulltextConfig {
        enabled: true,
        index_path: temp_dir.path().to_path_buf(),
        default_engine: EngineType::Bm25,
        sync: Default::default(),
        bm25: Default::default(),
        inversearch: Default::default(),
        cache_size: 100,
        max_result_cache: 1000,
        result_cache_ttl_secs: 60,
    };
    let manager = Arc::new(FulltextIndexManager::new(config).expect("Failed to create manager"));
    let coordinator = FulltextCoordinator::new(manager);
    (coordinator, temp_dir)
}

fn bench_indexing(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    let mut group = c.benchmark_group("indexing");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("bm25", size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let (coordinator, _temp) = setup_coordinator().await;
                    coordinator
                        .create_index(1, "Doc", "content", Some(EngineType::Bm25))
                        .await
                        .expect("Failed to create index");

                    for i in 0..size {
                        let vertex = create_test_vertex(
                            i as i64,
                            "Doc",
                            vec![("content", &format!("Document content number {}", i))],
                        );
                        coordinator
                            .on_vertex_inserted(1, &vertex)
                            .await
                            .expect("Failed to insert");
                    }

                    coordinator.commit_all().await.expect("Failed to commit");
                    black_box(&coordinator);
                });
            });
        });
    }

    group.finish();
}

fn bench_search(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    let mut group = c.benchmark_group("search");

    for doc_count in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("single_term", doc_count),
            doc_count,
            |b, &doc_count| {
                let coordinator = rt.block_on(async {
                    let (coordinator, _temp) = setup_coordinator().await;
                    coordinator
                        .create_index(1, "Doc", "content", Some(EngineType::Bm25))
                        .await
                        .expect("Failed to create index");

                    // Prepare data
                    for i in 0..doc_count {
                        let vertex = create_test_vertex(
                            i as i64,
                            "Doc",
                            vec![("content", &format!("Content {}", i))],
                        );
                        coordinator
                            .on_vertex_inserted(1, &vertex)
                            .await
                            .expect("Failed to insert");
                    }
                    coordinator.commit_all().await.expect("Failed to commit");
                    coordinator
                });

                b.iter(|| {
                    rt.block_on(async {
                        let _ = coordinator.search(1, "Doc", "content", "Content", 10).await;
                    });
                });
            },
        );
    }

    group.finish();
}

fn bench_engine_comparison(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    let mut group = c.benchmark_group("engine_comparison");

    for engine_type in [EngineType::Bm25, EngineType::Inversearch].iter() {
        let engine_name = format!("{:?}", engine_type);

        group.bench_function(format!("{}_index", engine_name), |b| {
            b.iter(|| {
                rt.block_on(async {
                    let (coordinator, _temp) = setup_coordinator().await;
                    coordinator
                        .create_index(1, "Doc", "content", Some(*engine_type))
                        .await
                        .expect("Failed to create index");

                    for i in 0..1000 {
                        let vertex = create_test_vertex(
                            i as i64,
                            "Doc",
                            vec![("content", &format!("Test content {}", i))],
                        );
                        coordinator
                            .on_vertex_inserted(1, &vertex)
                            .await
                            .expect("Failed to insert");
                    }
                    coordinator.commit_all().await.expect("Failed to commit");
                    black_box(&coordinator);
                });
            });
        });
    }

    group.finish();
}

fn bench_batch_operations(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    let mut group = c.benchmark_group("batch_operations");

    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("commit", batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    rt.block_on(async {
                        let (coordinator, _temp) = setup_coordinator().await;
                        coordinator
                            .create_index(1, "Doc", "content", Some(EngineType::Bm25))
                            .await
                            .expect("Failed to create index");

                        // Insert in batches
                        for batch in 0..10 {
                            for i in 0..batch_size {
                                let vid = batch * batch_size + i;
                                let vertex = create_test_vertex(
                                    vid as i64,
                                    "Doc",
                                    vec![("content", &format!("Batch content {}", vid))],
                                );
                                coordinator
                                    .on_vertex_inserted(1, &vertex)
                                    .await
                                    .expect("Failed to insert");
                            }
                            coordinator.commit_all().await.expect("Failed to commit");
                        }
                        black_box(&coordinator);
                    });
                });
            },
        );
    }

    group.finish();
}

fn bench_concurrent_search(c: &mut Criterion) {
    let rt = Runtime::new().expect("Failed to create runtime");

    let mut group = c.benchmark_group("concurrent_search");

    // Setup data once
    let coordinator = rt.block_on(async {
        let (coordinator, _temp) = setup_coordinator().await;
        coordinator
            .create_index(1, "Doc", "content", Some(EngineType::Bm25))
            .await
            .expect("Failed to create index");

        for i in 0..10000 {
            let vertex = create_test_vertex(
                i as i64,
                "Doc",
                vec![("content", &format!("Content {} with keywords", i))],
            );
            coordinator
                .on_vertex_inserted(1, &vertex)
                .await
                .expect("Failed to insert");
        }
        coordinator.commit_all().await.expect("Failed to commit");
        coordinator
    });

    group.bench_function("single_thread", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..100 {
                    let _ = coordinator
                        .search(1, "Doc", "content", "keywords", 10)
                        .await;
                }
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_indexing,
    bench_search,
    bench_engine_comparison,
    bench_batch_operations,
    bench_concurrent_search
);
criterion_main!(benches);
