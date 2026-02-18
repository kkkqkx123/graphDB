# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 1
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\rules\limit_pushdown\push_topn_down_index_scan.rs`: 2 warnings
- `src\query\optimizer\rules\index\index_covering_scan.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused `std::result::Result` that must be used

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\optimizer\rules\limit_pushdown\push_topn_down_index_scan.rs`: 2 occurrences

- Line 160: unused `std::result::Result` that must be used
- Line 198: unused `std::result::Result` that must be used

#### `src\query\optimizer\rules\index\index_covering_scan.rs`: 1 occurrences

- Line 195: unused `std::result::Result` that must be used

