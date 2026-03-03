# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 36
- **Total Warnings**: 5
- **Total Issues**: 41
- **Unique Error Patterns**: 23
- **Unique Warning Patterns**: 4
- **Files with Issues**: 7

## Error Statistics

**Total Errors**: 36

### Error Type Breakdown

- **error[E0599]**: 21 errors
- **error[E0308]**: 4 errors
- **error[E0004]**: 4 errors
- **error[E0407]**: 3 errors
- **error[E0119]**: 3 errors
- **error[E0277]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\strategy\subquery_unnesting.rs`: 17 errors
- `src\query\planner\rewrite\subquery_unnesting\simple_unnest.rs`: 9 errors
- `src\query\planner\plan\core\nodes\macros.rs`: 3 errors
- `src\query\planner\plan\core\nodes\data_processing_node.rs`: 3 errors
- `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 errors
- `src\query\planner\plan\core\nodes\plan_node_traits_impl.rs`: 1 errors
- `src\query\planner\plan\core\nodes\plan_node_visitor.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 5

### Warning Type Breakdown

- **warning**: 5 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\strategy\subquery_unnesting.rs`: 2 warnings
- `src\query\planner\rewrite\subquery_unnesting\simple_unnest.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no variant or associated item named `Eq` found for enum `operators::BinaryOperator` in the current scope: variant or associated item not found in `BinaryOperator`

**Total Occurrences**: 21  
**Unique Files**: 2

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 14 occurrences

- Line 123: no method named `input` found for reference `&data_processing_node::PatternApplyNode` in the current scope
- Line 130: no method named `condition` found for reference `&data_processing_node::PatternApplyNode` in the current scope: method not found in `&PatternApplyNode`
- Line 148: no method named `input` found for reference `&data_processing_node::PatternApplyNode` in the current scope
- ... 11 more occurrences in this file

#### `src\query\planner\rewrite\subquery_unnesting\simple_unnest.rs`: 7 occurrences

- Line 112: no variant or associated item named `Eq` found for enum `operators::BinaryOperator` in the current scope: variant or associated item not found in `BinaryOperator`
- Line 138: no method named `name` found for reference `&graph_scan_node::ScanVerticesNode` in the current scope: method not found in `&ScanVerticesNode`
- Line 139: no method named `vertex_count` found for struct `TagStatistics` in the current scope: field, not a method
- ... 4 more occurrences in this file

### error[E0004]: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered: pattern `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered

**Total Occurrences**: 4  
**Unique Files**: 3

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 2 occurrences

- Line 594: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered: pattern `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered
- Line 672: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered: pattern `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered

#### `src\query\planner\plan\core\nodes\plan_node_traits_impl.rs`: 1 occurrences

- Line 250: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered: pattern `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered

#### `src\query\planner\plan\core\nodes\plan_node_visitor.rs`: 1 occurrences

- Line 185: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered: pattern `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered

### error[E0308]: mismatched types: expected `ContextualExpression`, found `Option<_>`

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 3 occurrences

- Line 187: mismatched types: expected `ContextualExpression`, found `Option<_>`
- Line 217: mismatched types: expected `&ContextualExpression`, found `&Box<Expression>`
- Line 218: mismatched types: expected `&ContextualExpression`, found `&Box<Expression>`

#### `src\query\planner\rewrite\subquery_unnesting\simple_unnest.rs`: 1 occurrences

- Line 89: mismatched types: expected `ContextualExpression`, found `Option<_>`

### error[E0407]: method `set_id` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 3 occurrences

- Line 594: method `set_id` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`
- Line 598: method `type_name` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`
- Line 602: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

### error[E0119]: conflicting implementations of trait `PlanNodeClonable` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\macros.rs`: 3 occurrences

- Line 563: conflicting implementations of trait `PlanNodeClonable` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`
- Line 534: conflicting implementations of trait `plan_node_traits::PlanNode` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`
- Line 547: conflicting implementations of trait `plan_node_traits::SingleInputNode` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`

### error[E0277]: `?` couldn't convert the error to `rewrite::result::RewriteError`: the trait `From<query::planner::planner::PlannerError>` is not implemented for `rewrite::result::RewriteError`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\rewrite\subquery_unnesting\simple_unnest.rs`: 1 occurrences

- Line 235: `?` couldn't convert the error to `rewrite::result::RewriteError`: the trait `From<query::planner::planner::PlannerError>` is not implemented for `rewrite::result::RewriteError`

## Detailed Warning Categorization

### warning: unused import: `ScanVerticesNode`

**Total Occurrences**: 5  
**Unique Files**: 3

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 2 occurrences

- Line 33: unused import: `ScanVerticesNode`
- Line 327: unused variable: `optimizer`: help: if this is intentional, prefix it with an underscore: `_optimizer`

#### `src\query\planner\rewrite\subquery_unnesting\simple_unnest.rs`: 2 occurrences

- Line 33: unused import: `ScanVerticesNode`
- Line 255: unused variable: `rule`: help: if this is intentional, prefix it with an underscore: `_rule`

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 occurrences

- Line 549: unused doc comment

