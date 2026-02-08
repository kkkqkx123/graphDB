# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 86
- **Total Issues**: 89
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 41
- **Files with Issues**: 48

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0308]**: 1 errors
- **error[E0061]**: 1 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\mod.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 86

### Warning Type Breakdown

- **warning**: 86 warnings

### Files with Warnings (Top 10)

- `src\query\context\runtime_context.rs`: 11 warnings
- `src\storage\test_mock.rs`: 9 warnings
- `src\query\executor\logic\loops.rs`: 4 warnings
- `src\query\optimizer\rule_enum.rs`: 3 warnings
- `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_left_join.rs`: 3 warnings
- `src\query\optimizer\rules\predicate_pushdown\push_filter_down_cross_join.rs`: 3 warnings
- `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 3 warnings
- `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 warnings
- `src\query\executor\mod.rs`: 3 warnings
- `src\query\executor\result_processing\sort.rs`: 2 warnings

## Detailed Error Categorization

### error[E0432]: unresolved import `crate::query::executor::ExecutorEnum`: no `ExecutorEnum` in `query::executor`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\mod.rs`: 1 occurrences

- Line 109: unresolved import `crate::query::executor::ExecutorEnum`: no `ExecutorEnum` in `query::executor`

### error[E0061]: this function takes 2 arguments but 1 argument was supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\mod.rs`: 1 occurrences

- Line 126: this function takes 2 arguments but 1 argument was supplied

### error[E0308]: mismatched types: expected `CrossJoinNode`, found `Result<CrossJoinNode, PlannerError>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\mod.rs`: 1 occurrences

- Line 127: mismatched types: expected `CrossJoinNode`, found `Result<CrossJoinNode, PlannerError>`

## Detailed Warning Categorization

### warning: unused variable: `edge_key`: help: if this is intentional, prefix it with an underscore: `_edge_key`

**Total Occurrences**: 86  
**Unique Files**: 48

#### `src\query\context\runtime_context.rs`: 11 occurrences

- Line 99: use of deprecated type alias `query::context::runtime_context::ExecutionState`: 请使用 crate::query::core::ExecutorState
- Line 117: use of deprecated type alias `query::context::runtime_context::ExecutionState`: 请使用 crate::query::core::ExecutorState
- Line 203: use of deprecated type alias `query::context::runtime_context::ResultStatus`: 请使用 crate::query::core::RowStatus
- ... 8 more occurrences in this file

#### `src\storage\test_mock.rs`: 9 occurrences

- Line 180: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 184: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- Line 188: unused variable: `space`: help: if this is intentional, prefix it with an underscore: `_space`
- ... 6 more occurrences in this file

#### `src\query\executor\logic\loops.rs`: 4 occurrences

- Line 264: use of deprecated type alias `query::executor::logic::loops::LoopState`: 请使用 crate::query::core::LoopExecutionState
- Line 504: use of deprecated type alias `query::executor::logic::loops::LoopState`: 请使用 crate::query::core::LoopExecutionState
- Line 696: use of deprecated type alias `query::executor::logic::loops::LoopState`: 请使用 crate::query::core::LoopExecutionState
- ... 1 more occurrences in this file

#### `src\query\optimizer\rules\scan\index_full_scan.rs`: 3 occurrences

- Line 14: unused import: `crate::query::planner::plan::algorithms::IndexScan`
- Line 119: variable does not need to be mutable
- Line 137: variable does not need to be mutable

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_left_join.rs`: 3 occurrences

- Line 97: unused variable: `left_filter_picked`: help: if this is intentional, prefix it with an underscore: `_left_filter_picked`
- Line 106: unused variable: `remaining_after_right`: help: if this is intentional, prefix it with an underscore: `_remaining_after_right`
- Line 111: unused variable: `right_filter_picked`: help: if this is intentional, prefix it with an underscore: `_right_filter_picked`

#### `src\query\executor\mod.rs`: 3 occurrences

- Line 107: unused import: `NodeType`
- Line 110: unused import: `crate::storage::StorageClient`
- Line 58: use of deprecated type alias `query::executor::logic::loops::LoopState`: 请使用 crate::query::core::LoopExecutionState

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_hash_inner_join.rs`: 3 occurrences

- Line 97: unused variable: `left_filter_picked`: help: if this is intentional, prefix it with an underscore: `_left_filter_picked`
- Line 106: unused variable: `remaining_after_right`: help: if this is intentional, prefix it with an underscore: `_remaining_after_right`
- Line 111: unused variable: `right_filter_picked`: help: if this is intentional, prefix it with an underscore: `_right_filter_picked`

