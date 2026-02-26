# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 5
- **Total Warnings**: 1
- **Total Issues**: 6
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 1
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 5

### Error Type Breakdown

- **error[E0599]**: 3 errors
- **error[E0433]**: 2 errors

### Files with Errors (Top 10)

- `src\api\server\session\network_session.rs`: 2 errors
- `src\core\error\storage.rs`: 2 errors
- `src\core\error\manager.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\api\server\session\network_session.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no variant or associated item named `LockError` found for enum `core::error::storage::StorageError` in the current scope: variant or associated item not found in `StorageError`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\core\error\storage.rs`: 2 occurrences

- Line 82: no variant or associated item named `LockError` found for enum `core::error::storage::StorageError` in the current scope: variant or associated item not found in `StorageError`
- Line 102: no variant or associated item named `TransactionError` found for enum `core::error::storage::StorageError` in the current scope: variant or associated item not found in `StorageError`

#### `src\core\error\manager.rs`: 1 occurrences

- Line 110: no variant or associated item named `PermissionError` found for enum `ManagerError` in the current scope: variant or associated item not found in `ManagerError`

### error[E0433]: failed to resolve: use of undeclared type `QueryError`: use of undeclared type `QueryError`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\api\server\session\network_session.rs`: 2 occurrences

- Line 249: failed to resolve: use of undeclared type `QueryError`: use of undeclared type `QueryError`
- Line 520: failed to resolve: use of undeclared type `QueryError`: use of undeclared type `QueryError`

## Detailed Warning Categorization

### warning: unused import: `SessionError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\server\session\network_session.rs`: 1 occurrences

- Line 7: unused import: `SessionError`

