# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0412]**: 1 errors
- **error[E0433]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\validator_enum.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0412]: cannot find type `ClearSpaceValidator` in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 187: cannot find type `ClearSpaceValidator` in this scope

### error[E0433]: failed to resolve: use of undeclared type `ClearSpaceValidator`: use of undeclared type `ClearSpaceValidator`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 648: failed to resolve: use of undeclared type `ClearSpaceValidator`: use of undeclared type `ClearSpaceValidator`

