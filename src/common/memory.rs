use std::alloc::Layout;
use std::collections::HashMap;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Memory tracker to monitor allocations
#[derive(Debug)]
pub struct MemoryTracker {
    total_allocated: AtomicUsize,
    total_deallocated: AtomicUsize,
    current_allocated: AtomicUsize,
    peak_usage: AtomicUsize,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            total_allocated: AtomicUsize::new(0),
            total_deallocated: AtomicUsize::new(0),
            current_allocated: AtomicUsize::new(0),
            peak_usage: AtomicUsize::new(0),
        }
    }

    pub fn record_allocation(&self, size: usize) {
        let old_current = self.current_allocated.fetch_add(size, Ordering::SeqCst);
        self.total_allocated.fetch_add(size, Ordering::SeqCst);

        // Update peak if current is greater than previous peak
        let mut current_peak = self.peak_usage.load(Ordering::SeqCst);
        loop {
            if old_current + size > current_peak {
                match self.peak_usage.compare_exchange(
                    current_peak,
                    old_current + size,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(current) => current_peak = current,
                }
            } else {
                break;
            }
        }
    }

    pub fn record_deallocation(&self, size: usize) {
        self.current_allocated.fetch_sub(size, Ordering::SeqCst);
        self.total_deallocated.fetch_add(size, Ordering::SeqCst);
    }

    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::SeqCst)
    }

    pub fn total_deallocated(&self) -> usize {
        self.total_deallocated.load(Ordering::SeqCst)
    }

    pub fn current_allocated(&self) -> usize {
        self.current_allocated.load(Ordering::SeqCst)
    }

    pub fn peak_usage(&self) -> usize {
        self.peak_usage.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.total_allocated.store(0, Ordering::SeqCst);
        self.total_deallocated.store(0, Ordering::SeqCst);
        self.current_allocated.store(0, Ordering::SeqCst);
        self.peak_usage.store(0, Ordering::SeqCst);
    }
}

/// Global memory tracker
static MEMORY_TRACKER: once_cell::sync::Lazy<MemoryTracker> =
    once_cell::sync::Lazy::new(MemoryTracker::new);

/// Get a reference to the global memory tracker
pub fn memory_tracker() -> &'static MemoryTracker {
    &MEMORY_TRACKER
}

/// Custom memory pool implementation
pub struct MemoryPool {
    pool: Arc<Mutex<Vec<u8>>>,
    available_chunks: Arc<Mutex<HashMap<usize, Vec<usize>>>>, // size -> list of start indices
    chunk_size_map: Arc<Mutex<HashMap<usize, usize>>>,        // start index -> size
    total_size: usize,
    used_size: Arc<AtomicUsize>,
}

