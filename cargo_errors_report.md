# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 172
- **Total Issues**: 172
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 120
- **Files with Issues**: 95

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 172

### Warning Type Breakdown

- **warning**: 172 warnings

### Files with Warnings (Top 10)

- `src\query\executor\aggregation.rs`: 8 warnings
- `src\api\service\index_service.rs`: 8 warnings
- `src\query\executor\result_processing\sort.rs`: 8 warnings
- `src\query\validator\strategies\type_inference.rs`: 6 warnings
- `src\query\parser\lexer\lexer.rs`: 4 warnings
- `src\query\parser\mod.rs`: 4 warnings
- `src\query\planner\statements\path_planner.rs`: 4 warnings
- `src\query\context\managers\impl\storage_client_impl.rs`: 4 warnings
- `src\query\executor\result_processing\topn.rs`: 3 warnings
- `src\query\optimizer\plan_validator.rs`: 3 warnings

## Detailed Warning Categorization

### warning: field `dependencies_vec` is never read

**Total Occurrences**: 172  
**Unique Files**: 95

#### `src\query\executor\result_processing\sort.rs`: 8 occurrences

- Line 689: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- Line 714: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- Line 739: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- ... 5 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 8 occurrences

- Line 487: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 519: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 535: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- ... 5 more occurrences in this file

#### `src\api\service\index_service.rs`: 8 occurrences

- Line 13: unused import: `lru::LruCache`
- Line 14: unused import: `std::collections::HashMap`
- Line 15: unused imports: `Hash` and `Hasher`
- ... 5 more occurrences in this file

#### `src\query\validator\strategies\type_inference.rs`: 6 occurrences

- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`
- Line 37: trait `ExpressionValidationContext` is more private than the item `type_inference::TypeInference::validate_expression_type`: method `type_inference::TypeInference::validate_expression_type` is reachable at visibility `pub`
- Line 47: trait `ExpressionValidationContext` is more private than the item `type_inference::TypeInference::validate_expression_type_full`: method `type_inference::TypeInference::validate_expression_type_full` is reachable at visibility `pub`
- ... 3 more occurrences in this file

#### `src\query\planner\statements\path_planner.rs`: 4 occurrences

- Line 83: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`
- Line 84: unused variable: `max_hops`: help: if this is intentional, prefix it with an underscore: `_max_hops`
- Line 173: unused variable: `path_ctx`: help: if this is intentional, prefix it with an underscore: `_path_ctx`
- ... 1 more occurrences in this file

#### `src\query\parser\mod.rs`: 4 occurrences

- Line 18: ambiguous glob re-exports: the name `SetParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `ReturnParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `WithParser` in the type namespace is first re-exported here
- ... 1 more occurrences in this file

#### `src\query\parser\lexer\lexer.rs`: 4 occurrences

- Line 126: unused variable: `start_column`: help: if this is intentional, prefix it with an underscore: `_start_column`
- Line 1042: variable does not need to be mutable
- Line 1056: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\context\managers\impl\storage_client_impl.rs`: 4 occurrences

- Line 86: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`
- Line 115: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`
- Line 20: field `storage_path` is never read
- ... 1 more occurrences in this file

#### `src\query\optimizer\plan_validator.rs`: 3 occurrences

- Line 459: unused import: `OptGroup`
- Line 445: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`
- Line 462: function `create_test_context` is never used

#### `src\query\optimizer\predicate_pushdown.rs`: 3 occurrences

- Line 10: unused import: `crate::core::types::EdgeDirection`
- Line 62: variable does not need to be mutable
- Line 131: variable does not need to be mutable

#### `src\query\executor\result_processing\topn.rs`: 3 occurrences

- Line 641: type `TopNItem` is more private than the item `topn::TopNExecutor::<S>::push_to_heap`: method `topn::TopNExecutor::<S>::push_to_heap` is reachable at visibility `pub`
- Line 658: type `TopNItem` is more private than the item `topn::TopNExecutor::<S>::pop_from_heap`: method `topn::TopNExecutor::<S>::pop_from_heap` is reachable at visibility `pub`
- Line 287: multiple methods are never used

#### `src\index\binary.rs`: 3 occurrences

- Line 14: unused imports: `DurationValue`, `Edge`, `GeographyValue`, and `Vertex`
- Line 315: unused import: `TimeValue`
- Line 285: variable does not need to be mutable

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 256: methods `visit_property` and `visit_set` are never used
- Line 1220: fields `storage`, `validate_context`, `inputs`, `space`, and `vid_type` are never read
- Line 1240: multiple methods are never used

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`
- Line 520: unexpected `cfg` condition value: `system_monitor`
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\index\storage.rs`: 3 occurrences

