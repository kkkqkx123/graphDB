# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 11
- **Total Warnings**: 1
- **Total Issues**: 12
- **Unique Error Patterns**: 7
- **Unique Warning Patterns**: 1
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 11

### Error Type Breakdown

- **error[E0599]**: 8 errors
- **error[E0308]**: 2 errors
- **error[E0596]**: 1 errors

### Files with Errors (Top 10)

- `tests\cache_test.rs`: 10 errors
- `tests\integration_test.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\index\cache.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `is_empty` found for struct `bm25_service::index::cache::Cache<K, V>` in the current scope: method not found in `bm25_service::index::cache::Cache<i32, std::string::String>`

**Total Occurrences**: 8  
**Unique Files**: 1

#### `tests\cache_test.rs`: 8 occurrences

- Line 79: no method named `is_empty` found for struct `bm25_service::index::cache::Cache<K, V>` in the current scope: method not found in `bm25_service::index::cache::Cache<i32, std::string::String>`
- Line 83: no method named `is_empty` found for struct `bm25_service::index::cache::Cache<K, V>` in the current scope: method not found in `bm25_service::index::cache::Cache<i32, std::string::String>`
- Line 94: no method named `is_empty` found for struct `bm25_service::index::cache::Cache<K, V>` in the current scope: method not found in `bm25_service::index::cache::Cache<std::string::String, std::string::String>`
- ... 5 more occurrences in this file

### error[E0308]: mismatched types: expected `()`, found `Option<String>`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `tests\cache_test.rs`: 2 occurrences

- Line 49: mismatched types: expected `()`, found `Option<String>`
- Line 343: mismatched types: expected `()`, found `Option<_>`

### error[E0596]: cannot borrow `cache` as mutable, as it is not declared as mutable: not mutable

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_test.rs`: 1 occurrences

- Line 187: cannot borrow `cache` as mutable, as it is not declared as mutable: not mutable

## Detailed Warning Categorization

### warning: this `impl` can be derived

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\index\cache.rs`: 1 occurrences

- Line 12: this `impl` can be derived

