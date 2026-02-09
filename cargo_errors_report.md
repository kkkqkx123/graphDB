# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 20
- **Total Issues**: 20
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 16
- **Files with Issues**: 18

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 20

### Warning Type Breakdown

- **warning**: 20 warnings

### Files with Warnings (Top 10)

- `src\query\executor\factory.rs`: 2 warnings
- `src\storage\redb_storage.rs`: 2 warnings
- `src\query\optimizer\plan\node.rs`: 1 warnings
- `src\query\executor\data_modification.rs`: 1 warnings
- `src\storage\operations\redb_operations.rs`: 1 warnings
- `src\query\executor\search_executors.rs`: 1 warnings
- `src\index\fulltext.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 warnings
- `src\query\scheduler\async_scheduler.rs`: 1 warnings
- `src\query\validator\update_validator.rs`: 1 warnings

## Detailed Warning Categorization

### warning: method `mark_termination` is never used

**Total Occurrences**: 20  
**Unique Files**: 18

#### `src\query\executor\factory.rs`: 2 occurrences

- Line 961: unused variable: `start_vertex`: help: if this is intentional, prefix it with an underscore: `_start_vertex`
- Line 970: unused variable: `end_vertex`: help: if this is intentional, prefix it with an underscore: `_end_vertex`

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 34: field `db` is never read
- Line 239: associated items `update_edge_property` and `detect_format` are never used

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 289: method `InvalidPlanNode` should have a snake case name: help: convert the identifier to snake case: `invalid_plan_node`

#### `src\query\optimizer\rules\join\join_optimization.rs`: 1 occurrences

- Line 140: variant `NestedLoopJoin` is never constructed

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 293: fields `vertex_cache` and `edge_cache` are never read

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 390: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 413: field `condition` is never read

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 81: multiple methods are never used

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 93: methods `apply_filter_with_thread_pool` and `calculate_batch_size` are never used

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 110: methods `get_edge_direction`, `get_edge_types`, `get_max_steps`, and `has_same_edge` are never used

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 115: multiple methods are never used

#### `src\index\fulltext.rs`: 1 occurrences

- Line 340: field `config` is never read

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 317: methods `build_path`, `conjunct_paths`, `create_path`, `build_half_path`, and `bfs_shortest_path_single` are never used

