# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 103
- **Total Issues**: 103
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 89
- **Files with Issues**: 79

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 103

### Warning Type Breakdown

- **warning**: 103 warnings

### Files with Warnings (Top 10)

- `src\query\validator\strategies\type_inference.rs`: 7 warnings
- `src\query\executor\aggregation.rs`: 3 warnings
- `src\query\planner\match_planning\core\match_planner.rs`: 3 warnings
- `src\query\visitor\deduce_type_visitor.rs`: 3 warnings
- `src\query\executor\result_processing\topn.rs`: 3 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\executor\data_processing\join\hash_table.rs`: 3 warnings
- `src\query\planner\ngql\lookup_planner.rs`: 2 warnings
- `src\query\planner\match_planning\utils\finder.rs`: 2 warnings
- `src\query\validator\strategies\variable_validator.rs`: 2 warnings

## Detailed Warning Categorization

### warning: function `create_test_storage` is never used

**Total Occurrences**: 103  
**Unique Files**: 74

#### `src\query\validator\strategies\type_inference.rs`: 7 occurrences

- Line 77: unused variable: `arg`: help: try ignoring the field: `arg: _`
- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`
- Line 37: trait `ExpressionValidationContext` is more private than the item `type_inference::TypeInference::validate_expression_type`: method `type_inference::TypeInference::validate_expression_type` is reachable at visibility `pub`
- ... 4 more occurrences in this file

#### `src\query\planner\match_planning\core\match_planner.rs`: 3 occurrences

- Line 97: function `create_test_node_info` is never used
- Line 110: function `create_test_path` is never used
- Line 133: function `create_test_match_clause_context` is never used

#### `src\query\executor\aggregation.rs`: 3 occurrences

- Line 528: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 557: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 26: field `filter_condition` is never read

#### `src\query\executor\data_processing\join\hash_table.rs`: 3 occurrences

- Line 802: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`
- Line 170: method `clear` is never used
- Line 335: field `config` is never read

#### `src\query\executor\result_processing\topn.rs`: 3 occurrences

- Line 641: type `TopNItem` is more private than the item `topn::TopNExecutor::<S>::push_to_heap`: method `topn::TopNExecutor::<S>::push_to_heap` is reachable at visibility `pub`
- Line 658: type `TopNItem` is more private than the item `topn::TopNExecutor::<S>::pop_from_heap`: method `topn::TopNExecutor::<S>::pop_from_heap` is reachable at visibility `pub`
- Line 287: multiple methods are never used

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 520: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 254: methods `visit_property` and `visit_set` are never used
- Line 1192: fields `storage`, `validate_context`, `inputs`, `space`, and `vid_type` are never read
- Line 1212: multiple methods are never used

#### `src\query\planner\ngql\lookup_planner.rs`: 2 occurrences

- Line 119: unused variable: `score_expr`: help: if this is intentional, prefix it with an underscore: `_score_expr`
- Line 284: unused variable: `is_edge`: help: if this is intentional, prefix it with an underscore: `_is_edge`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 247: unused import: `std::collections::HashMap`
- Line 251: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\planner\match_planning\utils\finder.rs`: 2 occurrences

- Line 349: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`
- Line 356: unused variable: `finder`: help: if this is intentional, prefix it with an underscore: `_finder`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 2 occurrences

- Line 77: methods `lookup_vertex_by_id` and `lookup_edge_by_id` are never used
- Line 755: associated functions `vertex_matches_values`, `edge_matches_values`, `extract_vertex_field_value`, and `extract_edge_field_value` are never used

#### `src\storage\native_storage.rs`: 2 occurrences

- Line 14: field `schema_tree` is never read
- Line 88: method `value_from_bytes` is never used

#### `src\query\executor\data_modification.rs`: 2 occurrences

- Line 193: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`
- Line 388: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 815: variable does not need to be mutable
- Line 647: function `create_large_test_dataset` is never used

#### `src\core\context\manager.rs`: 2 occurrences

- Line 96: field `created_at` is never read
- Line 306: method `is_max_contexts_exceeded` is never used

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 182: struct `DummyPlanNode` is never constructed
- Line 191: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\query\validator\strategies\expression_operations.rs`: 2 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`
- Line 384: method `validate_pattern_comprehension` is never used

#### `src\query\executor\cypher\factory.rs`: 1 occurrences

- Line 161: function `create_test_storage` is never used

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 323: unused import: `std::fs`

#### `src\query\parser\cypher\parser.rs`: 1 occurrences

- Line 257: variable does not need to be mutable

#### `src\expression\visitor.rs`: 1 occurrences

- Line 282: unused variable: `children`: help: if this is intentional, prefix it with an underscore: `_children`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 468: unused import: `DedupNode as Dedup`

#### `src\query\planner\match_planning\utils\connection_strategy.rs`: 1 occurrences

- Line 488: unused import: `std::sync::Arc`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\context\managers\impl\storage_client_impl.rs`: 1 occurrences

- Line 218: associated functions `vertex_key` and `edge_key_string` are never used

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 100: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 244: field `planners` is never read

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\match_planning\clauses\return_clause_planner.rs`: 1 occurrences

- Line 215: function `get_yield_columns` is never used

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 313: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 247: variable does not need to be mutable

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 478: function `is_filter_expression` is never used

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 226: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 945: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\core\type_utils.rs`: 1 occurrences

- Line 136: associated functions `test_check_compatibility`, `test_check_compatibility_batch`, `test_literal_type`, `test_binary_operation_result_type`, and `test_should_cache_expression` are never used

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 52: fields `signals` and `signal_info` are never read

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 567: unused import: `SortNode`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 47: field `data` is never read

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 489: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 27: field `props` is never read

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 204: methods `validate_expression_cycles` and `calculate_expression_depth` are never used

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 33: field `exchange` is never read

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 337: method `args_to_hash` is never used

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 525: unused import: `crate::core::value::NullType`

#### `src\query\planner\match_planning\paths\shortest_path_planner.rs`: 1 occurrences

- Line 17: field `match_clause_ctx` is never read

#### `src\cache\cache_impl\adaptive.rs`: 1 occurrences

- Line 12: variants `LFU` and `Hybrid` are never constructed

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 478: unused import: `crate::core::value::NullType`

#### `src\query\planner\match_planning\utils\connection_builder.rs`: 1 occurrences

- Line 220: unused import: `crate::query::context::ast::base::AstContext`

#### `src\query\planner\ngql\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\planner\match_planning\clauses\order_by_planner.rs`: 1 occurrences

- Line 195: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `src\query\parser\cypher\expression_converter.rs`: 1 occurrences

- Line 299: unused imports: `FunctionCall`, `ListExpression`, `MapExpression`, `PropertyExpression`, and `UnaryExpression`

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 252: method `update_iterator_and_value` is never used

#### `src\query\planner\match_planning\core\match_clause_planner.rs`: 1 occurrences

- Line 36: field `paths` is never read

#### `src\query\planner\ngql\go_planner.rs`: 1 occurrences

- Line 61: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 256: field `edge_types` is never read

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 293: unused import: `crate::core::value::NullType`

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

