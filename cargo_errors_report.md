# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 74
- **Total Issues**: 74
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 49
- **Files with Issues**: 58

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 74

### Warning Type Breakdown

- **warning**: 74 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 warnings
- `src\query\optimizer\rule_enum.rs`: 3 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 3 warnings
- `src\query\validator\insert_vertices_validator.rs`: 2 warnings
- `src\storage\redb_storage.rs`: 2 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings
- `src\query\optimizer\index_optimization.rs`: 2 warnings
- `src\query\optimizer\rules\mod.rs`: 2 warnings
- `src\query\executor\data_access.rs`: 2 warnings
- `src\core\types\expression\visitor.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

**Total Occurrences**: 74  
**Unique Files**: 58

#### `src\query\optimizer\rule_enum.rs`: 3 occurrences

- Line 81: unreachable pattern: no value can reach this
- Line 128: unreachable pattern: no value can reach this
- Line 176: unreachable pattern: no value can reach this

#### `src\query\optimizer\predicate_pushdown.rs`: 3 occurrences

- Line 615: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 723: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 901: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 occurrences

- Line 14: unused import: `crate::query::planner::plan::algorithms::IndexScan`
- Line 119: variable does not need to be mutable
- Line 137: variable does not need to be mutable

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 373: unused import: `PasswordInfo`
- Line 105: multiple methods are never used

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 965: variable does not need to be mutable
- Line 1013: variable does not need to be mutable

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 87: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 88: unused variable: `boundary`: help: if this is intentional, prefix it with an underscore: `_boundary`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 150: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 178: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\optimizer\index_optimization.rs`: 2 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 731: methods `optimize_union_all_index_scans`, `try_merge_index_scans`, `are_index_scans_mergeable`, and `reorder_index_scans` are never used

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 2 occurrences

- Line 133: unused variable: `op`: help: try ignoring the field: `op: _`
- Line 149: unused variable: `op`: help: try ignoring the field: `op: _`

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 22: field `id_generator` is never read
- Line 227: associated items `update_edge_property` and `detect_format` are never used

#### `src\query\validator\insert_vertices_validator.rs`: 2 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 12: field `base` is never read

#### `src\query\optimizer\rules\mod.rs`: 2 occurrences

- Line 36: unused import: `predicate_pushdown::*`
- Line 40: unused import: `index::*`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_index_scan.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\optimizer\rules\elimination\eliminate_row_collect.rs`: 1 occurrences

- Line 4: unused import: `crate::query::optimizer::rule_patterns::PatternBuilder`

#### `src\core\result\builder.rs`: 1 occurrences

- Line 18: field `capacity` is never read

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 20: field `default_limit` is never read

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 952: function `create_plan_node_with_output_var` is never used

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 289: method `InvalidPlanNode` should have a snake case name: help: convert the identifier to snake case: `invalid_plan_node`

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\transformation\top_n.rs`: 1 occurrences

- Line 94: variable does not need to be mutable

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 567: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\optimizer\rules\merge\collapse_project.rs`: 1 occurrences

- Line 4: unused import: `PatternBuilder`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 21: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 475: variable does not need to be mutable

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 362: field `condition` is never read

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 67: field `max_size` is never read

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 314: fields `space_id`, `tag_id`, `index_id`, `scan_limits`, and `return_columns` are never read

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 136: variant `NestedLoopJoin` is never constructed

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\api\service\graph_service.rs`: 1 occurrences

- Line 104: type `TxState` is more private than the item `graph_service::TransactionManager::get_transaction_state`: method `graph_service::TransactionManager::get_transaction_state` is reachable at visibility `pub`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\storage\processor\base.rs`: 1 occurrences

- Line 544: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\query\optimizer\rules\merge\combine_filter.rs`: 1 occurrences

- Line 4: unused import: `PatternBuilder`

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 293: fields `vertex_cache` and `edge_cache` are never read

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 844: method `explore_node` is never used

#### `src\storage\metadata\extended_schema.rs`: 1 occurrences

- Line 50: method `save_schema_snapshot` is never used

#### `src\query\optimizer\rules\join\join_optimization.rs`: 1 occurrences

- Line 140: variant `NestedLoopJoin` is never constructed

#### `src\query\optimizer\rules\elimination\eliminate_append_vertices.rs`: 1 occurrences

- Line 4: unused import: `crate::query::optimizer::rule_patterns::PatternBuilder`

#### `src\query\optimizer\rules\aggregate\push_filter_down_aggregate.rs`: 1 occurrences

- Line 35: unused import: `crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 8: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

