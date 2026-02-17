# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 6
- **Total Issues**: 6
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 6
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `src\query\validator\insert_vertices_validator.rs`: 3 warnings
- `src\query\executor\graph_query_executor.rs`: 1 warnings
- `src\storage\operations\redb_operations.rs`: 1 warnings
- `src\query\planner\statements\insert_planner.rs`: 1 warnings

## Detailed Warning Categorization

### warning: value assigned to `deleted_count` is never read

**Total Occurrences**: 6  
**Unique Files**: 4

#### `src\query\validator\insert_vertices_validator.rs`: 3 occurrences

- Line 433: unused import: `crate::core::Value`
- Line 393: unused variable: `prop_idx`: help: if this is intentional, prefix it with an underscore: `_prop_idx`
- Line 438: struct `MockSchemaManager` is never constructed

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 441: value assigned to `deleted_count` is never read

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 6: unused import: `TagInsertSpec`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 323: unused variable: `is_all_tags`: help: try ignoring the field: `is_all_tags: _`

