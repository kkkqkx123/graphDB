# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0308]**: 2 errors

### Files with Errors (Top 10)

- `src\query\optimizer\subquery_optimization.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `Box<Expression>`, found `Expression`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\subquery_optimization.rs`: 2 occurrences

- Line 386: mismatched types: expected `Box<Expression>`, found `Expression`
- Line 387: mismatched types: expected `Box<Expression>`, found `Expression`

