//! Lazy Decompression Support
//!
//! Provides operations that can be performed on compressed data without
//! full decompression. This improves query performance by avoiding
//! unnecessary decompression overhead.
//!
//! # Supported Operations
//!
//! - `equals`: Check if a value equals a target
//! - `compare`: Compare a value with a target
//! - `find`: Find all rows matching a value
//! - `filter`: Filter rows by predicate

use std::cmp::Ordering;

use crate::core::Value;

use super::bitpacking::BitPackedColumn;
use super::dictionary::DictionaryColumn;
use super::fsst::FsstColumn;
use super::rle::RleEncoder;

pub trait LazyDecompress {
    fn get_encoded_size(&self) -> usize;

    fn get_row_count(&self) -> usize;

    fn is_null(&self, row_idx: usize) -> bool;
}

pub trait LazyCompare: LazyDecompress {
    fn equals(&self, row_idx: usize, value: &Value) -> bool;

    fn compare(&self, row_idx: usize, value: &Value) -> Option<Ordering>;

    fn less_than(&self, row_idx: usize, value: &Value) -> bool {
        matches!(self.compare(row_idx, value), Some(Ordering::Less))
    }

    fn greater_than(&self, row_idx: usize, value: &Value) -> bool {
        matches!(self.compare(row_idx, value), Some(Ordering::Greater))
    }

    fn less_than_or_equal(&self, row_idx: usize, value: &Value) -> bool {
        matches!(
            self.compare(row_idx, value),
            Some(Ordering::Less | Ordering::Equal)
        )
    }

    fn greater_than_or_equal(&self, row_idx: usize, value: &Value) -> bool {
        matches!(
            self.compare(row_idx, value),
            Some(Ordering::Greater | Ordering::Equal)
        )
    }
}

pub trait LazyFind: LazyDecompress {
    fn find_value(&self, value: &Value) -> Vec<usize>;

    fn find_value_range(&self, min: &Value, max: &Value) -> Vec<usize>;

    fn count_value(&self, value: &Value) -> usize {
        self.find_value(value).len()
    }
}

impl LazyDecompress for BitPackedColumn {
    fn get_encoded_size(&self) -> usize {
        self.memory_usage()
    }

    fn get_row_count(&self) -> usize {
        self.len()
    }

    fn is_null(&self, row_idx: usize) -> bool {
        BitPackedColumn::is_null(self, row_idx)
    }
}

impl LazyCompare for BitPackedColumn {
    fn equals(&self, row_idx: usize, value: &Value) -> bool {
        if self.is_null(row_idx) {
            return false;
        }

        let target = match value {
            Value::SmallInt(i) => *i as i64,
            Value::Int(i) => *i as i64,
            Value::BigInt(i) => *i,
            _ => return false,
        };

        self.get(row_idx) == Some(target)
    }

    fn compare(&self, row_idx: usize, value: &Value) -> Option<Ordering> {
        if self.is_null(row_idx) {
            return None;
        }

        let current = self.get(row_idx)?;
        let target = match value {
            Value::SmallInt(i) => *i as i64,
            Value::Int(i) => *i as i64,
            Value::BigInt(i) => *i,
            _ => return None,
        };

        Some(current.cmp(&target))
    }
}

impl LazyFind for BitPackedColumn {
    fn find_value(&self, value: &Value) -> Vec<usize> {
        let target = match value {
            Value::SmallInt(i) => *i as i64,
            Value::Int(i) => *i as i64,
            Value::BigInt(i) => *i,
            _ => return Vec::new(),
        };

        (0..self.len())
            .filter(|&i| !self.is_null(i) && self.get(i) == Some(target))
            .collect()
    }

    fn find_value_range(&self, min: &Value, max: &Value) -> Vec<usize> {
        let min_val = match min {
            Value::SmallInt(i) => *i as i64,
            Value::Int(i) => *i as i64,
            Value::BigInt(i) => *i,
            _ => return Vec::new(),
        };

        let max_val = match max {
            Value::SmallInt(i) => *i as i64,
            Value::Int(i) => *i as i64,
            Value::BigInt(i) => *i,
            _ => return Vec::new(),
        };

        (0..self.len())
            .filter(|&i| {
                if self.is_null(i) {
                    return false;
                }
                match self.get(i) {
                    Some(v) => v >= min_val && v <= max_val,
                    None => false,
                }
            })
            .collect()
    }
}

impl LazyDecompress for DictionaryColumn {
    fn get_encoded_size(&self) -> usize {
        self.memory_usage()
    }

    fn get_row_count(&self) -> usize {
        self.len()
    }

    fn is_null(&self, row_idx: usize) -> bool {
        DictionaryColumn::is_null(self, row_idx)
    }
}

impl LazyCompare for DictionaryColumn {
    fn equals(&self, row_idx: usize, value: &Value) -> bool {
        if self.is_null(row_idx) {
            return false;
        }

        match value {
            Value::String(s) => {
                if let Some(Value::String(current_str)) = self.get(row_idx) {
                    return current_str == *s;
                }
                false
            }
            _ => false,
        }
    }

