# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 17
- **Total Issues**: 17
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 10
- **Files with Issues**: 8

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 17

### Warning Type Breakdown

- **warning**: 17 warnings

### Files with Warnings (Top 10)

- `src\core\stats\aggregated_stats.rs`: 4 warnings
- `src\core\stats\latency_histogram.rs`: 3 warnings
- `src\core\stats\profile.rs`: 3 warnings
- `src\config\server\security.rs`: 2 warnings
- `src\config\common\storage.rs`: 2 warnings
- `src\api\server\grpc\server.rs`: 1 warnings
- `src\query\executor\explain\execution_stats_context.rs`: 1 warnings
- `src\core\stats\slow_query_logger.rs`: 1 warnings

## Detailed Warning Categorization

### warning: this `impl` can be derived

**Total Occurrences**: 17  
**Unique Files**: 8

#### `src\core\stats\aggregated_stats.rs`: 4 occurrences

- Line 287: casting to the same type is unnecessary (`f64` -> `f64`): help: try: `self.avg_duration_us`
- Line 465: manual implementation of `.is_multiple_of()`: help: replace with: `query_num.is_multiple_of((1.0 / sample_rate) as u64)`
- Line 759: field assignment outside of initializer for an instance created with Default::default()
- ... 1 more occurrences in this file

#### `src\core\stats\latency_histogram.rs`: 3 occurrences

- Line 152: manual `RangeInclusive::contains` implementation: help: use: `(49..=51).contains(&p50)`
- Line 160: manual `RangeInclusive::contains` implementation: help: use: `(94..=96).contains(&p95)`
- Line 168: manual `RangeInclusive::contains` implementation: help: use: `(98..=100).contains(&p99)`

#### `src\core\stats\profile.rs`: 3 occurrences

- Line 209: field assignment outside of initializer for an instance created with Default::default()
- Line 230: field assignment outside of initializer for an instance created with Default::default()
- Line 244: field assignment outside of initializer for an instance created with Default::default()

#### `src\config\common\storage.rs`: 2 occurrences

- Line 16: this `impl` can be derived
- Line 45: this `impl` can be derived

#### `src\config\server\security.rs`: 2 occurrences

- Line 20: this `impl` can be derived
- Line 182: this `impl` can be derived

#### `src\api\server\grpc\server.rs`: 1 occurrences

- Line 487: unused import: `super::*`

#### `src\core\stats\slow_query_logger.rs`: 1 occurrences

- Line 373: this loop could be written as a `while let` loop: help: try: `while let Ok(log_entry) = rx.recv() { .. }`

#### `src\query\executor\explain\execution_stats_context.rs`: 1 occurrences

- Line 180: field assignment outside of initializer for an instance created with Default::default()

