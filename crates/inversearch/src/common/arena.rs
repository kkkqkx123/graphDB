//! Arena Distributor Module
//!
//! Provides efficient memory allocation for temporary data structures and batch operations

use bumpalo::Bump;

/// Arena Distributor Packer
pub struct Arena {
    inner: Bump,
}

impl Arena {
    /// Create a new Arena with default capacity
    pub fn new() -> Self {
        Self { inner: Bump::new() }
    }

    /// Create a new Arena, specifying the initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Bump::with_capacity(capacity),
        }
    }

    /// Assign a value to the Arena
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.inner.alloc(value)
    }

    /// Assign a slice to Arena
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> &mut [T] {
        self.inner.alloc_slice_copy(slice)
    }

    /// Assign a string to Arena
    pub fn alloc_str(&self, s: &str) -> &mut str {
        self.inner.alloc_str(s)
    }

    /// Get the size of allocated memory
    pub fn allocated_bytes(&self) -> usize {
        self.inner.allocated_bytes()
    }

    /// Reset Arena, release all allocations
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get the underlying Bump allocator
    pub fn inner(&self) -> &Bump {
        &self.inner
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// Arena-based String Segmenter
pub struct ArenaTokenizer<'a> {
    arena: &'a Bump,
}

impl<'a> ArenaTokenizer<'a> {
    /// Creating a New Arena Segmenter
    pub fn new(arena: &'a Bump) -> Self {
        Self { arena }
    }

    /// Segmentation and storage in Arena
    pub fn tokenize(&self, text: &str) -> Vec<&'a str> {
        let mut tokens = Vec::new();

        for word in text.split_whitespace() {
            let token: &'a str = self.arena.alloc_str(word);
            tokens.push(token);
        }

        tokens
    }

    /// Segment and store to Arena (with delimiter)
    pub fn tokenize_with_sep(&self, text: &str, sep: char) -> Vec<&'a str> {
        let mut tokens = Vec::new();

        for word in text.split(sep) {
            if !word.is_empty() {
                let token: &'a str = self.arena.alloc_str(word);
                tokens.push(token);
            }
        }

        tokens
    }
}

/// Arena-based temporary vectors
pub struct ArenaVec<'a, T> {
    arena: &'a Bump,
    items: Vec<&'a mut T>,
}

impl<'a, T> ArenaVec<'a, T> {
    /// Create a new Arena vector
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            items: Vec::new(),
        }
    }

    /// Adding Elements
    pub fn push(&mut self, value: T) {
        let item = self.arena.alloc(value);
        self.items.push(item);
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if it is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get Element
    pub fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).map(|item| &**item)
    }

    /// Iterate over elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|item| &**item)
    }
}

/// Arena-based string builder
pub struct ArenaStringBuilder<'a> {
    arena: &'a Bump,
    buffer: Vec<&'a str>,
}

impl<'a> ArenaStringBuilder<'a> {
    /// Creating a new string builder
    pub fn new(arena: &'a Bump) -> Self {
        Self {
            arena,
            buffer: Vec::new(),
        }
    }

    /// Append string fragment
    pub fn append(&mut self, s: &str) {
        let slice = self.arena.alloc_str(s);
        self.buffer.push(slice);
    }

    /// Build the final string (needs to be copied to the heap)
    pub fn build(&self) -> String {
        self.buffer.concat()
    }

    /// Get all clips
    pub fn slices(&self) -> &[&'a str] {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_basic() {
        let mut arena = Arena::new();

        // Assign some value
        let a = arena.alloc(42);
        let b = arena.alloc("hello");

        assert_eq!(*a, 42);
        assert_eq!(*b, "hello");

        // It should be able to be reassigned after a reset
        arena.reset();
        let c = arena.alloc(100);
        assert_eq!(*c, 100);
    }

    #[test]
    fn test_arena_tokenizer() {
        let arena = Bump::new();
        let tokenizer = ArenaTokenizer::new(&arena);

        let tokens = tokenizer.tokenize("hello world rust");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], "hello");
        assert_eq!(tokens[1], "world");
        assert_eq!(tokens[2], "rust");
    }

    #[test]
    fn test_arena_vec() {
        let arena = Bump::new();
        let mut vec = ArenaVec::new(&arena);

        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(0), Some(&1));
        assert_eq!(vec.get(1), Some(&2));
        assert_eq!(vec.get(2), Some(&3));
    }

    #[test]
    fn test_arena_string_builder() {
        let arena = Bump::new();
        let mut builder = ArenaStringBuilder::new(&arena);

        builder.append("hello");
        builder.append(" ");
        builder.append("world");

        assert_eq!(builder.build(), "hello world");
    }
}
