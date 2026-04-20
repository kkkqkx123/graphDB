# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 3
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\core\stats\aggregated_stats.rs`: 1 warnings
- `tests\integration_core.rs`: 1 warnings
- `src\config\server\security.rs`: 1 warnings

## Detailed Warning Categorization

### warning: `Box::new(_)` of default value: help: try: `Box::default()`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `tests\integration_core.rs`: 1 occurrences

- Line 330: `Box::new(_)` of default value: help: try: `Box::default()`

#### `src\config\server\security.rs`: 1 occurrences

- Line 171: this `impl` can be derived

#### `src\core\stats\aggregated_stats.rs`: 1 occurrences

- Line 784: manual `RangeInclusive::contains` implementation: help: use: `(20..=80).contains(&sampled)`

