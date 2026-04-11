# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 26
- **Total Warnings**: 13
- **Total Issues**: 39
- **Unique Error Patterns**: 6
- **Unique Warning Patterns**: 10
- **Files with Issues**: 11

## Error Statistics

**Total Errors**: 26

### Error Type Breakdown

- **error[E0599]**: 22 errors
- **error[E0282]**: 1 errors
- **error[E0432]**: 1 errors
- **error[E0308]**: 1 errors
- **error[E0560]**: 1 errors

### Files with Errors (Top 10)

- `tests\integration_transaction.rs`: 21 errors
- `tests\integration_fulltext_search.rs`: 5 errors

## Warning Statistics

**Total Warnings**: 13

### Warning Type Breakdown

- **warning**: 13 warnings

### Files with Warnings (Top 10)

- `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 3 warnings
- `src\transaction\sync_handle.rs`: 2 warnings
- `src\query\validator\statements\insert_vertices_validator.rs`: 2 warnings
- `src\sync\coordinator\mod.rs`: 1 warnings
- `src\sync\batch\processor.rs`: 1 warnings
- `src\query\executor\result_processing\agg_function_manager.rs`: 1 warnings
- `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 warnings
- `src\search\config.rs`: 1 warnings
- `src\transaction\index_buffer.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no function or associated item named `with_mode` found for struct `graphdb::sync::SyncManager` in the current scope: function or associated item not found in `graphdb::sync::SyncManager`

**Total Occurrences**: 22  
**Unique Files**: 2

#### `tests\integration_transaction.rs`: 21 occurrences

- Line 226: no method named `expect` found for opaque type `impl Future<Output = Result<(), TransactionError>>` in the current scope: method not found in `impl Future<Output = Result<(), TransactionError>>`
- Line 358: no method named `expect` found for opaque type `impl Future<Output = Result<(), TransactionError>>` in the current scope: method not found in `impl Future<Output = Result<(), TransactionError>>`
- Line 396: no method named `expect` found for opaque type `impl Future<Output = Result<(), TransactionError>>` in the current scope: method not found in `impl Future<Output = Result<(), TransactionError>>`
- ... 18 more occurrences in this file

#### `tests\integration_fulltext_search.rs`: 1 occurrences

- Line 655: no function or associated item named `with_mode` found for struct `graphdb::sync::SyncManager` in the current scope: function or associated item not found in `graphdb::sync::SyncManager`

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_fulltext_search.rs`: 1 occurrences

- Line 664: type annotations needed: cannot infer type

### error[E0432]: unresolved import `graphdb::sync::manager::SyncMode`: no `SyncMode` in `sync::manager`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_fulltext_search.rs`: 1 occurrences

- Line 28: unresolved import `graphdb::sync::manager::SyncMode`: no `SyncMode` in `sync::manager`

### error[E0560]: struct `graphdb::search::SyncConfig` has no field named `mode`: `graphdb::search::SyncConfig` does not have this field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_fulltext_search.rs`: 1 occurrences

- Line 78: struct `graphdb::search::SyncConfig` has no field named `mode`: `graphdb::search::SyncConfig` does not have this field

### error[E0308]: mismatched types: expected `Arc<SyncCoordinator>`, found `Arc<FulltextCoordinator>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_fulltext_search.rs`: 1 occurrences

- Line 85: mismatched types: expected `Arc<SyncCoordinator>`, found `Arc<FulltextCoordinator>`

## Detailed Warning Categorization

### warning: this function has too many arguments (8/7)

**Total Occurrences**: 13  
**Unique Files**: 9

#### `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 3 occurrences

- Line 34: this function has too many arguments (10/7)
- Line 82: this function has too many arguments (10/7)
- Line 180: this function has too many arguments (9/7)

#### `src\query\validator\statements\insert_vertices_validator.rs`: 2 occurrences

- Line 156: this function has too many arguments (8/7)
- Line 202: this `if let` can be collapsed into the outer `if let`

#### `src\transaction\sync_handle.rs`: 2 occurrences

- Line 32: this function has too many arguments (9/7)
- Line 82: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\executor\result_processing\agg_function_manager.rs`: 1 occurrences

- Line 468: you seem to use `.enumerate()` and immediately discard the index

#### `src\sync\batch\processor.rs`: 1 occurrences

- Line 211: you should consider adding a `Default` implementation for `TransactionBatchBuffer`

#### `src\transaction\index_buffer.rs`: 1 occurrences

- Line 176: field assignment outside of initializer for an instance created with Default::default()

#### `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 occurrences

- Line 378: you seem to use `.enumerate()` and immediately discard the index

#### `src\search\config.rs`: 1 occurrences

- Line 17: this `impl` can be derived

#### `src\sync\coordinator\mod.rs`: 1 occurrences

- Line 1: module has the same name as its containing module

