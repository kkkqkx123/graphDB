# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 7
- **Total Issues**: 7
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 6
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 7

### Warning Type Breakdown

- **warning**: 7 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\deduce_type_visitor.rs`: 6 warnings
- `src\query\optimizer\fold_constant_expr_visitor.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused imports: `Edge` and `Vertex`

**Total Occurrences**: 7  
**Unique Files**: 2

#### `src\query\optimizer\deduce_type_visitor.rs`: 6 occurrences

- Line 15: unused imports: `Edge` and `Vertex`
- Line 17: unused import: `crate::core::EdgeDirection`
- Line 19: unused import: `crate::core::error::StorageError`
- ... 3 more occurrences in this file

#### `src\query\optimizer\fold_constant_expr_visitor.rs`: 1 occurrences

- Line 65: field `state` is never read

