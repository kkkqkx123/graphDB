# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 67
- **Total Warnings**: 12
- **Total Issues**: 79
- **Unique Error Patterns**: 9
- **Unique Warning Patterns**: 6
- **Files with Issues**: 11

## Error Statistics

**Total Errors**: 67

### Error Type Breakdown

- **error[E0560]**: 63 errors
- **error[E0432]**: 3 errors
- **error[E0063]**: 1 errors

### Files with Errors (Top 10)

- `tests\integration_logging.rs`: 63 errors
- `tests\sync_fault_tolerance.rs`: 3 errors
- `tests\integration_api.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 12

### Warning Type Breakdown

- **warning**: 12 warnings

### Files with Warnings (Top 10)

- `src\core\stats\slow_query_logger.rs`: 2 warnings
- `src\config\common\storage.rs`: 2 warnings
- `src\config\server\security.rs`: 2 warnings
- `src\core\stats\aggregated_stats.rs`: 2 warnings
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 warnings
- `src\query\executor\result_processing\transformations\pattern_apply.rs`: 1 warnings
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 warnings
- `src\query\executor\result_processing\transformations\unwind.rs`: 1 warnings

## Detailed Error Categorization

### error[E0560]: struct `graphdb::config::Config` has no field named `database`: `graphdb::config::Config` does not have this field

**Total Occurrences**: 63  
**Unique Files**: 1

#### `tests\integration_logging.rs`: 63 occurrences

- Line 35: struct `graphdb::config::Config` has no field named `database`: `graphdb::config::Config` does not have this field
- Line 41: struct `graphdb::config::Config` has no field named `transaction`: `graphdb::config::Config` does not have this field
- Line 45: struct `graphdb::config::Config` has no field named `log`: `graphdb::config::Config` does not have this field
- ... 60 more occurrences in this file

### error[E0432]: unresolved import `graphdb::sync::compensation`: could not find `compensation` in `sync`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `tests\sync_fault_tolerance.rs`: 3 occurrences

- Line 169: unresolved import `graphdb::sync::compensation`: could not find `compensation` in `sync`
- Line 186: unresolved import `graphdb::sync::compensation`: could not find `compensation` in `sync`
- Line 223: unresolved import `graphdb::sync::compensation`: could not find `compensation` in `sync`

### error[E0063]: missing field `query_resource` in initializer of `graphdb::config::CommonConfig`: missing `query_resource`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `tests\integration_api.rs`: 1 occurrences

- Line 684: missing field `query_resource` in initializer of `graphdb::config::CommonConfig`: missing `query_resource`

## Detailed Warning Categorization

### warning: this `impl` can be derived

**Total Occurrences**: 12  
**Unique Files**: 8

#### `src\config\common\storage.rs`: 2 occurrences

- Line 16: this `impl` can be derived
- Line 45: this `impl` can be derived

#### `src\core\stats\slow_query_logger.rs`: 2 occurrences

- Line 60: fields `current_file_size` and `current_file_path` are never read
- Line 371: this loop could be written as a `while let` loop: help: try: `while let Ok(log_entry) = rx.recv() { .. }`

#### `src\core\stats\aggregated_stats.rs`: 2 occurrences

- Line 287: casting to the same type is unnecessary (`f64` -> `f64`): help: try: `self.avg_duration_us`
- Line 465: manual implementation of `.is_multiple_of()`: help: replace with: `query_num.is_multiple_of((1.0 / sample_rate) as u64)`

#### `src\config\server\security.rs`: 2 occurrences

- Line 20: this `impl` can be derived
- Line 182: this `impl` can be derived

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 12: unused import: `Executor`

#### `src\query\executor\result_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 14: unused import: `Executor`

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 10: unused import: `Executor`

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 13: unused import: `Executor`

