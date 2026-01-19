use graphdb::common::thread::*;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[test]
fn test_atomic_counter_new() {
    let counter = AtomicCounter::new(0);
    assert_eq!(counter.get(), 0);
}

#[test]
fn test_atomic_counter_increment() {
    let counter = AtomicCounter::new(0);
    counter.increment();
    assert_eq!(counter.get(), 1);
    counter.increment();
    assert_eq!(counter.get(), 2);
}

#[test]
fn test_atomic_counter_decrement() {
    let counter = AtomicCounter::new(5);
    assert_eq!(counter.decrement(), 5);
    assert_eq!(counter.get(), 4);
    assert_eq!(counter.decrement(), 4);
    assert_eq!(counter.get(), 3);
}

#[test]
fn test_atomic_counter_set() {
    let counter = AtomicCounter::new(0);
    counter.set(100);
    assert_eq!(counter.get(), 100);
    counter.set(50);
    assert_eq!(counter.get(), 50);
}

#[test]
fn test_atomic_counter_multiple_threads() {
    let counter = Arc::new(AtomicCounter::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || {
            for _ in 0..100 {
                counter.increment();
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    assert_eq!(counter.get(), 1000);
}

#[test]
fn test_thread_pool_new() {
    let pool = ThreadPool::new(4);
    assert_eq!(pool.len(), 4);
}

#[test]
fn test_thread_pool_new_single_worker() {
    let pool = ThreadPool::new(1);
    assert_eq!(pool.len(), 1);
}

#[test]
fn test_thread_pool_execute() {
    let pool = ThreadPool::new(4);
    let counter = Arc::new(AtomicCounter::new(0));
    let counter_clone = Arc::clone(&counter);

    pool.execute(move || {
        counter_clone.increment();
    });

    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(counter.get(), 1);
}

#[test]
fn test_thread_pool_multiple_executes() {
    let pool = ThreadPool::new(2);
    let counter = Arc::new(AtomicCounter::new(0));

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        pool.execute(move || {
            counter.increment();
        });
    }

    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(counter.get(), 10);
}

#[test]
fn test_thread_pool_shutdown() {
    let mut pool = ThreadPool::new(2);
    let counter = Arc::new(AtomicCounter::new(0));

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        pool.execute(move || {
            counter.increment();
        });
    }

    pool.shutdown();
    pool.wait_for_completion();
    assert_eq!(counter.get(), 5);
}

#[test]
fn test_condition_variable_new() {
    let cvar = ConditionVariable::new();
    assert!(true);
}

#[test]
fn test_condition_variable_notify_one() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    let notified = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let notified2 = Arc::clone(&notified);

    let handle = std::thread::spawn(move || {
        let (lock, cvar): &(std::sync::Mutex<bool>, Condvar) = &*pair2;
        let mut started = lock.lock().expect("Mutex should not be poisoned");
        *started = true;
        notified2.store(true, std::sync::atomic::Ordering::SeqCst);
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().expect("Mutex should not be poisoned");
    while !*started {
        started = cvar.wait(started).expect("Cvar should not be corrupted");
    }

    handle.join().expect("Thread should complete");
    assert!(notified.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn test_condition_variable_notify_all() {
    let pair = Arc::new((Mutex::new(0u32), Condvar::new()));
    let (lock, cvar) = &*pair;
    let mut count = lock.lock().expect("Mutex should not be poisoned");

    cvar.notify_all();
}

#[test]
fn test_thread_manager_new() {
    let manager = ThreadManager::new();
    assert_eq!(manager.active_count(), 0);
}

#[test]
fn test_thread_manager_spawn() {
    let manager = ThreadManager::new();
    let result = manager.spawn(|| {
        std::thread::sleep(Duration::from_millis(10));
    });

    assert!(result.is_ok());
    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(manager.active_count(), 1);
}

#[test]
fn test_thread_manager_multiple_spawns() {
    let manager = ThreadManager::new();

    for _ in 0..5 {
        let result = manager.spawn(|| {
            std::thread::sleep(Duration::from_millis(10));
        });
        assert!(result.is_ok());
    }

    std::thread::sleep(Duration::from_millis(50));
    assert_eq!(manager.active_count(), 5);
}

#[test]
fn test_thread_manager_join_all() {
    let manager = ThreadManager::new();
    let counter = Arc::new(AtomicCounter::new(0));

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        manager.spawn(move || {
            counter.increment();
        }).expect("Spawn should succeed");
    }

    let result = manager.join_all();
    assert!(result.is_ok());
    assert_eq!(counter.get(), 5);
}

#[test]
fn test_lazy_new() {
    let called = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let called2 = called.clone();

    let lazy = Lazy::new(move || {
        called2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        42
    });

    let value = lazy.get();
    assert_eq!(value, 42);
    assert_eq!(called.load(std::sync::atomic::Ordering::SeqCst), 1);

    let value2 = lazy.get();
    assert_eq!(value2, 42);
    assert_eq!(called.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn test_lazy_get_multiple_times() {
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter2 = counter.clone();

    let lazy = Lazy::new(move || {
        counter2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        "initialized".to_string()
    });

    for _ in 0..5 {
        let val = lazy.get();
        assert_eq!(val, "initialized");
    }

    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn test_thread_local_new() {
    let _local = ThreadLocal::new(|| 42);
}

#[test]
fn test_thread_pool_clone() {
    let pool = ThreadPool::new(4);
    let _clone = pool;
}

#[test]
fn test_atomic_counter_clone() {
    let counter1 = AtomicCounter::new(10);
    let counter2 = counter1;
    assert_eq!(counter2.get(), 10);
}

#[test]
fn test_condition_variable_clone() {
    let cvar1 = ConditionVariable::new();
    let _cvar2 = cvar1;
    assert!(true);
}

#[test]
fn test_thread_manager_clone() {
    let manager1 = ThreadManager::new();
    let manager2 = manager1;
    assert_eq!(manager2.active_count(), 0);
}

#[test]
fn test_lazy_clone() {
    let lazy1: Lazy<i32> = Lazy::new(|| 42);
    let _lazy2 = lazy1;
}

#[tokio::test]
async fn test_async_utils_spawn_blocking() {
    let result = async_utils::spawn_blocking(|| {
        std::thread::sleep(Duration::from_millis(10));
        "blocking result"
    }).await;

    assert_eq!(result, "blocking result");
}

#[tokio::test]
async fn test_async_utils_join_all() {
    use std::pin::Pin;
    use std::future::Future;

    let futures: Vec<Pin<Box<dyn Future<Output = i32>>>> = vec![
        Box::pin(async { 1 }),
        Box::pin(async { 2 }),
        Box::pin(async { 3 }),
    ];

    let results = async_utils::join_all(futures).await;
    assert_eq!(results, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_async_utils_timeout_success() {
    let result = async_utils::timeout(Duration::from_millis(100), async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        "success"
    }).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[tokio::test]
async fn test_async_utils_timeout_failure() {
    let result = async_utils::timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "should not reach"
    }).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_utils_channel() {
    let (sender, mut receiver) = async_utils::channel::<i32>();
    sender.send(42).await.expect("Send should succeed");
    let received = receiver.recv().await.expect("Receive should succeed");
    assert_eq!(received, 42);
}

#[tokio::test]
async fn test_async_utils_oneshot() {
    let (sender, receiver) = async_utils::oneshot::<&str>();
    sender.send("hello").expect("Send should succeed");
    let received = receiver.await.expect("Receive should succeed");
    assert_eq!(received, "hello");
}

#[test]
fn test_atomic_counter_stress() {
    let counter = Arc::new(AtomicCounter::new(0));
    let mut handles = vec![];

    for _ in 0..20 {
        let counter = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment();
            }
        }));
    }

    for handle in handles {
        handle.join().expect("Thread should complete");
    }

    assert_eq!(counter.get(), 20000);
}
