# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 12
- **Total Issues**: 12
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 12
- **Files with Issues**: 8

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 12

### Warning Type Breakdown

- **warning**: 12 warnings

### Files with Warnings (Top 10)

- `src\query\executor\data_processing\graph_traversal\algorithms\bfs_shortest.rs`: 3 warnings
- `src\api\embedded\c_api\database.rs`: 2 warnings
- `src\storage\redb_storage.rs`: 2 warnings
- `src\query\executor\graph_query_executor.rs`: 1 warnings
- `src\query\executor\statement_executors\ddl_executor.rs`: 1 warnings
- `src\storage\vertex_storage.rs`: 1 warnings
- `src\storage\user_storage.rs`: 1 warnings
- `src\query\executor\statement_executors\query_executor.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `PropertyDef`

**Total Occurrences**: 12  
**Unique Files**: 8

#### `src\query\executor\data_processing\graph_traversal\algorithms\bfs_shortest.rs`: 3 occurrences

- Line 9: unused import: `crate::core::error::DBError`
- Line 10: unused import: `NullType`
- Line 13: unused import: `crate::query::executor::expression::evaluator::traits::ExpressionContext`

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 10: unused import: `ExtendedSchemaManager`
- Line 35: field `users` is never read

#### `src\api\embedded\c_api\database.rs`: 2 occurrences

- Line 5: unused import: `crate::api::core::CoreError`
- Line 11: unused import: `StorageClient`

#### `src\storage\vertex_storage.rs`: 1 occurrences

- Line 1: unused import: `PropertyDef`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 89: methods `execute_statement`, `execute_assignment`, `execute_set_operation`, and `execute_subgraph` are never used

#### `src\query\executor\statement_executors\ddl_executor.rs`: 1 occurrences

- Line 17: unused import: `PropertyChange`

#### `src\query\executor\statement_executors\query_executor.rs`: 1 occurrences

- Line 326: unused variable: `clause`: help: if this is intentional, prefix it with an underscore: `_clause`

#### `src\storage\user_storage.rs`: 1 occurrences

- Line 147: unused import: `crate::core::types::PasswordInfo`

