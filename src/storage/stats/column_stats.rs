//! Column Statistics Collection
//!
//! Provides statistical information about column data distributions
//! for query optimization and analysis.
//!
//! # Features
//!
//! - Column-level statistics (null count, distinct count, min/max)
//! - Histogram-based data distribution analysis
//! - Most common values tracking
//! - Average length calculation for string types

use std::collections::HashMap;

use crate::core::{DataType, Value};

#[derive(Debug, Clone)]
pub struct HistogramBucket {
    pub lower: Value,
    pub upper: Value,
    pub count: usize,
    pub distinct_count: usize,
}

impl HistogramBucket {
    pub fn new(lower: Value, upper: Value) -> Self {
        Self {
            lower,
            upper,
            count: 0,
            distinct_count: 0,
        }
    }

    pub fn contains_value(&self, value: &Value) -> bool {
        match (&self.lower, &self.upper, value) {
            (Value::Int(l), Value::Int(u), Value::Int(v)) => v >= l && v <= u,
            (Value::Double(l), Value::Double(u), Value::Double(v)) => v >= l && v <= u,
            (Value::String(l), Value::String(u), Value::String(v)) => v >= l && v <= u,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Histogram {
    pub buckets: Vec<HistogramBucket>,
    pub most_common_values: Vec<(Value, usize)>,
    pub total_count: usize,
}

impl Histogram {
    pub fn new(bucket_count: usize) -> Self {
        Self {
            buckets: Vec::with_capacity(bucket_count),
            most_common_values: Vec::new(),
            total_count: 0,
        }
    }

    pub fn add_value(&mut self, value: &Value) {
        self.total_count += 1;

        for bucket in &mut self.buckets {
            if bucket.contains_value(value) {
                bucket.count += 1;
                break;
            }
        }

        let entry = self.most_common_values.iter_mut().find(|(v, _)| v == value);
        if let Some((_, count)) = entry {
            *count += 1;
        } else {
            self.most_common_values.push((value.clone(), 1));
        }
    }

    pub fn sort_most_common(&mut self) {
        self.most_common_values.sort_by(|a, b| b.1.cmp(&a.1));
    }

    pub fn estimate_selectivity(&self, lower: &Value, upper: &Value) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let mut matching_count = 0;
        for bucket in &self.buckets {
            let overlaps = match (lower, upper, &bucket.lower, &bucket.upper) {
                (Value::Int(l), Value::Int(u), Value::Int(bl), Value::Int(bu)) => {
                    l <= bu && u >= bl
                }
                (Value::Double(l), Value::Double(u), Value::Double(bl), Value::Double(bu)) => {
                    l <= bu && u >= bl
                }
                _ => false,
            };

            if overlaps {
                matching_count += bucket.count;
            }
        }

        matching_count as f64 / self.total_count as f64
    }
}

#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub column_name: String,
    pub data_type: DataType,
    pub null_count: usize,
    pub distinct_count: usize,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub avg_length: f64,
    pub total_count: usize,
    pub histogram: Option<Histogram>,
}

impl ColumnStatistics {
    pub fn new(column_name: String, data_type: DataType) -> Self {
        Self {
            column_name,
            data_type,
            null_count: 0,
            distinct_count: 0,
            min_value: None,
            max_value: None,
            avg_length: 0.0,
            total_count: 0,
            histogram: None,
        }
    }

    pub fn selectivity(&self, value: &Value) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        if self.distinct_count > 0 {
            1.0 / self.distinct_count as f64
        } else {
            0.0
        }
    }

    pub fn range_selectivity(&self, lower: &Value, upper: &Value) -> f64 {
        if let Some(ref histogram) = self.histogram {
            histogram.estimate_selectivity(lower, upper)
        } else {
            0.5
        }
    }

    pub fn null_ratio(&self) -> f64 {
        if self.total_count > 0 {
            self.null_count as f64 / self.total_count as f64
        } else {
            0.0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.total_count == 0
    }
}

pub struct StatsCollector {
    column_stats: HashMap<String, ColumnStatistics>,
    sample_rate: f64,
    histogram_buckets: usize,
}

impl StatsCollector {
    pub fn new(sample_rate: f64, histogram_buckets: usize) -> Self {
        Self {
            column_stats: HashMap::new(),
            sample_rate: sample_rate.clamp(0.01, 1.0),
            histogram_buckets,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(1.0, 10)
    }

    pub fn init_column(&mut self, column_name: String, data_type: DataType) {
        let stats = ColumnStatistics::new(column_name.clone(), data_type);
        self.column_stats.insert(column_name, stats);
    }

    pub fn collect_values(&mut self, column_name: &str, values: &[Option<Value>]) {
        let stats = self.column_stats.entry(column_name.to_string()).or_insert_with(|| {
            ColumnStatistics::new(
                column_name.to_string(),
                if !values.is_empty() {
                    if let Some(Some(v)) = values.first() {
                        v.data_type()
                    } else {
                        DataType::String
                    }
                } else {
                    DataType::String
                },
            )
        });

        let mut distinct_values = std::collections::HashSet::new();
        let mut total_length = 0.0;
        let mut length_count = 0;
        let mut min_val: Option<Value> = None;
        let mut max_val: Option<Value> = None;

        let mut histogram = Histogram::new(self.histogram_buckets);

        for value in values {
            stats.total_count += 1;

            match value {
                None => {
                    stats.null_count += 1;
                }
                Some(v) => {
                    distinct_values.insert(v.clone());

                    if let Value::String(s) = v {
                        total_length += s.len() as f64;
                        length_count += 1;
                    }

                    min_val = Some(match &min_val {
                        Some(existing) => Self::min_value(existing, v).clone(),
                        None => v.clone(),
                    });

                    max_val = Some(match &max_val {
                        Some(existing) => Self::max_value(existing, v).clone(),
                        None => v.clone(),
                    });

                    histogram.add_value(v);
                }
            }
        }

        stats.distinct_count = distinct_values.len();
        stats.min_value = min_val;
        stats.max_value = max_val;

        if length_count > 0 {
            stats.avg_length = total_length / length_count as f64;
        }

        histogram.sort_most_common();
        stats.histogram = Some(histogram);
    }

    pub fn collect_from_column(&mut self, column_name: &str, data: &[Value], null_bitmap: &[bool]) {
        let values: Vec<Option<Value>> = data
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if i < null_bitmap.len() && null_bitmap[i] {
                    None
                } else {
                    Some(v.clone())
                }
            })
            .collect();

