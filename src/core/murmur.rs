//! MurmurHash2 implementation
//!
//! This module provides MurmurHash2 algorithm implementation similar to NebulaGraph's MurmurHash2.

use std::hash::Hasher;

const M: u32 = 0x5bd1e995;
const R: u8 = 24;

/// Compute MurmurHash2 for byte data with a given seed
pub fn murmurhash2(data: &[u8], seed: u32) -> u32 {
    let mut h: u32 = seed ^ (data.len() as u32);
    let mut pos = 0;

    // Process 4-byte chunks
    while pos + 4 <= data.len() {
        let mut k = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        k = k.wrapping_mul(M);
        k ^= k >> R;
        k = k.wrapping_mul(M);
        h = h.wrapping_mul(M) ^ k;
        pos += 4;
    }

    // Handle remaining bytes
    let remaining = data.len() - pos;
    if remaining >= 3 {
        h ^= (data[pos + 2] as u32) << 16;
    }
    if remaining >= 2 {
        h ^= (data[pos + 1] as u32) << 8;
    }
    if remaining >= 1 {
        h ^= data[pos] as u32;
        h = h.wrapping_mul(M);
    }

    h ^= h >> 13;
    h = h.wrapping_mul(M);
    h ^ (h >> 15)
}

/// Compute MurmurHash2 for string data with a given seed
pub fn murmurhash2_str(s: &str, seed: u32) -> u32 {
    murmurhash2(s.as_bytes(), seed)
}

/// Compute MurmurHash2 for integer with a given seed
pub fn murmurhash2_int(n: u32, seed: u32) -> u32 {
    murmurhash2(&n.to_le_bytes(), seed)
}

/// Compute MurmurHash2 for 64-bit integer with a given seed
pub fn murmurhash2_int64(n: u64, seed: u32) -> u32 {
    murmurhash2(&n.to_le_bytes(), seed)
}

/// Compute MurmurHash2 for f64 with a given seed
pub fn murmurhash2_f64(f: f64, seed: u32) -> u32 {
    murmurhash2(&f.to_bits().to_le_bytes(), seed)
}

/// Compute MurmurHash2 for f32 with a given seed
pub fn murmurhash2_f32(f: f32, seed: u32) -> u32 {
    murmurhash2(&f.to_bits().to_le_bytes(), seed)
}

/// A Hasher implementation that uses MurmurHash2 algorithm
#[derive(Debug)]
pub struct MurmurHasher {
    seed: u32,
    buffer: Vec<u8>,
}

impl MurmurHasher {
    /// Create a new MurmurHasher with the default seed
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Create a new MurmurHasher with a specific seed
    pub fn with_seed(seed: u32) -> Self {
        MurmurHasher {
            seed,
            buffer: Vec::new(),
        }
    }
}

impl Default for MurmurHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for MurmurHasher {
    fn finish(&self) -> u64 {
        murmurhash2(&self.buffer, self.seed) as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    fn write_u8(&mut self, i: u8) {
        self.buffer.push(i);
    }

    fn write_u16(&mut self, i: u16) {
        self.buffer.extend_from_slice(&i.to_le_bytes());
    }

    fn write_u32(&mut self, i: u32) {
        self.buffer.extend_from_slice(&i.to_le_bytes());
    }

    fn write_u64(&mut self, i: u64) {
        self.buffer.extend_from_slice(&i.to_le_bytes());
    }

    fn write_u128(&mut self, i: u128) {
        self.buffer.extend_from_slice(&i.to_le_bytes());
    }

    fn write_usize(&mut self, i: usize) {
        self.buffer.extend_from_slice(&i.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_murmurhash2_consistency() {
        // Same input should always produce same hash
        let data = b"hello world";
        let seed = 42;
        let hash1 = murmurhash2(data, seed);
        let hash2 = murmurhash2(data, seed);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_murmurhash2_different_inputs() {
        // Different inputs should likely produce different hashes
        let data1 = b"hello world";
        let data2 = b"hello worle"; // Different by 1 character
        let seed = 42;
        let hash1 = murmurhash2(data1, seed);
        let hash2 = murmurhash2(data2, seed);
        // Note: This might occasionally collide, but should be rare
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_murmurhash2_different_seeds() {
        // Same input with different seeds should produce different hashes
        let data = b"hello world";
        let hash1 = murmurhash2(data, 42);
        let hash2 = murmurhash2(data, 43);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_murmurhash2_empty() {
        let hash = murmurhash2(b"", 0);
        // Hash of empty data should be deterministic
        assert_eq!(hash, murmurhash2(b"", 0));
    }

    #[test]
    fn test_murmurhash2_various_sizes() {
        // Test with various data sizes
        assert_ne!(murmurhash2(b"a", 0), 0);
        assert_ne!(murmurhash2(b"ab", 0), 0);
        assert_ne!(murmurhash2(b"abc", 0), 0);
        assert_ne!(murmurhash2(b"abcd", 0), 0);
        assert_ne!(
            murmurhash2(b"hello world this is a longer test string", 0),
            0
        );
    }

    #[test]
    fn test_murmurhash2_str() {
        let s = "hello world";
        let hash1 = murmurhash2(s.as_bytes(), 42);
        let hash2 = murmurhash2_str(s, 42);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_murmurhasher() {
        // Test the Hasher implementation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = MurmurHasher::new();
        hasher.write(b"hello world");
        let murmur_hash = hasher.finish();

        // Compare with Rust's default hasher (should be different)
        let mut default_hasher = DefaultHasher::new();
        default_hasher.write(b"hello world");
        let default_hash = default_hasher.finish();

        // The hashes should be different (different algorithms)
        assert_ne!(murmur_hash, default_hash);
    }

    #[test]
    fn test_murmurhasher_with_different_types() {
        // Test that MurmurHasher can handle different data types
        use std::hash::Hasher;

        let mut hasher = MurmurHasher::new();
        hasher.write_u32(42);
        hasher.write_u64(12345);
        let result = hasher.finish();

        assert_ne!(result, 0);
    }
}