    fn compare(&self, row_idx: usize, value: &Value) -> Option<Ordering> {
        if self.is_null(row_idx) {
            return None;
        }

        let current = self.get(row_idx)?;
        match (&current, value) {
            (Value::String(current_str), Value::String(s)) => {
                Some(current_str.as_str().cmp(s.as_str()))
            }
            _ => None,
        }
    }
}

impl LazyFind for DictionaryColumn {
    fn find_value(&self, value: &Value) -> Vec<usize> {
        let target = match value {
            Value::String(s) => s.clone(),
            _ => return Vec::new(),
        };

        (0..self.len())
            .filter(|&i| {
                if self.is_null(i) {
                    return false;
                }
                match self.get(i) {
                    Some(Value::String(s)) => s == target,
                    _ => false,
                }
            })
            .collect()
    }

    fn find_value_range(&self, min: &Value, max: &Value) -> Vec<usize> {
        let min_str = match min {
            Value::String(s) => s.clone(),
            _ => return Vec::new(),
        };

        let max_str = match max {
            Value::String(s) => s.clone(),
            _ => return Vec::new(),
        };

        (0..self.len())
            .filter(|&i| {
                if self.is_null(i) {
                    return false;
                }
                match self.get(i) {
                    Some(Value::String(s)) => s >= min_str && s <= max_str,
                    _ => false,
                }
            })
            .collect()
    }
}

impl LazyDecompress for FsstColumn {
    fn get_encoded_size(&self) -> usize {
        self.memory_usage()
    }

    fn get_row_count(&self) -> usize {
        self.len()
    }

    fn is_null(&self, row_idx: usize) -> bool {
        FsstColumn::is_null(self, row_idx)
    }
}

impl LazyCompare for FsstColumn {
    fn equals(&self, row_idx: usize, value: &Value) -> bool {
        if self.is_null(row_idx) {
            return false;
        }

        match value {
            Value::String(s) => self
                .get(row_idx)
                .map(|decoded| decoded == *s)
                .unwrap_or(false),
            _ => false,
        }
    }

    fn compare(&self, row_idx: usize, value: &Value) -> Option<Ordering> {
        if self.is_null(row_idx) {
            return None;
        }

        let current = self.get(row_idx)?;
        match value {
            Value::String(s) => Some(current.as_str().cmp(s.as_str())),
            _ => None,
        }
    }
}

impl LazyFind for FsstColumn {
    fn find_value(&self, value: &Value) -> Vec<usize> {
        let target = match value {
            Value::String(s) => s.clone(),
            _ => return Vec::new(),
        };

        (0..self.len())
            .filter(|&i| {
                if self.is_null(i) {
                    return false;
                }
                self.get(i).map(|s| s == target).unwrap_or(false)
            })
            .collect()
    }

    fn find_value_range(&self, min: &Value, max: &Value) -> Vec<usize> {
        let min_str = match min {
            Value::String(s) => s.clone(),
            _ => return Vec::new(),
        };

        let max_str = match max {
            Value::String(s) => s.clone(),
            _ => return Vec::new(),
        };

        (0..self.len())
            .filter(|&i| {
                if self.is_null(i) {
                    return false;
                }
                self.get(i)
                    .map(|s| s >= min_str && s <= max_str)
                    .unwrap_or(false)
            })
            .collect()
    }
}

impl<T: Clone + PartialEq> LazyDecompress for RleEncoder<T> {
    fn get_encoded_size(&self) -> usize {
        self.memory_usage()
    }

    fn get_row_count(&self) -> usize {
        self.len()
    }

    fn is_null(&self, _row_idx: usize) -> bool {
        false
    }
}

pub struct LazyFilter<'a, C: LazyDecompress> {
    column: &'a C,
    predicate: Box<dyn Fn(usize) -> bool + 'a>,
}

impl<'a, C: LazyDecompress> LazyFilter<'a, C> {
    pub fn new<F>(column: &'a C, predicate: F) -> Self
    where
        F: Fn(usize) -> bool + 'a,
    {
        Self {
            column,
            predicate: Box::new(predicate),
        }
    }

    pub fn collect(&self) -> Vec<usize> {
        (0..self.column.get_row_count())
            .filter(|&i| (self.predicate)(i))
            .collect()
    }

    pub fn count(&self) -> usize {
        (0..self.column.get_row_count())
            .filter(|&i| (self.predicate)(i))
            .count()
    }

    pub fn first(&self) -> Option<usize> {
        (0..self.column.get_row_count()).find(|&i| (self.predicate)(i))
    }

    pub fn any(&self) -> bool {
        (0..self.column.get_row_count()).any(|i| (self.predicate)(i))
    }

    pub fn all(&self) -> bool {
        (0..self.column.get_row_count()).all(|i| (self.predicate)(i))
    }
}

pub struct LazyStats {
    pub total_rows: usize,
    pub null_count: usize,
    pub distinct_count: Option<usize>,
}

impl LazyStats {
    pub fn from_column<C: LazyDecompress + ?Sized>(column: &C) -> Self {
        let total_rows = column.get_row_count();
        let null_count = (0..total_rows).filter(|&i| column.is_null(i)).count();

        Self {
            total_rows,
            null_count,
            distinct_count: None,
        }
    }

