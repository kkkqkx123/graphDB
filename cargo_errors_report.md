# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 3
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\api\server\permission\permission_checker.rs`: 1 warnings
- `src\api\server\http\server.rs`: 1 warnings
- `src\api\embedded\embedded_api.rs`: 1 warnings

## Detailed Warning Categorization

### warning: fields `schema_api`, `storage`, and `txn_manager` are never read

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\api\embedded\embedded_api.rs`: 1 occurrences

- Line 41: fields `schema_api`, `storage`, and `txn_manager` are never read

#### `src\api\server\permission\permission_checker.rs`: 1 occurrences

- Line 222: unused variable: `session`: help: if this is intentional, prefix it with an underscore: `_session`

#### `src\api\server\http\server.rs`: 1 occurrences

- Line 18: fields `query_api`, `txn_api`, `schema_api`, `auth_service`, and `permission_manager` are never read

