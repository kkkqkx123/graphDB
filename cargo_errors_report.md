# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 145
- **Total Issues**: 145
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 93
- **Files with Issues**: 70

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 145

### Warning Type Breakdown

- **warning**: 145 warnings

### Files with Warnings (Top 10)

- `src\storage\plan\nodes\mod.rs`: 8 warnings
- `src\query\optimizer\elimination_rules.rs`: 8 warnings
- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\context\runtime_context.rs`: 5 warnings
- `src\query\executor\graph_query_executor.rs`: 4 warnings
- `src\query\validator\insert_vertices_validator.rs`: 4 warnings
- `src\query\planner\statements\match_planner.rs`: 4 warnings
- `src\storage\iterator\composite.rs`: 4 warnings
- `src\query\executor\search_executors.rs`: 3 warnings
- `src\query\planner\statements\paths\match_path_planner.rs`: 3 warnings

## Detailed Warning Categorization

### warning: multiple methods are never used

**Total Occurrences**: 145  
**Unique Files**: 70

#### `src\query\optimizer\elimination_rules.rs`: 8 occurrences

- Line 87: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 171: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 316: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- ... 5 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 8 occurrences

- Line 319: unused import: `DataSet`
- Line 321: unused import: `crate::query::executor::executor_enum::ExecutorEnum`
- Line 322: unused import: `crate::query::executor::base::BaseExecutor`
- ... 5 more occurrences in this file

#### `src\storage\plan\nodes\mod.rs`: 8 occurrences

- Line 2: unused imports: `Edge` and `Vertex`
- Line 15: field `schema` is never read
- Line 32: field `output_schema` is never read
- ... 5 more occurrences in this file

#### `src\query\context\runtime_context.rs`: 5 occurrences

- Line 9: unused import: `crate::core::Value`
- Line 10: unused import: `crate::core::EdgeDirection`
- Line 12: unused import: `crate::query::planner::plan::management::ddl::space_ops::Schema`
- ... 2 more occurrences in this file

#### `src\storage\iterator\composite.rs`: 4 occurrences

- Line 120: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`
- Line 141: unused variable: `row`: help: if this is intentional, prefix it with an underscore: `_row`
- Line 120: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 4 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable
- Line 36: field `thread_pool` is never read
- ... 1 more occurrences in this file

#### `src\query\validator\insert_vertices_validator.rs`: 4 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 48: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 79: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\match_planner.rs`: 4 occurrences

- Line 75: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 296: unreachable pattern: no value can reach this
- Line 470: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\paths\match_path_planner.rs`: 3 occurrences

- Line 433: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 439: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 461: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\storage\iterator\predicate.rs`: 3 occurrences

- Line 10: unused import: `std::any::Any`
- Line 11: unused import: `std::collections::HashMap`
- Line 488: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\query\validator\update_validator.rs`: 3 occurrences

- Line 34: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 168: unused variable: `op`: help: try ignoring the field: `op: _`
- Line 13: field `base` is never read

#### `src\query\validator\insert_edges_validator.rs`: 3 occurrences

- Line 53: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 84: unused variable: `edge_name`: help: if this is intentional, prefix it with an underscore: `_edge_name`
- Line 12: field `base` is never read

#### `src\storage\transaction\log.rs`: 3 occurrences

- Line 15: unused import: `self`
- Line 460: unused variable: `flushed`: help: if this is intentional, prefix it with an underscore: `_flushed`
- Line 458: unused variable: `min_lsn`: help: if this is intentional, prefix it with an underscore: `_min_lsn`

#### `src\query\executor\search_executors.rs`: 3 occurrences

- Line 13: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 358: value assigned to `vertices` is never read
- Line 315: fields `space_id`, `tag_id`, `index_id`, `scan_limits`, and `return_columns` are never read

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 occurrences

- Line 24: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`
- Line 479: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 485: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\storage\plan\executors\mod.rs`: 3 occurrences

- Line 1: unused import: `ColumnSchema`
- Line 2: unused imports: `StorageError` and `Vertex`
- Line 3: unused import: `ScanResult`

#### `src\storage\transaction\traits.rs`: 2 occurrences

- Line 10: unused import: `Value`
- Line 11: unused imports: `LockManager`, `LogRecord`, `TransactionLog`, and `VersionVec`

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`
- Line 55: unused variable: `match_expression`: help: if this is intentional, prefix it with an underscore: `_match_expression`

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 86: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 353: unreachable pattern: no value can reach this

