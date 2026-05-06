//! Encoding Module
//!
//! Provides compression encodings for columnar storage:
//! - Dictionary encoding for low-cardinality strings
//! - RLE (Run-Length Encoding) for repeated values
//! - BitPacking for small-range integers
//! - Varint for variable-length integer encoding
//! - FSST for long string compression
//! - ALP for floating-point compression
//! - Lazy decompression for compressed queries
//! - Tiered compression strategy selector

pub mod alp;
pub mod bitpacking;
pub mod dictionary;
pub mod fsst;
pub mod lazy;
pub mod rle;
pub mod selector;
pub mod varint;

use crate::core::{DataType, Value};

pub use alp::{AlpColumn, AlpEncoder, AlpFloatType, select_alp, select_alp_f32};
pub use bitpacking::{BitPackedColumn, BitPackedIntColumn, BitPackedIterator, select_bitpacking};
pub use dictionary::{DictionaryColumn, DictionaryEncoder, StringDictionary};
pub use fsst::{FsstColumn, FsstEncoder, FsstSymbolTable, select_fsst};
pub use lazy::{LazyCompare, LazyDecompress, LazyFilter, LazyFind, LazyStats};
pub use rle::{RleBoolColumn, RleEncoder, RleIntColumn, RleRun};
pub use selector::{
    ColumnStats, CompressionConfig, CompressionSelector, DataTemperature, TierConfig,
    TieredCompressionStrategy,
};
pub use varint::{SignedVarint, Varint, VarintReader, VarintWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingType {
    None,
    Dictionary,
    Rle,
    BitPacking,
    Fsst,
    Alp,
}

impl Default for EncodingType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
pub struct EncodingStats {
    pub encoding_type: EncodingType,
    pub original_size: usize,
    pub encoded_size: usize,
    pub compression_ratio: f64,
}

impl EncodingStats {
    pub fn new(encoding_type: EncodingType, original_size: usize, encoded_size: usize) -> Self {
        let compression_ratio = if original_size > 0 {
            (original_size as f64 - encoded_size as f64) / original_size as f64
        } else {
            0.0
        };
        Self {
            encoding_type,
            original_size,
            encoded_size,
            compression_ratio,
        }
    }
}

pub fn select_encoding(values: &[Option<Value>], data_type: &DataType) -> EncodingType {
    if values.is_empty() {
        return EncodingType::None;
    }

    match data_type {
        DataType::String => select_string_encoding(values),
        DataType::Int | DataType::SmallInt | DataType::BigInt => select_int_encoding(values),
        _ => EncodingType::None,
    }
}

fn select_string_encoding(values: &[Option<Value>]) -> EncodingType {
    let non_null_count = values.iter().filter(|v| v.is_some()).count();
    if non_null_count == 0 {
        return EncodingType::None;
    }

    let mut unique_values: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_length = 0;

    for value in values.iter().flatten() {
        if let Value::String(s) = value {
            *unique_values.entry(s.clone()).or_insert(0) += 1;
            total_length += s.len();
        }
    }

    let cardinality = unique_values.len();
    let cardinality_ratio = cardinality as f64 / non_null_count as f64;

    if cardinality_ratio < 0.5 && cardinality < 10000 {
        let dict_size = cardinality * 20 + non_null_count * 4;
        if dict_size < total_length {
            return EncodingType::Dictionary;
        }
    }

    EncodingType::None
}

fn select_int_encoding(values: &[Option<Value>]) -> EncodingType {
    let non_null_values: Vec<i64> = values
        .iter()
        .filter_map(|v| {
            v.as_ref().and_then(|val| match val {
                Value::SmallInt(i) => Some(*i as i64),
                Value::Int(i) => Some(*i as i64),
                Value::BigInt(i) => Some(*i),
                _ => None,
            })
        })
        .collect();

    if non_null_values.is_empty() {
        return EncodingType::None;
    }

    let mut runs = 1;
    for i in 1..non_null_values.len() {
        if non_null_values[i] != non_null_values[i - 1] {
            runs += 1;
        }
    }

    let run_ratio = runs as f64 / non_null_values.len() as f64;
    if run_ratio < 0.3 {
        return EncodingType::Rle;
    }

    let min_val = *non_null_values.iter().min().unwrap_or(&0);
    let max_val = *non_null_values.iter().max().unwrap_or(&0);
    let range = (max_val - min_val) as u64;

    if range > 0 {
        let bit_width = (64 - range.leading_zeros()) as u8;
        if bit_width < 32 && non_null_values.len() >= 100 {
            return EncodingType::BitPacking;
        }
    }

    EncodingType::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_string_encoding_dictionary() {
        let values: Vec<Option<Value>> = vec![
            Some(Value::String("very_long_string_apple".to_string())),
            Some(Value::String("very_long_string_banana".to_string())),
            Some(Value::String("very_long_string_apple".to_string())),
            Some(Value::String("very_long_string_apple".to_string())),
            Some(Value::String("very_long_string_banana".to_string())),
        ];

        let encoding = select_encoding(&values, &DataType::String);
        assert_eq!(encoding, EncodingType::Dictionary);
    }

    #[test]
    fn test_select_string_encoding_none() {
        let values: Vec<Option<Value>> = vec![
            Some(Value::String("unique1".to_string())),
            Some(Value::String("unique2".to_string())),
            Some(Value::String("unique3".to_string())),
        ];

        let encoding = select_encoding(&values, &DataType::String);
        assert_eq!(encoding, EncodingType::None);
    }

    #[test]
    fn test_select_int_encoding_rle() {
        let values: Vec<Option<Value>> = vec![
            Some(Value::Int(1)),
            Some(Value::Int(1)),
            Some(Value::Int(1)),
            Some(Value::Int(2)),
            Some(Value::Int(2)),
            Some(Value::Int(2)),
            Some(Value::Int(2)),
        ];

        let encoding = select_encoding(&values, &DataType::Int);
        assert_eq!(encoding, EncodingType::Rle);
    }

    #[test]
    fn test_select_int_encoding_bitpacking() {
        let values: Vec<Option<Value>> = (0..200)
            .map(|i| Some(Value::Int(i % 100)))
            .collect();

        let encoding = select_encoding(&values, &DataType::Int);
        assert_eq!(encoding, EncodingType::BitPacking);
    }
}