impl MemoryPool {
    pub fn new(size: usize) -> Self {
        let mut pool = Vec::with_capacity(size);
        pool.resize(size, 0);

        let mut available_chunks = HashMap::new();
        if size > 0 {
            available_chunks.insert(size, vec![0]); // Start with one chunk of `size` at position 0
        }

        Self {
            pool: Arc::new(Mutex::new(pool)),
            available_chunks: Arc::new(Mutex::new(available_chunks)),
            chunk_size_map: Arc::new(Mutex::new(HashMap::new())),
            total_size: size,
            used_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn allocate(&self, size: usize) -> Option<*mut u8> {
        let mut available_chunks = self
            .available_chunks
            .lock()
            .expect("Available chunks lock should not be poisoned");

        // Look for a suitable chunk (first fit)
        if let Some(&start_idx) = available_chunks.get(&size).and_then(|v| v.first()) {
            // Found an available chunk of exact size
            available_chunks
                .get_mut(&size)
                .expect("Chunk should exist")
                .remove(0);
            if available_chunks[&size].is_empty() {
                available_chunks.remove(&size);
            }

            // Mark this chunk as used
            {
                let mut chunk_map = self
                    .chunk_size_map
                    .lock()
                    .expect("Chunk size map lock was poisoned");
                chunk_map.insert(start_idx, size);
            }

            self.used_size.fetch_add(size, Ordering::SeqCst);

            let pool = self.pool.lock().expect("Pool lock was poisoned");
            Some(pool.as_ptr() as *mut u8).map(|p| unsafe { p.add(start_idx) })
        } else {
            // Try to find a larger chunk that can be split
            let mut suitable_chunk = None;
            for (&chunk_size, indices) in available_chunks.iter() {
                if chunk_size >= size && !indices.is_empty() {
                    suitable_chunk = Some((chunk_size, indices[0]));
                    break;
                }
            }

            if let Some((original_chunk_size, start_idx)) = suitable_chunk {
                // Remove the chunk from available list
                available_chunks
                    .get_mut(&original_chunk_size)
                    .expect("Original chunk should exist")
                    .retain(|&x| x != start_idx);
                if available_chunks[&original_chunk_size].is_empty() {
                    available_chunks.remove(&original_chunk_size);
                }

                // If there's remaining space after allocation, add it back as available
                if original_chunk_size > size {
                    let remaining_size = original_chunk_size - size;
                    let remaining_start_idx = start_idx + size;
                    available_chunks
                        .entry(remaining_size)
                        .or_insert_with(Vec::new)
                        .push(remaining_start_idx);
                }

                // Mark the allocated portion as used
                {
                    let mut chunk_map = self
                        .chunk_size_map
                        .lock()
                        .expect("Chunk size map lock was poisoned");
                    chunk_map.insert(start_idx, size);
                }

                self.used_size.fetch_add(size, Ordering::SeqCst);

                let pool = self.pool.lock().expect("Pool lock was poisoned");
                Some(pool.as_ptr() as *mut u8).map(|p| unsafe { p.add(start_idx) })
            } else {
                // No suitable chunk found, allocation failed
                None
            }
        }
    }

    pub fn deallocate(&self, ptr: *mut u8, size: usize) {
        if ptr.is_null() {
            return;
        }

        // Calculate the index in our pool
        let pool_ptr = self.pool.lock().expect("Pool lock was poisoned").as_ptr() as *mut u8;
        let start_idx = unsafe { ptr.offset_from(pool_ptr) as usize };

        // Verify this is a valid pointer from our pool
        let mut chunk_size_map = self
            .chunk_size_map
            .lock()
            .expect("Chunk size map lock was poisoned");
        if chunk_size_map.get(&start_idx) != Some(&size) {
            // This pointer doesn't belong to our pool or size doesn't match
            return;
        }

        // Remove from used map
        chunk_size_map.remove(&start_idx);
        drop(chunk_size_map);

        // Add to available chunks
        let mut available_chunks = self
            .available_chunks
            .lock()
            .expect("Available chunks lock was poisoned");
        available_chunks
            .entry(size)
            .or_insert_with(Vec::new)
            .push(start_idx);

        self.used_size.fetch_sub(size, Ordering::SeqCst);
    }

    pub fn total_size(&self) -> usize {
        self.total_size
    }

    pub fn used_size(&self) -> usize {
        self.used_size.load(Ordering::SeqCst)
    }

    pub fn available_size(&self) -> usize {
        self.total_size - self.used_size.load(Ordering::SeqCst)
    }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub current_allocated: usize,
    pub peak_usage: usize,
    pub pool_total_size: usize,
    pub pool_used_size: usize,
    pub pool_available_size: usize,
}

/// Get memory statistics
pub fn get_memory_stats(pool: Option<&MemoryPool>) -> MemoryStats {
    MemoryStats {
        total_allocated: MEMORY_TRACKER.total_allocated(),
        total_deallocated: MEMORY_TRACKER.total_deallocated(),
        current_allocated: MEMORY_TRACKER.current_allocated(),
        peak_usage: MEMORY_TRACKER.peak_usage(),
        pool_total_size: match pool {
            Some(p) => p.total_size(),
            None => 0,
        },
        pool_used_size: match pool {
            Some(p) => p.used_size(),
            None => 0,
        },
        pool_available_size: match pool {
            Some(p) => p.available_size(),
            None => 0,
        },
    }
}

/// Memory utilities
pub mod memory_utils {
    use super::*;
    use std::mem;

    /// Get the size of a type in bytes
    pub fn size_of<T>() -> usize {
        mem::size_of::<T>()
    }

    /// Get the alignment of a type
    pub fn align_of<T>() -> usize {
        mem::align_of::<T>()
    }

    /// Fill a buffer with a specific value
    pub fn fill_buffer(buffer: &mut [u8], value: u8) {
        for b in buffer.iter_mut() {
            *b = value;
        }
    }

    /// Compare two byte buffers
    pub fn compare_buffers(buf1: &[u8], buf2: &[u8]) -> bool {
        buf1 == buf2
    }

    /// Copy memory from one location to another
    pub unsafe fn copy_memory(src: *const u8, dest: *mut u8, size: usize) {
        ptr::copy_nonoverlapping(src, dest, size);
    }

    /// Set memory to a value
    pub unsafe fn set_memory(ptr: *mut u8, value: u8, size: usize) {
        ptr::write_bytes(ptr, value, size);
    }

    /// Get the memory address of a value as a usize
    pub fn get_memory_address<T: ?Sized>(r: &T) -> usize {
        r as *const T as *const () as usize
    }
}

/// A simple object pool to reuse objects and reduce allocations
pub struct ObjectPool<T> {
    pool: Arc<Mutex<Vec<T>>>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T: Clone + 'static> ObjectPool<T> {
    pub fn new(factory: Box<dyn Fn() -> T + Send + Sync>, max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::new())),
            factory,
            max_size,
        }
    }

    pub fn get(&self) -> T {
        let mut pool = self.pool.lock().expect("Object pool lock was poisoned");
        if let Some(obj) = pool.pop() {
            obj
        } else {
            (self.factory)()
        }
    }

    pub fn return_obj(&self, obj: T) {
        let mut pool = self.pool.lock().expect("Object pool lock was poisoned");
        if pool.len() < self.max_size {
            pool.push(obj);
        }
        // Otherwise, the object is dropped
    }

    pub fn len(&self) -> usize {
        self.pool
            .lock()
            .expect("Object pool lock was poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.pool
            .lock()
            .expect("Object pool lock was poisoned")
            .is_empty()
    }
}

