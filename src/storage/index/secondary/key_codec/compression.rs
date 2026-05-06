//! Index Key Compression
//!
//! This module provides compression algorithms for index keys.
//! Compression can significantly reduce memory usage for large indexes.
//!
//! ## Compression Types
//!
//! - `Prefix`: Removes common prefixes from keys
//! - `Dictionary`: Replaces frequent values with shorter IDs
//! - `Delta`: Stores only differences from a base key

use crate::core::{StorageError, StorageResult};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Prefix,
    Dictionary,
    Delta,
}

#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub compression_type: CompressionType,
    pub min_prefix_length: usize,
    pub dictionary_threshold: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            compression_type: CompressionType::Prefix,
            min_prefix_length: 4,
            dictionary_threshold: 100,
        }
    }
}

impl CompressionConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_compression_type(mut self, compression_type: CompressionType) -> Self {
        self.compression_type = compression_type;
        self
    }

    pub fn with_min_prefix_length(mut self, min_prefix_length: usize) -> Self {
        self.min_prefix_length = min_prefix_length;
        self
    }

    pub fn with_dictionary_threshold(mut self, threshold: usize) -> Self {
        self.dictionary_threshold = threshold;
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.compression_type != CompressionType::None
    }
}

#[derive(Debug, Clone)]
pub struct PrefixCompressor {
    common_prefix: Vec<u8>,
}

impl PrefixCompressor {
    pub fn new() -> Self {
        Self {
            common_prefix: Vec::new(),
        }
    }

    pub fn train(keys: &[Vec<u8>]) -> StorageResult<Self> {
        if keys.is_empty() {
            return Ok(Self::new());
        }

        let first = &keys[0];
        let mut prefix_len = 0;

        for i in 0..first.len() {
            let byte = first[i];
            if keys.iter().all(|k| k.len() > i && k[i] == byte) {
                prefix_len = i + 1;
            } else {
                break;
            }
        }

        Ok(Self {
            common_prefix: first[..prefix_len].to_vec(),
        })
    }

    pub fn compress(&self, key: &[u8]) -> Vec<u8> {
        if self.common_prefix.is_empty() || key.len() < self.common_prefix.len() {
            return key.to_vec();
        }

        if &key[..self.common_prefix.len()] == self.common_prefix.as_slice() {
            let mut result = vec![0x01];
            result.extend_from_slice(&key[self.common_prefix.len()..]);
            result
        } else {
            let mut result = vec![0x00];
            result.extend_from_slice(key);
            result
        }
    }

    pub fn decompress(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        if compressed.is_empty() {
            return Ok(Vec::new());
        }

        match compressed[0] {
            0x01 => {
                let mut result = self.common_prefix.clone();
                result.extend_from_slice(&compressed[1..]);
                Ok(result)
            }
            0x00 => Ok(compressed[1..].to_vec()),
            _ => Err(StorageError::InvalidInput(
                "Invalid compression flag".to_string(),
            )),
        }
    }

    pub fn prefix(&self) -> &[u8] {
        &self.common_prefix
    }

    pub fn prefix_len(&self) -> usize {
        self.common_prefix.len()
    }
}

impl Default for PrefixCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DictionaryCompressor {
    dictionary: HashMap<Vec<u8>, u32>,
    reverse_dict: Vec<Vec<u8>>,
}

impl DictionaryCompressor {
    pub fn new() -> Self {
        Self {
            dictionary: HashMap::new(),
            reverse_dict: Vec::new(),
        }
    }

    pub fn train(values: &[Vec<u8>], threshold: usize) -> Self {
        let mut freq: HashMap<Vec<u8>, usize> = HashMap::new();

        for value in values {
            *freq.entry(value.clone()).or_insert(0) += 1;
        }

        let mut dictionary: HashMap<Vec<u8>, u32> = HashMap::new();
        let mut reverse_dict: Vec<Vec<u8>> = Vec::new();
        let mut next_id: u32 = 0;

        for (value, count) in freq {
            if count >= threshold {
                dictionary.insert(value.clone(), next_id);
                reverse_dict.push(value);
                next_id += 1;
            }
        }

        Self {
            dictionary,
            reverse_dict,
        }
    }

    pub fn compress(&self, value: &[u8]) -> Vec<u8> {
        if let Some(&id) = self.dictionary.get(value) {
            let mut result = vec![0x01];
            result.extend_from_slice(&id.to_le_bytes());
            result
        } else {
            let mut result = vec![0x00];
            let len = value.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
            result.extend_from_slice(value);
            result
        }
    }

