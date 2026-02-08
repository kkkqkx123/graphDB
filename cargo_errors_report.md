# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 94
- **Total Issues**: 94
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 57
- **Files with Issues**: 68

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 94

### Warning Type Breakdown

- **warning**: 94 warnings

### Files with Warnings (Top 10)

- `src\storage\test_mock.rs`: 9 warnings
- `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 3 warnings
- `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 warnings
- `src\query\optimizer\rule_enum.rs`: 3 warnings
- `src\query\optimizer\rules\predicate_pushdown\push_filter_down_cross_join.rs`: 3 warnings
- `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_left_join.rs`: 3 warnings
- `src\query\executor\result_processing\sort.rs`: 2 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings
- `src\query\executor\data_access.rs`: 2 warnings
- `src\query\executor\graph_query_executor.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::index::IndexStatus`

**Total Occurrences**: 94  
**Unique Files**: 67

#### `src\storage\test_mock.rs`: 9 occurrences

- Line 180: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 184: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 188: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- ... 6 more occurrences in this file

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_cross_join.rs`: 3 occurrences

- Line 97: unused variable: `left_filter_picked`: help: if this is intentional, prefix it with an underscore: `_left_filter_picked`
- Line 106: unused variable: `remaining_after_right`: help: if this is intentional, prefix it with an underscore: `_remaining_after_right`
- Line 111: unused variable: `right_filter_picked`: help: if this is intentional, prefix it with an underscore: `_right_filter_picked`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 3 occurrences

- Line 97: unused variable: `left_filter_picked`: help: if this is intentional, prefix it with an underscore: `_left_filter_picked`
- Line 106: unused variable: `remaining_after_right`: help: if this is intentional, prefix it with an underscore: `_remaining_after_right`
- Line 111: unused variable: `right_filter_picked`: help: if this is intentional, prefix it with an underscore: `_right_filter_picked`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_left_join.rs`: 3 occurrences

- Line 97: unused variable: `left_filter_picked`: help: if this is intentional, prefix it with an underscore: `_left_filter_picked`
- Line 106: unused variable: `remaining_after_right`: help: if this is intentional, prefix it with an underscore: `_remaining_after_right`
- Line 111: unused variable: `right_filter_picked`: help: if this is intentional, prefix it with an underscore: `_right_filter_picked`

#### `src\query\optimizer\rule_enum.rs`: 3 occurrences

- Line 77: unreachable pattern: no value can reach this
- Line 121: unreachable pattern: no value can reach this
- Line 166: unreachable pattern: no value can reach this

#### `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 occurrences

- Line 14: unused import: `crate::query::planner::plan::algorithms::IndexScan`
- Line 119: variable does not need to be mutable
- Line 137: variable does not need to be mutable

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 965: variable does not need to be mutable
- Line 1013: variable does not need to be mutable

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_join.rs`: 2 occurrences

- Line 95: unused variable: `filter_unpicked`: help: if this is intentional, prefix it with an underscore: `_filter_unpicked`
- Line 97: unused variable: `filter_picked`: help: if this is intentional, prefix it with an underscore: `_filter_picked`

#### `src\query\validator\insert_vertices_validator.rs`: 2 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 12: field `base` is never read

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 23: field `id_generator` is never read
- Line 228: associated items `update_edge_property` and `detect_format` are never used

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 388: unused import: `PasswordInfo`
- Line 115: multiple methods are never used

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 2 occurrences

- Line 133: unused variable: `op`: help: try ignoring the field: `op: _`
- Line 149: unused variable: `op`: help: try ignoring the field: `op: _`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 612: variable does not need to be mutable
- Line 580: variable does not need to be mutable

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 150: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 178: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\query\executor\admin\index\show_edge_index_status.rs`: 1 occurrences

- Line 9: unused import: `crate::index::IndexStatus`

#### `src\query\optimizer\rules\transformation\top_n.rs`: 1 occurrences

- Line 94: variable does not need to be mutable

#### `src\query\executor\admin\index\show_tag_index_status.rs`: 1 occurrences

- Line 9: unused import: `crate::index::IndexStatus`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 244: unused variable: `current_node`: help: if this is intentional, prefix it with an underscore: `_current_node`

#### `src\query\executor\data_processing\graph_traversal\shortest_path.rs`: 1 occurrences

- Line 192: method `mark_termination` is never used

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 844: method `explore_node` is never used

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\executor\search_executors.rs`: 1 occurrences

- Line 314: fields `space_id`, `tag_id`, `index_id`, `scan_limits`, and `return_columns` are never read

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\rules\index\index_scan.rs`: 1 occurrences

- Line 44: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\optimizer\rules\merge\combine_filter.rs`: 1 occurrences

- Line 4: unused import: `PatternBuilder`

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\api\service\graph_service.rs`: 1 occurrences

- Line 104: type `TxState` is more private than the item `graph_service::TransactionManager::get_transaction_state`: method `graph_service::TransactionManager::get_transaction_state` is reachable at visibility `pub`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_expand_all.rs`: 1 occurrences

- Line 9: unused import: `crate::query::planner::plan::core::nodes::PlanNodeEnum`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_all_paths.rs`: 1 occurrences

- Line 9: unused import: `crate::query::planner::plan::core::nodes::PlanNodeEnum`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 398: methods `compare_values`, `extract_sort_values`, `invert_sort_values`, `invert_value_for_sorting`, `optimize_heap_capacity`, and `exceeds_memory_limit` are never used

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 74: multiple methods are never used

#### `src\query\optimizer\rules\elimination\eliminate_append_vertices.rs`: 1 occurrences

- Line 4: unused import: `crate::query::optimizer::rule_patterns::PatternBuilder`

#### `src\core\result\builder.rs`: 1 occurrences

- Line 18: field `capacity` is never read

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 567: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\storage\operations\redb_operations.rs`: 1 occurrences

- Line 293: fields `vertex_cache` and `edge_cache` are never read

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\merge\collapse_project.rs`: 1 occurrences

- Line 4: unused import: `PatternBuilder`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 362: field `condition` is never read

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\join\join_optimization.rs`: 1 occurrences

- Line 140: variant `NestedLoopJoin` is never constructed

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 552: variable does not need to be mutable

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 20: field `default_limit` is never read

#### `src\query\optimizer\rules\index\tag_index_full_scan.rs`: 1 occurrences

- Line 44: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\optimizer\rules\elimination\eliminate_row_collect.rs`: 1 occurrences

- Line 4: unused import: `crate::query::optimizer::rule_patterns::PatternBuilder`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\storage\processor\base.rs`: 1 occurrences

- Line 586: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\storage\metadata\extended_schema.rs`: 1 occurrences

- Line 50: method `save_schema_snapshot` is never used

#### `src\query\optimizer\rules\aggregate\push_filter_down_aggregate.rs`: 1 occurrences

- Line 35: unused import: `crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 9: unused import: `crate::query::planner::plan::core::nodes::PlanNodeEnum`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 67: field `max_size` is never read

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 289: method `InvalidPlanNode` should have a snake case name: help: convert the identifier to snake case: `invalid_plan_node`

#### `src\query\optimizer\rules\index\union_all_edge_index_scan.rs`: 1 occurrences

- Line 30: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\validator\insert_edges_validator.rs`: 1 occurrences

- Line 12: field `base` is never read

#### `src\query\validator\update_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_index_scan.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\index\edge_index_full_scan.rs`: 1 occurrences

- Line 75: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 93: methods `apply_filter_with_thread_pool` and `calculate_batch_size` are never used

