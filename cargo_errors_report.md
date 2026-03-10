# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 3
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\api\server\http\handlers\function.rs`: 2 warnings
- `src\api\server\http\handlers\config.rs`: 1 warnings

## Detailed Warning Categorization

### warning: variable does not need to be mutable

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\api\server\http\handlers\function.rs`: 2 occurrences

- Line 20: variable does not need to be mutable
- Line 155: field `registered_at` is never read

#### `src\api\server\http\handlers\config.rs`: 1 occurrences

- Line 80: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`