/// Memory leak detector (simplified for demonstration)
pub struct MemoryLeakDetector {
    allocations: Arc<Mutex<HashMap<usize, (Layout, String)>>>,
}

impl MemoryLeakDetector {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record_allocation(&self, ptr: usize, layout: Layout, location: String) {
        let mut allocations = self
            .allocations
            .lock()
            .expect("Allocations lock was poisoned");
        allocations.insert(ptr, (layout, location));
    }

    pub fn record_deallocation(&self, ptr: usize) {
        let mut allocations = self
            .allocations
            .lock()
            .expect("Allocations lock was poisoned");
        allocations.remove(&ptr);
    }

    pub fn report_leaks(&self) -> Vec<(usize, Layout, String)> {
        let allocations = self
            .allocations
            .lock()
            .expect("Allocations lock was poisoned");
        allocations
            .iter()
            .map(|(&ptr, (layout, location))| (ptr, *layout, location.clone()))
            .collect()
    }

    pub fn has_leaks(&self) -> bool {
        let allocations = self
            .allocations
            .lock()
            .expect("Allocations lock was poisoned");
        !allocations.is_empty()
    }
}

static LEAK_DETECTOR: once_cell::sync::Lazy<MemoryLeakDetector> =
    once_cell::sync::Lazy::new(MemoryLeakDetector::new);

pub fn leak_detector() -> &'static MemoryLeakDetector {
    &LEAK_DETECTOR
}

