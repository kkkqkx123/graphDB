//! Compression type definition and compression/decompression helpers for storage layer.
//!
//! This module provides the `CompressionType` enum for configuring
//! compression in flush operations, along with `compress_payload` and
//! `decompress_payload` helpers used by the table flush/load pipeline.
//!
//! Every persisted file uses the compression marker format:
//! - Marker 0x00: raw data follows
//! - Marker 0x01: [4-byte CRC32][4-byte compressed_len][zstd compressed data]
//!
//! Files without a marker (older format) are rejected. There is no
//! backward compatibility with pre-marker file formats.

use crate::core::{StorageError, StorageResult};

/// Compression type with optional compression level
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
    const NONE_TAG: u8 = 0;
    const ZSTD_TAG: u8 = 1;

    pub fn from_u8(value: u8) -> Self {
        if value == Self::NONE_TAG {
            CompressionType::None
        } else {
            let level = ((value >> 4) & 0x0F) as i32;
            let level = if level == 0 { 3 } else { level };
            CompressionType::Zstd { level }
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            CompressionType::None => Self::NONE_TAG,
            CompressionType::Zstd { level } => {
                let clamped_level = level.clamp(1, 15) as u8;
                Self::ZSTD_TAG | (clamped_level << 4)
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, CompressionType::Zstd { .. })
    }

    pub fn level(&self) -> Option<i32> {
        match self {
            CompressionType::None => None,
            CompressionType::Zstd { level } => Some(*level),
        }
    }
}

const COMPRESSION_MARKER_NONE: u8 = 0x00;
const COMPRESSION_MARKER_ZSTD: u8 = 0x01;

/// Compress payload with the given strategy.
/// Output format: [1-byte marker][payload]
/// - Marker 0x00: raw data follows
/// - Marker 0x01: [4-byte CRC32][4-byte compressed_len][compressed data]
pub fn compress_payload(data: &[u8], ct: CompressionType) -> StorageResult<Vec<u8>> {
    let mut result = Vec::new();
    match ct {
        CompressionType::None => {
            result.push(COMPRESSION_MARKER_NONE);
            result.extend_from_slice(data);
        }
        CompressionType::Zstd { level } => {
            result.push(COMPRESSION_MARKER_ZSTD);
            let compressed = zstd::encode_all(data, level)
                .map_err(|e| StorageError::io_error(format!("zstd compress failed: {}", e)))?;
            let checksum = crc32fast::hash(&compressed);
            result.extend_from_slice(&checksum.to_le_bytes());
            result.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
            result.extend_from_slice(&compressed);
        }
    }
    Ok(result)
}

/// Decompress payload.
/// Accepts only marker 0x00 (raw) or 0x01 (zstd).
/// Rejects anything else — no backward compat with older format.
pub fn decompress_payload(data: &[u8]) -> StorageResult<Vec<u8>> {
    if data.is_empty() {
        return Err(StorageError::deserialize_error(
            "empty data, expected compression marker",
        ));
    }
    match data[0] {
        COMPRESSION_MARKER_NONE => Ok(data[1..].to_vec()),
        COMPRESSION_MARKER_ZSTD => {
            if data.len() < 9 {
                return Err(StorageError::deserialize_error(
                    "truncated compressed data header",
                ));
            }
            let checksum =
                u32::from_le_bytes(data[1..5].try_into().map_err(|_| {
                    StorageError::deserialize_error("failed to read zstd checksum")
                })?);
            let compressed_len = u32::from_le_bytes(data[5..9].try_into().map_err(|_| {
                StorageError::deserialize_error("failed to read zstd compressed length")
            })?) as usize;
            let compressed_end = 9 + compressed_len;
            if compressed_end > data.len() {
                return Err(StorageError::deserialize_error("truncated compressed data"));
            }
            let compressed = &data[9..compressed_end];
            let actual_checksum = crc32fast::hash(compressed);
            if checksum != actual_checksum {
                return Err(StorageError::deserialize_error(
                    "compressed data checksum mismatch",
                ));
            }
            zstd::decode_all(compressed)
                .map_err(|e| StorageError::io_error(format!("zstd decompress failed: {}", e)))
        }
        marker => Err(StorageError::deserialize_error(format!(
            "unknown compression marker: {:#04x}, expected 0x00 or 0x01",
            marker
        ))),
    }
}

