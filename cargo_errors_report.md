# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 17
- **Total Issues**: 17
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 5
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 17

### Warning Type Breakdown

- **warning**: 17 warnings

### Files with Warnings (Top 10)

- `src\core\value\comparison.rs`: 16 warnings
- `src\core\query_result\iterator_enum.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused `std::result::Result` that must be used

**Total Occurrences**: 17  
**Unique Files**: 2

#### `src\core\value\comparison.rs`: 16 occurrences

- Line 43: casting to the same type is unnecessary (`i64` -> `i64`): help: try: `*a`
- Line 44: casting to the same type is unnecessary (`i64` -> `i64`): help: try: `*a`
- Line 45: casting to the same type is unnecessary (`i64` -> `i64`): help: try: `*a`
- ... 13 more occurrences in this file

#### `src\core\query_result\iterator_enum.rs`: 1 occurrences

- Line 213: unused `std::result::Result` that must be used

