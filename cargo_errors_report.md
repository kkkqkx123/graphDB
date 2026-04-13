# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 93
- **Total Warnings**: 6
- **Total Issues**: 99
- **Unique Error Patterns**: 9
- **Unique Warning Patterns**: 5
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 93

### Error Type Breakdown

- **error[E0599]**: 47 errors
- **error[E0282]**: 46 errors

### Files with Errors (Top 10)

- `tests\fulltext_integration_test.rs`: 93 errors

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `tests\sync_2pc_protocol.rs`: 4 warnings
- `src\storage\extend\fulltext_storage.rs`: 1 warnings
- `tests\sync_transaction_basic.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `create_index` found for struct `std::sync::Arc<graphdb::sync::SyncCoordinator>` in the current scope: method not found in `std::sync::Arc<graphdb::sync::SyncCoordinator>`

**Total Occurrences**: 47  
**Unique Files**: 1

#### `tests\fulltext_integration_test.rs`: 47 occurrences

- Line 49: no method named `create_index` found for struct `std::sync::Arc<graphdb::sync::SyncCoordinator>` in the current scope: method not found in `std::sync::Arc<graphdb::sync::SyncCoordinator>`
- Line 72: no method named `on_vertex_inserted` found for struct `std::sync::Arc<graphdb::sync::SyncCoordinator>` in the current scope
- Line 76: no method named `on_vertex_inserted` found for struct `std::sync::Arc<graphdb::sync::SyncCoordinator>` in the current scope
- ... 44 more occurrences in this file

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 46  
**Unique Files**: 1

#### `tests\fulltext_integration_test.rs`: 46 occurrences

- Line 48: type annotations needed: cannot infer type
- Line 71: type annotations needed: cannot infer type
- Line 75: type annotations needed: cannot infer type
- ... 43 more occurrences in this file

## Detailed Warning Categorization

### warning: this `if let` can be collapsed into the outer `if let`

**Total Occurrences**: 6  
**Unique Files**: 3

#### `tests\sync_2pc_protocol.rs`: 4 occurrences

- Line 383: casting to the same type is unnecessary (`i64` -> `i64`): help: try: `((i * 10 + 1))`
- Line 401: this assertion is always `true`
- Line 459: casting to the same type is unnecessary (`i64` -> `i64`): help: try: `((i + 1))`
- ... 1 more occurrences in this file

#### `src\storage\extend\fulltext_storage.rs`: 1 occurrences

- Line 164: this `if let` can be collapsed into the outer `if let`

#### `tests\sync_transaction_basic.rs`: 1 occurrences

- Line 250: length comparison to one: help: using `!is_empty` is clearer and more explicit: `!results.is_empty()`

