# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 8
- **Total Warnings**: 3
- **Total Issues**: 11
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 1
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 8

### Error Type Breakdown

- **error[E0422]**: 4 errors
- **error[E0599]**: 2 errors
- **error[E0412]**: 2 errors

### Files with Errors (Top 10)

- `src\query\validator\insert_vertices_validator.rs`: 3 errors
- `src\query\validator\insert_edges_validator.rs`: 3 errors
- `src\query\validator\match_validator.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\validator\fetch_vertices_validator.rs`: 1 warnings
- `src\query\validator\delete_validator.rs`: 1 warnings
- `src\query\validator\fetch_edges_validator.rs`: 1 warnings

## Detailed Error Categorization

### error[E0422]: cannot find struct, variant or union type `InsertStmt` in this scope: not found in this scope

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\validator\insert_edges_validator.rs`: 2 occurrences

- Line 381: cannot find struct, variant or union type `InsertStmt` in this scope: not found in this scope
- Line 637: cannot find struct, variant or union type `InsertStmt` in this scope: not found in this scope

#### `src\query\validator\insert_vertices_validator.rs`: 2 occurrences

- Line 334: cannot find struct, variant or union type `InsertStmt` in this scope: not found in this scope
- Line 565: cannot find struct, variant or union type `InsertStmt` in this scope: not found in this scope

### error[E0412]: cannot find type `InsertStmt` in this scope: not found in this scope

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 333: cannot find type `InsertStmt` in this scope: not found in this scope

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 380: cannot find type `InsertStmt` in this scope: not found in this scope

### error[E0599]: no method named `requires_space` found for struct `match_validator::MatchValidator` in the current scope: method not found in `MatchValidator`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\match_validator.rs`: 2 occurrences

- Line 911: no method named `requires_space` found for struct `match_validator::MatchValidator` in the current scope: method not found in `MatchValidator`
- Line 917: no method named `requires_write_permission` found for struct `match_validator::MatchValidator` in the current scope: method not found in `MatchValidator`

## Detailed Warning Categorization

### warning: variable does not need to be mutable

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 643: variable does not need to be mutable

#### `src\query\validator\fetch_edges_validator.rs`: 1 occurrences

- Line 460: variable does not need to be mutable

#### `src\query\validator\fetch_vertices_validator.rs`: 1 occurrences

- Line 405: variable does not need to be mutable

