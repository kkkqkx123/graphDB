# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 75
- **Total Issues**: 75
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 53
- **Files with Issues**: 45

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 75

### Warning Type Breakdown

- **warning**: 75 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\elimination_rules.rs`: 7 warnings
- `src\query\planner\statements\match_planner.rs`: 4 warnings
- `src\query\executor\result_processing\projection.rs`: 4 warnings
- `src\query\executor\graph_query_executor.rs`: 4 warnings
- `src\query\planner\statements\paths\match_path_planner.rs`: 3 warnings
- `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings
- `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 warnings
- `src\query\planner\statements\clauses\pagination_planner.rs`: 2 warnings
- `src\query\planner\planner.rs`: 2 warnings

## Detailed Warning Categorization

### warning: field `edge_types` is never read

**Total Occurrences**: 75  
**Unique Files**: 45

#### `src\query\optimizer\elimination_rules.rs`: 7 occurrences

- Line 86: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 169: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 312: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- ... 4 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 4 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable
- Line 36: field `thread_pool` is never read
- ... 1 more occurrences in this file

#### `src\query\planner\statements\match_planner.rs`: 4 occurrences

- Line 75: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 290: unreachable pattern: no value can reach this
- Line 464: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 4 occurrences

- Line 437: unused variable: `vertex1`: help: if this is intentional, prefix it with an underscore: `_vertex1`
- Line 450: unused variable: `vertex2`: help: if this is intentional, prefix it with an underscore: `_vertex2`
- Line 514: unused variable: `edge1`: help: if this is intentional, prefix it with an underscore: `_edge1`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 occurrences

- Line 23: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`
- Line 461: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 467: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 3 occurrences

- Line 415: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 421: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 443: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`
- Line 20: field `default_limit` is never read

#### `src\query\executor\factory.rs`: 2 occurrences

- Line 42: unused import: `AlterEdgeExecutor`
- Line 49: unused imports: `EdgeAlterInfo`, `EdgeManageInfo`, `IndexManageInfo`, `SpaceManageInfo`, `TagAlterInfo`, and `TagManageInfo`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 207: variable does not need to be mutable

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\query\optimizer\rule_registry.rs`: 2 occurrences

- Line 115: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 116: unused variable: `group_node`: help: if this is intentional, prefix it with an underscore: `_group_node`

#### `src\api\service\index_service.rs`: 2 occurrences

- Line 504: unused `std::result::Result` that must be used
- Line 520: unused `std::result::Result` that must be used

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`
- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`
- Line 55: unused variable: `match_expression`: help: if this is intentional, prefix it with an underscore: `_match_expression`

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 86: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 347: unreachable pattern: no value can reach this

#### `src\query\planner\planner.rs`: 2 occurrences

- Line 191: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 67: field `max_size` is never read

#### `src\query\validator\base_validator.rs`: 2 occurrences

- Line 228: calls to `std::mem::drop` with a reference instead of an owned value does nothing
- Line 248: calls to `std::mem::drop` with a reference instead of an owned value does nothing

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 8: unused imports: `UpdateOp` and `UpdateTarget`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 97: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 184: value assigned to `last_changes` is never read

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 71: variable does not need to be mutable

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\query\optimizer\optimizer_config.rs`: 1 occurrences

- Line 4: unused import: `std::collections::HashMap`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 514: unused variable: `input_result`: help: if this is intentional, prefix it with an underscore: `_input_result`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 522: unused import: `crate::query::executor::traits::ExecutorStats`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 111: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 180: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 305: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\query\scheduler\execution_plan_analyzer.rs`: 1 occurrences

- Line 109: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`

