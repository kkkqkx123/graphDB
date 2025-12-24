# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 27
- **Total Warnings**: 0
- **Total Issues**: 27
- **Unique Error Patterns**: 14
- **Unique Warning Patterns**: 0
- **Files with Issues**: 10

## Error Statistics

**Total Errors**: 27

### Error Type Breakdown

- **error[E0407]**: 15 errors
- **error[E0599]**: 12 errors

### Files with Errors (Top 10)

- `src\query\planner\plan\management\security\user_ops.rs`: 7 errors
- `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 5 errors
- `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 errors
- `src\query\planner\plan\core\nodes\sort_node.rs`: 3 errors
- `src\query\planner\plan\algorithms\index_scan.rs`: 2 errors
- `src\query\planner\plan\core\nodes\join_node.rs`: 2 errors
- `src\query\planner\plan\core\nodes\traversal_node.rs`: 1 errors
- `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 errors
- `src\query\planner\plan\core\nodes\aggregate_node.rs`: 1 errors
- `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0407]: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

**Total Occurrences**: 15  
**Unique Files**: 6

#### `src\query\planner\plan\management\security\user_ops.rs`: 7 occurrences

- Line 68: method `dependencies` is not a member of trait `PlanNode`: not a member of trait `PlanNode`
- Line 144: method `dependencies` is not a member of trait `PlanNode`: not a member of trait `PlanNode`
- Line 220: method `dependencies` is not a member of trait `PlanNode`: not a member of trait `PlanNode`
- ... 4 more occurrences in this file

#### `src\query\planner\plan\core\nodes\sort_node.rs`: 3 occurrences

- Line 144: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`
- Line 319: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`
- Line 494: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

#### `src\query\planner\plan\core\nodes\join_node.rs`: 2 occurrences

- Line 165: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`
- Line 367: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

#### `src\query\planner\plan\core\nodes\traversal_node.rs`: 1 occurrences

- Line 743: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

#### `src\query\planner\plan\core\nodes\graph_scan_node.rs`: 1 occurrences

- Line 837: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

#### `src\query\planner\plan\core\nodes\aggregate_node.rs`: 1 occurrences

- Line 141: method `dependencies` is not a member of trait `super::plan_node_traits::PlanNode`: not a member of trait `super::plan_node_traits::PlanNode`

### error[E0599]: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope

**Total Occurrences**: 12  
**Unique Files**: 4

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 5 occurrences

- Line 516: no method named `dependencies` found for reference `&project_node::ProjectNode` in the current scope: method not found in `&ProjectNode`
- Line 527: no method named `dependencies` found for reference `&graph_scan_node::GetVerticesNode` in the current scope: method not found in `&GetVerticesNode`
- Line 528: no method named `dependencies` found for reference `&graph_scan_node::GetEdgesNode` in the current scope: method not found in `&GetEdgesNode`
- ... 2 more occurrences in this file

#### `src\query\planner\plan\algorithms\path_algorithms.rs`: 4 occurrences

- Line 144: no method named `visit_multi_shortest_path` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`
- Line 267: no method named `visit_bfs_shortest` found for mutable reference `&mut V` in the current scope
- Line 401: no method named `visit_all_paths` found for mutable reference `&mut V` in the current scope
- ... 1 more occurrences in this file

#### `src\query\planner\plan\algorithms\index_scan.rs`: 2 occurrences

- Line 135: no method named `visit_index_scan` found for mutable reference `&mut V` in the current scope
- Line 244: no method named `visit_fulltext_index_scan` found for mutable reference `&mut V` in the current scope: method not found in `&mut V`

#### `src\query\planner\ngql\fetch_vertices_planner.rs`: 1 occurrences

- Line 68: no method named `add_dependency` found for struct `graph_scan_node::GetVerticesNode` in the current scope: method not found in `GetVerticesNode`

