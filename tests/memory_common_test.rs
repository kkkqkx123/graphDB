use graphdb::common::memory::*;
use std::alloc::Layout;

#[test]
fn test_memory_tracker_new() {
    let tracker = MemoryTracker::new();
    assert_eq!(tracker.total_allocated(), 0);
    assert_eq!(tracker.total_deallocated(), 0);
    assert_eq!(tracker.current_allocated(), 0);
    assert_eq!(tracker.peak_usage(), 0);
}

#[test]
fn test_memory_tracker_allocation() {
    let tracker = MemoryTracker::new();

    tracker.record_allocation(100);
    assert_eq!(tracker.total_allocated(), 100);
    assert_eq!(tracker.current_allocated(), 100);
    assert_eq!(tracker.peak_usage(), 100);

    tracker.record_allocation(50);
    assert_eq!(tracker.total_allocated(), 150);
    assert_eq!(tracker.current_allocated(), 150);
    assert_eq!(tracker.peak_usage(), 150);
}

#[test]
fn test_memory_tracker_deallocation() {
    let tracker = MemoryTracker::new();

    tracker.record_allocation(100);
    tracker.record_deallocation(30);

    assert_eq!(tracker.total_deallocated(), 30);
    assert_eq!(tracker.current_allocated(), 70);
}

#[test]
fn test_memory_tracker_peak_usage() {
    let tracker = MemoryTracker::new();
    tracker.record_allocation(100);
    tracker.record_allocation(50);
    tracker.record_deallocation(100);
    tracker.record_allocation(200);
    assert!(tracker.peak_usage() >= 250);
}

#[test]
fn test_memory_tracker_reset() {
    let tracker = MemoryTracker::new();

    tracker.record_allocation(100);
    tracker.record_allocation(50);
    tracker.record_deallocation(30);

    tracker.reset();

    assert_eq!(tracker.total_allocated(), 0);
    assert_eq!(tracker.total_deallocated(), 0);
    assert_eq!(tracker.current_allocated(), 0);
    assert_eq!(tracker.peak_usage(), 0);
}

#[test]
fn test_memory_pool_new() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");
    assert_eq!(pool.total_size(), 1024);
    assert_eq!(pool.used_size(), 0);
    assert_eq!(pool.available_size(), 1024);
}

#[test]
fn test_memory_pool_zero_size() {
    let result = MemoryPool::new(0);
    assert!(result.is_err());
}

#[test]
fn test_memory_pool_allocation() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");

    let chunk = pool.allocate(100).expect("分配失败");
    assert_eq!(pool.used_size(), 100);
    assert_eq!(pool.available_size(), 924);
    assert_eq!(chunk.len(), 100);
}

#[test]
fn test_memory_pool_multiple_allocations() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");

    let chunk1 = pool.allocate(100).expect("分配失败");
    let chunk2 = pool.allocate(50).expect("分配失败");
    let chunk3 = pool.allocate(200).expect("分配失败");

    assert_eq!(pool.used_size(), 350);
    assert_eq!(pool.available_size(), 674);
}

#[test]
fn test_memory_pool_allocation_too_large() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");
    let result = pool.allocate(2048);
    assert!(result.is_err());
}

#[test]
fn test_memory_pool_zero_allocation() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");
    let result = pool.allocate(0);
    assert!(result.is_err());
}

#[test]
fn test_memory_chunk_write_read() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");
    let mut chunk = pool.allocate(100).expect("分配失败");

    let value = chunk.with_mut_slice(|slice| {
        slice[0] = 42;
        slice[99] = 99;
        42
    });
    assert_eq!(value, 42);

    let read_value = chunk.with_slice(|slice| slice[0]);
    assert_eq!(read_value, 42);
}

#[test]
fn test_memory_chunk_len_is_empty() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");
    let chunk = pool.allocate(100).expect("分配失败");

    assert_eq!(chunk.len(), 100);
    assert!(!chunk.is_empty());
}

#[test]
fn test_memory_chunk_drop_frees() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");

    {
        let chunk = pool.allocate(100).expect("分配失败");
        assert_eq!(pool.used_size(), 100);
    }

    assert_eq!(pool.used_size(), 0);
}

#[test]
fn test_memory_chunk_fragmentation_handling() {
    let pool = MemoryPool::new(1024).expect("内存池创建失败");

    let _chunk1 = pool.allocate(300).expect("分配失败");
    let _chunk2 = pool.allocate(300).expect("分配失败");
    let _chunk3 = pool.allocate(300).expect("分配失败");

    assert_eq!(pool.used_size(), 900);
    assert!(pool.available_size() < 200);
}

#[test]
fn test_object_pool_new() {
    let factory = Box::new(|| 0i32);
    let pool = ObjectPool::new(factory, 10);

    assert!(pool.is_empty());
    assert_eq!(pool.len(), 0);
}

#[test]
fn test_object_pool_get_return() {
    let factory = Box::new(|| 42i32);
    let pool = ObjectPool::new(factory, 10);

    let obj1 = pool.get();
    assert_eq!(obj1, 42);

    pool.return_obj(obj1);
    assert_eq!(pool.len(), 1);
    assert!(!pool.is_empty());

    let obj2 = pool.get();
    assert_eq!(obj2, 42);
    assert!(pool.is_empty());
}

