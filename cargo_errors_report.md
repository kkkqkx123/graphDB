# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 0
- **Total Issues**: 1
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `src\api\server\http\handlers\statistics.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0599]: no variant or associated item named `internal_error` found for enum `HttpError` in the current scope: variant or associated item not found in `HttpError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\server\http\handlers\statistics.rs`: 1 occurrences

- Line 129: no variant or associated item named `internal_error` found for enum `HttpError` in the current scope: variant or associated item not found in `HttpError`

