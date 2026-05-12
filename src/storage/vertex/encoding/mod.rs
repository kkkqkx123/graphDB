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
#[derive(Default)]
pub enum EncodingType {
    #[default]
    None,
    Dictionary,
    Rle,
    BitPacking,
    Fsst,
    Alp,
}

pub trait EncodedColumn: Send + Sync {
    fn get(&self, row_idx: usize) -> Option<Value>;
    
    fn len(&self) -> usize;
    
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    fn is_null(&self, row_idx: usize) -> bool;
    
    fn memory_usage(&self) -> usize;
    
    fn encoding_type(&self) -> EncodingType;
    
    fn compression_ratio(&self) -> f64 {
        0.0
    }
}

#[derive(Debug, Clone, Default)]
pub enum ColumnEncoding {
    #[default]
    None,
    Fsst(FsstColumn),
    Dictionary(DictionaryColumn),
    RleInt(RleIntColumn),
    RleBool(RleBoolColumn),
    BitPacked(BitPackedIntColumn),
    Alp(AlpColumn),
}

impl ColumnEncoding {
    pub fn encoding_type(&self) -> EncodingType {
        match self {
            Self::None => EncodingType::None,
            Self::Fsst(_) => EncodingType::Fsst,
            Self::Dictionary(_) => EncodingType::Dictionary,
            Self::RleInt(_) | Self::RleBool(_) => EncodingType::Rle,
            Self::BitPacked(_) => EncodingType::BitPacking,
            Self::Alp(_) => EncodingType::Alp,
        }
    }
    
    pub fn get(&self, row_idx: usize) -> Option<Value> {
        match self {
            Self::None => None,
            Self::Fsst(col) => col.get(row_idx).map(Value::String),
            Self::Dictionary(col) => col.get(row_idx),
            Self::RleInt(col) => col.get(row_idx),
            Self::RleBool(col) => col.get(row_idx),
            Self::BitPacked(col) => col.get(row_idx),
            Self::Alp(col) => col.get_value(row_idx),
        }
    }
    
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Fsst(col) => col.len(),
            Self::Dictionary(col) => col.len(),
            Self::RleInt(col) => col.len(),
            Self::RleBool(col) => col.len(),
            Self::BitPacked(col) => col.len(),
            Self::Alp(col) => col.len(),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    pub fn is_null(&self, row_idx: usize) -> bool {
        match self {
            Self::None => true,
            Self::Fsst(col) => col.is_null(row_idx),
            Self::Dictionary(col) => col.is_null(row_idx),
            Self::RleInt(col) => col.is_null(row_idx),
            Self::RleBool(col) => col.is_null(row_idx),
            Self::BitPacked(col) => col.is_null(row_idx),
            Self::Alp(col) => col.is_null(row_idx),
        }
    }
    
