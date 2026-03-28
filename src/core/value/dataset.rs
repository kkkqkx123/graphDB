//! DataSet Type Module
//!
//! This module defines the DataSet type and its associated operations.

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// Simple dataset representation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DataSet {
    pub col_names: Vec<String>,
    pub rows: Vec<Vec<super::types::Value>>,
}

impl Default for DataSet {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSet {
    pub fn new() -> Self {
        Self {
            col_names: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Create a DataSet with column names
    pub fn with_columns(col_names: Vec<String>) -> Self {
        Self {
            col_names,
            rows: Vec::new(),
        }
    }

    /// Add a row
    pub fn add_row(&mut self, row: Vec<super::types::Value>) {
        self.rows.push(row);
    }

    /// Get row count
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Get column count
    pub fn col_count(&self) -> usize {
        self.col_names.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get index of specified column
    pub fn get_col_index(&self, col_name: &str) -> Option<usize> {
        self.col_names.iter().position(|name| name == col_name)
    }

    /// Get all values of specified column
    pub fn get_column(&self, col_name: &str) -> Option<Vec<super::types::Value>> {
        self.get_col_index(col_name).map(|index| {
            self.rows
                .iter()
                .filter_map(|row| row.get(index).cloned())
                .collect()
        })
    }

    /// Filter dataset
    pub fn filter<F>(&self, predicate: F) -> DataSet
    where
        F: Fn(&Vec<super::types::Value>) -> bool,
    {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self
                .rows
                .iter()
                .filter(|row| predicate(row))
                .cloned()
                .collect(),
        }
    }

    /// Map dataset
    pub fn map<F>(&self, mapper: F) -> DataSet
    where
        F: Fn(&Vec<super::types::Value>) -> Vec<super::types::Value>,
    {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().map(mapper).collect(),
        }
    }

    /// Sort dataset
    pub fn sort_by<F>(&mut self, comparator: F)
    where
        F: Fn(&Vec<super::types::Value>, &Vec<super::types::Value>) -> std::cmp::Ordering,
    {
        self.rows.sort_by(comparator);
    }

    /// Join two datasets
    pub fn join(&self, other: &DataSet, on: &str) -> Result<DataSet, String> {
        let left_index = self
            .get_col_index(on)
            .ok_or_else(|| format!("Left dataset column not found: {}", on))?;
        let right_index = other
            .get_col_index(on)
            .ok_or_else(|| format!("Right dataset column not found: {}", on))?;

        let mut result = DataSet::new();
        result.col_names = self
            .col_names
            .iter()
            .chain(other.col_names.iter())
            .filter(|name| *name != on)
            .cloned()
            .collect();

        for left_row in &self.rows {
            if let Some(left_key) = left_row.get(left_index) {
                for right_row in &other.rows {
                    if let Some(right_key) = right_row.get(right_index) {
                        if left_key == right_key {
                            let mut merged_row = left_row.clone();
                            for (i, val) in right_row.iter().enumerate() {
                                if i != right_index {
                                    merged_row.push(val.clone());
                                }
                            }
                            result.add_row(merged_row);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Group dataset
    pub fn group_by<F, K>(&self, key_fn: F) -> Vec<(K, DataSet)>
    where
        F: Fn(&Vec<super::types::Value>) -> K,
        K: std::hash::Hash + Eq + Clone,
    {
        let mut groups: HashMap<K, Vec<Vec<super::types::Value>>> = HashMap::new();

        for row in &self.rows {
            let key = key_fn(row);
            groups.entry(key).or_default().push(row.clone());
        }

        groups
            .into_iter()
            .map(|(key, rows)| {
                let dataset = DataSet {
                    col_names: self.col_names.clone(),
                    rows,
                };
                (key, dataset)
            })
            .collect()
    }

    /// Aggregate dataset
    pub fn aggregate<F, R>(&self, aggregator: F) -> Vec<R>
    where
        F: Fn(&Vec<super::types::Value>) -> R,
    {
        self.rows.iter().map(aggregator).collect()
    }

    /// Limit rows
    pub fn limit(&self, n: usize) -> DataSet {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().take(n).cloned().collect(),
        }
    }

    /// Skip rows
    pub fn skip(&self, n: usize) -> DataSet {
        DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().skip(n).cloned().collect(),
        }
    }

    /// Merge datasets
    pub fn union(&self, other: &DataSet) -> Result<DataSet, String> {
        if self.col_names != other.col_names {
            return Err("Column names mismatch, cannot merge".to_string());
        }

        Ok(DataSet {
            col_names: self.col_names.clone(),
            rows: self.rows.iter().chain(other.rows.iter()).cloned().collect(),
        })
    }

    /// Calculate intersection
    pub fn intersect(&self, other: &DataSet) -> DataSet {
        use std::collections::HashSet;
        let other_set: HashSet<&Vec<super::types::Value>> = other.rows.iter().collect();
        DataSet {
            col_names: self.col_names.clone(),
            rows: self
                .rows
                .iter()
                .filter(|row| other_set.contains(row))
                .cloned()
                .collect(),
        }
    }

    /// Calculate difference
    pub fn except(&self, other: &DataSet) -> DataSet {
        use std::collections::HashSet;
        let other_set: HashSet<&Vec<super::types::Value>> = other.rows.iter().collect();
        DataSet {
            col_names: self.col_names.clone(),
            rows: self
                .rows
                .iter()
                .filter(|row| !other_set.contains(row))
                .cloned()
                .collect(),
        }
    }

    /// Transpose dataset
    pub fn transpose(&self) -> DataSet {
        if self.rows.is_empty() {
            return DataSet::new();
        }

        let col_count = self.col_names.len();
        let mut transposed = DataSet::new();
        transposed.col_names = (0..self.row_count())
            .map(|i| format!("row_{}", i))
            .collect();

        for col in 0..col_count {
            let mut new_row = Vec::new();
            for row in &self.rows {
                if let Some(val) = row.get(col) {
                    new_row.push(val.clone());
                }
            }
            transposed.add_row(new_row);
        }

        transposed
    }

    /// Get unique values
    pub fn distinct(&self, col_name: &str) -> Vec<super::types::Value> {
        use std::collections::HashSet;
        if let Some(index) = self.get_col_index(col_name) {
            let mut unique = HashSet::new();
            for row in &self.rows {
                if let Some(val) = row.get(index) {
                    unique.insert(val.clone());
                }
            }
            unique.into_iter().collect()
        } else {
            Vec::new()
        }
    }

    /// Estimate memory usage
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // col_names capacity
        size += self.col_names.capacity() * std::mem::size_of::<String>();
        for col_name in &self.col_names {
            size += col_name.capacity();
        }

        // rows capacity
        size += self.rows.capacity() * std::mem::size_of::<Vec<super::types::Value>>();
        for row in &self.rows {
            size += row.capacity() * std::mem::size_of::<super::types::Value>();
            for value in row {
                size += value.estimated_size();
            }
        }

        size
    }
}