#[test]
fn test_object_pool_max_size() {
    let factory = Box::new(|| 0i32);
    let pool = ObjectPool::new(factory, 3);

    pool.return_obj(1);
    pool.return_obj(2);
    pool.return_obj(3);
    pool.return_obj(4);

    assert_eq!(pool.len(), 3);
}

#[test]
fn test_object_pool_reuse() {
    let call_count = std::sync::atomic::AtomicUsize::new(0);
    let call_count_clone = std::sync::Arc::new(call_count);
    let call_count_inner = std::sync::Arc::clone(&call_count_clone);
    let factory = Box::new(move || {
        call_count_inner.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        0i32
    });
    let pool = ObjectPool::new(factory, 10);

    let obj1 = pool.get();
    pool.return_obj(obj1);
    let obj2 = pool.get();

    assert_eq!(call_count_clone.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn test_leak_detector_new() {
    let detector = MemoryLeakDetector::new();
    assert!(!detector.has_leaks());
    assert!(detector.report_leaks().is_empty());
}

#[test]
fn test_leak_detector_record_allocation() {
    let detector = MemoryLeakDetector::new();

    let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
    detector.record_allocation(0x1000, layout, "test_location".to_string());

    assert!(detector.has_leaks());
    let leaks = detector.report_leaks();
    assert_eq!(leaks.len(), 1);
    assert_eq!(leaks[0].0, 0x1000);
}

#[test]
fn test_leak_detector_record_deallocation() {
    let detector = MemoryLeakDetector::new();

    let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
    detector.record_allocation(0x1000, layout, "test_location".to_string());
    detector.record_deallocation(0x1000);

    assert!(!detector.has_leaks());
    assert!(detector.report_leaks().is_empty());
}

#[test]
fn test_leak_detector_multiple_allocations() {
    let detector = MemoryLeakDetector::new();

    let layout1 = Layout::from_size_align(100, 8).expect("布局创建失败");
    let layout2 = Layout::from_size_align(200, 16).expect("布局创建失败");

    detector.record_allocation(0x1000, layout1, "location1".to_string());
    detector.record_allocation(0x2000, layout2, "location2".to_string());

    assert!(detector.has_leaks());
    let leaks = detector.report_leaks();
    assert_eq!(leaks.len(), 2);
}

#[test]
fn test_memory_utils_size_of() {
    use graphdb::common::memory::memory_utils;

    assert_eq!(memory_utils::size_of::<i32>(), 4);
    assert_eq!(memory_utils::size_of::<u64>(), 8);
    assert_eq!(memory_utils::size_of::<f64>(), 8);
    assert!(memory_utils::size_of::<String>() > 0);
}

#[test]
fn test_memory_utils_align_of() {
    use graphdb::common::memory::memory_utils;

    assert_eq!(memory_utils::align_of::<i32>(), std::mem::align_of::<i32>());
    assert_eq!(memory_utils::align_of::<u64>(), std::mem::align_of::<u64>());
}

#[test]
fn test_memory_utils_fill_buffer() {
    use graphdb::common::memory::memory_utils;

    let mut buffer = [0u8; 10];
    memory_utils::fill_buffer(&mut buffer, 5);
    assert_eq!(buffer, [5u8; 10]);
}

#[test]
fn test_memory_utils_compare_buffers() {
    use graphdb::common::memory::memory_utils;

    assert!(memory_utils::compare_buffers(&[1, 2, 3], &[1, 2, 3]));
    assert!(!memory_utils::compare_buffers(&[1, 2, 3], &[1, 2, 4]));
    assert!(!memory_utils::compare_buffers(&[1, 2], &[1, 2, 3]));
}

#[test]
fn test_memory_utils_get_memory_address() {
    use graphdb::common::memory::memory_utils;

    let value = 42;
    let addr = memory_utils::get_memory_address(&value);
    assert!(addr > 0);
}

#[test]
fn test_memory_stats() {
    use graphdb::common::memory::memory_tracker;

    let tracker = memory_tracker();
    let before_allocated = tracker.total_allocated();

    let layout = Layout::from_size_align(100, 8).expect("布局创建失败");
    let _ptr = unsafe { std::alloc::alloc(layout) };
    tracker.record_allocation(100);
    tracker.record_deallocation(100);
    unsafe { std::alloc::dealloc(_ptr, layout) };

    let stats = get_memory_stats(None);
    assert!(stats.total_allocated >= before_allocated + 100);
    assert!(stats.current_allocated >= 0);
}

#[test]
fn test_memory_config_default() {
    let config = MemoryConfig::default();
    assert!(config.enable_tracking);
    assert!(config.enable_pooling);
    assert_eq!(config.pool_size, 10 * 1024 * 1024);
    assert_eq!(config.max_object_pool_size, 100);
    assert_eq!(config.log_memory_stats_interval, Some(300));
}

#[test]
fn test_memory_config_custom() {
    let config = MemoryConfig {
        enable_tracking: false,
        enable_pooling: false,
        pool_size: 1024,
        max_object_pool_size: 5,
        log_memory_stats_interval: None,
    };

    assert!(!config.enable_tracking);
    assert!(!config.enable_pooling);
    assert_eq!(config.pool_size, 1024);
    assert_eq!(config.max_object_pool_size, 5);
    assert!(config.log_memory_stats_interval.is_none());
}
