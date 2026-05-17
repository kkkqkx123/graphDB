# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 2
- **Total Issues**: 2
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 2
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\storage\container\volatile\windows.rs`: 1 warnings
- `src\storage\container\persistent\mod.rs`: 1 warnings

## Detailed Warning Categorization

### warning: method `is_empty` is never used

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\storage\container\volatile\windows.rs`: 1 occurrences

- Line 66: method `is_empty` is never used

#### `src\storage\container\persistent\mod.rs`: 1 occurrences

- Line 26: unused import: `crate::storage::container::mmap::IDataContainer`

