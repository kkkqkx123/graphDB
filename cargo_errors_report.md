# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 25
- **Total Issues**: 29
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 20
- **Files with Issues**: 13

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0432]**: 3 errors
- **error[E0282]**: 1 errors

### Files with Errors (Top 10)

- `tests\integration_vector_query.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 25

### Warning Type Breakdown

- **warning**: 25 warnings

### Files with Warnings (Top 10)

- `src\query\validator\statements\insert_vertices_validator.rs`: 6 warnings
- `src\storage\event_storage.rs`: 4 warnings
- `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 3 warnings
- `crates\vector-client\src\embedding\service.rs`: 3 warnings
- `src\sync\vector_sync.rs`: 2 warnings
- `src\query\executor\result_processing\agg_function_manager.rs`: 1 warnings
- `src\query\planning\statements\dml\insert_planner.rs`: 1 warnings
- `src\sync\batch.rs`: 1 warnings
- `crates\inversearch\src\config\validator.rs`: 1 warnings
- `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 warnings

## Detailed Error Categorization

### error[E0432]: unresolved import `graphdb::vector::config`: could not find `config` in `vector`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `tests\integration_vector_query.rs`: 3 occurrences

- Line 13: unresolved import `graphdb::vector::config`: could not find `config` in `vector`
- Line 14: unresolved import `graphdb::vector::coordinator`: could not find `coordinator` in `vector`
- Line 15: unresolved import `graphdb::vector::manager`: could not find `manager` in `vector`

### error[E0282]: type annotations needed: cannot infer type

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_vector_query.rs`: 1 occurrences

- Line 29: type annotations needed: cannot infer type

## Detailed Warning Categorization

### warning: unused import: `std::fmt`

**Total Occurrences**: 25  
**Unique Files**: 12

#### `src\query\validator\statements\insert_vertices_validator.rs`: 6 occurrences

- Line 163: unused variable: `row_idx`: help: if this is intentional, prefix it with an underscore: `_row_idx`
- Line 164: unused variable: `tag_idx`: help: if this is intentional, prefix it with an underscore: `_tag_idx`
- Line 187: unused variable: `prop_idx`: help: if this is intentional, prefix it with an underscore: `_prop_idx`
- ... 3 more occurrences in this file

#### `src\storage\event_storage.rs`: 4 occurrences

- Line 165: unused variable: `old_vertex`: help: if this is intentional, prefix it with an underscore: `_old_vertex`
- Line 151: deref which would be done by auto-deref: help: try: `&vertex_id`
- Line 194: deref which would be done by auto-deref: help: try: `&vertex_id`
- ... 1 more occurrences in this file

#### `crates\vector-client\src\embedding\service.rs`: 3 occurrences

- Line 40: field `usage` is never read
- Line 51: fields `prompt_tokens` and `total_tokens` are never read
- Line 118: method `add_auth` is never used

#### `src\query\planning\plan\core\nodes\data_access\vector_search.rs`: 3 occurrences

- Line 34: this function has too many arguments (10/7)
- Line 82: this function has too many arguments (10/7)
- Line 180: this function has too many arguments (9/7)

#### `src\sync\vector_sync.rs`: 2 occurrences

- Line 331: useless use of `format!`: help: consider using `.to_string()`: `ctx.data.id.to_string()`
- Line 496: this function has too many arguments (8/7)

#### `crates\inversearch\src\config\validator.rs`: 1 occurrences

- Line 16: unused import: `std::fmt`

#### `src\sync\queue.rs`: 1 occurrences

- Line 126: field `shutdown_tx` is never read

#### `src\query\planning\statements\dml\insert_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::metadata::MetadataContext`

#### `src\query\executor\expression\functions\builtin\aggregate.rs`: 1 occurrences

- Line 379: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`

#### `src\query\executor\result_processing\agg_function_manager.rs`: 1 occurrences

- Line 469: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`

#### `src\api\core\query_api.rs`: 1 occurrences

- Line 22: field `vector_coordinator` is never read

#### `src\sync\batch.rs`: 1 occurrences

- Line 41: field `batch_size` is never read

