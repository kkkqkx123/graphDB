# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 22
- **Total Warnings**: 0
- **Total Issues**: 22
- **Unique Error Patterns**: 15
- **Unique Warning Patterns**: 0
- **Files with Issues**: 11

## Error Statistics

**Total Errors**: 22

### Error Type Breakdown

- **error[E0599]**: 10 errors
- **error[E0515]**: 5 errors
- **error[E0308]**: 5 errors
- **error[E0614]**: 2 errors

### Files with Errors (Top 10)

- `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 5 errors
- `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 errors
- `src\query\planner\plan\algorithms\index_scan.rs`: 2 errors
- `src\query\optimizer\predicate_pushdown.rs`: 2 errors
- `src\query\visitor\evaluable_expr_visitor.rs`: 2 errors
- `src\core\visitor.rs`: 2 errors
- `src\query\executor\result_processing\filter.rs`: 1 errors
- `src\query\executor\result_processing\topn.rs`: 1 errors
- `src\query\optimizer\operation_merge.rs`: 1 errors
- `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0599]: no method named `as_any` found for reference `&nodes::plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`

**Total Occurrences**: 10  
**Unique Files**: 5

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 occurrences

- Line 144: no method named `visit_multi_shortest_path` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`
- Line 267: no method named `visit_bfs_shortest` found for mutable reference `&mut V` in the current scope
- Line 401: no method named `visit_all_paths` found for mutable reference `&mut V` in the current scope
- ... 1 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 2 occurrences

- Line 53: no method named `as_any` found for reference `&nodes::plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`
- Line 168: no method named `as_any` found for reference `&nodes::plan_node_enum::PlanNodeEnum` in the current scope: method not found in `&PlanNodeEnum`

#### `src\query\planner\plan\algorithms\index_scan.rs`: 2 occurrences

- Line 135: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope
- Line 244: no method named `visit_fulltext_index_scan` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`

#### `src\query\planner\match_planning\clauses\unwind_planner.rs`: 1 occurrences

- Line 231: `core::types::expression::Expression` doesn't implement `std::fmt::Display`: `core::types::expression::Expression` cannot be formatted with the default formatter

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 515: no method named `dependencies` found for reference `&project_node::ProjectNode` in the current scope: method not found in `&ProjectNode`

### error[E0308]: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`

**Total Occurrences**: 5  
**Unique Files**: 4

#### `src\core\visitor.rs`: 2 occurrences

- Line 325: mismatched types: expected `&Option<Expression>`, found `Option<&Expression>`
- Line 338: arguments to this method are incorrect

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 58: mismatched types: expected `PlanNodeEnum`, found `Box<PlanNodeEnum>`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 311: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 411: mismatched types: expected `Option<&Box<dyn Executor<S>>>`, found `Option<Box<dyn Executor<S>>>`

### error[E0515]: cannot return value referencing local variable `guard`: returns a value referencing data owned by the current function

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 5 occurrences

- Line 113: cannot return value referencing local variable `guard`: returns a value referencing data owned by the current function
- Line 327: cannot return value referencing local variable `guard`: returns a value referencing data owned by the current function
- Line 513: cannot return value referencing local variable `guard`: returns a value referencing data owned by the current function
- ... 2 more occurrences in this file

### error[E0614]: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\visitor\evaluable_expr_visitor.rs`: 2 occurrences

- Line 215: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced
- Line 216: type `core::types::expression::Expression` cannot be dereferenced: can't be dereferenced

