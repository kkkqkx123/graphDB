# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 208
- **Total Issues**: 208
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 99
- **Files with Issues**: 102

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 208

### Warning Type Breakdown

- **warning**: 208 warnings

### Files with Warnings (Top 10)

- `src\storage\redb_storage.rs`: 40 warnings
- `src\query\optimizer\operation_merge.rs`: 16 warnings
- `src\query\optimizer\limit_pushdown.rs`: 12 warnings
- `src\query\context\symbol\symbol_table.rs`: 10 warnings
- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 7 warnings
- `src\query\optimizer\projection_pushdown.rs`: 5 warnings
- `src\api\service\graph_service.rs`: 5 warnings
- `src\query\optimizer\elimination_rules.rs`: 4 warnings
- `src\query\optimizer\engine\optimizer.rs`: 4 warnings

## Detailed Warning Categorization

### warning: unused imports: `Arc` and `Mutex`

**Total Occurrences**: 208  
**Unique Files**: 76

#### `src\storage\redb_storage.rs`: 40 occurrences

- Line 3: unused import: `crate::core::vertex_edge_path::Tag`
- Line 11: unused import: `FieldDef`
- Line 12: unused imports: `edge_type_info_to_schema` and `tag_info_to_schema`
- ... 37 more occurrences in this file

#### `src\query\optimizer\operation_merge.rs`: 16 occurrences

- Line 129: unused variable: `ctx`
- Line 229: unused variable: `node_ref`
- Line 225: unused variable: `ctx`
- ... 13 more occurrences in this file

#### `src\query\optimizer\limit_pushdown.rs`: 12 occurrences

- Line 46: unused variable: `input_id`
- Line 197: unused variable: `node_ref`
- Line 193: unused variable: `ctx`
- ... 9 more occurrences in this file

#### `src\query\context\symbol\symbol_table.rs`: 10 occurrences

- Line 161: unused variable: `symbol`
- Line 163: variable does not need to be mutable
- Line 173: unused variable: `symbol`
- ... 7 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 8 occurrences

- Line 319: unused import: `DataSet`
- Line 321: unused import: `crate::query::executor::executor_enum::ExecutorEnum`
- Line 322: unused import: `crate::query::executor::base::BaseExecutor`
- ... 5 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 7 occurrences

- Line 198: unused variable: `ctx`
- Line 727: unused variable: `node_ref`
- Line 723: unused variable: `ctx`
- ... 4 more occurrences in this file

#### `src\api\service\graph_service.rs`: 5 occurrences

- Line 8: unused import: `crate::utils::safe_lock`
- Line 336: variable does not need to be mutable
- Line 375: variable does not need to be mutable
- ... 2 more occurrences in this file

#### `src\query\optimizer\projection_pushdown.rs`: 5 occurrences

- Line 129: unused variable: `ctx`
- Line 131: unused variable: `child`
- Line 200: unused variable: `node_ref`
- ... 2 more occurrences in this file

#### `src\query\optimizer\engine\optimizer.rs`: 4 occurrences

- Line 416: unreachable pattern
- Line 470: value assigned to `last_changes` is never read
- Line 576: unused variable: `node_id`
- ... 1 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 4 occurrences

- Line 90: variable does not need to be mutable
- Line 429: variable does not need to be mutable
- Line 624: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 3 occurrences

- Line 138: unused variable: `id`
- Line 36: field `thread_pool` is never read
- Line 104: multiple methods are never used

#### `src\core\vertex_edge_path.rs`: 3 occurrences

- Line 268: unused variable: `v`
- Line 272: unused variable: `v`
- Line 378: unused variable: `v`

#### `src\storage\iterator\composite.rs`: 3 occurrences

- Line 120: unused variable: `idx`
- Line 141: unused variable: `row`
- Line 705: variable does not need to be mutable

#### `src\query\executor\batch.rs`: 2 occurrences

- Line 5: unused import: `async_trait::async_trait`
- Line 8: unused import: `mpsc`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 36: unused variable: `ast_ctx`
- Line 20: field `default_limit` is never read

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing
- Line 268: function cannot return without recursing

#### `src\storage\processor\base.rs`: 2 occurrences

