# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 53
- **Total Issues**: 56
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 33
- **Files with Issues**: 39

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0560]**: 3 errors

### Files with Errors (Top 10)

- `src\query\executor\graph_query_executor.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 53

### Warning Type Breakdown

- **warning**: 53 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\predicate_pushdown.rs`: 3 warnings
- `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 warnings
- `src\storage\redb_storage.rs`: 3 warnings
- `src\query\optimizer\rule_enum.rs`: 3 warnings
- `src\query\executor\data_access.rs`: 2 warnings
- `src\query\optimizer\rules\mod.rs`: 2 warnings
- `src\query\optimizer\plan_validator.rs`: 2 warnings
- `src\query\visitor\extract_filter_expr_visitor.rs`: 2 warnings
- `src\core\types\expression\visitor.rs`: 2 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings

## Detailed Error Categorization

### error[E0560]: struct `GraphQueryExecutor<S>` has no field named `thread_pool`: unknown field

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 3 occurrences

- Line 65: struct `GraphQueryExecutor<S>` has no field named `thread_pool`: unknown field
- Line 79: struct `GraphQueryExecutor<S>` has no field named `thread_pool`: unknown field
- Line 98: struct `GraphQueryExecutor<S>` has no field named `thread_pool`: unknown field

## Detailed Warning Categorization

### warning: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

**Total Occurrences**: 53  
**Unique Files**: 39

#### `src\query\optimizer\predicate_pushdown.rs`: 3 occurrences

- Line 615: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 723: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 901: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\storage\redb_storage.rs`: 3 occurrences

- Line 359: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`
- Line 1220: unused variable: `new_role`: help: if this is intentional, prefix it with an underscore: `_new_role`
- Line 1222: unused variable: `is_locked`: help: if this is intentional, prefix it with an underscore: `_is_locked`

#### `src\query\optimizer\rule_enum.rs`: 3 occurrences

- Line 81: unreachable pattern: no value can reach this
- Line 128: unreachable pattern: no value can reach this
- Line 176: unreachable pattern: no value can reach this

#### `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 occurrences

- Line 14: unused import: `crate::query::planner::plan::algorithms::IndexScan`
- Line 119: variable does not need to be mutable
- Line 137: variable does not need to be mutable

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 965: variable does not need to be mutable
- Line 1013: variable does not need to be mutable

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 2 occurrences

- Line 133: unused variable: `op`: help: try ignoring the field: `op: _`
- Line 149: unused variable: `op`: help: try ignoring the field: `op: _`

#### `src\query\optimizer\rules\mod.rs`: 2 occurrences

- Line 36: unused import: `predicate_pushdown::*`
- Line 40: unused import: `index::*`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 150: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 178: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 152: unused variable: `ids`: help: if this is intentional, prefix it with an underscore: `_ids`
- Line 531: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 87: unused variable: `node_ref`: help: if this is intentional, prefix it with an underscore: `_node_ref`
- Line 88: unused variable: `boundary`: help: if this is intentional, prefix it with an underscore: `_boundary`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 475: variable does not need to be mutable

#### `src\query\optimizer\rules\aggregate\push_filter_down_aggregate.rs`: 1 occurrences

- Line 35: unused import: `crate::query::planner::plan::core::nodes::aggregate_node::AggregateNode`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\elimination\eliminate_row_collect.rs`: 1 occurrences

- Line 4: unused import: `crate::query::optimizer::rule_patterns::PatternBuilder`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 25: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 8: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\merge\collapse_project.rs`: 1 occurrences

- Line 4: unused import: `PatternBuilder`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 21: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\merge\combine_filter.rs`: 1 occurrences

- Line 4: unused import: `PatternBuilder`

#### `src\query\optimizer\rules\transformation\top_n.rs`: 1 occurrences

- Line 94: variable does not need to be mutable

#### `src\query\optimizer\rules\elimination\eliminate_append_vertices.rs`: 1 occurrences

- Line 4: unused import: `crate::query::optimizer::rule_patterns::PatternBuilder`

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 373: unused import: `PasswordInfo`

#### `src\storage\processor\base.rs`: 1 occurrences

- Line 544: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 567: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_index_scan.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\optimizer\expression_utils.rs`: 1 occurrences

- Line 132: unused variable: `property`: help: try ignoring the field: `property: _`

#### `src\query\validator\insert_vertices_validator.rs`: 1 occurrences

- Line 204: unused import: `crate::core::Value`

