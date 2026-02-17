# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 0
- **Total Issues**: 4
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0502]**: 4 errors

### Files with Errors (Top 10)

- `src\query\parser\parser\ddl_parser.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0502]: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\query\parser\parser\ddl_parser.rs`: 4 occurrences

- Line 533: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here
- Line 537: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here
- Line 541: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here
- ... 1 more occurrences in this file

