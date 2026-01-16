//! 错误处理测试用例
//!
//!

use graphdb::core::error::DBError;
use graphdb::utils::error_handling::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
fn test_safe_lock_success() {
    let mutex = Mutex::new(42);
    let guard = safe_lock(&mutex).expect("Safe lock should succeed");
    assert_eq!(*guard, 42);
}

#[test]
fn test_safe_lock_poisoned() {
    use std::sync::mpsc;
    use std::thread;

    let mutex = Arc::new(Mutex::new(42));
    let mutex_clone = mutex.clone();

    // 在另一个线程中故意污染锁
    let (tx, rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        let _guard = mutex_clone
            .lock()
            .expect("Mutex should be accessible in test thread");
        // 故意 panic 来污染锁
        panic!("Intentional panic to poison the lock");
    });

    // 等待线程完成（应该会因为 panic 而结束）
    let _ = rx.recv();

    // 给一点时间让锁被污染
    thread::sleep(std::time::Duration::from_millis(10));

    // 测试安全锁获取
    let result = safe_lock(&mutex);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DBError::Lock(_)));
}

#[test]
fn test_expect_option_some() {
    let option = Some(42);
    let result = expect_option(option, "Should have value");
    assert_eq!(result.expect("Expect option should succeed"), 42);
}

#[test]
fn test_expect_option_none() {
    let option: Option<i32> = None;
    let result = expect_option(option, "Value should exist");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DBError::Internal(_)));
}

#[test]
fn test_expect_vec_last() {
    let vec = vec![1, 2, 3];
    let result = expect_vec_last(&vec, "Vector should not be empty");
    assert_eq!(result.expect("Expect vec last should succeed"), &3);
}

#[test]
fn test_expect_vec_last_empty() {
    let vec: Vec<i32> = vec![];
    let result = expect_vec_last(&vec, "Vector should not be empty");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DBError::Internal(_)));
}

#[test]
fn test_expect_first() {
    let vec = vec![1, 2, 3];
    let result = expect_first(vec.iter(), "Iterator should not be empty");
    assert_eq!(result.expect("Expect first should succeed"), &1);
}

#[test]
fn test_expect_first_empty() {
    let vec: Vec<i32> = vec![];
    let result = expect_first(vec.iter(), "Iterator should not be empty");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DBError::Internal(_)));
}

#[test]
fn test_expect_min() {
    let vec = vec![3, 1, 2];
    let result = expect_min(vec.iter(), "Iterator should not be empty");
    assert_eq!(result.expect("Expect min should succeed"), &1);
}

#[test]
fn test_expect_max() {
    let vec = vec![1, 3, 2];
    let result = expect_max(vec.iter(), "Iterator should not be empty");
    assert_eq!(result.expect("Expect max should succeed"), &3);
}

#[test]
fn test_expect_arc_mut() {
    let mut arc = Arc::new(42);
    let result = expect_arc_mut(&mut arc, "Should have unique reference");
    assert_eq!(*result.expect("Expect arc mut should succeed"), 42);
}

#[test]
fn test_expect_arc_mut_shared() {
    let mut arc_mut = Arc::new(42);
    let _shared = arc_mut.clone(); // 创建共享引用
    let result = expect_arc_mut(&mut arc_mut, "Should have unique reference");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DBError::Internal(_)));
}

#[test]
fn test_error_propagation() {
    use std::sync::mpsc;
    use std::thread;

    // 测试错误传播
    let mutex = Arc::new(Mutex::new(42));
    let mutex_clone = mutex.clone();

    // 在另一个线程中故意污染锁
    let (tx, rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        let _guard = mutex_clone
            .lock()
            .expect("Mutex should be accessible in test thread");
        panic!("Intentional panic to poison the lock");
    });

    // 等待线程完成
    let _ = rx.recv();

    // 给一点时间让锁被污染
    thread::sleep(std::time::Duration::from_millis(10));

    // 错误应该传播
    let result = safe_lock(&mutex);
    assert!(result.is_err());

    // 测试错误链
    let chained_result = result.and_then(|_| {
        // 这个闭包不会被执行，因为第一个操作失败了
        Ok(42)
    });

    assert!(chained_result.is_err());
}

#[test]
fn test_complex_error_scenario() {
    // 测试复杂的错误场景
    let data_map = Arc::new(Mutex::new(HashMap::new()));

    // 模拟一个复杂的操作序列
    let result = safe_lock(&data_map)
        .and_then(|mut map| {
            map.insert("key1".to_string(), 42);
            Ok(())
        })
        .and_then(|_| {
            // 这里会失败，因为锁已经被释放
            safe_lock(&data_map).and_then(|mut map| {
                map.insert("key2".to_string(), 24);
                Ok(())
            })
        });

    // 第一个操作应该成功，第二个操作应该失败
    assert!(result.is_ok());

    // 验证第一个操作的结果
    let map = safe_lock(&data_map).expect("Map should be accessible after first operation");
    assert_eq!(map.get("key1"), Some(&42));
    assert!(!map.contains_key("key2"));
}

#[test]
fn test_error_messages() {
    // 测试错误消息的质量
    let option: Option<i32> = None;
    let result = expect_option(option, "This value should exist for proper operation");

    match result {
        Ok(_) => panic!("Expected error but got success"),
        Err(e) => {
            let error_msg = e.to_string();
            assert!(error_msg.contains("This value should exist for proper operation"));
        }
    }
}

#[test]
fn test_lock_recovery() {
    use std::sync::mpsc;
    use std::thread;

    // 测试锁恢复机制
    let mutex = Arc::new(Mutex::new(vec![1, 2, 3]));
    let mutex_clone = mutex.clone();

    // 在另一个线程中故意污染锁
    let (tx, rx) = mpsc::channel::<()>();
    thread::spawn(move || {
        let _guard = mutex_clone
            .lock()
            .expect("Mutex should be accessible in test thread");
        panic!("Intentional panic to poison the lock");
    });

    // 等待线程完成
    let _ = rx.recv();

    // 给一点时间让锁被污染
    thread::sleep(std::time::Duration::from_millis(10));

    // 测试从污染的锁中恢复
    let result = safe_lock(&mutex);
    assert!(result.is_err());

    // 在实际应用中，我们可能需要从污染的锁中恢复数据
    match result {
        Err(DBError::Lock(_)) => {
            // 这里可以实现恢复逻辑
            println!("Lock error occurred");
        }
        _ => panic!("Expected lock error"),
    }
}
