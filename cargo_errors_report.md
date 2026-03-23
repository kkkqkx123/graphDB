# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 20
- **Total Warnings**: 15
- **Total Issues**: 35
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 8
- **Files with Issues**: 14

## Error Statistics

**Total Errors**: 20

### Error Type Breakdown

- **error[E0433]**: 17 errors
- **error[E0425]**: 3 errors

### Files with Errors (Top 10)

- `src\query\planner\rewrite\pattern.rs`: 5 errors
- `src\query\planner\statements\clauses\yield_planner.rs`: 5 errors
- `src\storage\operations\rollback.rs`: 3 errors
- `src\query\planner\rewrite\aggregate\push_filter_down_aggregate.rs`: 3 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 1 errors
- `src\query\planner\statements\statement_planner.rs`: 1 errors
- `src\query\planner\rewrite\merge\combine_filter.rs`: 1 errors
- `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 15

### Warning Type Breakdown

- **warning**: 15 warnings

### Files with Warnings (Top 10)

- `src\query\validator\helpers\variable_checker.rs`: 3 warnings
- `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 3 warnings
- `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 2 warnings
- `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 2 warnings
- `src\storage\operations\rollback.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\operation\project_node.rs`: 2 warnings
- `src\query\validator\helpers\expression_checker.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

**Total Occurrences**: 17  
**Unique Files**: 7

#### `src\query\planner\statements\clauses\yield_planner.rs`: 5 occurrences

- Line 281: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 311: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 356: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 2 more occurrences in this file

#### `src\query\planner\rewrite\pattern.rs`: 5 occurrences

- Line 308: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 313: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 353: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 2 more occurrences in this file

#### `src\query\planner\rewrite\aggregate\push_filter_down_aggregate.rs`: 3 occurrences

- Line 383: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 421: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 451: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\merge\combine_filter.rs`: 1 occurrences

- Line 176: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 1 occurrences

- Line 243: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 1 occurrences

- Line 243: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\planner\statements\statement_planner.rs`: 1 occurrences

- Line 148: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

### error[E0425]: cannot find function `encode_to_vec` in this scope: not found in this scope

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\storage\operations\rollback.rs`: 3 occurrences

- Line 498: cannot find function `encode_to_vec` in this scope: not found in this scope
- Line 507: cannot find function `encode_to_vec` in this scope: not found in this scope
- Line 558: cannot find function `encode_to_vec` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused import: `crate::core::Expression`

**Total Occurrences**: 15  
**Unique Files**: 7

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 3 occurrences

- Line 3: unused import: `crate::core::Expression`
- Line 5: unused import: `crate::core::types::expression::ExpressionMeta`
- Line 6: unused import: `crate::core::YieldColumn`

#### `src\query\validator\helpers\variable_checker.rs`: 3 occurrences

- Line 7: unused import: `crate::core::types::expression::Expression`
- Line 8: unused import: `crate::core::types::operators::BinaryOperator`
- Line 9: unused import: `crate::core::Value`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 2 occurrences

- Line 5: unused import: `crate::core::types::expression::ExpressionMeta`
- Line 6: unused import: `crate::core::YieldColumn`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 2 occurrences

- Line 15: unused import: `crate::core::Expression`
- Line 17: unused import: `crate::core::types::expression::ExpressionMeta`

#### `src\query\planner\plan\core\nodes\operation\project_node.rs`: 2 occurrences

- Line 8: unused import: `crate::core::types::expression::Expression`
- Line 9: unused import: `crate::core::types::expression::ExpressionMeta`

#### `src\storage\operations\rollback.rs`: 2 occurrences

- Line 6: unused import: `Tag`
- Line 10: unused import: `std::collections::HashMap`

#### `src\query\validator\helpers\expression_checker.rs`: 1 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionMeta`

