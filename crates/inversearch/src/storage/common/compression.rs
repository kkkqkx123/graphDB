//! 压缩/解压缩功能
//!
//! 提供数据压缩和解压缩功能，用于 WAL 快照等场景

use crate::error::{InversearchError, Result};

/// 压缩数据
///
/// 使用 zstd 算法压缩数据
///
/// # 参数
/// - `data`: 原始数据
/// - `level`: 压缩级别 (1-22)，值越大压缩率越高但速度越慢
///
/// # 返回
/// 压缩后的数据
pub fn compress_data(data: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::stream::encode_all(data, level)
        .map_err(|e| InversearchError::Serialization(format!("Compression error: {}", e)))
}

/// 解压缩数据
///
/// 使用 zstd 算法解压缩数据
///
/// # 参数
/// - `data`: 压缩后的数据
///
/// # 返回
/// 解压缩后的原始数据
pub fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    zstd::stream::decode_all(data)
        .map_err(|e| InversearchError::Serialization(format!("Decompression error: {}", e)))
}

/// 尝试解压缩数据
///
/// 如果解压缩失败，返回原始数据（可能未压缩）
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