- Line 153: unused import: `crate::storage::index::IndexType`
- Line 526: unused variable: `counters`

#### `src\query\optimizer\index_optimization.rs`: 2 occurrences

- Line 25: unused variable: `ctx`
- Line 731: methods `optimize_union_all_index_scans`, `try_merge_index_scans`, `are_index_scans_mergeable`, and `reorder_index_scans` are never used

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\query\planner\planner.rs`: 2 occurrences

- Line 210: unused variable: `query_context`
- Line 67: field `max_size` is never read

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\storage\transaction\wal.rs`: 2 occurrences

- Line 476: unused variable: `flushed`
- Line 474: unused variable: `min_lsn`

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 303: unreachable pattern
- Line 568: unused variable: `planner`

#### `src\query\executor\data_modification.rs`: 2 occurrences

- Line 7: unused import: `StorageError`
- Line 362: field `condition` is never read

#### `src\core\codec\row_buffer.rs`: 2 occurrences

- Line 6: unused imports: `CodecError` and `Result`
- Line 14: field `schema` is never read

#### `src\query\validator\insert_vertices_validator.rs`: 2 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 12: field `base` is never read

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`
- Line 177: unused variable: `variable`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`
- Line 531: unused variable: `idx`

#### `src\storage\iterator\predicate.rs`: 2 occurrences

- Line 161: unused variable: `col_name`
- Line 492: unused variable: `pred2`

#### `src\core\codec\mod.rs`: 2 occurrences

- Line 65: unused import: `crate::storage::types::FieldDef`
- Line 66: unused import: `crate::core::DataType`

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing
- Line 543: function cannot return without recursing

#### `src\core\codec\key_utils.rs`: 2 occurrences

- Line 132: unused variable: `vid_len`
- Line 11: associated constants `KEY_TYPE_SYSTEM` and `KEY_TYPE_INDEX` are never used

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing
- Line 611: function cannot return without recursing

#### `src\query\executor\search_executors.rs`: 2 occurrences

- Line 357: value assigned to `vertices` is never read
- Line 314: fields `space_id`, `tag_id`, `index_id`, `scan_limits`, and `return_columns` are never read

#### `src\storage\operations\redb_operations.rs`: 2 occurrences

- Line 3: unused import: `INDEXES_TABLE`
- Line 293: fields `vertex_cache` and `edge_cache` are never read

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing
- Line 463: function cannot return without recursing

#### `src\query\planner\plan\core\nodes\join_node.rs`: 2 occurrences

- Line 1056: unused variable: `l`
- Line 1057: unused variable: `r`

#### `src\storage\engine\redb_engine.rs`: 1 occurrences

- Line 6: unused imports: `Arc` and `Mutex`

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 9: unused import: `IndexStatus`

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 281: method `InvalidPlanNode` should have a snake case name

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`

#### `src\api\mod.rs`: 1 occurrences

- Line 2: unused imports: `error` and `warn`

#### `src\core\codec\field_accessor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::Value`

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\storage\metadata\schema_manager.rs`: 1 occurrences

- Line 5: unused import: `FieldDef`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`

#### `src\core\result\builder.rs`: 1 occurrences

- Line 18: field `capacity` is never read

#### `src\storage\metadata\redb_metadata.rs`: 1 occurrences

- Line 3: unused imports: `DataType` and `FieldDef`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\executor\base\storage_processor_executor.rs`: 1 occurrences

- Line 339: unused import: `crate::storage::index::IndexType`

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 353: unreachable pattern

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`

#### `src\storage\metadata\extended_schema.rs`: 1 occurrences

- Line 50: method `save_schema_snapshot` is never used

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\storage\schema.rs`: 1 occurrences

- Line 6: unused import: `super::DataType`

#### `src\core\codec\row_writer.rs`: 1 occurrences

- Line 4: unused import: `crate::storage::types::FieldDef`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 9: unused import: `IndexStatus`

#### `src\core\codec\row_reader.rs`: 1 occurrences

- Line 8: unused imports: `DateTimeValue`, `DateValue`, and `TimeValue`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 26: unused import: `crate::query::planner::statements::seeks::IndexInfo`

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 571: unused import: `crate::query::planner::statements::seeks::IndexInfo`

