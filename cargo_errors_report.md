# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 5
- **Total Issues**: 5
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 5
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 5

### Warning Type Breakdown

- **warning**: 5 warnings

### Files with Warnings (Top 10)

- `src\api\core\transaction_api.rs`: 2 warnings
- `src\api\core\schema_api.rs`: 1 warnings
- `src\api\server\http\server.rs`: 1 warnings
- `src\api\embedded\embedded_api.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `CoreError`

**Total Occurrences**: 5  
**Unique Files**: 4

#### `src\api\core\transaction_api.rs`: 2 occurrences

- Line 5: unused import: `TransactionId`
- Line 6: unused import: `SavepointId`

#### `src\api\core\schema_api.rs`: 1 occurrences

- Line 6: unused import: `CoreError`

#### `src\api\server\http\server.rs`: 1 occurrences

- Line 18: fields `query_api`, `txn_api`, `schema_api`, `auth_service`, and `permission_manager` are never read

#### `src\api\embedded\embedded_api.rs`: 1 occurrences

- Line 41: fields `schema_api`, `storage`, and `txn_manager` are never read