    pub fn null_ratio(&self) -> f64 {
        if self.total_rows == 0 {
            return 0.0;
        }
        self.null_count as f64 / self.total_rows as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitpacked_lazy_compare() {
        let values = vec![10, 20, 30, 40, 50];
        let column = BitPackedColumn::analyze(&values);

        assert!(column.equals(0, &Value::Int(10)));
        assert!(!column.equals(0, &Value::Int(20)));

        assert!(column.less_than(0, &Value::Int(20)));
        assert!(column.greater_than(4, &Value::Int(40)));
    }

    #[test]
    fn test_bitpacked_lazy_find() {
        let values = vec![10, 20, 30, 20, 10];
        let column = BitPackedColumn::analyze(&values);

        let found = column.find_value(&Value::Int(10));
        assert_eq!(found, vec![0, 4]);

        let found = column.find_value(&Value::Int(20));
        assert_eq!(found, vec![1, 3]);

        let range = column.find_value_range(&Value::Int(15), &Value::Int(25));
        assert_eq!(range, vec![1, 3]);
    }

    #[test]
    fn test_dictionary_lazy_compare() {
        let mut column = DictionaryColumn::new();

        column
            .set(0, Some(&Value::String("apple".to_string())))
            .unwrap();
        column
            .set(1, Some(&Value::String("banana".to_string())))
            .unwrap();
        column
            .set(2, Some(&Value::String("apple".to_string())))
            .unwrap();

        assert!(column.equals(0, &Value::String("apple".to_string())));
        assert!(!column.equals(0, &Value::String("banana".to_string())));
    }

    #[test]
    fn test_dictionary_lazy_find() {
        let mut column = DictionaryColumn::new();

        column
            .set(0, Some(&Value::String("apple".to_string())))
            .unwrap();
        column
            .set(1, Some(&Value::String("banana".to_string())))
            .unwrap();
        column
            .set(2, Some(&Value::String("apple".to_string())))
            .unwrap();
        column
            .set(3, Some(&Value::String("cherry".to_string())))
            .unwrap();

        let found = column.find_value(&Value::String("apple".to_string()));
        assert_eq!(found, vec![0, 2]);

        let count = column.count_value(&Value::String("apple".to_string()));
        assert_eq!(count, 2);
    }

    #[test]
    fn test_lazy_filter() {
        let values = vec![10, 20, 30, 40, 50];
        let column = BitPackedColumn::analyze(&values);

        let filter = LazyFilter::new(&column, |i| i >= 2);

        assert_eq!(filter.collect(), vec![2, 3, 4]);
        assert_eq!(filter.count(), 3);
        assert_eq!(filter.first(), Some(2));
    }

    #[test]
    fn test_lazy_stats() {
        let values: Vec<i64> = (0..100).collect();
        let column = BitPackedColumn::analyze(&values);

        let stats = LazyStats::from_column(&column);

        assert_eq!(stats.total_rows, 100);
        assert_eq!(stats.null_count, 0);
    }

    #[test]
    fn test_fsst_lazy_compare() {
        let strings = vec![
            Some("apple"),
            Some("banana"),
            Some("apple"),
            None,
            Some("cherry"),
        ];
        let column = FsstColumn::train_and_build(&strings, 100);

        assert!(column.equals(0, &Value::String("apple".to_string())));
        assert!(!column.equals(0, &Value::String("banana".to_string())));
        assert!(!column.equals(3, &Value::String("apple".to_string())));

        assert!(column.less_than(1, &Value::String("cherry".to_string())));
        assert!(column.greater_than(4, &Value::String("banana".to_string())));
    }

    #[test]
    fn test_fsst_lazy_find() {
        let strings = vec![
            Some("apple"),
            Some("banana"),
            Some("apple"),
            None,
            Some("cherry"),
        ];
        let column = FsstColumn::train_and_build(&strings, 100);

        let found = column.find_value(&Value::String("apple".to_string()));
        assert_eq!(found, vec![0, 2]);

        let count = column.count_value(&Value::String("apple".to_string()));
        assert_eq!(count, 2);

        let range = column.find_value_range(
            &Value::String("apple".to_string()),
            &Value::String("banana".to_string()),
        );
        assert_eq!(range.len(), 3);
    }

    #[test]
    fn test_fsst_lazy_stats() {
        let strings = vec![Some("apple"), Some("banana"), None, Some("cherry")];
        let column = FsstColumn::train_and_build(&strings, 100);

        let stats = LazyStats::from_column(&column);

        assert_eq!(stats.total_rows, 4);
        assert_eq!(stats.null_count, 1);
        assert!((stats.null_ratio() - 0.25).abs() < 0.001);
    }
}
