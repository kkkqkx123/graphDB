# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 74
- **Total Issues**: 74
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 55
- **Files with Issues**: 45

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 74

### Warning Type Breakdown

- **warning**: 74 warnings

### Files with Warnings (Top 10)

- `src\query\parser\lexer\lexer.rs`: 4 warnings
- `src\query\planner\statements\match_planner.rs`: 4 warnings
- `src\query\executor\factory.rs`: 4 warnings
- `src\query\executor\graph_query_executor.rs`: 4 warnings
- `src\query\planner\statements\paths\match_path_planner.rs`: 4 warnings
- `src\query\planner\statements\seeks\index_seek.rs`: 3 warnings
- `src\query\planner\statements\seeks\scan_seek.rs`: 3 warnings
- `src\query\parser\parser\expr_parser.rs`: 3 warnings
- `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 warnings
- `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

**Total Occurrences**: 74  
**Unique Files**: 45

#### `src\query\planner\statements\match_planner.rs`: 4 occurrences

- Line 92: unused variable: `stmt`: help: if this is intentional, prefix it with an underscore: `_stmt`
- Line 99: variable does not need to be mutable
- Line 119: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- ... 1 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 4 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable
- Line 36: field `thread_pool` is never read
- ... 1 more occurrences in this file

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 22: unused imports: `MultiShortestPathExecutor` and `ShortestPathExecutor`
- Line 45: unused imports: `EdgeAlterInfo`, `EdgeManageInfo`, `IndexManageInfo`, `SpaceManageInfo`, `TagAlterInfo`, and `TagManageInfo`
- Line 866: unused import: `AlterTagOp`
- ... 1 more occurrences in this file

#### `src\query\parser\lexer\lexer.rs`: 4 occurrences

- Line 909: variable does not need to be mutable
- Line 928: variable does not need to be mutable
- Line 961: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\planner\statements\paths\match_path_planner.rs`: 4 occurrences

- Line 5: unused import: `crate::core::types::Expression`
- Line 416: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 422: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 occurrences

- Line 23: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`
- Line 461: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 467: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 3 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`
- Line 7: unused import: `Value`
- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\parser\parser\expr_parser.rs`: 3 occurrences

- Line 8: unused import: `AggregateFunction`
- Line 9: unused imports: `BinaryOp` and `UnaryOp`
- Line 451: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\planner\statements\seeks\index_seek.rs`: 3 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`
- Line 7: unused import: `Value`
- Line 94: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\api\service\index_service.rs`: 2 occurrences

- Line 504: unused `std::result::Result` that must be used
- Line 520: unused `std::result::Result` that must be used

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 207: variable does not need to be mutable

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`
- Line 55: unused variable: `match_expression`: help: if this is intentional, prefix it with an underscore: `_match_expression`

#### `src\query\validator\base_validator.rs`: 2 occurrences

- Line 228: calls to `std::mem::drop` with a reference instead of an owned value does nothing
- Line 248: calls to `std::mem::drop` with a reference instead of an owned value does nothing

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`
- Line 134: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 2 occurrences

- Line 7: unused import: `Vertex`
- Line 191: method `mark_termination` is never used

#### `src\core\value\comparison.rs`: 1 occurrences

- Line 403: associated functions `cmp_coordinate_list` and `cmp_polygon_list` are never used

#### `src\query\executor\admin\space\create_space.rs`: 1 occurrences

- Line 8: unused import: `Value`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 7: unused import: `ExpressionVisitor`

#### `src\query\validator\validation_factory.rs`: 1 occurrences

- Line 8: unused import: `super::validation_interface::ValidationStrategyType`

#### `src\query\executor\admin\mod.rs`: 1 occurrences

- Line 13: unused import: `crate::storage::StorageEngine`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 19: unused import: `crate::core::types::Span`

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 8: unused imports: `UpdateOp` and `UpdateTarget`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 305: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 15: unused import: `std::collections::HashMap`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 170: method `clear` is never used

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 397: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\query\parser\expressions\mod.rs`: 1 occurrences

- Line 5: unused import: `Expression`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 1: unused import: `crate::core::error::DBError`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\planner\statements\seeks\seek_strategy_base.rs`: 1 occurrences

- Line 6: unused import: `StorageError`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 42: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 1 occurrences

- Line 11: unused imports: `IndexInfo` and `NodePattern`

