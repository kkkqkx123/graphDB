//! Cord implementation for efficient string manipulation
//!
//! This module provides a Cord data structure similar to NebulaGraph's Cord,
//! which efficiently handles large strings by storing them in blocks.

use std::fmt::Write;

const DEFAULT_BLOCK_SIZE: usize = 1024;

/// A Cord is a sequence of blocks for efficient string operations
pub struct Cord {
    blocks: Vec<Vec<u8>>,
    block_size: usize,
    total_len: usize,
}

impl Cord {
    /// Create a new empty Cord with default block size
    pub fn new() -> Self {
        Self::with_block_size(DEFAULT_BLOCK_SIZE)
    }

    /// Create a new empty Cord with a specific block size
    pub fn with_block_size(block_size: usize) -> Self {
        Cord {
            blocks: Vec::new(),
            block_size: std::cmp::max(block_size, 64), // Minimum block size
            total_len: 0,
        }
    }

    /// Get the total length of the Cord
    pub fn len(&self) -> usize {
        self.total_len
    }

    /// Check if the Cord is empty
    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }

    /// Add a string to the Cord
    pub fn append_str(&mut self, s: &str) -> &mut Self {
        self.write_all(s.as_bytes())
    }

    /// Add bytes to the Cord
    pub fn append_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.write_all(bytes)
    }

    /// Write data to the cord
    fn write_all(&mut self, data: &[u8]) -> &mut Self {
        if data.is_empty() {
            return self;
        }

        let mut remaining = data;
        while !remaining.is_empty() {
            // Get or create a block
            if self.blocks.is_empty()
                || self.blocks.last().map_or(0, |b| b.len()) == self.block_size
            {
                self.blocks.push(Vec::with_capacity(self.block_size));
            }

            let last_block = self
                .blocks
                .last_mut()
                .expect("Cord should have at least one block after push");
            let space_left = self.block_size - last_block.len();

            if space_left > 0 {
                let to_write = std::cmp::min(space_left, remaining.len());
                last_block.extend_from_slice(&remaining[..to_write]);
                remaining = &remaining[to_write..];
                self.total_len += to_write;
            }
        }

        self
    }

    /// Convert the Cord to a String
    pub fn to_string(&self) -> String {
        match String::from_utf8(self.flatten()) {
            Ok(s) => s,
            Err(_) => String::new(),
        }
    }

    /// Convert the Cord to Vec<u8>
    pub fn flatten(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.total_len);
        for block in &self.blocks {
            result.extend_from_slice(block);
        }
        result
    }

    /// Clear the Cord
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.total_len = 0;
    }

    /// Apply a function to each block in the Cord
    pub fn apply_to<F>(&self, mut f: F) -> bool
    where
        F: FnMut(&[u8]) -> bool,
    {
        for block in &self.blocks {
            if !f(block) {
                return false;
            }
        }
        true
    }
}

impl Default for Cord {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Cord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Debug for Cord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

// Implement write operators similar to the C++ version
impl Cord {
    /// Write any value that implements Display to the Cord
    pub fn write<T: std::fmt::Display>(&mut self, value: T) -> &mut Self {
        let mut s = String::new();
        write!(s, "{}", value).expect("String write should not fail for Display types");
        self.append_str(&s)
    }

    /// Write an i8 value to the Cord
    pub fn write_i8(&mut self, value: i8) -> &mut Self {
        self.write(value)
    }

    /// Write a u8 value to the Cord
    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.write(value)
    }

    /// Write an i16 value to the Cord
    pub fn write_i16(&mut self, value: i16) -> &mut Self {
        self.write(value)
    }

    /// Write a u16 value to the Cord
    pub fn write_u16(&mut self, value: u16) -> &mut Self {
        self.write(value)
    }

    /// Write an i32 value to the Cord
    pub fn write_i32(&mut self, value: i32) -> &mut Self {
        self.write(value)
    }

    /// Write a u32 value to the Cord
    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.write(value)
    }

    /// Write an i64 value to the Cord
    pub fn write_i64(&mut self, value: i64) -> &mut Self {
        self.write(value)
    }

    /// Write a u64 value to the Cord
    pub fn write_u64(&mut self, value: u64) -> &mut Self {
        self.write(value)
    }

    /// Write a char to the Cord
    pub fn write_char(&mut self, value: char) -> &mut Self {
        self.write(value)
    }

    /// Write a bool to the Cord
    pub fn write_bool(&mut self, value: bool) -> &mut Self {
        self.write(value)
    }

    /// Write a float to the Cord
    pub fn write_f32(&mut self, value: f32) -> &mut Self {
        self.write(value)
    }

    /// Write a double to the Cord
    pub fn write_f64(&mut self, value: f64) -> &mut Self {
        self.write(value)
    }

    /// Write a string to the Cord
    pub fn write_string(&mut self, value: &str) -> &mut Self {
        self.append_str(value)
    }

    /// Write a &str to the Cord
    pub fn write_str(&mut self, value: &str) -> &mut Self {
        self.append_str(value)
    }
}

// Operator overloading for convenient appending
impl Cord {
    pub fn append<T: std::fmt::Display>(&mut self, value: T) -> &mut Self {
        self.write(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cord_creation() {
        let cord = Cord::new();
        assert!(cord.is_empty());
        assert_eq!(cord.len(), 0);
    }

    #[test]
    fn test_cord_append_str() {
        let mut cord = Cord::new();
        cord.append_str("Hello");
        cord.append_str(" ");
        cord.append_str("World");

        assert_eq!(cord.len(), 11);
        assert_eq!(cord.to_string(), "Hello World");
    }

    #[test]
    fn test_cord_append_bytes() {
        let mut cord = Cord::new();
        cord.append_bytes(b"Hello");
        cord.append_bytes(b" ");
        cord.append_bytes(b"World");

        assert_eq!(cord.len(), 11);
        assert_eq!(cord.to_string(), "Hello World");
    }

    #[test]
    fn test_cord_write_values() {
        let mut cord = Cord::new();
        cord.write("Number: ")
            .write_i32(42)
            .write(", Float: ")
            .write_f64(3.14)
            .write(", Bool: ")
            .write_bool(true);

        assert_eq!(cord.to_string(), "Number: 42, Float: 3.14, Bool: true");
    }

    #[test]
    fn test_cord_flatten() {
        let mut cord = Cord::new();
        cord.append_str("Hello").append_str(" ").append_str("World");

        let flattened = cord.flatten();
        assert_eq!(flattened, b"Hello World");
    }

    #[test]
    fn test_cord_apply_to() {
        let mut cord = Cord::new();
        cord.append_str("Hello").append_str(" ").append_str("World");

        let mut result = Vec::new();
        cord.apply_to(|block| {
            result.push(block.to_vec());
            true
        });

        // Check that we got the blocks
        assert!(!result.is_empty());
    }

    #[test]
    fn test_cord_clear() {
        let mut cord = Cord::new();
        cord.append_str("Hello, World!");
        assert!(!cord.is_empty());

        cord.clear();
        assert!(cord.is_empty());
        assert_eq!(cord.len(), 0);
    }

    #[test]
    fn test_cord_with_small_block_size() {
        let mut cord = Cord::with_block_size(5); // Very small block size
        cord.append_str("Hello, this is a longer string for testing");

        // Make sure it still works correctly
        assert_eq!(
            cord.to_string(),
            "Hello, this is a longer string for testing"
        );
    }
}
