# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 10
- **Total Warnings**: 35
- **Total Issues**: 45
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 18
- **Files with Issues**: 15

## Error Statistics

**Total Errors**: 10

### Error Type Breakdown

- **error[E0433]**: 9 errors
- **error[E0063]**: 1 errors

### Files with Errors (Top 10)

- `tests\integration_logging.rs`: 9 errors
- `tests\integration_fulltext_search.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 35

### Warning Type Breakdown

- **warning**: 35 warnings

### Files with Warnings (Top 10)

- `src\sync\vector_sync.rs`: 10 warnings
- `src\sync\manager.rs`: 5 warnings
- `src\query\validator\statements\insert_vertices_validator.rs`: 3 warnings
- `src\storage\event_storage.rs`: 3 warnings
- `src\transaction\sync_handle.rs`: 3 warnings
- `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 3 warnings
- `src\transaction\index_buffer.rs`: 2 warnings
- `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 warnings
- `src\query\executor\result_processing\agg_function_manager.rs`: 1 warnings
- `src\sync\queue.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: could not find `vector` in `graphdb`: could not find `vector` in `graphdb`

**Total Occurrences**: 9  
**Unique Files**: 1

#### `tests\integration_logging.rs`: 9 occurrences

- Line 55: failed to resolve: could not find `vector` in `graphdb`: could not find `vector` in `graphdb`
- Line 265: failed to resolve: could not find `vector` in `graphdb`: could not find `vector` in `graphdb`
- Line 293: failed to resolve: could not find `vector` in `graphdb`: could not find `vector` in `graphdb`
- ... 6 more occurrences in this file

### error[E0063]: missing field `failure_policy` in initializer of `graphdb::search::SyncConfig`: missing `failure_policy`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_fulltext_search.rs`: 1 occurrences

- Line 77: missing field `failure_policy` in initializer of `graphdb::search::SyncConfig`: missing `failure_policy`

## Detailed Warning Categorization

### warning: field `batch_size` is never read

**Total Occurrences**: 35  
**Unique Files**: 13

#### `src\sync\vector_sync.rs`: 10 occurrences

- Line 72: variant `Update` is never constructed
- Line 139: fields `batch_size` and `batch_timeout_ms` are never read
- Line 279: use of `or_insert_with` to construct default value: help: try: `or_default()`
- ... 7 more occurrences in this file

#### `src\sync\manager.rs`: 5 occurrences

- Line 382: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 401: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 624: use of `or_insert_with` to construct default value: help: try: `or_default()`
- ... 2 more occurrences in this file

#### `src\storage\event_storage.rs`: 3 occurrences

- Line 151: deref which would be done by auto-deref: help: try: `&vertex_id`
- Line 194: deref which would be done by auto-deref: help: try: `&vertex_id`
- Line 307: deref which would be done by auto-deref: help: try: `&vertex_id`

#### `src\transaction\sync_handle.rs`: 3 occurrences

- Line 90: field `created_at` is never read
- Line 35: this function has too many arguments (9/7)
- Line 86: very complex type used. Consider factoring parts into `type` definitions

#### `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 3 occurrences

- Line 34: this function has too many arguments (10/7)
- Line 82: this function has too many arguments (10/7)
- Line 180: this function has too many arguments (9/7)

#### `src\query\validator\statements\insert_vertices_validator.rs`: 3 occurrences

- Line 156: this function has too many arguments (8/7)
- Line 190: you seem to use `.enumerate()` and immediately discard the index
- Line 207: this `if let` can be collapsed into the outer `if let`

#### `src\transaction\index_buffer.rs`: 2 occurrences

- Line 37: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 183: field assignment outside of initializer for an instance created with Default::default()

#### `src\sync\batch.rs`: 1 occurrences

- Line 44: field `batch_size` is never read

#### `src\api\core\query_api.rs`: 1 occurrences

- Line 22: field `vector_coordinator` is never read

#### `src\query\executor\result_processing\agg_function_manager.rs`: 1 occurrences

- Line 468: you seem to use `.enumerate()` and immediately discard the index

#### `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 occurrences

- Line 378: you seem to use `.enumerate()` and immediately discard the index

#### `src\sync\queue.rs`: 1 occurrences

- Line 126: field `shutdown_tx` is never read

#### `src\search\config.rs`: 1 occurrences

- Line 18: this `impl` can be derived

