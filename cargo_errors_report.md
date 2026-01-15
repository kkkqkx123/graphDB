# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 192
- **Total Issues**: 192
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 116
- **Files with Issues**: 98

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 192

### Warning Type Breakdown

- **warning**: 192 warnings

### Files with Warnings (Top 10)

- `src\query\executor\graph_query_executor.rs`: 22 warnings
- `src\core\value\conversion.rs`: 16 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 8 warnings
- `src\query\validator\strategies\type_inference.rs`: 7 warnings
- `src\query\context\managers\impl\index_manager_impl.rs`: 4 warnings
- `src\query\optimizer\plan_validator.rs`: 4 warnings
- `src\query\parser\mod.rs`: 4 warnings
- `src\query\context\request_context.rs`: 3 warnings
- `src\query\visitor\deduce_type_visitor.rs`: 3 warnings
- `src\query\executor\aggregation.rs`: 3 warnings

## Detailed Warning Categorization

### warning: unused import: `SortNode`

**Total Occurrences**: 192  
**Unique Files**: 93

#### `src\query\executor\graph_query_executor.rs`: 22 occurrences

- Line 108: unused variable: `clause`: help: if this is intentional, prefix it with an underscore: `_clause`
- Line 112: unused variable: `clause`: help: if this is intentional, prefix it with an underscore: `_clause`
- Line 116: unused variable: `clause`: help: if this is intentional, prefix it with an underscore: `_clause`
- ... 19 more occurrences in this file

#### `src\core\value\conversion.rs`: 16 occurrences

- Line 158: comparison is useless due to type limits
- Line 158: comparison is useless due to type limits
- Line 158: comparison is useless due to type limits
- ... 13 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 8 occurrences

- Line 70: unused variable: `new_filter_str`: help: if this is intentional, prefix it with an underscore: `_new_filter_str`
- Line 85: unused variable: `remaining_condition`: help: if this is intentional, prefix it with an underscore: `_remaining_condition`
- Line 139: unused variable: `new_filter_str`: help: if this is intentional, prefix it with an underscore: `_new_filter_str`
- ... 5 more occurrences in this file

#### `src\query\validator\strategies\type_inference.rs`: 7 occurrences

- Line 77: unused variable: `arg`: help: try ignoring the field: `arg: _`
- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`
- Line 37: trait `ExpressionValidationContext` is more private than the item `type_inference::TypeInference::validate_expression_type`: method `type_inference::TypeInference::validate_expression_type` is reachable at visibility `pub`
- ... 4 more occurrences in this file

#### `src\query\context\managers\impl\index_manager_impl.rs`: 4 occurrences

- Line 1155: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`
- Line 1168: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`
- Line 103: methods `lookup_vertex_by_id` and `lookup_edge_by_id` are never used
- ... 1 more occurrences in this file

#### `src\query\parser\mod.rs`: 4 occurrences

- Line 18: ambiguous glob re-exports: the name `SetParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `ReturnParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `WithParser` in the type namespace is first re-exported here
- ... 1 more occurrences in this file

#### `src\query\optimizer\plan_validator.rs`: 4 occurrences

- Line 457: unused import: `OptGroup`
- Line 443: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 443: unused variable: `group_id`: help: if this is intentional, prefix it with an underscore: `_group_id`
- ... 1 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 3 occurrences

- Line 536: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 565: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 26: field `filter_condition` is never read

#### `src\query\context\request_context.rs`: 3 occurrences

- Line 203: unused variable: `now`: help: if this is intentional, prefix it with an underscore: `_now`
- Line 1071: variable does not need to be mutable
- Line 1080: unused `std::result::Result` that must be used

#### `src\query\executor\result_processing\topn.rs`: 3 occurrences

- Line 641: type `TopNItem` is more private than the item `topn::TopNExecutor::<S>::push_to_heap`: method `topn::TopNExecutor::<S>::push_to_heap` is reachable at visibility `pub`
- Line 658: type `TopNItem` is more private than the item `topn::TopNExecutor::<S>::pop_from_heap`: method `topn::TopNExecutor::<S>::pop_from_heap` is reachable at visibility `pub`
- Line 287: multiple methods are never used

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 3 occurrences

- Line 23: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 25: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`
- Line 72: unused variable: `clause_ctx`: help: if this is intentional, prefix it with an underscore: `_clause_ctx`

#### `src\query\executor\data_processing\join\hash_table.rs`: 3 occurrences

- Line 802: unused variable: `evaluator`: help: if this is intentional, prefix it with an underscore: `_evaluator`
- Line 170: method `clear` is never used
- Line 335: field `config` is never read

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 256: methods `visit_property` and `visit_set` are never used
- Line 1199: fields `storage`, `validate_context`, `inputs`, `space`, and `vid_type` are never read
- Line 1219: multiple methods are never used

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 520: unexpected `cfg` condition value: `system_monitor`: help: remove the condition
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 2 occurrences

- Line 4: unused import: `QueryInfo`
- Line 13: field `order_items` is never read

