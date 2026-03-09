# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 2
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\executor\statement_executors\group_by_executor.rs`: 2 warnings
- `src\query\executor\statement_executors\set_operation_executor.rs`: 1 warnings

## Detailed Warning Categorization

### warning: field `id` is never read

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\executor\statement_executors\group_by_executor.rs`: 2 occurrences

- Line 30: field `id` is never read
- Line 146: associated function `apply_having` is never used

#### `src\query\executor\statement_executors\set_operation_executor.rs`: 1 occurrences

- Line 27: field `id` is never read

