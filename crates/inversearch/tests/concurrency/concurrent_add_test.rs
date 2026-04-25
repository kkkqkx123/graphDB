//! 并发添加文档测试
//!
//! 测试范围：
//! - 多线程并发添加
//! - 并发写入安全性
//! - 并发添加后数据一致性

use inversearch_service::{Index, IndexOptions};
use std::sync::Arc;
use std::thread;

/// 测试单线程批量添加基准
#[test]
fn test_sequential_add_baseline() {
    let mut index = Index::new(IndexOptions::default()).unwrap();

    for i in 1..=100 {
        index.add(i, &format!("Document {}", i), false).unwrap();
    }

    for i in 1..=100 {
        assert!(index.contains(i), "文档 {} 应该存在", i);
    }
}

/// 测试多线程并发添加（使用互斥锁）
#[test]
fn test_concurrent_add_with_mutex() {
    use std::sync::Mutex;

    let index = Arc::new(Mutex::new(Index::new(IndexOptions::default()).unwrap()));
    let mut handles = vec![];

    for thread_id in 0..4 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            for i in 1..=25 {
                let doc_id = thread_id * 25 + i;
                let mut idx = index_clone.lock().unwrap();
                idx.add(
                    doc_id as u64,
                    &format!("Thread {} Document {}", thread_id, i),
                    false,
                )
                .unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let idx = index.lock().unwrap();
    for i in 1..=100 {
        assert!(idx.contains(i as u64), "文档 {} 应该存在", i);
    }
}

/// 测试并发添加不重复 ID
#[test]
fn test_concurrent_add_unique_ids() {
    use std::sync::Mutex;

    let index = Arc::new(Mutex::new(Index::new(IndexOptions::default()).unwrap()));
    let mut handles = vec![];

    for thread_id in 0..4 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let start_id = (thread_id * 100 + 1) as u64;
            for i in 0..100 {
                let doc_id = start_id + i;
                let mut idx = index_clone.lock().unwrap();
                idx.add(doc_id, &format!("Document {}", doc_id), false)
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let idx = index.lock().unwrap();
    let mut count = 0;
    for thread_id in 0..4 {
        let start_id = (thread_id * 100 + 1) as u64;
        for i in 0..100 {
            let doc_id = start_id + i;
            if idx.contains(doc_id) {
                count += 1;
            }
        }
    }
    assert_eq!(count, 400, "应该有 400 个文档");
}

/// 测试并发添加和验证
#[test]
fn test_concurrent_add_and_verify() {
    use std::sync::Mutex;

    let index = Arc::new(Mutex::new(Index::new(IndexOptions::default()).unwrap()));
    let mut handles = vec![];

    for batch in 0..2 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            for i in 1..=50 {
                let doc_id = batch * 50 + i;
                let mut idx = index_clone.lock().unwrap();
                idx.add(doc_id as u64, &format!("Batch {} Doc {}", batch, i), false)
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let idx = index.lock().unwrap();
    let mut found = 0;
    for i in 1..=100 {
        if idx.contains(i as u64) {
            found += 1;
        }
    }
    assert_eq!(found, 100);
}

/// 测试并发添加相同 ID（后写入胜出）
#[test]
fn test_concurrent_add_same_id() {
    use std::sync::Mutex;

    let index = Arc::new(Mutex::new(Index::new(IndexOptions::default()).unwrap()));
    let mut handles = vec![];

    for thread_id in 0..3 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let mut idx = index_clone.lock().unwrap();
            idx.add(1, &format!("Thread {} content", thread_id), false)
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let idx = index.lock().unwrap();
    assert!(idx.contains(1), "文档 1 应该存在");
}

/// 测试并发添加大量文档
#[test]
fn test_concurrent_add_large_batch() {
    use std::sync::Mutex;

    let index = Arc::new(Mutex::new(Index::new(IndexOptions::default()).unwrap()));
    let mut handles = vec![];

    for thread_id in 0..4 {
        let index_clone = Arc::clone(&index);
        let handle = thread::spawn(move || {
            let start_id = (thread_id * 250 + 1) as u64;
            for i in 0..250 {
                let doc_id = start_id + i;
                let mut idx = index_clone.lock().unwrap();
                idx.add(doc_id, &format!("Document {}", doc_id), false)
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let idx = index.lock().unwrap();
    let mut count = 0;
    for i in 1..=1000 {
        if idx.contains(i as u64) {
            count += 1;
        }
    }
    assert_eq!(count, 1000);
}
