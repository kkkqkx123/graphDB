# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 176
- **Total Issues**: 177
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 74
- **Files with Issues**: 64

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0308]**: 1 errors

### Files with Errors (Top 10)

- `src\storage\metadata\extended_schema.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 176

### Warning Type Breakdown

- **warning**: 176 warnings

### Files with Warnings (Top 10)

- `src\storage\redb_storage.rs`: 36 warnings
- `src\query\optimizer\operation_merge.rs`: 16 warnings
- `src\query\optimizer\limit_pushdown.rs`: 12 warnings
- `src\query\context\symbol\symbol_table.rs`: 10 warnings
- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 7 warnings
- `src\query\optimizer\projection_pushdown.rs`: 5 warnings
- `src\api\service\graph_service.rs`: 4 warnings
- `src\query\optimizer\engine\optimizer.rs`: 4 warnings
- `src\core\vertex_edge_path.rs`: 3 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `i64`, found `i32`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\metadata\extended_schema.rs`: 1 occurrences

- Line 300: mismatched types: expected `i64`, found `i32`

## Detailed Warning Categorization

### warning: unreachable pattern: no value can reach this

**Total Occurrences**: 176  
**Unique Files**: 63

#### `src\storage\redb_storage.rs`: 36 occurrences

- Line 3: unused import: `crate::core::vertex_edge_path::Tag`
- Line 11: unused import: `FieldDef`
- Line 12: unused imports: `edge_type_info_to_schema` and `tag_info_to_schema`
- ... 33 more occurrences in this file

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
- Line 707: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 703: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- ... 4 more occurrences in this file

#### `src\query\optimizer\projection_pushdown.rs`: 5 occurrences

- Line 136: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 138: unused variable: `child`: help: if this is intentional, prefix it with an underscore: `_child`
- Line 207: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- ... 2 more occurrences in this file

#### `src\api\service\graph_service.rs`: 4 occurrences

- Line 8: unused import: `crate::utils::safe_lock`
- Line 336: variable does not need to be mutable
- Line 375: variable does not need to be mutable
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

#### `src\query\optimizer\elimination_rules.rs`: 3 occurrences

- Line 90: variable does not need to be mutable
- Line 429: variable does not need to be mutable
- Line 624: variable does not need to be mutable

#### `src\query\executor\batch.rs`: 2 occurrences

- Line 5: unused import: `async_trait::async_trait`
- Line 8: unused import: `mpsc`

#### `src\storage\iterator\predicate.rs`: 2 occurrences

- Line 161: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`
- Line 492: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\query\planner\plan\core\nodes\join_node.rs`: 2 occurrences

- Line 1056: unused variable: `l`: help: if this is intentional, prefix it with an underscore: `_l`
- Line 1057: unused variable: `r`: help: if this is intentional, prefix it with an underscore: `_r`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\storage\processor\base.rs`: 2 occurrences

- Line 159: unused import: `crate::storage::index::IndexType`
- Line 532: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\query\planner\statements\match_planner.rs`: 2 occurrences

- Line 303: unreachable pattern: no value can reach this
- Line 568: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\storage\iterator\composite.rs`: 2 occurrences

- Line 120: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`
- Line 141: unused variable: `row`: help: if this is intentional, prefix it with an underscore: `_row`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\storage\transaction\wal.rs`: 2 occurrences

- Line 476: unused variable: `flushed`: help: if this is intentional, prefix it with an underscore: `_flushed`
- Line 474: unused variable: `min_lsn`: help: if this is intentional, prefix it with an underscore: `_min_lsn`

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 353: unreachable pattern: no value can reach this

#### `src\storage\serializer\transaction_serializer.rs`: 1 occurrences

- Line 1: unused import: `crate::core::StorageError`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\storage\engine\redb_engine.rs`: 1 occurrences

- Line 6: unused imports: `Arc` and `Mutex`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 571: unused import: `crate::query::planner::statements::seeks::IndexInfo`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 3: unused import: `INDEXES_TABLE`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\executor\admin\index\edge_index.rs`: 1 occurrences

- Line 9: unused import: `IndexStatus`

#### `src\storage\schema.rs`: 1 occurrences

- Line 6: unused import: `super::DataType`

#### `src\storage\test_mock.rs`: 1 occurrences

- Line 26: unused import: `crate::query::planner::statements::seeks::IndexInfo`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 484: unused import: `crate::storage::redb_storage::DefaultStorage`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\api\mod.rs`: 1 occurrences

- Line 2: unused imports: `error` and `warn`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\base\storage_processor_executor.rs`: 1 occurrences

- Line 340: unused import: `crate::storage::index::IndexType`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 357: value assigned to `vertices` is never read

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 210: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 21: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\storage\metadata\schema_manager.rs`: 1 occurrences

- Line 5: unused import: `FieldDef`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 7: unused import: `StorageError`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\admin\index\tag_index.rs`: 1 occurrences

- Line 9: unused import: `IndexStatus`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 204: unused import: `crate::core::Value`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\storage\metadata\redb_metadata.rs`: 1 occurrences

- Line 3: unused imports: `DataType` and `FieldDef`

