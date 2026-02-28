# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 0
- **Total Issues**: 4
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0061]**: 2 errors
- **error**: 1 errors
- **error[E0382]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\strategy\partition_pruning.rs`: 3 errors
- `src\query\optimizer\strategy\materialization_strategy.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0061]: this function takes 1 argument but 0 arguments were supplied

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\strategy\partition_pruning.rs`: 2 occurrences

- Line 253: this function takes 1 argument but 0 arguments were supplied
- Line 265: this function takes 1 argument but 0 arguments were supplied

### error: lifetime may not live long enough: method was supposed to return data with lifetime `'2` but it is returning data with lifetime `'1`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\strategy\materialization_strategy.rs`: 1 occurrences

- Line 399: lifetime may not live long enough: method was supposed to return data with lifetime `'2` but it is returning data with lifetime `'1`

### error[E0382]: borrow of moved value: `selected_partitions`: value borrowed here after move

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\strategy\partition_pruning.rs`: 1 occurrences

- Line 354: borrow of moved value: `selected_partitions`: value borrowed here after move

