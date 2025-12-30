# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 52
- **Total Warnings**: 105
- **Total Issues**: 157
- **Unique Error Patterns**: 26
- **Unique Warning Patterns**: 69
- **Files with Issues**: 87

## Error Statistics

**Total Errors**: 52

### Error Type Breakdown

- **error[E0277]**: 18 errors
- **error[E0046]**: 16 errors
- **error[E0063]**: 8 errors
- **error[E0119]**: 3 errors
- **error[E0407]**: 3 errors
- **error[E0308]**: 3 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\core\value\types.rs`: 13 errors
- `src\query\executor\result_processing\projection.rs`: 5 errors
- `src\query\context\managers\impl\storage_client_impl.rs`: 4 errors
- `src\query\context\execution\query_execution.rs`: 4 errors
- `src\query\executor\cypher\clauses\match_path\result_builder.rs`: 3 errors
- `src\query\executor\cypher\clauses\match_path\pattern_matcher.rs`: 2 errors
- `src\query\executor\cypher\clauses\match_path\traversal_engine.rs`: 2 errors
- `src\query\executor\cypher\clauses\match_path\path_info.rs`: 1 errors
- `src\query\executor\factory.rs`: 1 errors
- `src\query\visitor\deduce_type_visitor.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 105

### Warning Type Breakdown