/// Compress a file in-place by reading it, compressing, and rewriting.
/// Always rewrites the file with the compression marker (even for None).
pub fn compress_file_inplace(path: &std::path::Path, ct: CompressionType) -> StorageResult<()> {
    let data = std::fs::read(path).map_err(|e| {
        StorageError::io_error(format!(
            "failed to read {} for compression: {}",
            path.display(),
            e
        ))
    })?;
    let compressed = compress_payload(&data, ct)?;
    std::fs::write(path, &compressed).map_err(|e| {
        StorageError::io_error(format!(
            "failed to write compressed {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}

/// Read a file and decompress it.
pub fn read_decompressed(path: &std::path::Path) -> StorageResult<Vec<u8>> {
    let data = std::fs::read(path)
        .map_err(|e| StorageError::io_error(format!("failed to read {}: {}", path.display(), e)))?;
    decompress_payload(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_type_default() {
        let default = CompressionType::default();
        assert_eq!(default, CompressionType::Zstd { level: 3 });
    }

    #[test]
    fn test_compression_type_none() {
        let none = CompressionType::None;
        assert!(!none.is_enabled());
        assert_eq!(none.level(), None);
        assert_eq!(none.to_u8(), 0);
        assert_eq!(CompressionType::from_u8(0), CompressionType::None);
    }

    #[test]
    fn test_compression_type_zstd_roundtrip() {
        let original = CompressionType::Zstd { level: 5 };
        let encoded = original.to_u8();
        let decoded = CompressionType::from_u8(encoded);
        assert_eq!(original, decoded);
        assert!(decoded.is_enabled());
        assert_eq!(decoded.level(), Some(5));
    }

    #[test]
    fn test_compression_type_level_clamping() {
        let high_level = CompressionType::Zstd { level: 20 };
        let encoded = high_level.to_u8();
        let decoded = CompressionType::from_u8(encoded);
        assert_eq!(decoded.level(), Some(15));
    }

    #[test]
    fn test_compression_type_serialization() {
        let test_cases = [
            (CompressionType::None, 0u8),
            (CompressionType::Zstd { level: 1 }, 0x11u8),
            (CompressionType::Zstd { level: 3 }, 0x31u8),
            (CompressionType::Zstd { level: 10 }, 0xA1u8),
        ];

        for (compression, expected) in test_cases {
            assert_eq!(compression.to_u8(), expected);
            let decoded = CompressionType::from_u8(expected);
            if let CompressionType::Zstd { level: _ } = compression {
                assert!(decoded.is_enabled());
            } else {
                assert!(!decoded.is_enabled());
            }
        }
    }

    #[test]
    fn test_compress_decompress_roundtrip_none() {
        let data = b"hello world";
        let compressed = compress_payload(data, CompressionType::None).unwrap();
        assert_eq!(compressed[0], COMPRESSION_MARKER_NONE);
        let decompressed = decompress_payload(&compressed).unwrap();
        assert_eq!(&decompressed, data);
    }

    #[test]
    fn test_compress_decompress_roundtrip_zstd() {
        let data = b"hello world this is a test string for zstd compression";
        let compressed = compress_payload(data, CompressionType::Zstd { level: 3 }).unwrap();
        assert_eq!(compressed[0], COMPRESSION_MARKER_ZSTD);
        let decompressed = decompress_payload(&compressed).unwrap();
        assert_eq!(&decompressed, data);
    }

    #[test]
    fn test_decompress_rejects_unknown_marker() {
        let data = vec![0xFF, 0x01, 0x02, 0x03];
        let result = decompress_payload(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_rejects_empty() {
        let result = decompress_payload(&[]);
        assert!(result.is_err());
    }
}
