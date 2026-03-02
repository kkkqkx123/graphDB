# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 1
- **Total Issues**: 5
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 1
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0609]**: 2 errors
- **error[E0308]**: 1 errors
- **error[E0004]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\utility\acl_validator.rs`: 3 errors
- `src\query\validator\statements\merge_validator.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\query\validator\clauses\group_by_validator.rs`: 1 warnings

## Detailed Error Categorization

### error[E0609]: no field `users` on type `&mut GrantValidator`: unknown field

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\utility\acl_validator.rs`: 2 occurrences

- Line 615: no field `users` on type `&mut GrantValidator`: unknown field
- Line 735: no field `users` on type `&mut RevokeValidator`: unknown field

### error[E0308]: mismatched types: expected `String`, found `Option<String>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\utility\acl_validator.rs`: 1 occurrences

- Line 487: mismatched types: expected `String`, found `Option<String>`

### error[E0004]: non-exhaustive patterns: `&ast::pattern::Pattern::Variable(_)` not covered: pattern `&ast::pattern::Pattern::Variable(_)` not covered

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\statements\merge_validator.rs`: 1 occurrences

- Line 455: non-exhaustive patterns: `&ast::pattern::Pattern::Variable(_)` not covered: pattern `&ast::pattern::Pattern::Variable(_)` not covered

## Detailed Warning Categorization

### warning: unused import: `crate::query::validator::structs::AliasType`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\clauses\group_by_validator.rs`: 1 occurrences

- Line 21: unused import: `crate::query::validator::structs::AliasType`