        self.collect_values(column_name, &values);
    }

    pub fn get_stats(&self, column_name: &str) -> Option<&ColumnStatistics> {
        self.column_stats.get(column_name)
    }

    pub fn get_stats_mut(&mut self, column_name: &str) -> Option<&mut ColumnStatistics> {
        self.column_stats.get_mut(column_name)
    }

    pub fn all_stats(&self) -> &HashMap<String, ColumnStatistics> {
        &self.column_stats
    }

    pub fn clear(&mut self) {
        self.column_stats.clear();
    }

    fn min_value<'a>(a: &'a Value, b: &'a Value) -> &'a Value {
        match (a, b) {
            (Value::Int(a_val), Value::Int(b_val)) => if a_val <= b_val { a } else { b },
            (Value::Double(a_val), Value::Double(b_val)) => if a_val <= b_val { a } else { b },
            (Value::String(a_val), Value::String(b_val)) => if a_val <= b_val { a } else { b },
            _ => a,
        }
    }

    fn max_value<'a>(a: &'a Value, b: &'a Value) -> &'a Value {
        match (a, b) {
            (Value::Int(a_val), Value::Int(b_val)) => if a_val >= b_val { a } else { b },
            (Value::Double(a_val), Value::Double(b_val)) => if a_val >= b_val { a } else { b },
            (Value::String(a_val), Value::String(b_val)) => if a_val >= b_val { a } else { b },
            _ => a,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_statistics_basic() {
        let mut collector = StatsCollector::with_defaults();

        let values: Vec<Option<Value>> = vec![
            Some(Value::Int(10)),
            Some(Value::Int(20)),
            Some(Value::Int(10)),
            Some(Value::Int(30)),
            None,
            Some(Value::Int(20)),
        ];

        collector.collect_values("age", &values);

        let stats = collector.get_stats("age").unwrap();
        assert_eq!(stats.total_count, 6);
        assert_eq!(stats.null_count, 1);
        assert_eq!(stats.distinct_count, 3);
        assert_eq!(stats.min_value, Some(Value::Int(10)));
        assert_eq!(stats.max_value, Some(Value::Int(30)));
    }

    #[test]
    fn test_histogram() {
        let mut histogram = Histogram::new(3);
        histogram.buckets.push(HistogramBucket::new(Value::Int(0), Value::Int(10)));
        histogram.buckets.push(HistogramBucket::new(Value::Int(11), Value::Int(20)));
        histogram.buckets.push(HistogramBucket::new(Value::Int(21), Value::Int(30)));

        for i in 0..30 {
            histogram.add_value(&Value::Int(i + 1));
        }

        histogram.sort_most_common();

        assert_eq!(histogram.total_count, 30);
        assert_eq!(histogram.buckets.len(), 3);
    }

    #[test]
    fn test_selectivity() {
        let mut collector = StatsCollector::with_defaults();

        let values: Vec<Option<Value>> = (0..100)
            .map(|i| Some(Value::Int(i)))
            .collect();

        collector.collect_values("id", &values);

        let stats = collector.get_stats("id").unwrap();
        let selectivity = stats.selectivity(&Value::Int(50));

        assert!(selectivity > 0.0);
        assert!(selectivity <= 1.0);
    }

    #[test]
    fn test_string_statistics() {
        let mut collector = StatsCollector::with_defaults();

        let values: Vec<Option<Value>> = vec![
            Some(Value::String("Alice".to_string())),
            Some(Value::String("Bob".to_string())),
            Some(Value::String("Alice".to_string())),
            Some(Value::String("Charlie".to_string())),
        ];

        collector.collect_values("name", &values);

        let stats = collector.get_stats("name").unwrap();
        assert_eq!(stats.distinct_count, 3);
        assert!(stats.avg_length > 0.0);
    }

    #[test]
    fn test_null_ratio() {
        let mut collector = StatsCollector::with_defaults();

        let values: Vec<Option<Value>> = vec![
            Some(Value::Int(1)),
            None,
            None,
            Some(Value::Int(2)),
        ];

        collector.collect_values("col", &values);

        let stats = collector.get_stats("col").unwrap();
        assert!((stats.null_ratio() - 0.5).abs() < 0.001);
    }
}
