# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 24
- **Total Warnings**: 0
- **Total Issues**: 24
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 0
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 24

### Error Type Breakdown

- **error[E0308]**: 21 errors
- **error[E0277]**: 3 errors

### Files with Errors (Top 10)

- `src\api\embedded\c_api\statement.rs`: 8 errors
- `src\api\embedded\c_api\transaction.rs`: 5 errors
- `src\api\embedded\c_api\database.rs`: 4 errors
- `src\api\embedded\result.rs`: 3 errors
- `src\api\embedded\c_api\query.rs`: 2 errors
- `src\api\embedded\c_api\session.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`

**Total Occurrences**: 21  
**Unique Files**: 5

#### `src\api\embedded\c_api\statement.rs`: 8 occurrences

- Line 64: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 96: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 133: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- ... 5 more occurrences in this file

#### `src\api\embedded\c_api\transaction.rs`: 5 occurrences

- Line 94: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 141: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 214: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- ... 2 more occurrences in this file

#### `src\api\embedded\c_api\database.rs`: 4 occurrences

- Line 65: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 146: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 268: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- ... 1 more occurrences in this file

#### `src\api\embedded\c_api\session.rs`: 2 occurrences

- Line 151: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 217: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`

#### `src\api\embedded\c_api\query.rs`: 2 occurrences

- Line 72: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`
- Line 148: mismatched types: expected `i32`, found `(i32, graphdb_extended_error_code_t)`

### error[E0277]: the trait bound `CoreError: serde::Serialize` is not satisfied: the trait `Serialize` is not implemented for `CoreError`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\api\embedded\result.rs`: 3 occurrences

- Line 14: the trait bound `CoreError: serde::Serialize` is not satisfied: the trait `Serialize` is not implemented for `CoreError`
- Line 19: the trait bound `CoreError: serde::Deserialize<'de>` is not satisfied: the trait `Deserialize<'_>` is not implemented for `CoreError`
- Line 14: the trait bound `CoreError: serde::Deserialize<'de>` is not satisfied: the trait `Deserialize<'_>` is not implemented for `CoreError`

