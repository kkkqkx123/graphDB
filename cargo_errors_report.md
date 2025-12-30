# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 159
- **Total Issues**: 159
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 111
- **Files with Issues**: 110

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 159

### Warning Type Breakdown

- **warning**: 159 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 4 warnings
- `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 warnings
- `src\query\context\ast\base.rs`: 4 warnings
- `src\query\executor\factory.rs`: 4 warnings
- `src\services\stats.rs`: 4 warnings
- `src\core\value\mod.rs`: 3 warnings
- `src\query\visitor\deduce_type_visitor.rs`: 3 warnings
- `src\query\executor\data_access.rs`: 3 warnings
- `src\query\planner\match_planning\core\match_planner.rs`: 3 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 3 warnings

## Detailed Warning Categorization

### warning: field `id` is never read

**Total Occurrences**: 159  
**Unique Files**: 104

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 340: unused variable: `node`: help: if this is intentional, prefix it with an underscore: `_node`
- Line 143: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 234: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 occurrences

- Line 17: field `dependencies` is never read
- Line 111: field `dependencies` is never read
- Line 224: field `dependencies` is never read
- ... 1 more occurrences in this file

#### `src\query\context\ast\base.rs`: 4 occurrences

- Line 37: unused variable: `query_type`: help: if this is intentional, prefix it with an underscore: `_query_type`
- Line 37: unused variable: `query_text`: help: if this is intentional, prefix it with an underscore: `_query_text`
- Line 124: unused variable: `query_type`: help: if this is intentional, prefix it with an underscore: `_query_type`
- ... 1 more occurrences in this file

#### `src\query\executor\result_processing\sort.rs`: 4 occurrences

- Line 204: unused variable: `estimated_memory`: help: if this is intentional, prefix it with an underscore: `_estimated_memory`
- Line 745: variable does not need to be mutable
- Line 88: field `config` is never read
- ... 1 more occurrences in this file

#### `src\services\stats.rs`: 4 occurrences

- Line 15: field `created_at` is never read
- Line 53: field `created_at` is never read
- Line 150: field `created_at` is never read
- ... 1 more occurrences in this file

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system-monitor`: help: remove the condition
- Line 525: unexpected `cfg` condition value: `system-monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\planner\match_planning\core\match_planner.rs`: 3 occurrences

- Line 98: function `create_test_node_info` is never used
- Line 111: function `create_test_path` is never used
- Line 134: function `create_test_match_clause_context` is never used

#### `src\core\value\mod.rs`: 3 occurrences

- Line 16: unused import: `comparison::*`
- Line 17: unused import: `operations::*`
- Line 18: unused import: `conversion::*`

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 136: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\executor\result_processing\topn.rs`: 3 occurrences

- Line 20: unused import: `HasStorage`
- Line 494: unused import: `crate::core::value::NullType`
- Line 377: field `original_index` is never read

#### `src\expression\evaluator\expression_evaluator.rs`: 3 occurrences

- Line 241: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`
- Line 242: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 243: unused variable: `escape_char`: help: if this is intentional, prefix it with an underscore: `_escape_char`

#### `src\query\executor\data_access.rs`: 3 occurrences

- Line 184: field `edge_type` is never read
- Line 241: fields `vertex_ids`, `edge_direction`, and `edge_types` are never read
- Line 310: fields `vertex_ids`, `edge_ids`, and `prop_names` are never read

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 496: methods `visit_property`, `visit_set`, and `parse_type_def` are never used
- Line 1136: fields `storage`, `validate_context`, `inputs`, `space`, and `vid_type` are never read
- Line 1156: multiple methods are never used

#### `src\query\executor\data_processing\join\hash_table.rs`: 3 occurrences

- Line 792: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`
- Line 158: method `clear` is never used
- Line 319: field `config` is never read

#### `src\common\thread.rs`: 2 occurrences

- Line 58: field `id` is never read
- Line 175: field `mutex` is never read

#### `src\query\executor\result_processing\limit.rs`: 2 occurrences

- Line 15: unused import: `HasStorage`
- Line 284: unused import: `crate::core::value::NullType`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 6: unused import: `ContextExt`
- Line 335: method `args_to_hash` is never used

#### `src\core\result\result_core.rs`: 2 occurrences

- Line 186: variable does not need to be mutable
- Line 252: method `update_iterator_and_value` is never used

#### `src\query\context\managers\retry.rs`: 2 occurrences

- Line 7: unused import: `ErrorCategory`
- Line 149: variable does not need to be mutable

#### `src\query\executor\data_processing\graph_traversal\traits.rs`: 2 occurrences

- Line 3: unused import: `crate::core::error::DBResult`
- Line 4: unused imports: `ExecutionResult` and `Executor`

#### `src\storage\native_storage.rs`: 2 occurrences

- Line 14: field `schema_tree` is never read
- Line 79: method `value_from_bytes` is never used

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\parser\expressions\expression_converter.rs`: 2 occurrences

