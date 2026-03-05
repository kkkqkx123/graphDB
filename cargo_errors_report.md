# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 9
- **Total Warnings**: 2
- **Total Issues**: 11
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 1
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 9

### Error Type Breakdown

- **error[E0596]**: 9 errors

### Files with Errors (Top 10)

- `src\query\validator\statements\insert_edges_validator.rs`: 9 errors

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\query\validator\statements\insert_edges_validator.rs`: 2 warnings

## Detailed Error Categorization

### error[E0596]: cannot borrow `validator` as mutable, as it is not declared as mutable: cannot borrow as mutable

**Total Occurrences**: 9  
**Unique Files**: 1

#### `src\query\validator\statements\insert_edges_validator.rs`: 9 occurrences

- Line 471: cannot borrow `validator` as mutable, as it is not declared as mutable: cannot borrow as mutable
- Line 493: cannot borrow `validator` as mutable, as it is not declared as mutable: cannot borrow as mutable
- Line 514: cannot borrow `validator` as mutable, as it is not declared as mutable: cannot borrow as mutable
- ... 6 more occurrences in this file

## Detailed Warning Categorization

### warning: variable does not need to be mutable

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\statements\insert_edges_validator.rs`: 2 occurrences

- Line 646: variable does not need to be mutable
- Line 665: variable does not need to be mutable

