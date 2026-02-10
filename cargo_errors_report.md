# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 26
- **Total Issues**: 26
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 16
- **Files with Issues**: 16

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 26

### Warning Type Breakdown

- **warning**: 26 warnings

### Files with Warnings (Top 10)

- `src\query\scheduler\execution_plan_analyzer.rs`: 6 warnings
- `src\query\executor\result_processing\topn.rs`: 4 warnings
- `src\query\optimizer\engine\optimizer.rs`: 2 warnings
- `src\query\executor\factory.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 1 warnings
- `src\query\planner\plan\core\nodes\join_node.rs`: 1 warnings
- `src\query\optimizer\rules\transformation\optimize_set_operation_input_order.rs`: 1 warnings
- `src\query\planner\statements\fetch_vertices_planner.rs`: 1 warnings
- `src\query\executor\result_processing\filter.rs`: 1 warnings
- `src\query\executor\batch.rs`: 1 warnings

## Detailed Warning Categorization

### warning: multiple methods are never used

**Total Occurrences**: 26  
**Unique Files**: 16

#### `src\query\scheduler\execution_plan_analyzer.rs`: 6 occurrences

- Line 384: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode`
- Line 360: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode`
- Line 408: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::BinaryInputNode`
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\topn.rs`: 4 occurrences

- Line 277: variable does not need to be mutable
- Line 310: unused variable: `heap_size`: help: if this is intentional, prefix it with an underscore: `_heap_size`
- Line 517: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used
- ... 1 more occurrences in this file

#### `src\query\executor\factory.rs`: 2 occurrences

- Line 13: unused import: `PlanNode`
- Line 13: unused import: `BinaryInputNode`

#### `src\query\optimizer\engine\optimizer.rs`: 2 occurrences

- Line 320: unused import: `BinaryInputNode`
- Line 195: unused import: `BinaryInputNode`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 92: multiple methods are never used

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 121: methods `get_edge_direction`, `get_edge_types`, `get_max_steps`, and `has_same_edge` are never used

#### `src\query\executor\batch.rs`: 1 occurrences

- Line 92: field `config` is never read

#### `src\query\planner\statements\fetch_vertices_planner.rs`: 1 occurrences

- Line 7: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\transformation\optimize_set_operation_input_order.rs`: 1 occurrences

- Line 10: unused import: `BinaryInputNode`

#### `src\query\planner\plan\core\nodes\plan_node_enum.rs`: 1 occurrences

- Line 4: unused import: `super::plan_node_traits::PlanNode`

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 269: method `batch_get_neighbors_with_vertices` is never used

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 124: method `calculate_batch_size` is never used

#### `src\query\planner\plan\core\nodes\join_node.rs`: 1 occurrences

- Line 172: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 6: unused import: `BinaryInputNode`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 81: multiple methods are never used

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 9: unused import: `rayon::prelude`

