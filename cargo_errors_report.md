# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 116
- **Total Issues**: 116
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 96
- **Files with Issues**: 69

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 116

### Warning Type Breakdown

- **warning**: 116 warnings

### Files with Warnings (Top 10)

- `src\query\planner\plan\core\nodes\factory.rs`: 12 warnings
- `src\expression\evaluator\expression_evaluator.rs`: 6 warnings
- `src\services\stats.rs`: 5 warnings
- `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 warnings
- `src\query\executor\data_modification.rs`: 4 warnings
- `src\query\planner\match_planning\core\match_planner.rs`: 4 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 3 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 3 warnings
- `src\query\executor\data_access.rs`: 3 warnings
- `src\query\executor\cypher\factory.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`

**Total Occurrences**: 116  
**Unique Files**: 66

#### `src\query\planner\plan\core\nodes\factory.rs`: 12 occurrences

- Line 35: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 31: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- Line 49: unused variable: `input`: help: if this is intentional, prefix it with an underscore: `_input`
- ... 9 more occurrences in this file

#### `src\expression\evaluator\expression_evaluator.rs`: 6 occurrences

- Line 304: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 304: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`
- Line 1009: unused variable: `regex_pattern`: help: if this is intentional, prefix it with an underscore: `_regex_pattern`
- ... 3 more occurrences in this file

#### `src\services\stats.rs`: 5 occurrences

- Line 15: field `created_at` is never read
- Line 53: field `created_at` is never read
- Line 88: field `created_at` is never read
- ... 2 more occurrences in this file

#### `src\query\executor\data_modification.rs`: 4 occurrences

- Line 122: field `condition` is never read
- Line 236: fields `condition` and `cascade` are never read
- Line 338: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 4 occurrences

- Line 17: field `dependencies` is never read
- Line 111: field `dependencies` is never read
- Line 224: field `dependencies` is never read
- ... 1 more occurrences in this file

#### `src\query\planner\match_planning\core\match_planner.rs`: 4 occurrences

- Line 90: unused imports: `AliasType` and `CypherClauseContext`
- Line 100: function `create_test_node_info` is never used
- Line 113: function `create_test_path` is never used
- ... 1 more occurrences in this file

#### `src\query\executor\data_access.rs`: 3 occurrences

- Line 205: field `edge_type` is never read
- Line 270: fields `vertex_ids`, `edge_direction`, and `edge_types` are never read
- Line 347: fields `vertex_ids`, `edge_ids`, and `prop_names` are never read

#### `src\query\planner\ngql\lookup_planner.rs`: 3 occurrences

- Line 52: variable `index_scan_node` is assigned to, but never used
- Line 87: value assigned to `index_scan_node` is never read
- Line 127: unused variable: `final_node`: help: if this is intentional, prefix it with an underscore: `_final_node`

#### `src\query\planner\match_planning\utils\finder.rs`: 3 occurrences

- Line 294: unused imports: `ReturnClauseContext`, `UnwindClauseContext`, `WhereClauseContext`, `WithClauseContext`, and `YieldClauseContext`
- Line 347: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 354: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 319: unused variable: `expr_context`: help: if this is intentional, prefix it with an underscore: `_expr_context`
- Line 30: field `props` is never read

#### `src\storage\native_storage.rs`: 2 occurrences

- Line 14: field `schema_tree` is never read
- Line 76: method `value_from_bytes` is never used

#### `src\query\optimizer\limit_pushdown.rs`: 2 occurrences

- Line 9: unused import: `std::sync::Arc`
- Line 888: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\planner.rs`: 2 occurrences

- Line 393: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 245: field `planners` is never read

#### `src\core\context\manager.rs`: 2 occurrences

- Line 95: field `created_at` is never read
- Line 305: method `is_max_contexts_exceeded` is never used

#### `src\query\parser\cypher\expression_converter.rs`: 2 occurrences

- Line 268: unused imports: `CaseAlternative`, `CaseExpression`, `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`
- Line 272: unused import: `UnaryOperator`

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 182: struct `DummyPlanNode` is never constructed
- Line 191: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\common\thread.rs`: 2 occurrences

