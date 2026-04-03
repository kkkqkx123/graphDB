//! 压缩/解压缩模块
//!
//! 提供不同压缩算法（Zstd、Lz4）的数据压缩和解压缩功能

use crate::error::Result;
use crate::serialize::types::CompressionAlgorithm;

/// 压缩数据
pub fn compress_data(data: &[u8], algorithm: CompressionAlgorithm, level: i32) -> Result<Vec<u8>> {
    match algorithm {
        CompressionAlgorithm::None => Ok(data.to_vec()),
        CompressionAlgorithm::Zstd => zstd::stream::encode_all(data, level).map_err(|e| {
            crate::error::InversearchError::Serialization(format!("Compression error: {}", e))
        }),
        CompressionAlgorithm::Lz4 => Ok(lz4_flex::compress(data)),
    }
}

/// 解压缩数据
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
