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

- **error[E0592]**: 1 errors
- **error[E0433]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\strategies\type_inference.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0592]: duplicate definitions with name `are_types_compatible`: duplicate definitions for `are_types_compatible`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 492: duplicate definitions with name `are_types_compatible`: duplicate definitions for `are_types_compatible`

### error[E0433]: failed to resolve: use of undeclared type `TypeInference`: use of undeclared type `TypeInference`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 932: failed to resolve: use of undeclared type `TypeInference`: use of undeclared type `TypeInference`

