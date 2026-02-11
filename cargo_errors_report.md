# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 20
- **Total Issues**: 20
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 7
- **Files with Issues**: 20

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 20

### Warning Type Breakdown

- **warning**: 20 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\rules\merge\merge_get_vertices_and_project.rs`: 1 warnings
- `src\query\executor\result_processing\topn.rs`: 1 warnings
- `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_edges.rs`: 1 warnings
- `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 warnings
- `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_edges.rs`: 1 warnings
- `src\query\executor\batch.rs`: 1 warnings
- `src\query\optimizer\rules\elimination\dedup_elimination.rs`: 1 warnings
- `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 warnings
- `src\query\executor\result_processing\filter.rs`: 1 warnings
- `src\query\scheduler\async_scheduler.rs`: 1 warnings

## Detailed Warning Categorization

### warning: methods `get_edge_direction`, `get_edge_types`, `get_max_steps`, and `has_same_edge` are never used

**Total Occurrences**: 20  
**Unique Files**: 20

#### `src\query\executor\data_processing\graph_traversal\all_paths.rs`: 1 occurrences

- Line 121: methods `get_edge_direction`, `get_edge_types`, `get_max_steps`, and `has_same_edge` are never used

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 81: multiple methods are never used

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 269: method `batch_get_neighbors_with_vertices` is never used

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_edges.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 92: multiple methods are never used

#### `src\query\optimizer\rules\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 5: unused doc comment

#### `src\query\optimizer\rules\merge\merge_get_nbrs_and_dedup.rs`: 1 occurrences

- Line 5: unused doc comment

#### `src\query\optimizer\rules\elimination\dedup_elimination.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\optimizer\rules\merge\merge_get_vertices_and_dedup.rs`: 1 occurrences

- Line 5: unused doc comment

#### `src\query\optimizer\rules\elimination\eliminate_filter.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\optimizer\rules\elimination\eliminate_row_collect.rs`: 1 occurrences

- Line 7: unused doc comment

#### `src\query\optimizer\rules\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 5: unused doc comment

#### `src\query\optimizer\rules\elimination\remove_noop_project.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_vertices.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_edges.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_index_scan.rs`: 1 occurrences

- Line 8: unused doc comment

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 124: method `calculate_batch_size` is never used

#### `src\query\executor\batch.rs`: 1 occurrences

- Line 92: field `config` is never read

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 992: methods `process_input_batch` and `process_batch` are never used