    pub fn decompress(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        if compressed.is_empty() {
            return Ok(Vec::new());
        }

        match compressed[0] {
            0x01 => {
                if compressed.len() < 5 {
                    return Err(StorageError::InvalidInput(
                        "Invalid dictionary compressed data".to_string(),
                    ));
                }
                let id = u32::from_le_bytes([
                    compressed[1],
                    compressed[2],
                    compressed[3],
                    compressed[4],
                ]);
                self.reverse_dict.get(id as usize).cloned().ok_or_else(|| {
                    StorageError::InvalidInput("Dictionary ID not found".to_string())
                })
            }
            0x00 => {
                if compressed.len() < 3 {
                    return Err(StorageError::InvalidInput(
                        "Invalid literal compressed data".to_string(),
                    ));
                }
                let len = u16::from_le_bytes([compressed[1], compressed[2]]) as usize;
                if compressed.len() < 3 + len {
                    return Err(StorageError::InvalidInput(
                        "Compressed data truncated".to_string(),
                    ));
                }
                Ok(compressed[3..3 + len].to_vec())
            }
            _ => Err(StorageError::InvalidInput(
                "Invalid compression flag".to_string(),
            )),
        }
    }

    pub fn dictionary_size(&self) -> usize {
        self.reverse_dict.len()
    }
}

impl Default for DictionaryCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DeltaCompressor {
    base: Vec<u8>,
}

impl DeltaCompressor {
    pub fn new() -> Self {
        Self { base: Vec::new() }
    }

    pub fn with_base(base: Vec<u8>) -> Self {
        Self { base }
    }

    pub fn compress(&self, key: &[u8]) -> Vec<u8> {
        if self.base.is_empty() {
            let mut result = vec![0x00];
            let len = key.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
            result.extend_from_slice(key);
            return result;
        }

        let common_len = key.len().min(self.base.len());
        let mut delta_start = 0;

        for i in 0..common_len {
            if key[i] != self.base[i] {
                delta_start = i;
                break;
            }
            delta_start = i + 1;
        }

        if delta_start == common_len && key.len() >= self.base.len() {
            let mut result = vec![0x01];
            result.push(delta_start as u8);
            let suffix_len = (key.len() - delta_start) as u8;
            result.push(suffix_len);
            result.extend_from_slice(&key[delta_start..]);
            result
        } else {
            let mut result = vec![0x00];
            let len = key.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
            result.extend_from_slice(key);
            result
        }
    }

    pub fn decompress(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        if compressed.is_empty() {
            return Ok(Vec::new());
        }

        match compressed[0] {
            0x01 => {
                if compressed.len() < 3 {
                    return Err(StorageError::InvalidInput(
                        "Invalid delta compressed data".to_string(),
                    ));
                }
                let delta_start = compressed[1] as usize;
                let suffix_len = compressed[2] as usize;

                if compressed.len() < 3 + suffix_len {
                    return Err(StorageError::InvalidInput(
                        "Delta data truncated".to_string(),
                    ));
                }

                let mut result = self.base.clone();
                if delta_start + suffix_len > result.len() {
                    result.resize(delta_start + suffix_len, 0);
                }
                result[delta_start..delta_start + suffix_len]
                    .copy_from_slice(&compressed[3..3 + suffix_len]);
                Ok(result)
            }
            0x00 => {
                if compressed.len() < 3 {
                    return Err(StorageError::InvalidInput(
                        "Invalid literal data".to_string(),
                    ));
                }
                let len = u16::from_le_bytes([compressed[1], compressed[2]]) as usize;
                if compressed.len() < 3 + len {
                    return Err(StorageError::InvalidInput(
                        "Compressed data truncated".to_string(),
                    ));
                }
                Ok(compressed[3..3 + len].to_vec())
            }
            _ => Err(StorageError::InvalidInput(
                "Invalid compression flag".to_string(),
            )),
        }
    }

    pub fn set_base(&mut self, base: Vec<u8>) {
        self.base = base;
    }

    pub fn base(&self) -> &[u8] {
        &self.base
    }
}

impl Default for DeltaCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IndexCompressor {
    config: CompressionConfig,
    prefix_compressor: Option<PrefixCompressor>,
    dictionary_compressor: Option<DictionaryCompressor>,
}