- Line 215: unused variable: `field_name`: help: if this is intentional, prefix it with an underscore: `_field_name`
- Line 252: unused variable: `field_name`: help: if this is intentional, prefix it with an underscore: `_field_name`
- Line 88: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\parser\parser\utils.rs`: 2 occurrences

- Line 7: unused imports: `OrderByClause`, `OrderByItem`, `ReturnClause`, `ReturnItem`, `YieldClause`, and `YieldItem`
- Line 9: unused import: `OrderDirection`

#### `src\query\executor\data_processing\join\hash_table.rs`: 2 occurrences

- Line 170: method `clear` is never used
- Line 335: field `config` is never read

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 2 occurrences

- Line 4: unused import: `QueryInfo`
- Line 13: fields `skip` and `limit` are never read

#### `src\query\context\request_context.rs`: 2 occurrences

- Line 1071: variable does not need to be mutable
- Line 1080: unused `std::result::Result` that must be used

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 2 occurrences

- Line 6: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`
- Line 14: field `yield_items` is never read

#### `src\query\optimizer\optimizer.rs`: 2 occurrences

- Line 305: struct `DummyPlanNode` is never constructed
- Line 314: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used

#### `src\query\validator\strategies\expression_operations.rs`: 2 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`
- Line 384: method `validate_pattern_comprehension` is never used

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 52: fields `storage` and `execution_context` are never read
- Line 70: methods `execute_executor` and `get_executable_executors` are never used

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 2 occurrences

- Line 8: unused import: `crate::query::validator::structs::CypherClauseKind`
- Line 12: field `with_items` is never read

#### `src\query\planner\statements\clauses\projection_planner.rs`: 2 occurrences

- Line 6: unused import: `crate::query::validator::structs::CypherClauseKind`
- Line 10: field `projection_items` is never read

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 23: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`
- Line 25: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 2 occurrences

- Line 4: unused import: `QueryInfo`
- Line 13: field `order_items` is never read

#### `src\query\context\managers\impl\index_manager_impl.rs`: 2 occurrences

- Line 103: methods `lookup_vertex_by_id` and `lookup_edge_by_id` are never used
- Line 1355: associated functions `vertex_matches_values`, `edge_matches_values`, `extract_vertex_field_value`, and `extract_edge_field_value` are never used

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 2 occurrences

- Line 6: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`
- Line 14: fields `unwind_expr` and `variable` are never read

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 2 occurrences

- Line 14: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`
- Line 25: field `return_items` is never read

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 424: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- Line 27: field `props` is never read

#### `src\query\planner\statements\seeks\index_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\core\context\manager.rs`: 2 occurrences

- Line 96: field `created_at` is never read
- Line 306: method `is_max_contexts_exceeded` is never used

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 26: field `query_context` is never read

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 321: unused import: `crate::storage::StorageEngine`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 317: unused import: `std::path::Path`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused imports: `CharsetInfo` and `SchemaChangeType`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\core\type_utils.rs`: 1 occurrences

- Line 136: associated functions `test_check_compatibility`, `test_check_compatibility_batch`, `test_literal_type`, `test_binary_operation_result_type`, and `test_should_cache_expression` are never used

#### `src\storage\memory_storage.rs`: 1 occurrences

- Line 15: field `vertex_props` is never read

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 8: unused import: `std::collections::HashSet`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 47: field `data` is never read

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\index\mod.rs`: 1 occurrences

- Line 18: ambiguous glob re-exports: the name `IndexStatus` in the type namespace is first re-exported here

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 12: unused import: `std::sync::Arc`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 478: function `is_filter_expression` is never used

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 457: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 368: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\api\session\session_manager.rs`: 1 occurrences

- Line 1: unused import: `error`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 10: unused import: `AtomicU64`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 152: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\planner\statements\go_planner.rs`: 1 occurrences

- Line 26: field `query_context` is never read

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\parser\clauses\skip_limit_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 204: methods `validate_expression_cycles` and `calculate_expression_depth` are never used

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 52: fields `signals` and `signal_info` are never read

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 527: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 5: unused imports: `PropertyDef` and `PropertyType`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 549: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 243: field `planners` is never read

#### `src\query\planner\statements\core\match_clause_planner.rs`: 1 occurrences

- Line 35: field `paths` is never read

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 252: method `update_iterator_and_value` is never used

#### `src\query\executor\graph_query_executor.rs`: 1 occurrences

- Line 79: multiple methods are never used

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 460: unused import: `super::*`

