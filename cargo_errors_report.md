# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 8
- **Total Warnings**: 0
- **Total Issues**: 8
- **Unique Error Patterns**: 6
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 8

### Error Type Breakdown

- **error**: 8 errors

### Files with Errors (Top 10)

- `src\api\core\schema_api.rs`: 7 errors
- `src\api\server\batch\manager.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error: invalid format string: expected `}` but string was terminated: expected `}` in format string

**Total Occurrences**: 8  
**Unique Files**: 2

#### `src\api\core\schema_api.rs`: 7 occurrences

- Line 359: expected `,`, found `space`: expected `,`
- Line 360: expected `,`, found `Type`: expected `,`
- Line 362: expected `,`, found `:`: expected `,`
- ... 4 more occurrences in this file

#### `src\api\server\batch\manager.rs`: 1 occurrences

- Line 145: invalid format string: expected `}` but string was terminated: expected `}` in format string

