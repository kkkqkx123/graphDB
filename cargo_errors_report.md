# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 2
- **Total Issues**: 4
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 2
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0425]**: 2 errors

### Files with Errors (Top 10)

- `src\query\optimizer\cost\node_estimators\graph_traversal.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 2

### Warning Type Breakdown

- **warning**: 2 warnings

### Files with Warnings (Top 10)

- `src\api\embedded\c_api\database.rs`: 2 warnings

## Detailed Error Categorization

### error[E0425]: cannot find value `cost` in this scope: not found in this scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\cost\node_estimators\graph_traversal.rs`: 2 occurrences

- Line 400: cannot find value `cost` in this scope: not found in this scope
- Line 401: cannot find value `output_rows` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused import: `crate::api::core::CoreError`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\api\embedded\c_api\database.rs`: 2 occurrences

- Line 5: unused import: `crate::api::core::CoreError`
- Line 11: unused import: `StorageClient`