- Line 58: field `id` is never read
- Line 175: field `mutex` is never read

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 2 occurrences

- Line 148: unused import: `std::collections::HashMap`
- Line 196: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\cache\parser_cache.rs`: 2 occurrences

- Line 522: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`
- Line 531: unused variable: `cached_expr`: help: if this is intentional, prefix it with an underscore: `_cached_expr`

#### `src\core\query_pipeline_manager.rs`: 2 occurrences

- Line 117: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 20: fields `storage`, `parser`, `planner`, and `optimizer` are never read

#### `src\query\executor\cypher\factory.rs`: 2 occurrences

- Line 152: unused import: `CypherExecutorTrait`
- Line 161: function `create_test_storage` is never used

#### `src\query\executor\result_processing\aggregation.rs`: 2 occurrences

- Line 284: unused variable: `i`: help: if this is intentional, prefix it with an underscore: `_i`
- Line 284: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 2 occurrences

- Line 11: field `parameters` is never read
- Line 31: methods `evaluate_arithmetic`, `evaluate_logical`, `evaluate_relational`, `evaluate_unary`, `evaluate_function`, and `cast_value` are never used

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 222: field `config` is never read

#### `src\core\type_utils.rs`: 1 occurrences

- Line 136: associated functions `test_check_compatibility`, `test_check_compatibility_batch`, `test_literal_type`, `test_binary_operation_result_type`, and `test_should_cache_expression` are never used

#### `src\stats\graph_stats.rs`: 1 occurrences

- Line 40: field `enable_space_level_metrics` is never read

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 1138: unused imports: `ExpandNode`, `ScanVerticesNode`, and `TraverseNode`

#### `src\cache\global_manager.rs`: 1 occurrences

- Line 138: creating a shared reference to mutable static: shared reference to mutable static

#### `src\query\planner\match_planning\core\match_clause_planner.rs`: 1 occurrences

- Line 37: field `paths` is never read

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 53: fields `signals` and `signal_info` are never read

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 48: field `data` is never read

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 32: field `exchange` is never read

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\cache\factory.rs`: 1 occurrences

- Line 12: unused import: `StatsCache`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 347: unused import: `UnaryOperator`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 334: method `args_to_hash` is never used

#### `src\cache\cache_impl\adaptive.rs`: 1 occurrences

- Line 12: variants `LFU` and `Hybrid` are never constructed

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 8: unused import: `crate::expression::ExpressionContext`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 381: field `original_index` is never read

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 18: field `match_clause_ctx` is never read

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 58: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 340: variable does not need to be mutable

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 466: unused import: `DedupNode as Dedup`

#### `src\expression\visitor.rs`: 1 occurrences

- Line 287: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 8: field `query_text` is never read

#### `src\query\planner\match_planning\clauses\return_clause_planner.rs`: 1 occurrences

- Line 215: function `get_yield_columns` is never used

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 564: unused import: `SortNode`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\core\context\mod.rs`: 1 occurrences

- Line 22: ambiguous glob re-exports: the name `SessionInfo` in the type namespace is first re-exported here

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 491: unused import: `std::sync::Arc`

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\visitor\mod.rs`: 1 occurrences

- Line 147: variable does not need to be mutable

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\context\execution\query_execution.rs`: 1 occurrences

- Line 560: variable does not need to be mutable

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 198: method `create_null_right_row` is never used

#### `src\core\mod.rs`: 1 occurrences

- Line 46: ambiguous glob re-exports: the name `SymbolType` in the type namespace is first re-exported here

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 84: method `execute_multi_way_cartesian_product` is never used

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 51: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\visitor\deduce_type_visitor.rs`: 1 occurrences

- Line 518: methods `visit_property`, `visit_set`, and `parse_type_def` are never used

#### `src\query\executor\cypher\clauses\match_path\expression_evaluator.rs`: 1 occurrences

- Line 318: variable does not need to be mutable

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 206: method `update_iterator_and_value` is never used

