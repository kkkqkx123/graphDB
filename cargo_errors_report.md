# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 7
- **Total Warnings**: 0
- **Total Issues**: 7
- **Unique Error Patterns**: 4
- **Unique Warning Patterns**: 0
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 7

### Error Type Breakdown

- **error[E0599]**: 2 errors
- **error**: 2 errors
- **error[E0308]**: 2 errors
- **error[E0004]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\rewrite\visitor.rs`: 2 errors
- `src\query\optimizer\strategy\subquery_unnesting.rs`: 2 errors
- `src\query\optimizer\strategy\materialization.rs`: 2 errors
- `src\query\planner\plan\core\nodes\plan_node_children.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `&ContextualExpression`, found `&Expression`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 2 occurrences

- Line 209: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 210: mismatched types: expected `&ContextualExpression`, found `&Expression`

### error: no rules expected `;`: no rules expected this token in macro call

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planner\rewrite\visitor.rs`: 2 occurrences

- Line 127: no rules expected `;`: no rules expected this token in macro call
- Line 158: no rules expected `;`: no rules expected this token in macro call

### error[E0599]: no method named `expression` found for reference `&operators::AggregateFunction` in the current scope: method not found in `&AggregateFunction`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\strategy\materialization.rs`: 2 occurrences

- Line 230: no method named `expression` found for reference `&operators::AggregateFunction` in the current scope: method not found in `&AggregateFunction`
- Line 368: no method named `expression` found for reference `&operators::AggregateFunction` in the current scope: method not found in `&AggregateFunction`

### error[E0004]: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered: pattern `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\plan_node_children.rs`: 1 occurrences

- Line 9: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered: pattern `&plan_node_enum::PlanNodeEnum::Materialize(_)` not covered

