# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 9
- **Total Issues**: 9
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 8
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 9

### Warning Type Breakdown

- **warning**: 9 warnings

### Files with Warnings (Top 10)

- `src\api\embedded\c_api\statement.rs`: 3 warnings
- `src\api\embedded\c_api\batch.rs`: 2 warnings
- `src\api\embedded\c_api\query.rs`: 1 warnings
- `src\api\embedded\c_api\transaction.rs`: 1 warnings
- `src\api\embedded\session.rs`: 1 warnings
- `src\api\embedded\c_api\result.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::api::embedded::c_api::result::GraphDbResultHandle`

**Total Occurrences**: 9  
**Unique Files**: 6

#### `src\api\embedded\c_api\statement.rs`: 3 occurrences

- Line 6: unused import: `crate::api::embedded::c_api::result::GraphDbResultHandle`
- Line 8: unused import: `graphdb_result_t`
- Line 12: unused import: `std::collections::HashMap`

#### `src\api\embedded\c_api\batch.rs`: 2 occurrences

- Line 11: unused import: `c_void`
- Line 84: unused variable: `tag_str`

#### `src\api\embedded\c_api\query.rs`: 1 occurrences

- Line 11: unused import: `c_void`

#### `src\api\embedded\session.rs`: 1 occurrences

- Line 110: method `inner` is never used

#### `src\api\embedded\c_api\result.rs`: 1 occurrences

- Line 14: field `current_row` is never read

#### `src\api\embedded\c_api\transaction.rs`: 1 occurrences

- Line 369: unused variable: `txn_handle`

