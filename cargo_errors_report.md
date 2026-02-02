# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 154
- **Total Issues**: 154
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 75
- **Files with Issues**: 65

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 154

### Warning Type Breakdown

- **warning**: 154 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\operation_merge.rs`: 16 warnings
- `src\query\optimizer\limit_pushdown.rs`: 12 warnings
- `src\query\context\symbol\symbol_table.rs`: 10 warnings
- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 7 warnings
- `src\query\optimizer\projection_pushdown.rs`: 5 warnings
- `src\api\service\graph_service.rs`: 5 warnings
- `src\query\optimizer\engine\optimizer.rs`: 4 warnings
- `src\query\optimizer\elimination_rules.rs`: 4 warnings
- `src\storage\iterator\composite.rs`: 3 warnings

## Detailed Warning Categorization

### warning: unused import: `StorageError`

**Total Occurrences**: 154  
**Unique Files**: 65

#### `src\query\optimizer\operation_merge.rs`: 16 occurrences

- Line 129: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 229: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 225: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 13 more occurrences in this file

#### `src\query\optimizer\limit_pushdown.rs`: 12 occurrences

- Line 46: unused variable: `input_id`: help: if this is intentional, prefix it with an underscore: `_input_id`
- Line 197: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 193: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 9 more occurrences in this file

#### `src\query\context\symbol\symbol_table.rs`: 10 occurrences

- Line 161: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 163: variable does not need to be mutable
- Line 173: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- ... 7 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 8 occurrences

- Line 319: unused import: `DataSet`
- Line 321: unused import: `crate::query::executor::executor_enum::ExecutorEnum`
- Line 322: unused import: `crate::query::executor::base::BaseExecutor`
- ... 5 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 7 occurrences

- Line 198: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 727: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 723: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 4 more occurrences in this file

#### `src\query\optimizer\projection_pushdown.rs`: 5 occurrences

- Line 129: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 131: unused variable: `child`: help: if this is intentional, prefix it with an underscore: `_child`
- Line 200: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- ... 2 more occurrences in this file

#### `src\api\service\graph_service.rs`: 5 occurrences

- Line 8: unused import: `crate::utils::safe_lock`
- Line 336: variable does not need to be mutable
- Line 375: variable does not need to be mutable
- ... 2 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 4 occurrences

- Line 90: variable does not need to be mutable
- Line 429: variable does not need to be mutable
- Line 624: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\optimizer\engine\optimizer.rs`: 4 occurrences

- Line 416: unreachable pattern: no value can reach this
- Line 470: value assigned to `last_changes` is never read
- Line 576: unused variable: `node_id`: help: if this is intentional, prefix it with an underscore: `_node_id`
- ... 1 more occurrences in this file

#### `src\core\vertex_edge_path.rs`: 3 occurrences

- Line 268: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 272: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 378: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`

#### `src\query\executor\graph_query_executor.rs`: 3 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 36: field `thread_pool` is never read
- Line 104: multiple methods are never used

#### `src\storage\iterator\composite.rs`: 3 occurrences

- Line 120: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`
- Line 141: unused variable: `row`: help: if this is intentional, prefix it with an underscore: `_row`
- Line 705: variable does not need to be mutable

#### `src\query\executor\data_modification.rs`: 2 occurrences

- Line 7: unused import: `StorageError`
- Line 362: field `condition` is never read

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\query\executor\batch.rs`: 2 occurrences

- Line 5: unused import: `async_trait::async_trait`
- Line 8: unused import: `mpsc`

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\storage\mod.rs`: 2 occurrences

- Line 25: ambiguous glob re-exports: the name `ByteKey` in the type namespace is first re-exported here
- Line 25: ambiguous glob re-exports: the name `ByteKey` in the value namespace is first re-exported here

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`
- Line 20: field `default_limit` is never read

#### `src\query\validator\insert_vertices_validator.rs`: 2 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 12: field `base` is never read

#### `src\query\executor\search_executors.rs`: 2 occurrences

- Line 357: value assigned to `vertices` is never read
- Line 314: fields `space_id`, `tag_id`, `index_id`, `scan_limits`, and `return_columns` are never read

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\storage\processor\base.rs`: 2 occurrences

- Line 142: unused import: `crate::storage::index::IndexType`
- Line 515: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\storage\transaction\wal.rs`: 2 occurrences

- Line 461: unused variable: `flushed`: help: if this is intentional, prefix it with an underscore: `_flushed`
- Line 459: unused variable: `min_lsn`: help: if this is intentional, prefix it with an underscore: `_min_lsn`

#### `src\query\optimizer\index_optimization.rs`: 2 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 731: methods `optimize_union_all_index_scans`, `try_merge_index_scans`, `are_index_scans_mergeable`, and `reorder_index_scans` are never used

#### `src\query\planner\plan\core\nodes\join_node.rs`: 2 occurrences

- Line 1056: unused variable: `l`: help: if this is intentional, prefix it with an underscore: `_l`
- Line 1057: unused variable: `r`: help: if this is intentional, prefix it with an underscore: `_r`

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 303: unreachable pattern: no value can reach this
- Line 568: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\query\planner\planner.rs`: 2 occurrences

- Line 210: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 67: field `max_size` is never read

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\api\mod.rs`: 1 occurrences

- Line 2: unused imports: `error` and `warn`

#### `src\storage\metadata\extended_schema.rs`: 1 occurrences

- Line 50: method `save_schema_snapshot` is never used

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 223: field `gc_config` is never read

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\executor\base\storage_processor_executor.rs`: 1 occurrences

- Line 334: unused import: `crate::storage::index::IndexType`

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\storage\iterator\predicate.rs`: 1 occurrences

- Line 486: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\core\result\builder.rs`: 1 occurrences

- Line 18: field `capacity` is never read

#### `src\storage\engine\memory_engine.rs`: 1 occurrences

- Line 19: field `snapshot` is never read

#### `src\storage\redb_storage.rs`: 1 occurrences

- Line 398: method `get_vertices_by_prop` is never used

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 281: method `InvalidPlanNode` should have a snake case name: help: convert the identifier to snake case: `invalid_plan_node`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\storage\engine\redb_engine.rs`: 1 occurrences

- Line 40: constant `SNAPSHOTS_TABLE` is never used

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 353: unreachable pattern: no value can reach this

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

