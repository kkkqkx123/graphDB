# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 3
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\expression\context\row_context.rs`: 1 warnings
- `src\expression\context\default_context.rs`: 1 warnings
- `src\expression\functions\registry.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `super::ExpressionFunction`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\expression\functions\registry.rs`: 1 occurrences

- Line 12: unused import: `super::ExpressionFunction`

#### `src\expression\context\default_context.rs`: 1 occurrences

- Line 31: field `paths` is never read

#### `src\expression\context\row_context.rs`: 1 occurrences

- Line 20: field `col_names` is never read

