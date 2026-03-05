# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 35
- **Total Issues**: 35
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 34
- **Files with Issues**: 48

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 35

### Warning Type Breakdown

- **warning**: 35 warnings

### Files with Warnings (Top 10)

- `src\api\embedded\c_api\error.rs`: 22 warnings
- `src\api\embedded\c_api\types.rs`: 10 warnings
- `src\api\embedded\c_api\session.rs`: 2 warnings
- `src\api\embedded\c_api\result.rs`: 1 warnings

## Detailed Warning Categorization

### warning: variant `GRAPHDB_NULL` should have an upper camel case name

**Total Occurrences**: 35  
**Unique Files**: 4

#### `src\api\embedded\c_api\error.rs`: 22 occurrences

- Line 12: variant `GRAPHDB_OK` should have an upper camel case name
- Line 14: variant `GRAPHDB_ERROR` should have an upper camel case name
- Line 16: variant `GRAPHDB_INTERNAL` should have an upper camel case name
- ... 19 more occurrences in this file

#### `src\api\embedded\c_api\types.rs`: 10 occurrences

- Line 12: variant `GRAPHDB_NULL` should have an upper camel case name
- Line 14: variant `GRAPHDB_BOOL` should have an upper camel case name
- Line 16: variant `GRAPHDB_INT` should have an upper camel case name
- ... 7 more occurrences in this file

#### `src\api\embedded\c_api\session.rs`: 2 occurrences

- Line 224: variable does not need to be mutable
- Line 243: variable does not need to be mutable

#### `src\api\embedded\c_api\result.rs`: 1 occurrences

- Line 14: field `current_row` is never read

