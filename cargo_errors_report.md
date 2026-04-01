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

- `src\query\validator\statements\lookup_validator.rs`: 2 warnings
- `src\query\executor\data_access\search.rs`: 1 warnings

## Detailed Warning Categorization

### warning: method `get_schema_name` is never used

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\validator\statements\lookup_validator.rs`: 2 occurrences

- Line 238: unneeded `return` statement
- Line 246: unneeded `return` statement

#### `src\query\executor\data_access\search.rs`: 1 occurrences

- Line 94: method `get_schema_name` is never used

