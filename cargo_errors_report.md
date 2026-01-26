# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 44
- **Total Issues**: 44
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 38
- **Files with Issues**: 53

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 44

### Warning Type Breakdown

- **warning**: 44 warnings

### Files with Warnings (Top 10)

- `src\query\parser\lexer\lexer.rs`: 4 warnings
- `src\query\executor\factory.rs`: 4 warnings
- `src\query\parser\expressions\expression_converter.rs`: 3 warnings
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 warnings
- `src\query\planner\statements\match_planner.rs`: 2 warnings
- `src\api\service\index_service.rs`: 2 warnings
- `src\query\executor\graph_query_executor.rs`: 2 warnings
- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 2 warnings
- `src\query\context\managers\schema_traits.rs`: 1 warnings
- `src\query\executor\admin\data\update.rs`: 1 warnings

## Detailed Warning Categorization

### warning: unused import: `std::collections::HashMap`

**Total Occurrences**: 44  
**Unique Files**: 31

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 22: unused imports: `MultiShortestPathExecutor` and `ShortestPathExecutor`
- Line 45: unused imports: `EdgeAlterInfo`, `EdgeManageInfo`, `IndexManageInfo`, `SpaceManageInfo`, `TagAlterInfo`, and `TagManageInfo`
- Line 842: unused import: `AlterTagOp`
- ... 1 more occurrences in this file

#### `src\query\parser\lexer\lexer.rs`: 4 occurrences

- Line 909: variable does not need to be mutable
- Line 928: variable does not need to be mutable
- Line 961: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\parser\expressions\expression_converter.rs`: 3 occurrences

- Line 700: unused variable: `target_type`
- Line 857: unused variable: `arg`
- Line 857: unused variable: `distinct`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`
- Line 207: variable does not need to be mutable

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 32: field `thread_pool` is never read
- Line 100: multiple methods are never used

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 96: unused variable: `match_ctx`
- Line 157: unused variable: `planner`

#### `src\api\service\index_service.rs`: 2 occurrences

- Line 504: unused `std::result::Result` that must be used
- Line 520: unused `std::result::Result` that must be used

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 2 occurrences

- Line 7: unused import: `Vertex`
- Line 191: method `mark_termination` is never used

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 15: unused import: `std::collections::HashMap`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 397: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\core\value\comparison.rs`: 1 occurrences

- Line 403: associated functions `cmp_coordinate_list` and `cmp_polygon_list` are never used

#### `src\query\executor\admin\mod.rs`: 1 occurrences

- Line 13: unused import: `crate::storage::StorageEngine`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 1: unused import: `crate::core::error::DBError`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\query\executor\admin\space\create_space.rs`: 1 occurrences

- Line 8: unused import: `Value`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 297: unused variable: `tag_name`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 170: method `clear` is never used

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 42: unused variable: `ctx`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 8: unused imports: `UpdateOp` and `UpdateTarget`

