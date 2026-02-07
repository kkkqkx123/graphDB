# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 20
- **Total Warnings**: 5
- **Total Issues**: 25
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 5
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 20

### Error Type Breakdown

- **error[E0433]**: 18 errors
- **error[E0412]**: 1 errors
- **error[E0405]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\result_processing\projection.rs`: 19 errors
- `src\query\executor\graph_query_executor.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 5

### Warning Type Breakdown

- **warning**: 5 warnings

### Files with Warnings (Top 10)

- `src\query\executor\graph_query_executor.rs`: 2 warnings
- `src\query\executor\result_processing\projection.rs`: 1 warnings
- `src\query\validator\insert_vertices_validator.rs`: 1 warnings
- `src\common\memory.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `UserInfo`: use of undeclared type `UserInfo`

**Total Occurrences**: 18  
**Unique Files**: 2

#### `src\query\executor\result_processing\projection.rs`: 17 occurrences

- Line 209: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 213: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- Line 215: failed to resolve: use of undeclared type `ExecutionResult`: use of undeclared type `ExecutionResult`
- ... 14 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 341: failed to resolve: use of undeclared type `UserInfo`: use of undeclared type `UserInfo`

### error[E0405]: cannot find trait `Executor` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 204: cannot find trait `Executor` in this scope: not found in this scope

### error[E0412]: cannot find type `ExecutionResult` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 205: cannot find type `ExecutionResult` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused doc comment: rustdoc does not generate documentation for macro invocations

**Total Occurrences**: 5  
**Unique Files**: 4

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 373: unused import: `PasswordInfo`
- Line 142: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 320: unused imports: `ExecutionResult` and `Executor`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 204: unused import: `crate::core::Value`

