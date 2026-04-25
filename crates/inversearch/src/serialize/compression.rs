//! Compression/Decompression Module
//!
//! Provide data compression and decompression functions with different compression algorithms (Zstd, Lz4)

use crate::error::Result;
use crate::serialize::types::CompressionAlgorithm;

/// Compressed data
pub fn compress_data(data: &[u8], algorithm: CompressionAlgorithm, level: i32) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Zstd => zstd::stream::encode_all(data, level).map_err(|e| {
            crate::error::InversearchError::Serialization(format!("Compression error: {}", e))
        }),
        CompressionAlgorithm::Lz4 => Ok(lz4_flex::compress(data)),
    }
}

/// Decompression data
pub fn decompress_data(data: &[u8], algorithm: CompressionAlgorithm) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Zstd => zstd::stream::decode_all(data).map_err(|e| {
            crate::error::InversearchError::Deserialization(format!("Decompression error: {}", e))
        }),
        CompressionAlgorithm::Lz4 => lz4_flex::decompress(data, usize::MAX).map_err(|e| {
            crate::error::InversearchError::Deserialization(format!(
                "Lz4 decompression error: {}",
                e
            ))
        }),
    }
}
