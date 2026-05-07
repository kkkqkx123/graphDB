//! Statistics Module
//!
//! Provides column statistics collection and management for query optimization.

pub mod column_stats;

pub use column_stats::{
    ColumnStatistics, Histogram, HistogramBucket, StatsCollector,
};