- **warning**: 105 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 6 warnings
- `src\query\context\managers\impl\index_manager_impl.rs`: 5 warnings
- `src\query\context\ast\base.rs`: 4 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 3 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\core\value\mod.rs`: 3 warnings
- `src\query\executor\factory.rs`: 3 warnings
- `src\query\context\managers\retry.rs`: 2 warnings
- `src\query\executor\result_processing\sort.rs`: 2 warnings

## Detailed Error Categorization

### error[E0277]: can't compare `&core::value::types::Value` with `i64`: no implementation for `&core::value::types::Value == i64`

**Total Occurrences**: 18  
**Unique Files**: 6

#### `src\core\value\types.rs`: 10 occurrences

- Line 173: the trait bound `f64: std::hash::Hash` is not satisfied: the trait `std::hash::Hash` is not implemented for `f64`
- Line 182: the trait bound `std::collections::HashMap<std::string::String, core::value::types::Value>: std::hash::Hash` is not satisfied: the trait `std::hash::Hash` is not implemented for `std::collections::HashMap<std::string::String, core::value::types::Value>`
- Line 183: the trait bound `std::collections::HashSet<core::value::types::Value>: std::hash::Hash` is not satisfied: the trait `std::hash::Hash` is not implemented for `std::collections::HashSet<core::value::types::Value>`
- ... 7 more occurrences in this file

#### `src\query\executor\cypher\clauses\match_path\result_builder.rs`: 3 occurrences

- Line 401: can't compare `i64` with `&core::value::types::Value`: no implementation for `i64 == &core::value::types::Value`
- Line 404: can't compare `i64` with `&core::value::types::Value`: no implementation for `i64 == &core::value::types::Value`
- Line 407: can't compare `i64` with `&core::value::types::Value`: no implementation for `i64 == &core::value::types::Value`

#### `src\query\executor\cypher\clauses\match_path\pattern_matcher.rs`: 2 occurrences

- Line 211: can't compare `i64` with `&core::value::types::Value`: no implementation for `i64 == &core::value::types::Value`
- Line 310: can't compare `i64` with `&core::value::types::Value`: no implementation for `i64 == &core::value::types::Value`

#### `src\query\executor\cypher\clauses\match_path\traversal_engine.rs`: 1 occurrences

- Line 205: can't compare `&core::value::types::Value` with `i64`: no implementation for `&core::value::types::Value == i64`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 1 occurrences

- Line 151: `(dyn storage_engine::StorageEngine + 'static)` doesn't implement `std::fmt::Debug`: `(dyn storage_engine::StorageEngine + 'static)` cannot be formatted using `{:?}` because it doesn't implement `std::fmt::Debug`

#### `src\query\executor\cypher\clauses\match_path\path_info.rs`: 1 occurrences

- Line 78: can't compare `i64` with `&core::value::types::Value`: no implementation for `i64 == &core::value::types::Value`

### error[E0046]: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

**Total Occurrences**: 16  
**Unique Files**: 16

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 537: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 496: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 247: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 320: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 988: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 304: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 578: not all trait items implemented, missing: `insert_vertex_to_index`, `delete_vertex_from_index`, `update_vertex_in_index`, `insert_edge_to_index`, `delete_edge_from_index`, `update_edge_in_index`: missing `insert_vertex_to_index`, `delete_vertex_from_index`, `update_vertex_in_index`, `insert_edge_to_index`, `delete_edge_from_index`, `update_edge_in_index` in implementation

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 220: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 486: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\result_processing\limit.rs`: 1 occurrences

- Line 289: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 372: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 348: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 499: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 413: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 949: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

#### `src\query\executor\data_processing\join\inner_join.rs`: 1 occurrences

- Line 335: not all trait items implemented, missing: `scan_edges_by_type`: missing `scan_edges_by_type` in implementation

### error[E0063]: missing field `id` in initializer of `vertex_edge_path::Vertex`: missing `id`

**Total Occurrences**: 8  
**Unique Files**: 4

#### `src\query\executor\result_processing\projection.rs`: 4 occurrences

- Line 567: missing field `id` in initializer of `vertex_edge_path::Vertex`: missing `id`
- Line 579: missing field `id` in initializer of `vertex_edge_path::Vertex`: missing `id`
- Line 643: missing field `id` in initializer of `vertex_edge_path::Edge`: missing `id`
- ... 1 more occurrences in this file

#### `src\query\context\managers\impl\storage_client_impl.rs`: 2 occurrences

- Line 528: missing field `id` in initializer of `vertex_edge_path::Edge`: missing `id`
- Line 674: missing field `id` in initializer of `vertex_edge_path::Edge`: missing `id`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 458: missing field `id` in initializer of `vertex_edge_path::Vertex`: missing `id`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 238: missing field `id` in initializer of `vertex_edge_path::Vertex`: missing `id`

### error[E0407]: method `build_index_async` is not a member of trait `IndexManager`: not a member of trait `IndexManager`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\context\execution\query_execution.rs`: 3 occurrences

- Line 607: method `build_index_async` is not a member of trait `IndexManager`: not a member of trait `IndexManager`
- Line 611: method `get_build_progress` is not a member of trait `IndexManager`: not a member of trait `IndexManager`
- Line 615: method `cancel_build` is not a member of trait `IndexManager`: not a member of trait `IndexManager`

### error[E0119]: conflicting implementations of trait `PartialEq` for type `core::value::types::Value`: conflicting implementation for `core::value::types::Value`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\core\value\types.rs`: 3 occurrences

- Line 167: conflicting implementations of trait `PartialEq` for type `core::value::types::Value`: conflicting implementation for `core::value::types::Value`
- Line 167: conflicting implementations of trait `std::cmp::Eq` for type `core::value::types::Value`: conflicting implementation for `core::value::types::Value`
- Line 167: conflicting implementations of trait `std::hash::Hash` for type `core::value::types::Value`: conflicting implementation for `core::value::types::Value`

### error[E0308]: mismatched types: expected `Value`, found `i64`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\context\managers\impl\storage_client_impl.rs`: 2 occurrences

- Line 290: mismatched types: expected `Value`, found `i64`
- Line 310: mismatched types: expected `Value`, found `i64`

#### `src\query\executor\cypher\clauses\match_path\traversal_engine.rs`: 1 occurrences

- Line 243: mismatched types: expected `&_`, found `i64`

### error[E0432]: unresolved import `index_manager::IndexBuildProgress`: no `IndexBuildProgress` in `query::context::managers::index_manager`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\managers\mod.rs`: 1 occurrences

- Line 14: unresolved import `index_manager::IndexBuildProgress`: no `IndexBuildProgress` in `query::context::managers::index_manager`

## Detailed Warning Categorization

### warning: unused import: `ManagerError`

**Total Occurrences**: 105  
**Unique Files**: 73

#### `src\query\planner\plan\core\nodes\factory.rs`: 6 occurrences

- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 50: unused variable: `columns`: help: if this is intentional, prefix it with an underscore: `_columns`
- Line 72: unused variable: `hash_keys_expr`: help: if this is intentional, prefix it with an underscore: `_hash_keys_expr`
- ... 3 more occurrences in this file

#### `src\query\context\managers\impl\index_manager_impl.rs`: 5 occurrences

- Line 42: unused variable: `field_value`: help: if this is intentional, prefix it with an underscore: `_field_value`
- Line 49: unused variable: `field_value`: help: if this is intentional, prefix it with an underscore: `_field_value`
- Line 157: variable does not need to be mutable
- ... 2 more occurrences in this file

#### `src\query\context\ast\base.rs`: 4 occurrences

- Line 37: unused variable: `query_type`: help: if this is intentional, prefix it with an underscore: `_query_type`
- Line 37: unused variable: `query_text`: help: if this is intentional, prefix it with an underscore: `_query_text`
- Line 124: unused variable: `query_type`: help: if this is intentional, prefix it with an underscore: `_query_type`
- ... 1 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 3 occurrences

- Line 241: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`
- Line 242: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 243: unused variable: `escape_char`: help: if this is intentional, prefix it with an underscore: `_escape_char`

#### `src\query\executor\factory.rs`: 3 occurrences

- Line 341: unused variable: `node`: help: if this is intentional, prefix it with an underscore: `_node`
- Line 144: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 235: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 136: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\core\value\mod.rs`: 3 occurrences

- Line 16: unused import: `comparison::*`
- Line 17: unused import: `operations::*`
- Line 18: unused import: `conversion::*`

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system-monitor`: help: remove the condition
- Line 525: unexpected `cfg` condition value: `system-monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\executor\data_processing\graph_traversal\traits.rs`: 2 occurrences

- Line 3: unused import: `crate::core::error::DBResult`
- Line 4: unused imports: `ExecutionResult` and `Executor`

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`
- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`

#### `src\query\context\managers\impl\index_binary.rs`: 2 occurrences

- Line 6: unused imports: `Write` and `self`
- Line 103: unused variable: `length`: help: if this is intentional, prefix it with an underscore: `_length`

#### `src\query\planner\ngql\go_planner.rs`: 2 occurrences

- Line 5: unused import: `crate::query::parser::ast::expr::Expr`
- Line 62: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\parser\expressions\expression_converter.rs`: 2 occurrences

- Line 6: unused import: `NullType`
- Line 457: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 204: unused variable: `estimated_memory`: help: if this is intentional, prefix it with an underscore: `_estimated_memory`
- Line 745: variable does not need to be mutable

#### `src\query\context\managers\retry.rs`: 2 occurrences

- Line 7: unused import: `ErrorCategory`
- Line 149: variable does not need to be mutable

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 2 occurrences

- Line 7: unused import: `crate::core::error::DBResult`
- Line 8: unused import: `crate::query::executor::traits::ExecutionResult`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 2 occurrences

- Line 11: unused import: `HasStorage`
- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\context\managers\storage_client.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 182: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 4: unused import: `ManagerError`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 7: unused import: `ManagerError`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 20: unused import: `HasStorage`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\core\context\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::Value`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 792: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\expression\evaluator\operations.rs`: 1 occurrences

- Line 7: unused import: `ExpressionErrorType`

#### `src\core\context\request.rs`: 1 occurrences

- Line 14: unused import: `SessionStatus`

#### `src\query\executor\recursion_detector.rs`: 1 occurrences

- Line 3: unused import: `HashMap`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\executor\base.rs`: 1 occurrences

- Line 7: unused import: `HasInput`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 12: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 259: unused import: `std::fs`

#### `src\query\executor\result_processing\limit.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 8: unused import: `ManagerError`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 20: unused import: `HasStorage`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 278: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 9: unused import: `SessionStatus`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 340: unused import: `UnaryOperator`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 264: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 23: unused import: `HasStorage`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\query\context\ast\common.rs`: 1 occurrences

- Line 4: unused import: `crate::query::parser::ast::expr::Expr`

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 186: variable does not need to be mutable

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 212: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\context\managers\index_manager.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 339: variable does not need to be mutable

