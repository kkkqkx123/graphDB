use std::io;

use crate::core::{StorageError, StorageResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    None,
    Zstd { level: i32 },
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::Zstd { level: 3 }
    }
}

impl CompressionType {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => CompressionType::None,
            2 => CompressionType::Zstd { level: 3 },
            _ => CompressionType::None,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            CompressionType::None => 0,
            CompressionType::Zstd { .. } => 2,
        }
    }
}

pub struct Compressor {
    compression: CompressionType,
}

impl Compressor {
    pub fn new(compression: CompressionType) -> Self {
        Self { compression }
    }

    pub fn compression_type(&self) -> CompressionType {
        self.compression
    }

    pub fn compress(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        match self.compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Zstd { level } => zstd::encode_all(data, level)
                .map_err(|e| StorageError::CompressError(e.to_string())),
        }
    }

    pub fn decompress(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        match self.compression {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Zstd { .. } => {
                zstd::decode_all(data).map_err(|e| StorageError::DecompressError(e.to_string()))
            }
        }
    }

    pub fn compress_size_estimate(&self, data_len: usize) -> usize {
        match self.compression {
            CompressionType::None => data_len,
            CompressionType::Zstd { .. } => data_len + (data_len / 10),
        }
    }
}

impl Default for Compressor {
    fn default() -> Self {
        Self::new(CompressionType::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_type_conversion() {
        assert_eq!(CompressionType::from_u8(0), CompressionType::None);
        assert_eq!(
            CompressionType::from_u8(2),
            CompressionType::Zstd { level: 3 }
        );

        assert_eq!(CompressionType::None.to_u8(), 0);
        assert_eq!(CompressionType::Zstd { level: 3 }.to_u8(), 2);
    }

    #[test]
    fn test_no_compression() {
        let compressor = Compressor::new(CompressionType::None);
        let data = b"hello world";

        let compressed = compressor.compress(data).expect("Compress failed");
        assert_eq!(compressed, data);

        let decompressed = compressor
            .decompress(&compressed)
            .expect("Decompress failed");
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_size_estimate() {
        let compressor = Compressor::new(CompressionType::None);
        assert_eq!(compressor.compress_size_estimate(1000), 1000);
    }

    #[test]
    fn test_zstd_compression() {
        let compressor = Compressor::new(CompressionType::Zstd { level: 3 });
        let data = b"hello world, this is a test for zstd compression";

        let compressed = compressor.compress(data).expect("Compress failed");
        assert!(compressed.len() < data.len(), "Zstd should compress data");

        let decompressed = compressor
            .decompress(&compressed)
            .expect("Decompress failed");
        assert_eq!(decompressed, data);
    }
}