#### `src\query\optimizer\rule_enum.rs`: 3 occurrences

- Line 77: unreachable pattern: no value can reach this
- Line 121: unreachable pattern: no value can reach this
- Line 166: unreachable pattern: no value can reach this

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_cross_join.rs`: 3 occurrences

- Line 97: unused variable: `left_filter_picked`: help: if this is intentional, prefix it with an underscore: `_left_filter_picked`
- Line 106: unused variable: `remaining_after_right`: help: if this is intentional, prefix it with an underscore: `_remaining_after_right`
- Line 111: unused variable: `right_filter_picked`: help: if this is intentional, prefix it with an underscore: `_right_filter_picked`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 612: variable does not need to be mutable
- Line 580: variable does not need to be mutable

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 965: variable does not need to be mutable
- Line 1013: variable does not need to be mutable

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 150: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 178: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\context\mod.rs`: 2 occurrences

- Line 39: use of deprecated type alias `query::context::runtime_context::ExecutionState`: 请使用 crate::query::core::ExecutorState
- Line 39: use of deprecated type alias `query::context::runtime_context::ResultStatus`: 请使用 crate::query::core::RowStatus

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 2 occurrences

- Line 133: unused variable: `op`: help: try ignoring the field: `op: _`
- Line 149: unused variable: `op`: help: try ignoring the field: `op: _`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 343: unused variable: `edge_key`: help: if this is intentional, prefix it with an underscore: `_edge_key`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\scheduler\mod.rs`: 1 occurrences

- Line 15: use of deprecated type alias `query::scheduler::async_scheduler::ExecutionState`: 请使用 SchedulerExecutionState

#### `src\query\optimizer\rules\index\union_all_edge_index_scan.rs`: 1 occurrences

- Line 30: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\optimizer\rules\transformation\top_n.rs`: 1 occurrences

- Line 94: variable does not need to be mutable

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 504: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`

#### `src\common\memory.rs`: 1 occurrences

- Line 222: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\optimizer\rules\merge\combine_filter.rs`: 1 occurrences

- Line 4: unused import: `PatternBuilder`

#### `src\query\executor\admin\index\show_edge_index_status.rs`: 1 occurrences

- Line 9: unused import: `crate::index::IndexStatus`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 101: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\query\executor\data_processing\graph_traversal\traverse.rs`: 1 occurrences

- Line 244: unused variable: `current_node`: help: if this is intentional, prefix it with an underscore: `_current_node`

#### `src\query\optimizer\rules\elimination\eliminate_append_vertices.rs`: 1 occurrences

- Line 4: unused import: `crate::query::optimizer::rule_patterns::PatternBuilder`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_scan_edges.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 19: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\query\planner\statements\match_planner.rs`: 1 occurrences

- Line 567: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_index_scan.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\optimizer\rules\index\index_scan.rs`: 1 occurrences

- Line 44: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\storage\processor\base.rs`: 1 occurrences

- Line 586: unused variable: `counters`: help: if this is intentional, prefix it with an underscore: `_counters`

#### `src\index\fulltext.rs`: 1 occurrences

- Line 414: unused variable: `index_name`: help: if this is intentional, prefix it with an underscore: `_index_name`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_all_paths.rs`: 1 occurrences

- Line 9: unused import: `crate::query::planner::plan::core::nodes::PlanNodeEnum`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 47: unused variable: `ids`: help: try ignoring the field: `ids: _`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 553: variable does not need to be mutable

#### `src\query\optimizer\rules\index\edge_index_full_scan.rs`: 1 occurrences

- Line 75: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_get_nbrs.rs`: 1 occurrences

- Line 9: unused import: `crate::query::planner::plan::core::nodes::PlanNodeEnum`

#### `src\query\optimizer\rules\index\tag_index_full_scan.rs`: 1 occurrences

- Line 44: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\optimizer\rules\predicate_pushdown\push_filter_down_join.rs`: 1 occurrences

- Line 97: unused variable: `filter_picked`: help: if this is intentional, prefix it with an underscore: `_filter_picked`

#### `src\query\executor\logic\mod.rs`: 1 occurrences

- Line 16: use of deprecated type alias `query::executor::logic::loops::LoopState`: 请使用 crate::query::core::LoopExecutionState

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`

#### `src\query\optimizer\rules\limit_pushdown\push_limit_down_get_vertices.rs`: 1 occurrences

- Line 10: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 92: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

