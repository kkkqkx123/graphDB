# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 134
- **Total Warnings**: 0
- **Total Issues**: 134
- **Unique Error Patterns**: 10
- **Unique Warning Patterns**: 0
- **Files with Issues**: 31

## Error Statistics

**Total Errors**: 134

### Error Type Breakdown

- **error[E0433]**: 119 errors
- **error[E0432]**: 14 errors
- **error[E0425]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\statements\insert_edges_validator.rs`: 35 errors
- `src\query\validator\clauses\limit_validator.rs`: 18 errors
- `src\api\embedded\c_api\batch.rs`: 8 errors
- `src\query\validator\strategies\helpers\expression_checker.rs`: 8 errors
- `src\query\validator\helpers\expression_checker.rs`: 8 errors
- `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 7 errors
- `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 6 errors
- `src\query\optimizer\analysis\expression.rs`: 6 errors
- `src\query\optimizer\strategy\materialization.rs`: 4 errors
- `src\query\validator\utility\update_config_validator.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `CString`: use of undeclared type `CString`

**Total Occurrences**: 119  
**Unique Files**: 22

#### `src\query\validator\statements\insert_edges_validator.rs`: 33 occurrences

- Line 486: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 487: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 489: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- ... 30 more occurrences in this file

#### `src\query\validator\clauses\limit_validator.rs`: 18 occurrences

- Line 332: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 338: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 339: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- ... 15 more occurrences in this file

#### `src\query\validator\helpers\expression_checker.rs`: 8 occurrences

- Line 571: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- Line 572: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 588: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- ... 5 more occurrences in this file

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 8 occurrences

- Line 572: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 574: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 589: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 5 more occurrences in this file

#### `src\api\embedded\c_api\batch.rs`: 8 occurrences

- Line 497: failed to resolve: use of unresolved module or unlinked crate `ptr`: use of unresolved module or unlinked crate `ptr`
- Line 510: failed to resolve: use of unresolved module or unlinked crate `ptr`: use of unresolved module or unlinked crate `ptr`
- Line 510: failed to resolve: use of unresolved module or unlinked crate `ptr`: use of unresolved module or unlinked crate `ptr`
- ... 5 more occurrences in this file

#### `src\query\optimizer\analysis\expression.rs`: 6 occurrences

- Line 562: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 575: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 590: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 6 occurrences

- Line 356: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 392: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 522: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 3 more occurrences in this file

#### `src\query\optimizer\strategy\materialization.rs`: 4 occurrences

- Line 505: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 506: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 514: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 4 occurrences

- Line 239: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 244: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 298: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\alias_strategy.rs`: 4 occurrences

- Line 336: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 338: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 347: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- ... 1 more occurrences in this file

#### `src\query\validator\utility\update_config_validator.rs`: 3 occurrences

- Line 265: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- Line 270: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`
- Line 276: failed to resolve: use of undeclared type `ExpressionMeta`: use of undeclared type `ExpressionMeta`

#### `src\api\embedded\c_api\query.rs`: 2 occurrences

- Line 226: failed to resolve: use of undeclared type `CString`: use of undeclared type `CString`
- Line 277: failed to resolve: use of undeclared type `CString`: use of undeclared type `CString`

#### `src\query\validator\clauses\order_by_validator.rs`: 2 occurrences

- Line 616: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 647: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 2 occurrences

- Line 240: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 240: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_inner_join.rs`: 2 occurrences

- Line 244: failed to resolve: use of undeclared type `Arc`: use of undeclared type `Arc`
- Line 244: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\clauses\with_validator.rs`: 2 occurrences

- Line 413: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 421: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 361: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`
- Line 460: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\strategies\pagination_strategy.rs`: 1 occurrences

- Line 180: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\statements\lookup_validator.rs`: 1 occurrences

- Line 530: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 434: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 1 occurrences

- Line 305: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

#### `src\query\validator\clauses\return_validator.rs`: 1 occurrences

- Line 380: failed to resolve: use of undeclared type `ExpressionAnalysisContext`: use of undeclared type `ExpressionAnalysisContext`

### error[E0432]: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

**Total Occurrences**: 14  
**Unique Files**: 12

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 3 occurrences

- Line 223: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`
- Line 282: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`
- Line 319: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 438: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 1 occurrences

- Line 376: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\set_validator.rs`: 1 occurrences

- Line 525: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\utility\update_config_validator.rs`: 1 occurrences

- Line 215: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\go_validator.rs`: 1 occurrences

- Line 524: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 604: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\fetch_edges_validator.rs`: 1 occurrences

- Line 482: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\update_validator.rs`: 1 occurrences

- Line 836: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 1 occurrences

- Line 390: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\strategies\helpers\type_checker.rs`: 1 occurrences

- Line 613: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

#### `src\query\validator\statements\remove_validator.rs`: 1 occurrences

- Line 261: unresolved import `ExpressionAnalysisContext`: no external crate `ExpressionAnalysisContext`

### error[E0425]: cannot find type `Expression` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 440: cannot find type `Expression` in this scope: not found in this scope

