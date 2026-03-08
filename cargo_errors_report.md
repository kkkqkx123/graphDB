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

- `src\core\result\result.rs`: 1 warnings
- `src\query\execution\resource_context.rs`: 1 warnings
- `src\core\result\result_iterator.rs`: 1 warnings

## Detailed Warning Categorization

### warning: associated function `from_builder` is never used

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\core\result\result.rs`: 1 occurrences

- Line 72: associated function `from_builder` is never used

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 1: unused import: `crate::core::value::Value`

#### `src\query\execution\resource_context.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

