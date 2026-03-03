# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 1
- **Total Issues**: 5
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 1
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0433]**: 3 errors
- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\statements\clauses\return_clause_planner.rs`: 3 errors
- `src\query\planner\statements\seeks\prop_index_seek.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_traverse.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 3 occurrences

- Line 284: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 294: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 304: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

### error[E0599]: no method named `visit` found for struct `OrConditionCollector` in the current scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\seeks\prop_index_seek.rs`: 1 occurrences

- Line 99: no method named `visit` found for struct `OrConditionCollector` in the current scope

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::ContextualExpression`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_traverse.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::ContextualExpression`

