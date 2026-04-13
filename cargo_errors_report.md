# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 144
- **Total Warnings**: 36
- **Total Issues**: 180
- **Unique Error Patterns**: 11
- **Unique Warning Patterns**: 25
- **Files with Issues**: 19

## Error Statistics

**Total Errors**: 144

### Error Type Breakdown

- **error[E0282]**: 136 errors
- **error[E0433]**: 3 errors
- **error[E0425]**: 3 errors
- **error[E0432]**: 1 errors
- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `tests\integration_fulltext_search.rs`: 133 errors
- `tests\integration_web_management.rs`: 11 errors

## Warning Statistics

**Total Warnings**: 36

### Warning Type Breakdown

- **warning**: 36 warnings

### Files with Warnings (Top 10)

- `src\sync\coordinator\coordinator.rs`: 10 warnings
- `tests\common\sync_helpers.rs`: 6 warnings
- `src\main.rs`: 4 warnings
- `src\utils\output\manager.rs`: 3 warnings
- `src\query\executor\factory\builders\fulltext_search_builder.rs`: 1 warnings
- `src\utils\output\stream.rs`: 1 warnings
- `src\storage\extend\fulltext_storage.rs`: 1 warnings
- `src\utils\output\json.rs`: 1 warnings
- `src\query\planning\fulltext_planner.rs`: 1 warnings
- `src\query\query_pipeline_manager.rs`: 1 warnings

## Detailed Error Categorization

### error[E0282]: type annotations needed for `(_, _)`

**Total Occurrences**: 136  
**Unique Files**: 2

#### `tests\integration_fulltext_search.rs`: 127 occurrences

- Line 75: type annotations needed for `(_, _)`
- Line 123: type annotations needed for `(_, _)`
- Line 125: type annotations needed: cannot infer type
- ... 124 more occurrences in this file

#### `tests\integration_web_management.rs`: 9 occurrences

- Line 89: type annotations needed for `(graphdb::api::server::WebState<_>, _)`
- Line 127: type annotations needed for `(graphdb::api::server::WebState<_>, _)`
- Line 164: type annotations needed for `(graphdb::api::server::WebState<_>, _)`
- ... 6 more occurrences in this file

### error[E0433]: failed to resolve: use of undeclared type `FulltextCoordinator`: use of undeclared type `FulltextCoordinator`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `tests\integration_fulltext_search.rs`: 2 occurrences

- Line 52: failed to resolve: use of undeclared type `FulltextCoordinator`: use of undeclared type `FulltextCoordinator`
- Line 70: failed to resolve: use of undeclared type `FulltextCoordinator`: use of undeclared type `FulltextCoordinator`

#### `tests\integration_web_management.rs`: 1 occurrences

- Line 22: failed to resolve: could not find `redb_storage` in `storage`: could not find `redb_storage` in `storage`

### error[E0425]: cannot find type `FulltextCoordinator` in this scope: not found in this scope

**Total Occurrences**: 3  
**Unique Files**: 1

#### `tests\integration_fulltext_search.rs`: 3 occurrences

- Line 38: cannot find type `FulltextCoordinator` in this scope: not found in this scope
- Line 56: cannot find type `FulltextCoordinator` in this scope: not found in this scope
- Line 74: cannot find type `FulltextCoordinator` in this scope: not found in this scope

### error[E0432]: unresolved import `graphdb::storage::redb_storage`: could not find `redb_storage` in `storage`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_web_management.rs`: 1 occurrences

- Line 30: unresolved import `graphdb::storage::redb_storage`: could not find `redb_storage` in `storage`

### error[E0599]: no method named `on_vertex_change` found for struct `graphdb::sync::SyncManager` in the current scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_fulltext_search.rs`: 1 occurrences

- Line 677: no method named `on_vertex_change` found for struct `graphdb::sync::SyncManager` in the current scope

## Detailed Warning Categorization

### warning: unused import: `crate::search::manager::FulltextIndexManager`

**Total Occurrences**: 36  
**Unique Files**: 17

#### `src\sync\coordinator\coordinator.rs`: 10 occurrences

- Line 567: unused variable: `id`: help: try ignoring the field: `id: _`
- Line 569: unused variable: `payload`: help: try ignoring the field: `payload: _`
- Line 573: unused variable: `text`: help: if this is intentional, prefix it with an underscore: `_text`
- ... 7 more occurrences in this file

#### `tests\common\sync_helpers.rs`: 6 occurrences

- Line 14: unused imports: `TransactionManagerConfig`, `TransactionManager`, and `TransactionOptions`
- Line 352: unused variable: `field_name`: help: if this is intentional, prefix it with an underscore: `_field_name`
- Line 357: unused variable: `space_id`: help: if this is intentional, prefix it with an underscore: `_space_id`
- ... 3 more occurrences in this file

#### `src\main.rs`: 4 occurrences

- Line 29: unused `std::result::Result` that must be used
- Line 30: unused `std::result::Result` that must be used
- Line 57: unused `std::result::Result` that must be used
- ... 1 more occurrences in this file

#### `src\utils\output\manager.rs`: 3 occurrences

- Line 3: unused import: `self`
- Line 20: this `impl` can be derived
- Line 127: this `repeat().take()` can be written more concisely: help: consider using `repeat_n()` instead: `std::iter::repeat_n(char, length)`

#### `src\query\executor\factory\builders\fulltext_search_builder.rs`: 1 occurrences

- Line 27: unused import: `crate::search::manager::FulltextIndexManager`

#### `src\utils\output\config.rs`: 1 occurrences

- Line 14: this `impl` can be derived

#### `src\utils\output\table.rs`: 1 occurrences

- Line 165: function `print_table_slices` is never used

#### `src\storage\extend\fulltext_storage.rs`: 1 occurrences

- Line 163: this `if let` can be collapsed into the outer `if let`

#### `crates\vector-client\src\engine\qdrant\config.rs`: 1 occurrences

- Line 18: function `convert_index_type` is never used

#### `src\sync\manager.rs`: 1 occurrences

- Line 311: unused variable: `vector_coord`: help: if this is intentional, prefix it with an underscore: `_vector_coord`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 212: this `if let` can be collapsed into the outer `if let`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 670: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\utils\output\stream.rs`: 1 occurrences

- Line 38: field `file` is never read

#### `crates\vector-client\src\engine\qdrant\mod.rs`: 1 occurrences

- Line 24: unused import: `convert_index_type`

#### `src\query\planning\fulltext_planner.rs`: 1 occurrences

- Line 33: field `metadata_context` is never read

#### `src\utils\output\json.rs`: 1 occurrences

- Line 7: unused import: `OutputError`

#### `tests\common\transaction_helpers.rs`: 1 occurrences

- Line 250: manual absolute difference pattern without using `abs_diff`: help: replace with `abs_diff`: `elapsed.abs_diff(expected)`

