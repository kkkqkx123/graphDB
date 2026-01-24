# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 29
- **Total Issues**: 29
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 27
- **Files with Issues**: 23

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 29

### Warning Type Breakdown

- **warning**: 29 warnings

### Files with Warnings (Top 10)

- `src\query\executor\aggregation.rs`: 2 warnings
- `src\query\scheduler\async_scheduler.rs`: 2 warnings
- `src\api\service\index_service.rs`: 2 warnings
- `src\query\executor\graph_query_executor.rs`: 2 warnings
- `src\query\planner\statements\match_planner.rs`: 2 warnings
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 warnings
- `src\query\executor\data_access.rs`: 1 warnings
- `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 warnings
- `src\core\value\comparison.rs`: 1 warnings
- `src\index\cache.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`

**Total Occurrences**: 29  
**Unique Files**: 23

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 207: variable does not need to be mutable

#### `src\query\executor\aggregation.rs`: 2 occurrences

- Line 8: unused import: `HasInput`
- Line 238: fields `col_names` and `output_var` are never read

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 96: unused variable: `match_ctx`: help: if this is intentional, prefix it with an underscore: `_match_ctx`
- Line 157: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\api\service\index_service.rs`: 2 occurrences

- Line 504: unused `std::result::Result` that must be used
- Line 520: unused `std::result::Result` that must be used

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 9: unused import: `ExecutionContext`
- Line 74: multiple methods are never used

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 31: field `thread_pool` is never read
- Line 99: multiple methods are never used

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 15: unused import: `std::collections::HashMap`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 330: unnecessary parentheses around function argument

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 397: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 169: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 546: fields `history_left_paths` and `history_right_paths` are never read

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 42: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 22: unused imports: `MultiShortestPathExecutor` and `ShortestPathExecutor`

#### `src\core\value\comparison.rs`: 1 occurrences

- Line 403: associated functions `cmp_coordinate_list` and `cmp_polygon_list` are never used

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 170: method `clear` is never used

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

