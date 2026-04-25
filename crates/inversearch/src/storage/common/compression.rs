//! Compression/Decompression Functions
//!
//! Provides data compression and decompression for scenarios such as WAL snapshots

use crate::error::{InversearchError, Result};

/// Compressed data
///
/// Compressing data using the zstd algorithm
///
/// # Parameters
/// - `data`: 原始数据
/// - `level`: 压缩级别 (1-22)，值越大压缩率越高但速度越慢
///
/// # Back
/// compressed data
pub fn compress_data(data: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::stream::encode_all(data, level)
        .map_err(|e| InversearchError::Serialization(format!("Compression error: {}", e)))
}

/// Decompression data
///
/// Decompressing data using the zstd algorithm
///
/// # Parameters
/// - `data`: 压缩后的数据
///
/// # Back
/// Raw data after decompression
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    zstd::stream::decode_all(data)
        .map_err(|e| InversearchError::Serialization(format!("Decompression error: {}", e)))
}

/// Try to decompress the data
///
/// If decompression fails, return raw data (possibly uncompressed)
pub fn try_decompress(data: &[u8]) -> Result<Vec<u8>> {
    match decompress_data(data) {
        Ok(decompressed) => Ok(decompressed),
        Err(_) => Ok(data.to_vec()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original = b"hello world, this is a test string for compression!";
        let compressed = compress_data(original, 3).unwrap();
        let decompressed = decompress_data(&compressed).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_try_decompress_compressed() {
        let original = b"test data";
        let compressed = compress_data(original, 3).unwrap();
        let result = try_decompress(&compressed).unwrap();

        assert_eq!(result, original);
    }

    #[test]
    fn test_try_decompress_uncompressed() {
        let original = b"uncompressed data";
        let result = try_decompress(original).unwrap();

        assert_eq!(result, original);
    }
}
