# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 128
- **Total Issues**: 128
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 88
- **Files with Issues**: 64

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 128

### Warning Type Breakdown

- **warning**: 128 warnings

### Files with Warnings (Top 10)

- `src\query\planner\planner.rs`: 11 warnings
- `src\query\optimizer\elimination_rules.rs`: 8 warnings
- `src\query\planner\statements\match_statement_planner.rs`: 6 warnings
- `src\query\executor\result_processing\projection.rs`: 6 warnings
- `src\query\planner\statements\match_planner.rs`: 5 warnings
- `src\query\executor\graph_query_executor.rs`: 4 warnings
- `src\query\executor\factory.rs`: 4 warnings
- `src\query\parser\lexer\lexer.rs`: 4 warnings
- `src\query\planner\statements\paths\match_path_planner.rs`: 4 warnings
- `src\query\planner\mod.rs`: 3 warnings

## Detailed Warning Categorization

### warning: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

**Total Occurrences**: 128  
**Unique Files**: 64

#### `src\query\planner\planner.rs`: 11 occurrences

- Line 11: unused import: `Arc`
- Line 165: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 166: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- ... 8 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 8 occurrences

- Line 8: unused import: `EliminationRule`
- Line 12: unused import: `crate::query::planner::plan::ProjectNode`
- Line 87: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- ... 5 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 6 occurrences

- Line 323: unused import: `crate::query::executor::HasStorage`
- Line 324: unused import: `ExecutorStats`
- Line 438: unused variable: `vertex1`: help: if this is intentional, prefix it with an underscore: `_vertex1`
- ... 3 more occurrences in this file

#### `src\query\planner\statements\match_statement_planner.rs`: 6 occurrences

- Line 14: unused import: `crate::query::planner::connector::SegmentsConnector`
- Line 26: unused import: `std::collections::HashMap`
- Line 21: unused import: `ClausePlanner`
- ... 3 more occurrences in this file

#### `src\query\planner\statements\match_planner.rs`: 5 occurrences

- Line 26: unused import: `std::collections::HashMap`
- Line 75: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 290: unreachable pattern: no value can reach this
- ... 2 more occurrences in this file

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 23: unused imports: `MultiShortestPathExecutor` and `ShortestPathExecutor`
- Line 48: unused imports: `EdgeAlterInfo`, `EdgeManageInfo`, `IndexManageInfo`, `SpaceManageInfo`, `TagAlterInfo`, and `TagManageInfo`
- Line 928: unused import: `AlterTagOp`
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

#### `src\query\executor\graph_query_executor.rs`: 4 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable
- Line 36: field `thread_pool` is never read
- ... 1 more occurrences in this file

#### `src\query\planner\mod.rs`: 3 occurrences

- Line 16: use of deprecated type alias `query::planner::planner::ConfigurablePlannerRegistry`: 请使用 StaticConfigurablePlannerRegistry 替代
- Line 17: use of deprecated type alias `query::planner::planner::PlannerRegistry`: 请使用 StaticPlannerRegistry 替代
- Line 17: use of deprecated type alias `query::planner::planner::SequentialPlanner`: 请使用 StaticSequentialPlanner 替代

#### `src\query\parser\parser\expr_parser.rs`: 3 occurrences

- Line 8: unused import: `AggregateFunction`
- Line 9: unused imports: `BinaryOp` and `UnaryOp`
- Line 451: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 occurrences

- Line 23: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`
- Line 461: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 467: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\query\planner\statements\seeks\index_seek.rs`: 3 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`
- Line 7: unused import: `Value`
- Line 94: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 3 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`
- Line 7: unused import: `Value`
- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\api\service\index_service.rs`: 2 occurrences

- Line 504: unused `std::result::Result` that must be used
- Line 520: unused `std::result::Result` that must be used

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 207: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 207: variable does not need to be mutable

#### `src\query\validator\base_validator.rs`: 2 occurrences

- Line 228: calls to `std::mem::drop` with a reference instead of an owned value does nothing
- Line 248: calls to `std::mem::drop` with a reference instead of an owned value does nothing

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`
- Line 20: field `default_limit` is never read

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 2 occurrences

- Line 7: unused import: `Vertex`
- Line 192: method `mark_termination` is never used

#### `src\query\executor\logic\loops.rs`: 2 occurrences

- Line 526: struct `CountExecutor` is never constructed
- Line 534: associated function `new` is never used

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`
- Line 55: unused variable: `match_expression`: help: if this is intentional, prefix it with an underscore: `_match_expression`

#### `src\query\optimizer\constant_folding.rs`: 2 occurrences

- Line 6: unused import: `super::rule_patterns::PatternBuilder`
- Line 63: unused import: `crate::core::Expression`

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 5: unused import: `SeekStrategyTraitObject`
- Line 134: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\optimizer\loop_unrolling.rs`: 2 occurrences

- Line 71: variable does not need to be mutable
- Line 344: associated function `is_simple_loop_body` is never used

#### `src\query\executor\executor_enum.rs`: 2 occurrences

- Line 11: unused imports: `Arc` and `Mutex`
- Line 26: unused import: `ExecutionContext`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 15: unused import: `std::collections::HashMap`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 111: function cannot return without recursing: cannot return without recursing

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\scheduler\execution_plan_analyzer.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`

#### `src\core\value\comparison.rs`: 1 occurrences

- Line 403: associated functions `cmp_coordinate_list` and `cmp_polygon_list` are never used

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\planner\statements\seeks\seek_strategy_base.rs`: 1 occurrences

- Line 6: unused import: `StorageError`

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 1 occurrences

- Line 10: unused macro definition: `impl_graph_traversal_executor`

#### `src\query\validator\validation_factory.rs`: 1 occurrences

- Line 8: unused import: `super::validation_interface::ValidationStrategyType`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 1 occurrences

- Line 11: unused imports: `IndexInfo` and `NodePattern`

#### `src\query\executor\admin\data\update.rs`: 1 occurrences

- Line 8: unused imports: `UpdateOp` and `UpdateTarget`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\executor\admin\space\create_space.rs`: 1 occurrences

- Line 8: unused import: `Value`

#### `src\query\optimizer\subquery_optimization.rs`: 1 occurrences

- Line 6: unused import: `super::rule_patterns::PatternBuilder`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 1: unused import: `crate::core::error::DBError`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 305: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 7: unused import: `ExpressionVisitor`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 97: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 170: method `clear` is never used

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 6: unused imports: `Edge` and `Vertex`

#### `src\query\parser\expressions\mod.rs`: 1 occurrences

- Line 5: unused import: `Expression`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 27: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\executor\special_executors.rs`: 1 occurrences

- Line 4: unused import: `DBError`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\admin\mod.rs`: 1 occurrences

- Line 13: unused import: `crate::storage::StorageEngine`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 7: unused import: `Planner`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 514: unused variable: `input_result`: help: if this is intentional, prefix it with an underscore: `_input_result`

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 180: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