#### `src\storage\transaction\lock.rs`: 2 occurrences

- Line 11: unused import: `crate::core::StorageError`
- Line 297: field `config` is never read

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`
- Line 20: field `default_limit` is never read

#### `src\core\result\builder.rs`: 2 occurrences

- Line 2: unused import: `ResultMeta`
- Line 18: field `capacity` is never read

#### `src\expression\storage\row_reader.rs`: 2 occurrences

- Line 313: unreachable pattern: no value can reach this
- Line 326: unreachable pattern: no value can reach this

#### `src\api\service\index_service.rs`: 2 occurrences

- Line 504: unused `std::result::Result` that must be used
- Line 520: unused `std::result::Result` that must be used

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\storage\iterator\storage_iter.rs`: 2 occurrences

- Line 10: unused imports: `Edge`, `Value`, and `Vertex`
- Line 11: unused import: `std::sync::Arc`

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\query\planner\planner.rs`: 2 occurrences

- Line 191: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 67: field `max_size` is never read

#### `src\storage\transaction\mvcc.rs`: 2 occurrences

- Line 11: unused import: `TransactionState`
- Line 223: field `gc_config` is never read

#### `src\storage\memory_storage.rs`: 2 occurrences

- Line 1: unused import: `SchemaManager`
- Line 9: unused import: `RowReaderWrapper`

#### `src\storage\transaction\snapshot.rs`: 2 occurrences

- Line 9: unused import: `LockType`
- Line 290: unused variable: `key_lock`: help: if this is intentional, prefix it with an underscore: `_key_lock`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\query\validator\delete_validator.rs`: 2 occurrences

- Line 32: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 13: field `base` is never read

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 305: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 50: unused import: `SpaceManageInfo`

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 71: variable does not need to be mutable

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\context\managers\impl\storage_client_impl.rs`: 1 occurrences

- Line 4: unused imports: `DelTags`, `EdgeKey`, `ExecResponse`, `NewEdge`, `NewVertex`, `UpdateResponse`, and `UpdatedProp`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\storage\redb_storage.rs`: 1 occurrences

- Line 408: method `get_vertices_by_prop` is never used

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 184: value assigned to `last_changes` is never read

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 100: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\scheduler\execution_plan_analyzer.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`

#### `src\query\visitor\ast_transformer.rs`: 1 occurrences

- Line 8: unused imports: `AlterStmt`, `Assignment`, `ChangePasswordStmt`, `CreateStmt`, `DeleteStmt`, `DescStmt`, `DropStmt`, `ExplainStmt`, `FetchStmt`, `FindPathStmt`, `GoStmt`, `InsertStmt`, `LookupStmt`, `MatchStmt`, `MergeStmt`, `PipeStmt`, `QueryStmt`, `RemoveStmt`, `ReturnStmt`, `SetStmt`, `ShowStmt`, `Stmt`, `SubgraphStmt`, `UnwindStmt`, `UpdateStmt`, `UseStmt`, and `WithStmt`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\core\result\iterator.rs`: 1 occurrences

- Line 2: unused import: `crate::core::DBResult`

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 180: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\storage\engine\redb_engine.rs`: 1 occurrences

- Line 40: constant `SNAPSHOTS_TABLE` is never used

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 43: unused variable: `input_id`: help: if this is intentional, prefix it with an underscore: `_input_id`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\expression\context\traits.rs`: 1 occurrences

- Line 5: unused import: `crate::core::error::ExpressionError`

#### `src\storage\metadata\schema_manager.rs`: 1 occurrences

- Line 6: unused import: `IndexInfo`

#### `src\storage\engine\memory_engine.rs`: 1 occurrences

- Line 19: field `snapshot` is never read

