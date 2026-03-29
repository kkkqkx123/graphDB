# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 16
- **Total Warnings**: 3
- **Total Issues**: 19
- **Unique Error Patterns**: 10
- **Unique Warning Patterns**: 3
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 16

### Error Type Breakdown

- **error[E0599]**: 14 errors
- **error[E0609]**: 2 errors

### Files with Errors (Top 10)

- `tests\integration_embedded_api.rs`: 16 errors

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\api\embedded\batch.rs`: 1 warnings
- `src\api\embedded\transaction.rs`: 1 warnings
- `src\api\embedded\c_api\batch.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `batch_size` found for struct `graphdb::api::embedded::BatchInserter<'sess, S>` in the current scope: method not found in `graphdb::api::embedded::BatchInserter<'_, graphdb::storage::RedbStorage>`

**Total Occurrences**: 14  
**Unique Files**: 1

#### `tests\integration_embedded_api.rs`: 14 occurrences

- Line 253: no method named `batch_size` found for struct `graphdb::api::embedded::BatchInserter<'sess, S>` in the current scope: method not found in `graphdb::api::embedded::BatchInserter<'_, graphdb::storage::RedbStorage>`
- Line 431: no method named `total_inserted` found for struct `graphdb::api::embedded::BatchResult` in the current scope: method not found in `graphdb::api::embedded::BatchResult`
- Line 439: no function or associated item named `new` found for struct `graphdb::api::embedded::BatchError` in the current scope: function or associated item not found in `graphdb::api::embedded::BatchError`
- ... 11 more occurrences in this file

### error[E0609]: no field `auto_commit` on type `graphdb::api::core::BatchConfig`: unknown field

**Total Occurrences**: 2  
**Unique Files**: 1

#### `tests\integration_embedded_api.rs`: 2 occurrences

- Line 470: no field `auto_commit` on type `graphdb::api::core::BatchConfig`: unknown field
- Line 472: no field `max_errors` on type `graphdb::api::core::BatchConfig`: unknown field

## Detailed Warning Categorization

### warning: unused import: `crate::core::Value`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\api\embedded\batch.rs`: 1 occurrences

- Line 238: unused import: `crate::core::Value`

#### `src\api\embedded\transaction.rs`: 1 occurrences

- Line 449: methods `commit_ref` and `rollback_ref` are never used

#### `src\api\embedded\c_api\batch.rs`: 1 occurrences

- Line 37: field `last_error` is never read

