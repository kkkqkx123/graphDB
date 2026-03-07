# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0603]**: 1 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\storage\iterator\storage_iter.rs`: 1 errors
- `src\storage\iterator\mod.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0432]: unresolved import `crate::storage::iterator::StorageIterator`: no `StorageIterator` in `storage::iterator`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\iterator\storage_iter.rs`: 1 occurrences

- Line 11: unresolved import `crate::storage::iterator::StorageIterator`: no `StorageIterator` in `storage::iterator`

### error[E0603]: unresolved item import `StorageIterator` is private: private unresolved item import

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\iterator\mod.rs`: 1 occurrences

- Line 21: unresolved item import `StorageIterator` is private: private unresolved item import

