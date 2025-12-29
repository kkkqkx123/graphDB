# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 147
- **Total Issues**: 147
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 111
- **Files with Issues**: 92

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 147

### Warning Type Breakdown

- **warning**: 147 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\query\executor\result_processing\sort.rs`: 5 warnings
- `src\services\stats.rs`: 5 warnings
- `src\query\executor\factory.rs`: 4 warnings
- `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 4 warnings
- `src\query\executor\data_processing\join\hash_table.rs`: 4 warnings
- `src\query\executor\data_modification.rs`: 4 warnings
- `src\query\executor\data_access.rs`: 3 warnings
- `src\query\visitor\deduce_type_visitor.rs`: 3 warnings

## Detailed Warning Categorization

### warning: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

**Total Occurrences**: 147  
**Unique Files**: 87

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 36: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 32: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 50: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\services\stats.rs`: 5 occurrences

- Line 15: field `created_at` is never read
- Line 53: field `created_at` is never read
- Line 88: field `created_at` is never read
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\sort.rs`: 5 occurrences

- Line 204: unused variable: `estimated_memory`: help: if this is intentional, prefix it with an underscore: `_estimated_memory`
- Line 767: variable does not need to be mutable
- Line 88: field `config` is never read
- ... 2 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 4 occurrences

- Line 5: unused import: `DataType`
- Line 241: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`
- Line 242: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\join\hash_table.rs`: 4 occurrences

- Line 9: unused import: `Evaluator`
- Line 792: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`
- Line 158: method `clear` is never used
- ... 1 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 4 occurrences

- Line 115: field `condition` is never read
- Line 218: fields `condition` and `cascade` are never read
- Line 295: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 occurrences

- Line 17: field `dependencies` is never read
- Line 111: field `dependencies` is never read
- Line 224: field `dependencies` is never read
- ... 1 more occurrences in this file

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 341: unused variable: `node`: help: if this is intentional, prefix it with an underscore: `_node`
- Line 144: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 235: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- ... 1 more occurrences in this file

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 496: methods `visit_property`, `visit_set`, and `parse_type_def` are never used
- Line 1131: fields `storage`, `validate_context`, `inputs`, `space`, and `vid_type` are never read
- Line 1151: multiple methods are never used

#### `src\query\executor\data_processing\join\left_join.rs`: 3 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`
- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 207: method `create_null_right_row` is never used

#### `src\query\executor\data_access.rs`: 3 occurrences

- Line 184: field `edge_type` is never read
- Line 241: fields `vertex_ids`, `edge_direction`, and `edge_types` are never read
- Line 310: fields `vertex_ids`, `edge_ids`, and `prop_names` are never read

#### `src\core\value\mod.rs`: 3 occurrences

- Line 16: unused import: `comparison::*`
- Line 17: unused import: `operations::*`
- Line 18: unused import: `conversion::*`

#### `src\query\planner\match_planning\core\match_planner.rs`: 3 occurrences

- Line 98: function `create_test_node_info` is never used
- Line 111: function `create_test_path` is never used
- Line 134: function `create_test_match_clause_context` is never used

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\parser\expressions\expression_converter.rs`: 2 occurrences

- Line 6: unused import: `NullType`
- Line 457: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\executor\data_processing\join\inner_join.rs`: 2 occurrences

- Line 10: unused import: `crate::expression::evaluator::expression_evaluator::ExpressionEvaluator`
- Line 11: unused import: `crate::expression::evaluator::traits::ExpressionContext`

#### `src\query\executor\result_processing\topn.rs`: 2 occurrences

- Line 20: unused import: `HasStorage`
- Line 377: field `original_index` is never read

#### `src\query\executor\data_processing\join\cross_join.rs`: 2 occurrences

- Line 12: unused import: `HasStorage`
- Line 84: method `execute_multi_way_cartesian_product` is never used

#### `src\core\context\manager.rs`: 2 occurrences

- Line 96: field `created_at` is never read
- Line 306: method `is_max_contexts_exceeded` is never used

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\query\executor\data_processing\graph_traversal\traits.rs`: 2 occurrences

- Line 3: unused import: `crate::core::error::DBResult`
- Line 4: unused imports: `ExecutionResult` and `Executor`

#### `src\storage\native_storage.rs`: 2 occurrences

- Line 14: field `schema_tree` is never read
- Line 76: method `value_from_bytes` is never used

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 2 occurrences

- Line 11: unused import: `HasStorage`
- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\executor\data_processing\graph_traversal\impls.rs`: 2 occurrences

- Line 7: unused import: `crate::core::error::DBResult`
- Line 8: unused import: `crate::query::executor::traits::ExecutionResult`

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 6: unused import: `ContextExt`
- Line 335: method `args_to_hash` is never used

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 345: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 352: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\common\thread.rs`: 2 occurrences

- Line 58: field `id` is never read
- Line 175: field `mutex` is never read

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 182: struct `DummyPlanNode` is never constructed
- Line 191: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 887: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 34: field `exchange` is never read

#### `src\query\executor\recursion_detector.rs`: 1 occurrences

- Line 3: unused import: `HashMap`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 8: field `query_text` is never read

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 100: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 18: field `match_clause_ctx` is never read

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 269: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 20: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\executor\result_processing\limit.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 340: unused import: `UnaryOperator`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 11: unused import: `HasStorage`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 23: unused import: `HasStorage`

#### `src\core\context\mod.rs`: 1 occurrences

- Line 5: unused import: `crate::core::Value`

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\executor\cypher\factory.rs`: 1 occurrences

- Line 161: function `create_test_storage` is never used

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 245: field `planners` is never read

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 30: field `props` is never read

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 339: variable does not need to be mutable

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 9: unused import: `SessionStatus`

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 60: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\expression\evaluator\operations.rs`: 1 occurrences

- Line 7: unused import: `ExpressionErrorType`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 4: unused import: `crate::core::types::expression::DataType`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 278: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 48: field `data` is never read

#### `src\query\planner\match_planning\clauses\return_clause_planner.rs`: 1 occurrences

- Line 215: function `get_yield_columns` is never used

#### `src\core\context\request.rs`: 1 occurrences

- Line 14: unused import: `SessionStatus`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 236: methods `visit_type_casting`, `visit_path_build`, and `visit_subscript_range` are never used

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 10: unused import: `HasStorage`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\planner\match_planning\core\match_clause_planner.rs`: 1 occurrences

- Line 37: field `paths` is never read

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\core\type_utils.rs`: 1 occurrences

- Line 136: associated functions `test_check_compatibility`, `test_check_compatibility_batch`, `test_literal_type`, `test_binary_operation_result_type`, and `test_should_cache_expression` are never used

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\executor\base.rs`: 1 occurrences

- Line 7: unused import: `HasInput`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 16: unused import: `HasStorage`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 561: variable does not need to be mutable

#### `src\cache\cache_impl\adaptive.rs`: 1 occurrences

- Line 12: variants `LFU` and `Hybrid` are never constructed

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 15: unused import: `HasStorage`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 18: unused import: `HasStorage`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 206: method `update_iterator_and_value` is never used

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 222: field `config` is never read

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 53: fields `signals` and `signal_info` are never read

#### `src\stats\graph_stats.rs`: 1 occurrences

- Line 40: field `enable_space_level_metrics` is never read

