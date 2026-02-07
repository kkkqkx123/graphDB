# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 121
- **Total Issues**: 121
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 69
- **Files with Issues**: 47

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 121

### Warning Type Breakdown

- **warning**: 121 warnings

### Files with Warnings (Top 10)

- `src\storage\redb_storage.rs`: 23 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 19 warnings
- `src\query\context\symbol\symbol_table.rs`: 7 warnings
- `src\query\executor\result_processing\projection.rs`: 5 warnings
- `src\api\service\graph_service.rs`: 5 warnings
- `src\query\optimizer\elimination_rules.rs`: 4 warnings
- `src\query\executor\graph_query_executor.rs`: 4 warnings
- `src\query\visitor\plan_node_visitor.rs`: 3 warnings
- `src\core\vertex_edge_path.rs`: 3 warnings
- `src\query\optimizer\engine\optimizer.rs`: 3 warnings

## Detailed Warning Categorization

### warning: unused import: `UpdateTarget`

**Total Occurrences**: 121  
**Unique Files**: 47

#### `src\storage\redb_storage.rs`: 23 occurrences

- Line 5: unused import: `UpdateTarget`
- Line 129: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 129: unused variable: `vertex_id`: help: if this is intentional, prefix it with an underscore: `_vertex_id`
- ... 20 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 19 occurrences

- Line 54: unused variable: `filter_condition`: help: if this is intentional, prefix it with an underscore: `_filter_condition`
- Line 61: unused variable: `new_child_node`: help: if this is intentional, prefix it with an underscore: `_new_child_node`
- Line 61: variable does not need to be mutable
- ... 16 more occurrences in this file

#### `src\query\context\symbol\symbol_table.rs`: 7 occurrences

- Line 161: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 173: unused variable: `symbol`: help: if this is intentional, prefix it with an underscore: `_symbol`
- Line 196: variable does not need to be mutable
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\projection.rs`: 5 occurrences

- Line 321: unused imports: `ExecutionResult` and `Executor`
- Line 334: variable does not need to be mutable
- Line 370: variable does not need to be mutable
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

#### `src\query\executor\graph_query_executor.rs`: 4 occurrences

- Line 334: unused import: `PasswordInfo`
- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 36: field `thread_pool` is never read
- ... 1 more occurrences in this file

#### `src\query\visitor\plan_node_visitor.rs`: 3 occurrences

- Line 413: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`
- Line 414: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`
- Line 415: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\optimizer\rule_enum.rs`: 3 occurrences

- Line 81: unreachable pattern: no value can reach this
- Line 128: unreachable pattern: no value can reach this
- Line 176: unreachable pattern: no value can reach this

#### `src\query\optimizer\engine\optimizer.rs`: 3 occurrences

- Line 555: value assigned to `last_changes` is never read
- Line 658: unused variable: `node_id`: help: if this is intentional, prefix it with an underscore: `_node_id`
- Line 636: unused variable: `root_group`: help: if this is intentional, prefix it with an underscore: `_root_group`

#### `src\core\vertex_edge_path.rs`: 3 occurrences

- Line 268: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 272: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`
- Line 378: unused variable: `v`: help: if this is intentional, prefix it with an underscore: `_v`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\query\optimizer\index_optimization.rs`: 2 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 731: methods `optimize_union_all_index_scans`, `try_merge_index_scans`, `are_index_scans_mergeable`, and `reorder_index_scans` are never used

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 150: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 178: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 87: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 88: unused variable: `boundary`: help: if this is intentional, prefix it with an underscore: `_boundary`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\query\validator\insert_vertices_validator.rs`: 2 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 12: field `base` is never read

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 47: unused import: `ChangePasswordExecutor`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 21: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 314: fields `space_id`, `tag_id`, `index_id`, `scan_limits`, and `return_columns` are never read

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 136: variant `NestedLoopJoin` is never constructed

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 362: field `condition` is never read

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 20: field `default_limit` is never read

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 8: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 567: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 277: method `InvalidPlanNode` should have a snake case name: help: convert the identifier to snake case: `invalid_plan_node`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\storage\metadata\extended_schema.rs`: 1 occurrences

- Line 50: method `save_schema_snapshot` is never used

#### `src\storage\processor\base.rs`: 1 occurrences

- Line 544: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 293: fields `vertex_cache` and `edge_cache` are never read

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\core\result\builder.rs`: 1 occurrences

- Line 18: field `capacity` is never read

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 67: field `max_size` is never read

