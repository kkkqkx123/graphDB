# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 18
- **Total Issues**: 18
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 14
- **Files with Issues**: 17

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 18

### Warning Type Breakdown

- **warning**: 18 warnings

### Files with Warnings (Top 10)

- `src\storage\redb_storage.rs`: 2 warnings
- `src\query\executor\data_modification.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 warnings
- `src\query\validator\delete_validator.rs`: 1 warnings
- `src\storage\operations\redb_operations.rs`: 1 warnings
- `src\query\executor\search_executors.rs`: 1 warnings
- `src\query\optimizer\rules\join\join_optimization.rs`: 1 warnings
- `src\query\scheduler\async_scheduler.rs`: 1 warnings
- `src\query\optimizer\plan\node.rs`: 1 warnings

## Detailed Warning Categorization

### warning: field `condition` is never read

**Total Occurrences**: 18  
**Unique Files**: 17

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 34: field `db` is never read
- Line 239: associated items `update_edge_property` and `detect_format` are never used

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 413: field `condition` is never read

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 548: method `bfs_shortest_path_single` is never used

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 81: multiple methods are never used

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\index\fulltext.rs`: 1 occurrences

- Line 340: field `config` is never read

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\optimizer\rules\join\join_optimization.rs`: 1 occurrences

- Line 140: variant `NestedLoopJoin` is never constructed

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 390: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 115: multiple methods are never used

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 293: fields `vertex_cache` and `edge_cache` are never read

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 93: methods `apply_filter_with_thread_pool` and `calculate_batch_size` are never used

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 110: methods `get_edge_direction`, `get_edge_types`, `get_max_steps`, and `has_same_edge` are never used

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 289: method `InvalidPlanNode` should have a snake case name: help: convert the identifier to snake case: `invalid_plan_node`

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

