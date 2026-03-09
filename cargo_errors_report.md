# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 20
- **Total Warnings**: 0
- **Total Issues**: 20
- **Unique Error Patterns**: 20
- **Unique Warning Patterns**: 0
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 20

### Error Type Breakdown

- **error[E0599]**: 16 errors
- **error[E0432]**: 4 errors

### Files with Errors (Top 10)

- `src\query\optimizer\analysis\reference_count.rs`: 17 errors
- `src\query\planner\statements\create_planner.rs`: 1 errors
- `src\query\planner\plan\core\nodes\traversal\traversal_node.rs`: 1 errors
- `src\query\planner\statements\insert_planner.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0599]: no method named `input` found for reference `&filter_node::FilterNode` in the current scope: private field, not a method

**Total Occurrences**: 16  
**Unique Files**: 1

#### `src\query\optimizer\analysis\reference_count.rs`: 16 occurrences

- Line 247: no method named `input` found for reference `&filter_node::FilterNode` in the current scope: private field, not a method
- Line 251: no method named `input` found for reference `&project_node::ProjectNode` in the current scope: private field, not a method
- Line 255: no method named `input` found for reference `&sort_node::SortNode` in the current scope: private field, not a method
- ... 13 more occurrences in this file

### error[E0432]: unresolved import `crate::query::planner::plan::core::nodes::plan_node_traits`: could not find `plan_node_traits` in `nodes`

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\query\optimizer\analysis\reference_count.rs`: 1 occurrences

- Line 7: unresolved import `crate::query::planner::plan::core::nodes::plan_node_traits`: could not find `plan_node_traits` in `nodes`

#### `src\query\planner\plan\core\nodes\traversal\traversal_node.rs`: 1 occurrences

- Line 7: unresolved import `super::super::common`: could not find `common` in `super`

#### `src\query\planner\statements\create_planner.rs`: 1 occurrences

- Line 13: unresolved imports `crate::query::planner::plan::core::nodes::control_flow_node`, `crate::query::planner::plan::core::nodes::insert_nodes`: could not find `control_flow_node` in `nodes`, could not find `insert_nodes` in `nodes`

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 13: unresolved import `crate::query::planner::plan::core::nodes::insert_nodes`: could not find `insert_nodes` in `nodes`

