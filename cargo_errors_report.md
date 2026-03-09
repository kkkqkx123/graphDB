# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 10
- **Total Issues**: 10
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 10
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 10

### Warning Type Breakdown

- **warning**: 10 warnings

### Files with Warnings (Top 10)

- `src\query\executor\graph_query_executor.rs`: 3 warnings
- `src\storage\redb_storage.rs`: 2 warnings
- `src\api\embedded\c_api\database.rs`: 2 warnings
- `src\query\executor\statement_executors\ddl_executor.rs`: 1 warnings
- `src\storage\user_storage.rs`: 1 warnings
- `src\storage\vertex_storage.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `PropertyChange`

**Total Occurrences**: 10  
**Unique Files**: 6

#### `src\query\executor\graph_query_executor.rs`: 3 occurrences

- Line 8: unused import: `crate::query::executor::admin as admin_executor`
- Line 12: unused imports: `AlterStmt`, `DescStmt`, and `DropStmt`
- Line 250: methods `execute_delete`, `execute_update`, `execute_insert`, and `execute_merge` are never used

#### `src\api\embedded\c_api\database.rs`: 2 occurrences

- Line 5: unused import: `crate::api::core::CoreError`
- Line 11: unused import: `StorageClient`

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 10: unused import: `ExtendedSchemaManager`
- Line 35: field `users` is never read

#### `src\query\executor\statement_executors\ddl_executor.rs`: 1 occurrences

- Line 17: unused import: `PropertyChange`

#### `src\storage\vertex_storage.rs`: 1 occurrences

- Line 1: unused import: `PropertyDef`

#### `src\storage\user_storage.rs`: 1 occurrences

- Line 147: unused import: `crate::core::types::PasswordInfo`