impl IndexCompressor {
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            prefix_compressor: None,
            dictionary_compressor: None,
        }
    }

    pub fn with_default() -> Self {
        Self::new(CompressionConfig::default())
    }

    pub fn disabled() -> Self {
        Self::new(CompressionConfig {
            compression_type: CompressionType::None,
            ..Default::default()
        })
    }

    pub fn train_keys(&mut self, keys: &[Vec<u8>]) -> StorageResult<()> {
        match self.config.compression_type {
            CompressionType::Prefix => {
                self.prefix_compressor = Some(PrefixCompressor::train(keys)?);
            }
            CompressionType::Dictionary => {
                let values: Vec<Vec<u8>> = keys.to_vec();
                self.dictionary_compressor = Some(DictionaryCompressor::train(
                    &values,
                    self.config.dictionary_threshold,
                ));
            }
            _ => {}
        }
        Ok(())
    }

    pub fn compress_key(&self, key: &[u8]) -> Vec<u8> {
        match self.config.compression_type {
            CompressionType::Prefix => self
                .prefix_compressor
                .as_ref()
                .map(|c| c.compress(key))
                .unwrap_or_else(|| key.to_vec()),
            CompressionType::Dictionary => self
                .dictionary_compressor
                .as_ref()
                .map(|c| c.compress(key))
                .unwrap_or_else(|| key.to_vec()),
            _ => key.to_vec(),
        }
    }

    pub fn decompress_key(&self, compressed: &[u8]) -> StorageResult<Vec<u8>> {
        match self.config.compression_type {
            CompressionType::Prefix => self
                .prefix_compressor
                .as_ref()
                .map(|c| c.decompress(compressed))
                .unwrap_or_else(|| Ok(compressed.to_vec())),
            CompressionType::Dictionary => self
                .dictionary_compressor
                .as_ref()
                .map(|c| c.decompress(compressed))
                .unwrap_or_else(|| Ok(compressed.to_vec())),
            _ => Ok(compressed.to_vec()),
        }
    }

    pub fn compression_ratio(&self, original: &[Vec<u8>], compressed: &[Vec<u8>]) -> f64 {
        if original.is_empty() || compressed.is_empty() {
            return 0.0;
        }

        let original_size: usize = original.iter().map(|v| v.len()).sum();
        let compressed_size: usize = compressed.iter().map(|v| v.len()).sum();

        if original_size == 0 {
            return 0.0;
        }

        1.0 - (compressed_size as f64 / original_size as f64)
    }

    pub fn is_enabled(&self) -> bool {
        self.config.is_enabled()
    }

    pub fn config(&self) -> &CompressionConfig {
        &self.config
    }
}

impl Default for IndexCompressor {
    fn default() -> Self {
        Self::with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_compressor() {
        let keys = vec![
            b"prefix_key1".to_vec(),
            b"prefix_key2".to_vec(),
            b"prefix_key3".to_vec(),
        ];

        let compressor = PrefixCompressor::train(&keys).expect("Training failed");
        assert_eq!(compressor.prefix(), b"prefix_key");

        let compressed = compressor.compress(b"prefix_key4");
        assert!(compressed.len() < b"prefix_key4".len());

        let decompressed = compressor
            .decompress(&compressed)
            .expect("Decompression failed");
        assert_eq!(decompressed, b"prefix_key4".to_vec());
    }

    #[test]
    fn test_dictionary_compressor() {
        let values = vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
            b"value1".to_vec(),
            b"value1".to_vec(),
            b"value2".to_vec(),
        ];

        let compressor = DictionaryCompressor::train(&values, 2);
        assert!(compressor.dictionary_size() >= 2);

        let compressed = compressor.compress(b"value1");
        assert!(compressed.len() < b"value1".len());

        let decompressed = compressor
            .decompress(&compressed)
            .expect("Decompression failed");
        assert_eq!(decompressed, b"value1".to_vec());
    }

    #[test]
    fn test_delta_compressor() {
        let compressor = DeltaCompressor::with_base(b"key_prefix_001".to_vec());

        let compressed = compressor.compress(b"key_prefix_002");

        let decompressed = compressor
            .decompress(&compressed)
            .expect("Decompression failed");
        assert_eq!(decompressed, b"key_prefix_002".to_vec());
    }

    #[test]
    fn test_index_compressor() {
        let config = CompressionConfig {
            compression_type: CompressionType::Prefix,
            min_prefix_length: 4,
            dictionary_threshold: 100,
        };

        let mut compressor = IndexCompressor::new(config);

        let keys = vec![
            b"index_key_001".to_vec(),
            b"index_key_002".to_vec(),
            b"index_key_003".to_vec(),
        ];

        compressor.train_keys(&keys).expect("Training failed");

        let compressed: Vec<Vec<u8>> = keys.iter().map(|k| compressor.compress_key(k)).collect();
        let ratio = compressor.compression_ratio(&keys, &compressed);

        assert!(ratio > 0.0, "Compression should reduce size");
    }

    #[test]
    fn test_compression_config() {
        let config = CompressionConfig::default();
        assert_eq!(config.compression_type, CompressionType::Prefix);
        assert!(config.is_enabled());

        let disabled_config = CompressionConfig::new()
            .with_compression_type(CompressionType::None);
        assert!(!disabled_config.is_enabled());
    }
}
