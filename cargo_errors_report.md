# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 6
- **Total Warnings**: 0
- **Total Issues**: 6
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 0
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 6

### Error Type Breakdown

- **error[E0433]**: 4 errors
- **error[E0061]**: 2 errors

### Files with Errors (Top 10)

- `src\query\planner\rewrite\pattern.rs`: 3 errors
- `src\query\planner\statements\match_statement_planner.rs`: 2 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\planner\rewrite\pattern.rs`: 3 occurrences

- Line 310: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 354: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 364: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 172: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

### error[E0061]: this method takes 4 arguments but 3 arguments were supplied

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 289: this method takes 4 arguments but 3 arguments were supplied
- Line 770: this method takes 3 arguments but 2 arguments were supplied

