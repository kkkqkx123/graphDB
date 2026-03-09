# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 81
- **Total Warnings**: 11
- **Total Issues**: 92
- **Unique Error Patterns**: 35
- **Unique Warning Patterns**: 6
- **Files with Issues**: 11

## Error Statistics

**Total Errors**: 81

### Error Type Breakdown

- **error[E0310]**: 46 errors
- **error[E0599]**: 33 errors
- **error[E0503]**: 2 errors

### Files with Errors (Top 10)

- `src\query\executor\statement_executors\ddl_executor.rs`: 23 errors
- `src\query\executor\statement_executors\system_executor.rs`: 16 errors
- `src\query\executor\statement_executors\dml_executor.rs`: 12 errors
- `src\query\executor\statement_executors\cypher_executor.rs`: 10 errors
- `src\query\executor\statement_executors\query_executor.rs`: 10 errors
- `src\query\executor\statement_executors\user_executor.rs`: 8 errors
- `src\query\executor\graph_query_executor.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 11

### Warning Type Breakdown

- **warning**: 11 warnings

### Files with Warnings (Top 10)

- `src\api\embedded\c_api\database.rs`: 2 warnings
- `src\query\executor\statement_executors\user_executor.rs`: 1 warnings
- `src\query\executor\statement_executors\cypher_executor.rs`: 1 warnings
- `src\storage\user_storage.rs`: 1 warnings
- `src\storage\redb_storage.rs`: 1 warnings
- `src\query\executor\statement_executors\ddl_executor.rs`: 1 warnings
- `src\query\executor\statement_executors\query_executor.rs`: 1 warnings
- `src\query\executor\statement_executors\system_executor.rs`: 1 warnings
- `src\query\executor\statement_executors\dml_executor.rs`: 1 warnings
- `src\storage\vertex_storage.rs`: 1 warnings

## Detailed Error Categorization

### error[E0310]: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds

**Total Occurrences**: 46  
**Unique Files**: 4

#### `src\query\executor\statement_executors\ddl_executor.rs`: 22 occurrences

- Line 236: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 237: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 251: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- ... 19 more occurrences in this file

#### `src\query\executor\statement_executors\dml_executor.rs`: 10 occurrences

- Line 99: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 100: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 170: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- ... 7 more occurrences in this file

#### `src\query\executor\statement_executors\query_executor.rs`: 8 occurrences

- Line 138: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 147: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 148: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- ... 5 more occurrences in this file

#### `src\query\executor\statement_executors\cypher_executor.rs`: 6 occurrences

- Line 37: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 46: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- Line 47: the parameter type `S` may not live long enough: the parameter type `S` must be valid for the static lifetime..., ...so that the type `S` will meet its required lifetime bounds
- ... 3 more occurrences in this file

### error[E0599]: no method named `open` found for struct `create_user::CreateUserExecutor` in the current scope

**Total Occurrences**: 33  
**Unique Files**: 6

#### `src\query\executor\statement_executors\system_executor.rs`: 16 occurrences

- Line 30: no method named `open` found for struct `switch_space::SwitchSpaceExecutor` in the current scope
- Line 31: no method named `execute` found for struct `switch_space::SwitchSpaceExecutor` in the current scope: method not found in `SwitchSpaceExecutor<S>`
- Line 58: no method named `open` found for struct `ShowSpacesExecutor` in the current scope
- ... 13 more occurrences in this file

#### `src\query\executor\statement_executors\user_executor.rs`: 8 occurrences

- Line 31: no method named `open` found for struct `create_user::CreateUserExecutor` in the current scope
- Line 33: no method named `execute` found for struct `create_user::CreateUserExecutor` in the current scope: method not found in `CreateUserExecutor<S>`
- Line 53: no method named `open` found for struct `alter_user::AlterUserExecutor` in the current scope
- ... 5 more occurrences in this file

#### `src\query\executor\statement_executors\cypher_executor.rs`: 4 occurrences

- Line 65: no method named `transform` found for struct `ReturnPlanner` in the current scope: method not found in `ReturnPlanner`
- Line 109: no method named `transform` found for struct `WithPlanner` in the current scope: method not found in `WithPlanner`
- Line 153: no method named `transform` found for struct `YieldPlanner` in the current scope: method not found in `YieldPlanner`
- ... 1 more occurrences in this file

#### `src\query\executor\statement_executors\query_executor.rs`: 2 occurrences

- Line 46: no method named `transform` found for struct `match_statement_planner::MatchStatementPlanner` in the current scope: method not found in `MatchStatementPlanner`
- Line 89: no method named `transform` found for struct `GoPlanner` in the current scope: method not found in `GoPlanner`

#### `src\query\executor\statement_executors\dml_executor.rs`: 2 occurrences

- Line 42: no method named `transform` found for struct `insert_planner::InsertPlanner` in the current scope: method not found in `InsertPlanner`
- Line 391: no method named `transform` found for struct `MergePlanner` in the current scope: method not found in `MergePlanner`

#### `src\query\executor\statement_executors\ddl_executor.rs`: 1 occurrences

- Line 187: no method named `transform` found for struct `create_planner::CreatePlanner` in the current scope: method not found in `CreatePlanner`

### error[E0503]: cannot use `self.id` because it was mutably borrowed: use of borrowed `*self`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 108: cannot use `self.id` because it was mutably borrowed: use of borrowed `*self`
- Line 134: cannot use `self.id` because it was mutably borrowed: use of borrowed `*self`

## Detailed Warning Categorization

### warning: unused import: `crate::api::core::CoreError`

**Total Occurrences**: 11  
**Unique Files**: 10

#### `src\api\embedded\c_api\database.rs`: 2 occurrences

- Line 5: unused import: `crate::api::core::CoreError`
- Line 11: unused import: `StorageClient`

#### `src\storage\redb_storage.rs`: 1 occurrences

- Line 10: unused import: `ExtendedSchemaManager`

#### `src\query\executor\statement_executors\system_executor.rs`: 1 occurrences

- Line 5: unused import: `DBResult`

#### `src\query\executor\statement_executors\user_executor.rs`: 1 occurrences

- Line 5: unused import: `DBResult`

#### `src\storage\user_storage.rs`: 1 occurrences

- Line 147: unused import: `crate::core::types::PasswordInfo`

#### `src\storage\vertex_storage.rs`: 1 occurrences

- Line 1: unused import: `PropertyDef`

#### `src\query\executor\statement_executors\query_executor.rs`: 1 occurrences

- Line 5: unused import: `DBResult`

#### `src\query\executor\statement_executors\ddl_executor.rs`: 1 occurrences

- Line 5: unused import: `DBResult`

#### `src\query\executor\statement_executors\cypher_executor.rs`: 1 occurrences

- Line 5: unused import: `DBResult`

#### `src\query\executor\statement_executors\dml_executor.rs`: 1 occurrences

- Line 5: unused import: `DBResult`