    pub fn memory_usage(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Fsst(col) => col.memory_usage(),
            Self::Dictionary(col) => col.memory_usage(),
            Self::RleInt(col) => col.memory_usage(),
            Self::RleBool(col) => col.memory_usage(),
            Self::BitPacked(col) => col.memory_usage(),
            Self::Alp(col) => col.memory_usage(),
        }
    }
    
    pub fn compression_ratio(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Fsst(col) => col.compression_ratio(),
            Self::Dictionary(col) => {
                let dict_size = col.memory_usage();
                if dict_size == 0 {
                    return 0.0;
                }
                0.0
            }
            Self::RleInt(col) => {
                let mem = col.memory_usage();
                let original = col.len() * 8;
                if original == 0 { 0.0 } else { (original - mem) as f64 / original as f64 }
            }
            Self::RleBool(col) => {
                let mem = col.memory_usage();
                let original = col.len();
                if original == 0 { 0.0 } else { (original - mem) as f64 / original as f64 }
            }
            Self::BitPacked(col) => col.compression_ratio(),
            Self::Alp(col) => col.compression_ratio(),
        }
    }
    
    pub fn is_encoded(&self) -> bool {
        !matches!(self, Self::None)
    }
    
    pub fn set(&mut self, row_idx: usize, value: Option<&Value>) -> crate::core::StorageResult<()> {
        use crate::core::StorageError;
        
        match self {
            Self::None => {
                Err(StorageError::invalid_operation(
                    "Cannot set value on unencoded column through ColumnEncoding".to_string(),
                ))
            }
            Self::Fsst(col) => {
                match value {
                    Some(Value::String(s)) => {
                        col.set(row_idx, Some(s.as_str()));
                        Ok(())
                    }
                    Some(v) => Err(StorageError::type_mismatch(DataType::String, v.data_type())),
                    None => {
                        col.set(row_idx, None);
                        Ok(())
                    }
                }
            }
            Self::Dictionary(col) => {
                col.set(row_idx, value)?;
                Ok(())
            }
            Self::RleInt(col) => {
                col.append(value)?;
                Ok(())
            }
            Self::RleBool(col) => {
                col.append(value)?;
                Ok(())
            }
            Self::BitPacked(col) => {
                col.set(row_idx, value)?;
                Ok(())
            }
            Self::Alp(col) => {
                let float_val = value.and_then(|v| match v {
                    Value::Float(f) => Some(*f as f64),
                    Value::Double(d) => Some(*d),
                    _ => None,
                });
                col.set(row_idx, float_val)?;
                Ok(())
            }
        }
    }
    
    pub fn clear(&mut self) {
        match self {
            Self::None => {}
            Self::Fsst(col) => col.clear(),
            Self::Dictionary(col) => col.clear(),
            Self::RleInt(col) => col.clear(),
            Self::RleBool(col) => col.clear(),
            Self::BitPacked(col) => col.clear(),
            Self::Alp(col) => col.clear(),
        }
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
    
    #[test]
    fn test_column_encoding_none() {
        let encoding = ColumnEncoding::None;
        
        assert_eq!(encoding.encoding_type(), EncodingType::None);
        assert!(!encoding.is_encoded());
        assert_eq!(encoding.len(), 0);
        assert!(encoding.is_empty());
        assert_eq!(encoding.memory_usage(), 0);
        assert_eq!(encoding.compression_ratio(), 0.0);
        assert!(encoding.get(0).is_none());
        assert!(encoding.is_null(0));
    }
    
    #[test]
    fn test_column_encoding_fsst() {
        let strings = vec![
            Some("hello world"),
            None,
            Some("hello rust"),
        ];
        let col = FsstColumn::train_and_build(&strings, 100);
        let encoding = ColumnEncoding::Fsst(col);
        
        assert_eq!(encoding.encoding_type(), EncodingType::Fsst);
        assert!(encoding.is_encoded());
        assert_eq!(encoding.len(), 3);
        assert!(!encoding.is_empty());
        assert!(encoding.memory_usage() > 0);
        assert!(encoding.get(0).is_some());
        assert!(encoding.is_null(1));
        assert!(!encoding.is_null(0));
    }
    
    #[test]
    fn test_column_encoding_dictionary() {
        let mut col = DictionaryColumn::new();
        col.set(0, Some(&Value::String("apple".to_string()))).unwrap();
        col.set(1, Some(&Value::String("banana".to_string()))).unwrap();
        col.set(2, None).unwrap();
        
        let encoding = ColumnEncoding::Dictionary(col);
        
        assert_eq!(encoding.encoding_type(), EncodingType::Dictionary);
        assert!(encoding.is_encoded());
        assert_eq!(encoding.len(), 3);
        assert!(encoding.get(0).is_some());
        assert!(encoding.is_null(2));
    }
    
    #[test]
    fn test_column_encoding_rle_int() {
        let mut col = RleIntColumn::new();
        col.append(Some(&Value::Int(1))).unwrap();
        col.append(Some(&Value::Int(1))).unwrap();
        col.append(Some(&Value::Int(2))).unwrap();
        
        let encoding = ColumnEncoding::RleInt(col);
        
        assert_eq!(encoding.encoding_type(), EncodingType::Rle);
        assert!(encoding.is_encoded());
        assert_eq!(encoding.len(), 3);
        assert!(encoding.get(0).is_some());
    }
    
    #[test]
    fn test_column_encoding_bitpacked() {
        let values = vec![
            Some(Value::Int(10)),
            Some(Value::Int(20)),
            Some(Value::Int(30)),
        ];
        let col = BitPackedIntColumn::analyze(&values, DataType::Int).unwrap();
        
        let encoding = ColumnEncoding::BitPacked(col);
        
        assert_eq!(encoding.encoding_type(), EncodingType::BitPacking);
        assert!(encoding.is_encoded());
        assert_eq!(encoding.len(), 3);
        assert!(encoding.get(0).is_some());
    }
    
    #[test]
    fn test_column_encoding_alp() {
        let values = vec![
            Some(Value::Double(1.5)),
            Some(Value::Double(2.5)),
            None,
        ];
        let col = AlpColumn::analyze_values(&values, DataType::Double).unwrap();
        
        let encoding = ColumnEncoding::Alp(col);
        
        assert_eq!(encoding.encoding_type(), EncodingType::Alp);
        assert!(encoding.is_encoded());
        assert_eq!(encoding.len(), 3);
        assert!(encoding.get(0).is_some());
        assert!(encoding.is_null(2));
    }
    
    #[test]
    fn test_column_encoding_set_fsst() {
        let strings = vec![Some("hello")];
        let col = FsstColumn::train_and_build(&strings, 100);
        let mut encoding = ColumnEncoding::Fsst(col);
        
        encoding.set(0, Some(&Value::String("world".to_string()))).unwrap();
        assert_eq!(encoding.get(0), Some(Value::String("world".to_string())));
        
        encoding.set(0, None).unwrap();
        assert!(encoding.is_null(0));
    }
    
    #[test]
    fn test_column_encoding_clear() {
        let strings = vec![Some("hello"), Some("world")];
        let col = FsstColumn::train_and_build(&strings, 100);
        let mut encoding = ColumnEncoding::Fsst(col);
        
        assert_eq!(encoding.len(), 2);
        
        encoding.clear();
        assert_eq!(encoding.len(), 0);
        assert!(encoding.is_empty());
    }
}
