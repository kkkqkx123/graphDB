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

- `src\query\validator\fetch_vertices_validator.rs`: 1 warnings
- `src\query\validator\fetch_edges_validator.rs`: 1 warnings
- `src\query\validator\with_validator.rs`: 1 warnings

## Detailed Warning Categorization

### warning: function `create_fetch_vertices_stmt` is never used

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\validator\fetch_vertices_validator.rs`: 1 occurrences

- Line 342: function `create_fetch_vertices_stmt` is never used

#### `src\query\validator\fetch_edges_validator.rs`: 1 occurrences

- Line 379: function `create_fetch_edges_stmt` is never used

#### `src\query\validator\with_validator.rs`: 1 occurrences

- Line 364: unused variable: `where_expr`

