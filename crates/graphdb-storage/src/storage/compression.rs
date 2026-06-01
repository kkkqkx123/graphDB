//! Compression type definition for storage layer
//!
//! This module provides the `CompressionType` enum used for configuring
//! compression in flush operations and other storage operations.
//!
//! Note: Actual compression/decompression logic is implemented in:
//! - `src/transaction/wal/writer/compression.rs` for WAL compression
//! - Column encoding modules for columnar compression

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
}
