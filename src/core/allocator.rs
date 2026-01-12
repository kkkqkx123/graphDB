//! Arena allocator for efficient memory management
//!
//! This module provides an arena allocator similar to NebulaGraph's Arena,
//! optimized for allocating many small objects with the same lifetime.
//!
//! # Safety
//! This module uses unsafe code for low-level memory management. All unsafe blocks
//! are carefully bounded and follow Rust's safety invariants:
//! - Memory allocations are properly aligned and sized
//! - Pointers are always valid when dereferenced
//! - Memory is properly deallocated on drop

use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

/// Minimum chunk size for arena allocation (4KB)
const MIN_CHUNK_SIZE: usize = 4096;
/// Maximum chunk size for arena allocation
// const MAX_CHUNK_SIZE: usize = u16::MAX as usize; // 注释掉未使用的常量
/// Memory alignment for allocations
const ALIGNMENT: usize = std::mem::align_of::<usize>(); // 使用usize的对齐方式

/// Type representing maximum alignment
#[repr(align(16))]
// struct MaxAlignT(u8); // 注释掉未使用的对齐类型

/// A chunk of memory in the arena
struct Chunk {
    data: *mut u8,
    size: usize,
    next: Option<Box<Chunk>>,
}

impl Chunk {
    /// Create a new chunk with the specified size
    ///
    /// # Safety
    /// This function uses unsafe code to allocate raw memory.
    /// The returned pointer is guaranteed to be valid for the specified size
    /// and properly aligned.
    fn new(size: usize) -> Option<Box<Chunk>> {
        if size == 0 {
            return None;
        }

        let layout = Layout::from_size_align(size, ALIGNMENT).ok()?;
        unsafe {
            let data = alloc(layout);
            if data.is_null() {
                return None;
            }
            Some(Box::new(Chunk {
                data,
                size,
                next: None,
            }))
        }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        // Safety: self.data was allocated with Layout::from_size_align(self.size, ALIGNMENT)
        // and is still valid at this point.
        let layout = Layout::from_size_align(self.size, ALIGNMENT)
            .expect("Chunk size should be valid for layout creation");
        unsafe {
            dealloc(self.data, layout);
        }
    }
}

/// Arena allocator for efficient small object allocation
pub struct Arena {
    chunks: Option<Box<Chunk>>,
    current_ptr: *mut u8,
    end_ptr: *mut u8,
    #[cfg(debug_assertions)]
    allocated_size: usize,
}

impl Arena {
    /// Create a new arena allocator
    pub fn new() -> Self {
        Arena {
            chunks: None,
            current_ptr: ptr::null_mut(),
            end_ptr: ptr::null_mut(),
            #[cfg(debug_assertions)]
            allocated_size: 0,
        }
    }

    /// Allocate memory with the specified size and alignment
    pub fn allocate(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        if size == 0 {
            return None;
        }

        // Calculate aligned size
        let adjusted_size = (size + align - 1) & !(align - 1);

        // Check if we have enough space in the current chunk
        if self.has_space(adjusted_size) {
            let ptr = self.current_ptr;
            self.current_ptr = unsafe { self.current_ptr.add(adjusted_size) };

            #[cfg(debug_assertions)]
            {
                self.allocated_size += adjusted_size;
            }

            Some(ptr)
        } else {
            // Allocate a new chunk
            self.allocate_new_chunk(adjusted_size)
        }
    }

    /// Allocate memory with default alignment
    pub fn alloc(&mut self, size: usize) -> Option<*mut u8> {
        self.allocate(size, ALIGNMENT)
    }

    /// Allocate aligned memory (similar to C++ Arena::allocateAligned)
    pub fn allocate_aligned(&mut self, size: usize) -> Option<*mut u8> {
        self.allocate(size, ALIGNMENT)
    }

    /// Check if the current chunk has enough space
    ///
    /// # Safety
    /// This function assumes that current_ptr and end_ptr are valid pointers
    /// pointing to the same allocation.
    fn has_space(&self, size: usize) -> bool {
        if self.current_ptr.is_null() || self.end_ptr.is_null() {
            return false;
        }

        // Safety: Both pointers are non-null and point to the same allocation
        unsafe {
            let offset = self.end_ptr.offset_from(self.current_ptr) as usize;
            offset >= size
        }
    }

    /// Allocate a new chunk and allocate the requested memory in it
    fn allocate_new_chunk(&mut self, size: usize) -> Option<*mut u8> {
        // Calculate chunk size (at least the requested size, but aligned to MIN_CHUNK_SIZE)
        let chunk_size = std::cmp::max(size, MIN_CHUNK_SIZE);

        let new_chunk = Chunk::new(chunk_size)?;

        // Save the old chunk as the next chunk
        let old_chunks = self.chunks.take();
        let mut boxed_chunk = new_chunk;
        boxed_chunk.next = old_chunks;

        // Update pointers
        self.current_ptr = boxed_chunk.data;
        self.end_ptr = unsafe { self.current_ptr.add(boxed_chunk.size) };
        self.chunks = Some(boxed_chunk);

        // Now allocate from the new chunk
        if self.has_space(size) {
            let ptr = self.current_ptr;
            self.current_ptr = unsafe { self.current_ptr.add(size) };

            #[cfg(debug_assertions)]
            {
                self.allocated_size += size;
            }

            Some(ptr)
        } else {
            // This shouldn't happen if our math is correct
            None
        }
    }

    /// Allocate and initialize a value in the arena
    pub fn alloc_item<T>(&mut self, value: T) -> Option<&mut T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        if let Some(ptr) = self.allocate(size, align) {
            unsafe {
                std::ptr::write(ptr as *mut T, value);
                Some(&mut *(ptr as *mut T))
            }
        } else {
            None
        }
    }

    /// Get the total allocated size (only available in debug builds)
    #[cfg(debug_assertions)]
    pub fn allocated_size(&self) -> usize {
        self.allocated_size
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        // All chunks will be automatically dropped when self.chunks is dropped
        // due to the Box chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_allocation() {
        let mut arena = Arena::new();

        // Allocate some memory
        let ptr1 = arena.alloc(10).expect("Allocation should succeed");
        let ptr2 = arena.alloc(20).expect("Allocation should succeed");

        // Write some data
        unsafe {
            std::ptr::write_bytes(ptr1, 1, 10);
            std::ptr::write_bytes(ptr2, 2, 20);
        }

        // Verify the data
        unsafe {
            for i in 0..10 {
                assert_eq!(*ptr1.add(i), 1);
            }
            for i in 0..20 {
                assert_eq!(*ptr2.add(i), 2);
            }
        }
    }

    #[test]
    fn test_arena_item_allocation() {
        let mut arena = Arena::new();

        // Allocate and store a value
        let value: &mut i32 = arena
            .alloc_item(42)
            .expect("Item allocation should succeed");
        assert_eq!(*value, 42);

        // Modify the value
        *value = 100;
        assert_eq!(*value, 100);
    }

    #[test]
    fn test_arena_aligned_allocation() {
        let mut arena = Arena::new();

        // Allocate aligned memory
        let ptr = arena
            .allocate_aligned(32)
            .expect("Allocation should succeed");

        // Check alignment
        assert_eq!(ptr as usize % ALIGNMENT, 0);
    }
}
