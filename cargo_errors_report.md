# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 49
- **Total Warnings**: 0
- **Total Issues**: 49
- **Unique Error Patterns**: 7
- **Unique Warning Patterns**: 0
- **Files with Issues**: 41

## Error Statistics

**Total Errors**: 49

### Error Type Breakdown

- **error[E0432]**: 41 errors
- **error[E0433]**: 6 errors
- **error[E0583]**: 1 errors
- **error[E0761]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 3 errors
- `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 3 errors
- `src\query\planner\statements\insert_planner.rs`: 2 errors
- `src\query\executor\graph_query_executor.rs`: 2 errors
- `src\query\planner\plan\core\nodes\project_node.rs`: 2 errors
- `src\query\planner\rewrite\merge\collapse_project.rs`: 2 errors
- `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 1 errors
- `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 errors
- `src\query\validator\strategies\alias_strategy.rs`: 1 errors
- `src\core\types\expression\utils.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0432]: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

**Total Occurrences**: 41  
**Unique Files**: 35

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 3 occurrences

- Line 222: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`
- Line 281: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`
- Line 318: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 3 occurrences

- Line 28: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`
- Line 95: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`
- Line 235: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\plan\core\nodes\project_node.rs`: 2 occurrences

- Line 80: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`
- Line 7: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 2 occurrences

- Line 5: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`
- Line 282: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_hash_left_join.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

#### `src\query\validator\helpers\expression_checker.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 5: unresolved import `crate::core::types::expression::context`: could not find `context` in `expression`

#### `src\query\planner\rewrite\context.rs`: 1 occurrences

- Line 11: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\rewrite\merge\combine_filter.rs`: 1 occurrences

- Line 5: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

#### `src\query\planner\plan\core\nodes\filter_node.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 150: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 occurrences

- Line 600: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 1 occurrences

- Line 32: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\validator\utility\update_config_validator.rs`: 1 occurrences

- Line 215: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\validator\expression_analyzer.rs`: 1 occurrences

- Line 723: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_cross_join.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

#### `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 1 occurrences

- Line 36: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\validator\helpers\type_checker.rs`: 1 occurrences

- Line 611: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\validator\strategies\alias_strategy.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 9: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\rewrite\predicate_pushdown\push_filter_down_inner_join.rs`: 1 occurrences

- Line 8: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 669: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 657: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\rewrite\expression_utils.rs`: 1 occurrences

- Line 19: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\core\types\expression\serializable.rs`: 1 occurrences

- Line 8: unresolved import `super::context`: could not find `context` in `super`

#### `src\query\validator\strategies\helpers\type_checker.rs`: 1 occurrences

- Line 611: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\parser\parser\parse_context.rs`: 1 occurrences

- Line 3: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 155: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 270: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 180: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\validator\helpers\variable_checker.rs`: 1 occurrences

- Line 6: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 7: unresolved import `crate::core::types::expression::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types::expression`

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 1 occurrences

- Line 9: unresolved import `crate::core::types::ExpressionAnalysisContext`: no `ExpressionAnalysisContext` in `core::types`

### error[E0433]: failed to resolve: could not find `ExpressionAnalysisContext` in `expression`: could not find `ExpressionAnalysisContext` in `expression`

**Total Occurrences**: 6  
**Unique Files**: 4

#### `src\query\planner\statements\insert_planner.rs`: 2 occurrences

- Line 236: failed to resolve: could not find `ExpressionAnalysisContext` in `expression`: could not find `ExpressionAnalysisContext` in `expression`
- Line 246: failed to resolve: could not find `ExpressionAnalysisContext` in `expression`: could not find `ExpressionAnalysisContext` in `expression`

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 159: failed to resolve: could not find `ExpressionAnalysisContext` in `expression`: could not find `ExpressionAnalysisContext` in `expression`
- Line 334: failed to resolve: could not find `ExpressionAnalysisContext` in `expression`: could not find `ExpressionAnalysisContext` in `expression`

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 1476: failed to resolve: could not find `ExpressionAnalysisContext` in `expression`: could not find `ExpressionAnalysisContext` in `expression`

#### `src\query\planner\statements\statement_planner.rs`: 1 occurrences

- Line 149: failed to resolve: could not find `ExpressionAnalysisContext` in `expression`: could not find `ExpressionAnalysisContext` in `expression`

### error[E0761]: file for module `context` found at both "src\query\validator\context.rs" and "src\query\validator\context\mod.rs"

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\mod.rs`: 1 occurrences

- Line 12: file for module `context` found at both "src\query\validator\context.rs" and "src\query\validator\context\mod.rs"

### error[E0583]: file not found for module `context`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\mod.rs`: 1 occurrences

- Line 10: file not found for module `context`

