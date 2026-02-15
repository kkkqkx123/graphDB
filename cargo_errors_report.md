# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 1
- **Total Issues**: 2
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 1
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0502]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\data_processing\join\hash_table.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\query\executor\data_processing\join\hash_table.rs`: 1 warnings

## Detailed Error Categorization

### error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable: mutable borrow occurs here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 412: cannot borrow `*self` as mutable because it is also borrowed as immutable: mutable borrow occurs here

## Detailed Warning Categorization

### warning: variable `spilled_count` is assigned to, but never used

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 446: variable `spilled_count` is assigned to, but never used

