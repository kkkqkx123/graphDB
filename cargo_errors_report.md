# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 7
- **Total Warnings**: 0
- **Total Issues**: 7
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 0
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 7

### Error Type Breakdown

- **error[E0061]**: 4 errors
- **error[E0425]**: 1 errors
- **error[E0308]**: 1 errors
- **error[E0507]**: 1 errors

### Files with Errors (Top 10)

- `src\api\server\permission\permission_checker.rs`: 4 errors
- `src\query\planner\statements\insert_planner.rs`: 2 errors
- `src\query\parser\parser\tests.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0061]: this method takes 4 arguments but 3 arguments were supplied

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\api\server\permission\permission_checker.rs`: 4 occurrences

- Line 542: this method takes 4 arguments but 3 arguments were supplied
- Line 613: this method takes 4 arguments but 3 arguments were supplied
- Line 616: this method takes 4 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

### error[E0425]: cannot find value `ast` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 448: cannot find value `ast` in this scope: not found in this scope

### error[E0308]: mismatched types: expected `&Stmt`, found `&Arc<Ast>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 305: mismatched types: expected `&Stmt`, found `&Arc<Ast>`

### error[E0507]: cannot move out of an `Arc`: move occurs because value has type `stmt::Stmt`, which does not implement the `Copy` trait

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\parser\tests.rs`: 1 occurrences

- Line 11: cannot move out of an `Arc`: move occurs because value has type `stmt::Stmt`, which does not implement the `Copy` trait

