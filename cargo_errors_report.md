# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 0
- **Total Issues**: 3
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0425]**: 2 errors
- **error[E0433]**: 1 errors

### Files with Errors (Top 10)

- `build.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0425]: cannot find type `PathBuf` in this scope: not found in this scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `build.rs`: 2 occurrences

- Line 53: cannot find type `PathBuf` in this scope: not found in this scope
- Line 74: cannot find type `PathBuf` in this scope: not found in this scope

### error[E0433]: failed to resolve: use of undeclared type `PathBuf`: use of undeclared type `PathBuf`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `build.rs`: 1 occurrences

- Line 54: failed to resolve: use of undeclared type `PathBuf`: use of undeclared type `PathBuf`