/// Memory management configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub enable_tracking: bool,
    pub enable_pooling: bool,
    pub pool_size: usize,
    pub max_object_pool_size: usize,
    pub log_memory_stats_interval: Option<u64>, // seconds
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_tracking: true,
            enable_pooling: true,
            pool_size: 10 * 1024 * 1024, // 10MB
            max_object_pool_size: 100,
            log_memory_stats_interval: Some(300), // 5 minutes
        }
    }
}

/// Initialize memory management with the given configuration
pub fn init_memory_management(config: &MemoryConfig) {
    // Initialize tracking if enabled
    if config.enable_tracking {
        MEMORY_TRACKER.reset();
    }

    // Print initial stats if needed
    if config.enable_tracking {
        let stats = get_memory_stats(None);
        println!("Memory tracking initialized. Current stats: {:?}", stats);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();

        tracker.record_allocation(100);
        assert_eq!(tracker.total_allocated(), 100);
        assert_eq!(tracker.current_allocated(), 100);

        tracker.record_allocation(50);
        assert_eq!(tracker.total_allocated(), 150);
        assert_eq!(tracker.current_allocated(), 150);

        tracker.record_deallocation(30);
        assert_eq!(tracker.total_deallocated(), 30);
        assert_eq!(tracker.current_allocated(), 120);

        tracker.reset();
        assert_eq!(tracker.total_allocated(), 0);
        assert_eq!(tracker.current_allocated(), 0);
    }

    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::new(1024); // 1KB pool

        assert_eq!(pool.total_size(), 1024);
        assert_eq!(pool.available_size(), 1024);
        assert_eq!(pool.used_size(), 0);

        // Allocate 100 bytes
        let ptr1 = pool.allocate(100);
        assert!(ptr1.is_some());
        assert_eq!(pool.available_size(), 924); // 1024 - 100
        assert_eq!(pool.used_size(), 100);

        // Allocate another 50 bytes
        let ptr2 = pool.allocate(50);
        assert!(ptr2.is_some());
        assert_eq!(pool.available_size(), 874); // 1024 - 100 - 50
        assert_eq!(pool.used_size(), 150);

        // Deallocate first chunk
        if let Some(p) = ptr1 {
            pool.deallocate(p, 100);
        }
        assert_eq!(pool.available_size(), 974); // 874 + 100
        assert_eq!(pool.used_size(), 50);
    }

    #[test]
    fn test_object_pool() {
        let factory = Box::new(|| 0i32);
        let pool = ObjectPool::new(factory, 10);

        assert!(pool.is_empty());

        // Get an object
        let obj = pool.get();
        assert_eq!(obj, 0);
        assert!(pool.is_empty());

        // Return the object
        pool.return_obj(obj);
        assert!(!pool.is_empty());
        assert_eq!(pool.len(), 1);

        // Get it again
        let obj2 = pool.get();
        assert_eq!(obj2, 0);
        assert!(pool.is_empty());
    }

    #[test]
    fn test_memory_utils() {
        // Test size_of
        assert_eq!(memory_utils::size_of::<i32>(), 4);
        assert_eq!(memory_utils::size_of::<u64>(), 8);

        // Test align_of
        assert_eq!(memory_utils::align_of::<i32>(), std::mem::align_of::<i32>());

        // Test buffer operations
        let mut buffer = [0u8; 10];
        memory_utils::fill_buffer(&mut buffer, 5);
        assert_eq!(buffer, [5u8; 10]);

        assert!(memory_utils::compare_buffers(&[1, 2, 3], &[1, 2, 3]));
        assert!(!memory_utils::compare_buffers(&[1, 2, 3], &[1, 2, 4]));
    }

    #[test]
    fn test_leak_detector() {
        let detector = MemoryLeakDetector::new();

        assert!(!detector.has_leaks());

        let layout = Layout::from_size_align(100, 8).expect("Failed to create layout");
        detector.record_allocation(0x1000, layout, "test_location".to_string());

        assert!(detector.has_leaks());

        detector.record_deallocation(0x1000);

        assert!(!detector.has_leaks());
    }
}
