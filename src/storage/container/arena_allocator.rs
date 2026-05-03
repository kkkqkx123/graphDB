//! Arena Allocator
//!
//! Provides a high-performance arena-based memory allocator for batch allocations
//! with efficient deallocation patterns.

use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

use super::types::{ContainerError, ContainerResult};

/// Default chunk size (64KB)
const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Minimum alignment for allocations
const MIN_ALIGNMENT: usize = 8;

/// Memory chunk in the arena
struct Chunk {
    /// Pointer to the chunk memory
    data: NonNull<u8>,
    /// Total size of the chunk
    size: usize,
    /// Used bytes in this chunk
    used: usize,
}

impl Chunk {
    fn new(size: usize) -> ContainerResult<Self> {
        let layout = Layout::from_size_align(size, MIN_ALIGNMENT)
            .map_err(|e| ContainerError::InvalidSize(e.to_string()))?;

        let ptr = unsafe { alloc(layout) };
        let data = NonNull::new(ptr).ok_or(ContainerError::OutOfMemory)?;

        Ok(Self {
            data,
            size,
            used: 0,
        })
    }

    fn allocate(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let offset = self.used;
        let aligned_offset = (offset + align - 1) & !(align - 1);
        let new_used = aligned_offset + size;

        if new_used > self.size {
            return None;
        }

        self.used = new_used;
        let ptr = unsafe {
            let base = self.data.as_ptr();
            NonNull::new_unchecked(base.add(aligned_offset))
        };
        Some(ptr)
    }

    fn remaining(&self) -> usize {
        self.size - self.used
    }

    fn reset(&mut self) {
        self.used = 0;
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        unsafe {
            let layout = Layout::from_size_align_unchecked(self.size, MIN_ALIGNMENT);
            dealloc(self.data.as_ptr(), layout);
        }
    }
}

unsafe impl Send for Chunk {}

/// Arena allocator for efficient batch allocations
pub struct ArenaAllocator {
    /// Current chunk
    current: UnsafeCell<Option<Chunk>>,
    /// List of all chunks
    chunks: UnsafeCell<Vec<Chunk>>,
    /// Default chunk size
    chunk_size: usize,
    /// Total allocated bytes
    total_allocated: AtomicUsize,
    /// Total used bytes
    total_used: AtomicUsize,
}

impl ArenaAllocator {
    /// Create a new arena allocator with default chunk size
    pub fn new() -> ContainerResult<Self> {
        Self::with_chunk_size(DEFAULT_CHUNK_SIZE)
    }

    /// Create a new arena allocator with specified chunk size
    pub fn with_chunk_size(chunk_size: usize) -> ContainerResult<Self> {
        let chunk = Chunk::new(chunk_size)?;
        Ok(Self {
            current: UnsafeCell::new(Some(chunk)),
            chunks: UnsafeCell::new(Vec::new()),
            chunk_size,
            total_allocated: AtomicUsize::new(chunk_size),
            total_used: AtomicUsize::new(0),
        })
    }

    /// Allocate memory with the given size and alignment
    pub fn allocate(&self, size: usize, align: usize) -> ContainerResult<NonNull<u8>> {
        let align = align.max(MIN_ALIGNMENT);
        let size = (size + align - 1) & !(align - 1);

        unsafe {
            if let Some(ref mut chunk) = *self.current.get() {
                if let Some(ptr) = chunk.allocate(size, align) {
                    self.total_used.fetch_add(size, Ordering::Relaxed);
                    return Ok(ptr);
                }
            }

            let old_chunk = (*self.current.get()).take();
            if let Some(chunk) = old_chunk {
                (*self.chunks.get()).push(chunk);
            }

            let new_chunk_size = size.max(self.chunk_size);
            let mut new_chunk = Chunk::new(new_chunk_size)?;
            self.total_allocated
                .fetch_add(new_chunk_size, Ordering::Relaxed);

            let ptr = new_chunk
                .allocate(size, align)
                .expect("New chunk should have enough space");
            *self.current.get() = Some(new_chunk);
            self.total_used.fetch_add(size, Ordering::Relaxed);
            Ok(ptr)
        }
    }

    /// Allocate memory for a type T
    pub fn allocate_type<T>(&self) -> ContainerResult<NonNull<T>> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let ptr = self.allocate(size, align)?;
        Ok(unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut T) })
    }

    /// Allocate memory for a slice of type T
    pub fn allocate_slice<T>(&self, count: usize) -> ContainerResult<NonNull<T>> {
        if count == 0 {
            return Ok(NonNull::dangling());
        }

        let size = std::mem::size_of::<T>() * count;
        let align = std::mem::align_of::<T>();
        let ptr = self.allocate(size, align)?;
        Ok(unsafe { NonNull::new_unchecked(ptr.as_ptr() as *mut T) })
    }

    /// Allocate and copy a byte slice
    pub fn allocate_bytes(&self, bytes: &[u8]) -> ContainerResult<NonNull<u8>> {
        let ptr = self.allocate(bytes.len(), 1)?;
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.as_ptr(), bytes.len());
        }
        Ok(ptr)
    }

    /// Reset the arena, freeing all allocations
    pub fn reset(&self) {
        unsafe {
            if let Some(ref mut chunk) = *self.current.get() {
                chunk.reset();
            }

            for chunk in (*self.chunks.get()).iter_mut() {
                chunk.reset();
            }
        }
        self.total_used.store(0, Ordering::Relaxed);
    }

    /// Get total allocated bytes
    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::Relaxed)
    }

    /// Get total used bytes
    pub fn total_used(&self) -> usize {
        self.total_used.load(Ordering::Relaxed)
    }

    /// Get the number of chunks
    pub fn chunk_count(&self) -> usize {
        unsafe {
            let current_count = if (*self.current.get()).is_some() { 1 } else { 0 };
            (*self.chunks.get()).len() + current_count
        }
    }

    /// Get memory utilization (used / allocated)
    pub fn utilization(&self) -> f64 {
        let allocated = self.total_allocated();
        if allocated == 0 {
            0.0
        } else {
            self.total_used() as f64 / allocated as f64
        }
    }
}

