# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 9
- **Total Warnings**: 1
- **Total Issues**: 10
- **Unique Error Patterns**: 5
- **Unique Warning Patterns**: 1
- **Files with Issues**: 5

## Error Statistics

**Total Errors**: 9

### Error Type Breakdown

- **error[E0061]**: 4 errors
- **error[E0599]**: 3 errors
- **error[E0433]**: 2 errors

### Files with Errors (Top 10)

- `src\query\planner\statements\seeks\edge_seek.rs`: 4 errors
- `src\query\planner\statements\seeks\prop_index_seek.rs`: 2 errors
- `src\query\planner\statements\seeks\variable_prop_index_seek.rs`: 2 errors
- `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\storage\operations\redb_operations.rs`: 1 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 5 arguments but 3 arguments were supplied

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\query\planner\statements\seeks\edge_seek.rs`: 4 occurrences

- Line 213: this function takes 5 arguments but 3 arguments were supplied
- Line 221: this function takes 5 arguments but 3 arguments were supplied
- Line 239: this function takes 5 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

### error[E0599]: no variant or associated item named `Parameter` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\planner\statements\seeks\variable_prop_index_seek.rs`: 1 occurrences

- Line 331: no variant or associated item named `Parameter` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 605: no function or associated item named `new` found for struct `all_paths::SelfLoopDedup` in the current scope: function or associated item not found in `SelfLoopDedup`

#### `src\query\planner\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 306: no variant or associated item named `Constant` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`

### error[E0433]: failed to resolve: could not find `BinaryOp` in `core`: could not find `BinaryOp` in `core`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\planner\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 301: failed to resolve: could not find `BinaryOp` in `core`: could not find `BinaryOp` in `core`

#### `src\query\planner\statements\seeks\variable_prop_index_seek.rs`: 1 occurrences

- Line 326: failed to resolve: could not find `BinaryOp` in `core`: could not find `BinaryOp` in `core`

## Detailed Warning Categorization

### warning: value assigned to `deleted_count` is never read

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 441: value assigned to `deleted_count` is never read