#### `src\query\planner\statements\seeks\index_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 305: struct `DummyPlanNode` is never constructed
- Line 314: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\storage\native_storage.rs`: 2 occurrences

- Line 14: field `schema_tree` is never read
- Line 88: method `value_from_bytes` is never used

#### `src\query\planner\statements\utils\connection_builder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\parser\parser\utils.rs`: 2 occurrences

- Line 7: unused imports: `OrderByClause`, `OrderByItem`, `ReturnClause`, `ReturnItem`, `YieldClause`, and `YieldItem`
- Line 9: unused import: `OrderDirection`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 2 occurrences

- Line 6: unused import: `crate::query::validator::structs::CypherClauseKind`
- Line 10: field `projection_items` is never read

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 2 occurrences

- Line 8: unused import: `crate::query::validator::structs::CypherClauseKind`
- Line 12: field `with_items` is never read

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 247: unused import: `std::collections::HashMap`
- Line 251: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\context\managers\impl\storage_client_impl.rs`: 2 occurrences

- Line 20: field `storage_path` is never read
- Line 201: associated functions `vertex_key` and `edge_key_string` are never used

#### `src\query\executor\data_modification.rs`: 2 occurrences

- Line 201: unused variable: `id_str`: help: if this is intentional, prefix it with an underscore: `_id_str`
- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\core\expression_utils.rs`: 2 occurrences

- Line 7: unused import: `std::collections::HashSet`
- Line 669: associated function `get_constant_value` is never used

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 2 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 14: fields `unwind_expr` and `variable` are never read

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 2 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 14: fields `return_items` and `distinct` are never read

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\lookup_planner.rs`: 2 occurrences

- Line 119: unused variable: `score_expr`: help: if this is intentional, prefix it with an underscore: `_score_expr`
- Line 284: unused variable: `is_edge`: help: if this is intentional, prefix it with an underscore: `_is_edge`

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\query\planner\statements\clauses\yield_planner.rs`: 2 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 14: field `yield_items` is never read

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 822: variable does not need to be mutable
- Line 654: function `create_large_test_dataset` is never used

#### `src\query\planner\statements\utils\finder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\core\context\manager.rs`: 2 occurrences

- Line 96: field `created_at` is never read
- Line 306: method `is_max_contexts_exceeded` is never used

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 4: unused import: `QueryInfo`
- Line 13: fields `skip` and `limit` are never read

#### `src\query\validator\strategies\expression_operations.rs`: 2 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`
- Line 384: method `validate_pattern_comprehension` is never used

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 572: unused import: `SortNode`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 247: variable does not need to be mutable

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 496: unused import: `crate::core::value::NullType`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 317: unused import: `std::path::Path`

#### `src\query\planner\statements\subgraph_planner.rs`: 1 occurrences

- Line 52: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 252: method `update_iterator_and_value` is never used

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\context\ast\cypher_ast_context.rs`: 1 occurrences

- Line 226: unused variable: `label`: help: if this is intentional, prefix it with an underscore: `_label`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 152: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\cache\cache_impl\adaptive.rs`: 1 occurrences

- Line 12: variants `LFU` and `Hybrid` are never constructed

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 967: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\go_planner.rs`: 1 occurrences

- Line 61: unused variable: `expand_node`: help: if this is intentional, prefix it with an underscore: `_expand_node`

#### `src\core\type_utils.rs`: 1 occurrences

- Line 136: associated functions `test_check_compatibility`, `test_check_compatibility_batch`, `test_literal_type`, `test_binary_operation_result_type`, and `test_should_cache_expression` are never used

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\executor\data_processing\join\full_outer_join.rs`: 1 occurrences

- Line 107: unused variable: `right_col_map`: help: if this is intentional, prefix it with an underscore: `_right_col_map`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 374: unused variable: `var_name`: help: if this is intentional, prefix it with an underscore: `_var_name`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 244: field `planners` is never read

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 47: field `data` is never read

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 321: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 360: unused import: `crate::core::value::NullType`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused imports: `CharsetInfo` and `SchemaChangeType`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 468: unused import: `DedupNode as Dedup`

#### `src\core\context\base.rs`: 1 occurrences

- Line 100: unused variable: `event`: help: if this is intentional, prefix it with an underscore: `_event`

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 52: fields `signals` and `signal_info` are never read

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 204: methods `validate_expression_cycles` and `calculate_expression_depth` are never used

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 549: unused import: `crate::core::value::NullType`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 337: method `args_to_hash` is never used

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 5: unused imports: `PropertyDef` and `PropertyType`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 10: unused import: `AtomicU64`

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 27: field `props` is never read

#### `src\query\parser\clauses\skip_limit_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 486: unused import: `crate::core::value::NullType`

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 478: function `is_filter_expression` is never used

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\planner\statements\core\match_clause_planner.rs`: 1 occurrences

- Line 36: field `paths` is never read

#### `src\query\parser\parser\main_parser.rs`: 1 occurrences

- Line 238: unused variable: `yield_clause`: help: if this is intentional, prefix it with an underscore: `_yield_clause`