- Line 6: unused import: `NullType`
- Line 457: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 2 occurrences

- Line 7: unused import: `crate::core::error::DBResult`
- Line 8: unused import: `crate::query::executor::traits::ExecutionResult`

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 23: unused import: `HasStorage`
- Line 944: unused import: `crate::core::value::NullType`

#### `src\core\context\manager.rs`: 2 occurrences

- Line 96: field `created_at` is never read
- Line 306: method `is_max_contexts_exceeded` is never used

#### `src\query\executor\result_processing\sample.rs`: 2 occurrences

- Line 18: unused import: `HasStorage`
- Line 491: unused import: `crate::core::value::NullType`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 2 occurrences

- Line 57: methods `lookup_vertex_by_id` and `lookup_edge_by_id` are never used
- Line 554: associated functions `vertex_matches_values`, `edge_matches_values`, `extract_vertex_field_value`, and `extract_edge_field_value` are never used

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 182: struct `DummyPlanNode` is never constructed
- Line 191: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 2 occurrences

- Line 11: unused import: `HasStorage`
- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\executor\data_processing\join\cross_join.rs`: 2 occurrences

- Line 12: unused import: `HasStorage`
- Line 84: method `execute_multi_way_cartesian_product` is never used

#### `src\query\executor\data_modification.rs`: 2 occurrences

- Line 182: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`
- Line 360: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`
- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`

#### `src\query\executor\result_processing\filter.rs`: 2 occurrences

- Line 20: unused import: `HasStorage`
- Line 299: unused import: `crate::core::value::NullType`

#### `src\query\planner\ngql\go_planner.rs`: 2 occurrences

- Line 5: unused import: `crate::query::parser::ast::expr::Expr`
- Line 62: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 53: fields `signals` and `signal_info` are never read

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 48: field `data` is never read

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 4: unused import: `ManagerError`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 278: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\context\request.rs`: 1 occurrences

- Line 14: unused import: `SessionStatus`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\context\managers\storage_client.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\planner\match_planning\core\match_clause_planner.rs`: 1 occurrences

- Line 37: field `paths` is never read

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 340: unused import: `UnaryOperator`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 236: methods `visit_type_casting`, `visit_path_build`, and `visit_subscript_range` are never used

#### `src\query\context\managers\index_manager.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\expression\evaluator\operations.rs`: 1 occurrences

- Line 7: unused import: `ExpressionErrorType`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 9: unused import: `SessionStatus`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 212: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 241: variable does not need to be mutable

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 245: field `planners` is never read

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 18: field `match_clause_ctx` is never read

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 259: unused import: `std::fs`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\cache\cache_impl\adaptive.rs`: 1 occurrences

- Line 12: variants `LFU` and `Hybrid` are never constructed

#### `src\core\context\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::Value`

#### `src\query\context\ast\common.rs`: 1 occurrences

- Line 4: unused import: `crate::query::parser::ast::expr::Expr`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 8: unused import: `ManagerError`

#### `src\query\context\managers\impl\storage_client_impl.rs`: 1 occurrences

- Line 187: associated functions `vertex_key` and `edge_key_string` are never used

#### `src\core\type_utils.rs`: 1 occurrences

- Line 136: associated functions `test_check_compatibility`, `test_check_compatibility_batch`, `test_literal_type`, `test_binary_operation_result_type`, and `test_should_cache_expression` are never used

#### `src\query\executor\recursion_detector.rs`: 1 occurrences

- Line 3: unused import: `HashMap`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\executor\cypher\factory.rs`: 1 occurrences

- Line 161: function `create_test_storage` is never used

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 7: unused import: `ManagerError`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\query\planner\match_planning\clauses\return_clause_planner.rs`: 1 occurrences

- Line 215: function `get_yield_columns` is never used

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 222: field `config` is never read

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused import: `ManagerError`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 34: field `exchange` is never read

#### `src\query\executor\base.rs`: 1 occurrences

- Line 7: unused import: `HasInput`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 530: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 205: method `create_null_right_row` is never used

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 100: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\stats\graph_stats.rs`: 1 occurrences

- Line 40: field `enable_space_level_metrics` is never read

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 316: unused import: `crate::storage::StorageEngine`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 242: unused imports: `Direction` and `Value`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 264: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 480: unused import: `crate::core::value::NullType`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 30: field `props` is never read

