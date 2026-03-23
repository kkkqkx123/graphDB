# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 91
- **Total Warnings**: 0
- **Total Issues**: 91
- **Unique Error Patterns**: 13
- **Unique Warning Patterns**: 0
- **Files with Issues**: 27

## Error Statistics

**Total Errors**: 91

### Error Type Breakdown

- **error[E0433]**: 68 errors
- **error[E0432]**: 12 errors
- **error[E0425]**: 11 errors

### Files with Errors (Top 10)

- `src\query\validator\helpers\variable_checker.rs`: 15 errors
- `src\storage\operations\rollback.rs`: 11 errors
- `src\query\planner\rewrite\aggregate\push_filter_down_aggregate.rs`: 6 errors
- `src\query\planner\statements\statement_planner.rs`: 5 errors
- `src\query\planner\rewrite\pattern.rs`: 5 errors
- `src\query\planner\statements\clauses\yield_planner.rs`: 5 errors
- `src\query\planner\statements\clauses\return_clause_planner.rs`: 4 errors
- `src\query\validator\strategies\helpers\expression_checker.rs`: 4 errors
- `src\query\validator\helpers\expression_checker.rs`: 4 errors
- `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

**Total Occurrences**: 68  
**Unique Files**: 19

#### `src\query\validator\helpers\variable_checker.rs`: 15 occurrences

- Line 334: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- Line 335: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 337: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- ... 12 more occurrences in this file

#### `src\query\planner\rewrite\aggregate\push_filter_down_aggregate.rs`: 6 occurrences

- Line 381: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 381: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 419: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\pattern.rs`: 5 occurrences

- Line 308: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 313: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 353: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 2 more occurrences in this file

#### `src\query\planner\statements\clauses\yield_planner.rs`: 5 occurrences

- Line 281: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 311: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 356: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 2 more occurrences in this file

#### `src\query\validator\helpers\expression_checker.rs`: 4 occurrences

- Line 581: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 598: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 611: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 4 occurrences

- Line 578: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- Line 595: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- Line 608: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\operation\project_node.rs`: 3 occurrences

- Line 98: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 127: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 132: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 3 occurrences

- Line 237: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 247: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 257: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 3 occurrences

- Line 193: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 198: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 219: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\storage\operations\rollback.rs`: 3 occurrences

- Line 489: failed to resolve: use of undeclared type `Vertex`: use of undeclared type `Vertex`
- Line 498: failed to resolve: use of undeclared type `Vertex`: use of undeclared type `Vertex`
- Line 549: failed to resolve: use of undeclared type `Vertex`: use of undeclared type `Vertex`

#### `src\query\planner\statements\statement_planner.rs`: 3 occurrences

- Line 148: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 149: failed to resolve: use of undeclared type `Ast`: use of undeclared type `Ast`
- Line 178: failed to resolve: use of undeclared type `ValidatedStatement`: use of undeclared type `ValidatedStatement`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 2 occurrences

- Line 241: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 241: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 2 occurrences

- Line 241: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 241: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\expression_analyzer.rs`: 2 occurrences

- Line 727: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 743: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`

#### `src\query\planner\rewrite\merge\combine_filter.rs`: 2 occurrences

- Line 174: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 174: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 338: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- Line 349: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`

#### `src\query\planner\statements\insert_planner.rs`: 2 occurrences

- Line 233: failed to resolve: use of undeclared type `Ast`: use of undeclared type `Ast`
- Line 442: failed to resolve: use of undeclared type `Ast`: use of undeclared type `Ast`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 168: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 163: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

### error[E0432]: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

**Total Occurrences**: 12  
**Unique Files**: 12

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 174: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\helpers\type_checker.rs`: 1 occurrences

- Line 625: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 132: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_expand_all.rs`: 1 occurrences

- Line 139: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 82: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\plan\core\nodes\insert\insert_nodes.rs`: 1 occurrences

- Line 141: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 76: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 156: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 98: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\visitor.rs`: 1 occurrences

- Line 458: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 181: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 151: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

### error[E0425]: cannot find type `Ast` in this scope: not found in this scope

**Total Occurrences**: 11  
**Unique Files**: 3

#### `src\storage\operations\rollback.rs`: 8 occurrences

- Line 391: cannot find type `Vertex` in this scope: not found in this scope
- Line 397: cannot find type `Vertex` in this scope: not found in this scope
- Line 412: cannot find type `Vertex` in this scope: not found in this scope
- ... 5 more occurrences in this file

#### `src\query\planner\statements\statement_planner.rs`: 2 occurrences

- Line 74: cannot find type `ValidatedStatement` in this scope: not found in this scope
- Line 137: cannot find type `Ast` in this scope: not found in this scope

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 226: cannot find type `Ast` in this scope: not found in this scope

