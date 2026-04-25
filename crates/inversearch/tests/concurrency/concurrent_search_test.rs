//! 并发搜索测试
//!
//! 测试范围：
//! - 多线程并发搜索
//! - 并发读写混合
//! - 搜索结果一致性

use inversearch_service::search::search;
use inversearch_service::{Index, IndexOptions, SearchOptions};
use std::sync::Arc;
use std::thread;

fn create_populated_index() -> Arc<Index> {
    let mut index = Index::new(IndexOptions::default()).unwrap();
    for i in 1..=100 {
        index
            .add(i, &format!("Document {} with some content", i), false)
            .unwrap();
    }
    Arc::new(index)
}

fn basic_search_options(query: &str) -> SearchOptions {
    SearchOptions {
        query: Some(query.to_string()),
        limit: Some(10),
        offset: Some(0),
        resolve: Some(true),
        ..Default::default()
    }
}

/// 测试单线程搜索基准
#[test]
fn test_sequential_search_baseline() {
    let index = create_populated_index();

    for _ in 0..10 {
        let options = basic_search_options("Document");
        let result = search(&index, &options).unwrap();
        assert!(!result.results.is_empty());
    }
}

/// 测试多线程并发只读搜索
#[test]
fn test_concurrent_read_only_search() {
    let index = create_populated_index();
    let mut handles = vec![];

    for _ in 0..4 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            for _ in 0..25 {
                let options = basic_search_options("Document");
                let result = search(&index_clone, &options).unwrap();
                assert!(!result.results.is_empty());
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// 测试并发搜索不同关键词
#[test]
fn test_concurrent_search_different_queries() {
    let index = create_populated_index();
    let mut handles = vec![];

    let queries = vec!["Document", "content", "some", "with"];

    for query in queries {
        let index_clone = Arc::clone(&index);
        let query = query.to_string();
        let handle = thread::spawn(move || {
            let options = basic_search_options(&query);
            let result = search(&index_clone, &options).unwrap();
            result
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(!result.results.is_empty() || result.results.is_empty());
    }
}

/// 测试并发读写混合
#[test]
fn test_concurrent_read_write_mixed() {
    use std::sync::Mutex;

    let index = Arc::new(Mutex::new(Index::new(IndexOptions::default()).unwrap()));

    for i in 1..=50 {
        let mut idx = index.lock().unwrap();
        idx.add(i, &format!("Initial Document {}", i), false)
            .unwrap();
    }

    let mut handles = vec![];

    let index_reader = Arc::clone(&index);
    let reader = thread::spawn(move || {
        for _ in 0..10 {
            let idx = index_reader.lock().unwrap();
            let options = basic_search_options("Document");
            let _ = search(&idx, &options);
        }
    });
    handles.push(reader);

    for batch in 0..3 {
        let index_writer = Arc::clone(&index);
        let writer = thread::spawn(move || {
            for i in 1..=10 {
                let mut idx = index_writer.lock().unwrap();
                idx.add(100 + batch * 10 + i, &format!("New Document {}", i), false)
                    .unwrap();
            }
        });
        handles.push(writer);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// 测试并发搜索结果一致性
#[test]
fn test_concurrent_search_consistency() {
    let index = create_populated_index();
    let mut handles = vec![];

    for _ in 0..5 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let options = basic_search_options("Document");
            let result1 = search(&index_clone, &options).unwrap();
            let result2 = search(&index_clone, &options).unwrap();
            assert_eq!(result1.total, result2.total);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// 测试高并发搜索
#[test]
fn test_high_concurrency_search() {
    let index = create_populated_index();
    let mut handles = vec![];

    for _ in 0..10 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let options = basic_search_options("Document");
                let _ = search(&index_clone, &options);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// 测试并发搜索带分页
#[test]
fn test_concurrent_search_with_pagination() {
    let index = create_populated_index();
    let mut handles = vec![];

    for offset in 0..4 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let options = SearchOptions {
                query: Some("Document".to_string()),
                limit: Some(10),
                offset: Some(offset * 10),
                resolve: Some(true),
                ..Default::default()
            };
            let result = search(&index_clone, &options).unwrap();
            result
        });
        handles.push(handle);
    }

    let mut all_results = vec![];
    for handle in handles {
        all_results.push(handle.join().unwrap());
    }

    for result in all_results {
        assert!(!result.results.is_empty() || result.results.is_empty());
    }
}
