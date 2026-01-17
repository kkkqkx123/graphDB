# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 12
- **Total Warnings**: 0
- **Total Issues**: 12
- **Unique Error Patterns**: 7
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 12

### Error Type Breakdown

- **error[E0061]**: 6 errors
- **error[E0609]**: 3 errors
- **error[E0369]**: 1 errors
- **error[E0599]**: 1 errors
- **error[E0507]**: 1 errors

### Files with Errors (Top 10)

- `src\api\service\index_service.rs`: 8 errors
- `src\index\storage.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0061]: this method takes 5 arguments but 4 arguments were supplied

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\api\service\index_service.rs`: 6 occurrences

- Line 687: this method takes 5 arguments but 4 arguments were supplied
- Line 711: this method takes 5 arguments but 4 arguments were supplied
- Line 728: this method takes 5 arguments but 4 arguments were supplied
- ... 3 more occurrences in this file

### error[E0609]: no field `query_count` on type `&index::stats::IndexQueryStats`: unknown field

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\index\storage.rs`: 3 occurrences

- Line 639: no field `query_count` on type `&index::stats::IndexQueryStats`: unknown field
- Line 640: no field `hit_count` on type `&index::stats::IndexQueryStats`: unknown field
- Line 641: no field `miss_count` on type `&index::stats::IndexQueryStats`: unknown field

### error[E0369]: binary operation `==` cannot be applied to type `std::result::Result<i32, IndexServiceError>`: std::result::Result<i32, IndexServiceError>, {integer}

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\service\index_service.rs`: 1 occurrences

- Line 694: binary operation `==` cannot be applied to type `std::result::Result<i32, IndexServiceError>`: std::result::Result<i32, IndexServiceError>, {integer}

### error[E0507]: cannot move out of a shared reference: move occurs because value has type `(i32, std::string::String, core::value::types::Value)`, which does not implement the `Copy` trait

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\service\index_service.rs`: 1 occurrences

- Line 401: cannot move out of a shared reference: move occurs because value has type `(i32, std::string::String, core::value::types::Value)`, which does not implement the `Copy` trait

### error[E0599]: no method named `hit_rate` found for reference `&index::stats::IndexQueryStats` in the current scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\index\storage.rs`: 1 occurrences

- Line 642: no method named `hit_rate` found for reference `&index::stats::IndexQueryStats` in the current scope

