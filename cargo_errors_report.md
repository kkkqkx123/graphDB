# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 0
- **Total Issues**: 3
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0308]**: 3 errors

### Files with Errors (Top 10)

- `src\query\optimizer\core\selectivity.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `f64`, found `Option<{float}>`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\optimizer\core\selectivity.rs`: 3 occurrences

- Line 405: mismatched types: expected `f64`, found `Option<{float}>`
- Line 409: mismatched types: expected `f64`, found `Option<{float}>`
- Line 413: mismatched types: expected `f64`, found `Option<{float}>`

