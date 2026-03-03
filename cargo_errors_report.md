# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 49
- **Total Warnings**: 13
- **Total Issues**: 62
- **Unique Error Patterns**: 31
- **Unique Warning Patterns**: 12
- **Files with Issues**: 5

## Error Statistics

**Total Errors**: 49

### Error Type Breakdown

- **error[E0599]**: 30 errors
- **error[E0308]**: 7 errors
- **error[E0407]**: 5 errors
- **error[E0119]**: 3 errors
- **error**: 2 errors
- **error[E0432]**: 1 errors
- **error[E0050]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\strategy\materialization.rs`: 21 errors
- `src\query\optimizer\strategy\subquery_unnesting.rs`: 17 errors
- `src\query\planner\rewrite\visitor.rs`: 5 errors
- `src\query\planner\plan\core\nodes\data_processing_node.rs`: 3 errors
- `src\query\planner\plan\core\nodes\macros.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 13

### Warning Type Breakdown

- **warning**: 13 warnings

### Files with Warnings (Top 10)

- `src\query\planner\rewrite\visitor.rs`: 8 warnings
- `src\query\optimizer\strategy\subquery_unnesting.rs`: 2 warnings
- `src\query\optimizer\strategy\materialization.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no method named `input` found for reference `&data_processing_node::PatternApplyNode` in the current scope

**Total Occurrences**: 30  
**Unique Files**: 2

#### `src\query\optimizer\strategy\materialization.rs`: 16 occurrences

- Line 245: no variant or associated item named `Join` found for enum `plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`
- Line 226: no method named `input` found for reference `&filter_node::FilterNode` in the current scope: private field, not a method
- Line 228: no method named `input` found for reference `&project_node::ProjectNode` in the current scope: private field, not a method
- ... 13 more occurrences in this file

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 14 occurrences

- Line 123: no method named `input` found for reference `&data_processing_node::PatternApplyNode` in the current scope
- Line 130: no method named `condition` found for reference `&data_processing_node::PatternApplyNode` in the current scope: method not found in `&PatternApplyNode`
- Line 148: no method named `input` found for reference `&data_processing_node::PatternApplyNode` in the current scope
- ... 11 more occurrences in this file

### error[E0308]: mismatched types: expected `ContextualExpression`, found `Option<_>`

**Total Occurrences**: 7  
**Unique Files**: 2

#### `src\query\optimizer\strategy\materialization.rs`: 4 occurrences

- Line 218: mismatched types: expected `ContextualExpression`, found `Option<_>`
- Line 232: mismatched types: expected `&Expression`, found `&AggregateFunction`
- Line 274: mismatched types: expected `ContextualExpression`, found `Option<_>`
- ... 1 more occurrences in this file

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 3 occurrences

- Line 187: mismatched types: expected `ContextualExpression`, found `Option<_>`
- Line 217: mismatched types: expected `&ContextualExpression`, found `&Box<Expression>`
- Line 218: mismatched types: expected `&ContextualExpression`, found `&Box<Expression>`

### error[E0407]: method `set_id` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

**Total Occurrences**: 5  
**Unique Files**: 2

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 3 occurrences

- Line 594: method `set_id` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`
- Line 598: method `type_name` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`
- Line 602: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

#### `src\query\planner\rewrite\visitor.rs`: 2 occurrences

- Line 490: method `visit_insert_vertices` is not a member of trait `PlanNodeVisitor`: not a member of trait `PlanNodeVisitor`
- Line 494: method `visit_insert_edges` is not a member of trait `PlanNodeVisitor`: not a member of trait `PlanNodeVisitor`

### error[E0119]: conflicting implementations of trait `PlanNodeClonable` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\macros.rs`: 3 occurrences

- Line 563: conflicting implementations of trait `PlanNodeClonable` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`
- Line 534: conflicting implementations of trait `plan_node_traits::PlanNode` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`
- Line 547: conflicting implementations of trait `plan_node_traits::SingleInputNode` for type `MaterializeNode`: conflicting implementation for `MaterializeNode`

### error: no rules expected `;`: no rules expected this token in macro call

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planner\rewrite\visitor.rs`: 2 occurrences

- Line 127: no rules expected `;`: no rules expected this token in macro call
- Line 158: no rules expected `;`: no rules expected this token in macro call

### error[E0050]: method `visit_default` has 2 parameters but the declaration in trait `plan_node_visitor::PlanNodeVisitor::visit_default` has 1: expected 1 parameter, found 2

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\rewrite\visitor.rs`: 1 occurrences

- Line 122: method `visit_default` has 2 parameters but the declaration in trait `plan_node_visitor::PlanNodeVisitor::visit_default` has 1: expected 1 parameter, found 2

### error[E0432]: unresolved import `crate::query::planner::plan::core::nodes::MaterializeNode`: no `MaterializeNode` in `query::planner::plan::core::nodes`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\strategy\materialization.rs`: 1 occurrences

- Line 359: unresolved import `crate::query::planner::plan::core::nodes::MaterializeNode`: no `MaterializeNode` in `query::planner::plan::core::nodes`

## Detailed Warning Categorization

### warning: unused doc comment

**Total Occurrences**: 13  
**Unique Files**: 4

#### `src\query\planner\rewrite\visitor.rs`: 8 occurrences

- Line 15: unused import: `SingleInputNode`
- Line 20: unused import: `crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode`
- Line 22: unused imports: `DedupNode`, `PatternApplyNode`, `RollUpApplyNode`, and `UnwindNode`
- ... 5 more occurrences in this file

#### `src\query\optimizer\strategy\materialization.rs`: 2 occurrences

- Line 33: unused import: `ReferenceCountAnalysis`
- Line 406: unused variable: `optimizer`: help: if this is intentional, prefix it with an underscore: `_optimizer`

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 2 occurrences

- Line 33: unused import: `ScanVerticesNode`
- Line 327: unused variable: `optimizer`: help: if this is intentional, prefix it with an underscore: `_optimizer`

#### `src\query\planner\plan\core\nodes\data_processing_node.rs`: 1 occurrences

- Line 549: unused doc comment