impl Default for ArenaAllocator {
    fn default() -> Self {
        Self::new().expect("Failed to create default arena allocator")
    }
}

impl Drop for ArenaAllocator {
    fn drop(&mut self) {
        unsafe {
            *self.current.get() = None;
            self.chunks.get_mut().clear();
        }
    }
}

unsafe impl Sync for ArenaAllocator {}

/// Thread-local arena allocator wrapper
pub struct ThreadLocalArena {
    arena: ArenaAllocator,
}

impl ThreadLocalArena {
    pub fn new() -> ContainerResult<Self> {
        Ok(Self {
            arena: ArenaAllocator::new()?,
        })
    }

    pub fn with_chunk_size(chunk_size: usize) -> ContainerResult<Self> {
        Ok(Self {
            arena: ArenaAllocator::with_chunk_size(chunk_size)?,
        })
    }

    pub fn allocate(&self, size: usize, align: usize) -> ContainerResult<NonNull<u8>> {
        self.arena.allocate(size, align)
    }

    pub fn allocate_type<T>(&self) -> ContainerResult<NonNull<T>> {
        self.arena.allocate_type()
    }

    pub fn allocate_slice<T>(&self, count: usize) -> ContainerResult<NonNull<T>> {
        self.arena.allocate_slice(count)
    }

    pub fn allocate_bytes(&self, bytes: &[u8]) -> ContainerResult<NonNull<u8>> {
        self.arena.allocate_bytes(bytes)
    }

    pub fn reset(&self) {
        self.arena.reset();
    }

    pub fn total_allocated(&self) -> usize {
        self.arena.total_allocated()
    }

    pub fn total_used(&self) -> usize {
        self.arena.total_used()
    }
}

impl Default for ThreadLocalArena {
    fn default() -> Self {
        Self::new().expect("Failed to create default thread local arena")
    }
}

/// Global arena pool for multi-threaded allocation
pub struct ArenaPool {
    arenas: Vec<ArenaAllocator>,
    current: AtomicUsize,
}

impl ArenaPool {
    /// Create a new arena pool with the specified number of arenas
    pub fn new(arena_count: usize) -> ContainerResult<Self> {
        let arenas = (0..arena_count)
            .map(|_| ArenaAllocator::new())
            .collect::<ContainerResult<Vec<_>>>()?;

        Ok(Self {
            arenas,
            current: AtomicUsize::new(0),
        })
    }

    /// Create a new arena pool with custom chunk size
    pub fn with_chunk_size(arena_count: usize, chunk_size: usize) -> ContainerResult<Self> {
        let arenas = (0..arena_count)
            .map(|_| ArenaAllocator::with_chunk_size(chunk_size))
            .collect::<ContainerResult<Vec<_>>>()?;

        Ok(Self {
            arenas,
            current: AtomicUsize::new(0),
        })
    }

    /// Get an arena from the pool (round-robin)
    pub fn get_arena(&self) -> &ArenaAllocator {
        let idx = self.current.fetch_add(1, Ordering::Relaxed) % self.arenas.len();
        &self.arenas[idx]
    }

    /// Reset all arenas
    pub fn reset_all(&self) {
        for arena in &self.arenas {
            arena.reset();
        }
    }

    /// Get total allocated bytes across all arenas
    pub fn total_allocated(&self) -> usize {
        self.arenas.iter().map(|a| a.total_allocated()).sum()
    }

    /// Get total used bytes across all arenas
    pub fn total_used(&self) -> usize {
        self.arenas.iter().map(|a| a.total_used()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        let arena = ArenaAllocator::new().expect("Failed to create arena");

        let ptr1 = arena.allocate(100, 8).expect("Failed to allocate");
        assert!(!ptr1.as_ptr().is_null());

        let ptr2 = arena.allocate(200, 8).expect("Failed to allocate");
        assert!(!ptr2.as_ptr().is_null());

        assert!(arena.total_used() >= 300);
    }

    #[test]
    fn test_arena_reset() {
        let arena = ArenaAllocator::new().expect("Failed to create arena");

        arena.allocate(100, 8).expect("Failed to allocate");
        assert!(arena.total_used() > 0);

        arena.reset();
        assert_eq!(arena.total_used(), 0);
    }

    #[test]
    fn test_arena_slice() {
        let arena = ArenaAllocator::new().expect("Failed to create arena");
        let ptr = arena.allocate_slice::<u64>(10).expect("Failed to allocate slice");

        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr.as_ptr(), 10);
            for i in 0..10 {
                slice[i] = i as u64;
            }
            for i in 0..10 {
                assert_eq!(slice[i], i as u64);
            }
        }
    }

    #[test]
    fn test_arena_pool() {
        let pool = ArenaPool::new(4).expect("Failed to create pool");

        for _ in 0..10 {
            let arena = pool.get_arena();
            arena.allocate(100, 8).expect("Failed to allocate");
        }

        assert!(pool.total_used() > 0);
    }
}
