# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 130
- **Total Issues**: 130
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 89
- **Files with Issues**: 89

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 130

### Warning Type Breakdown

- **warning**: 130 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 8 warnings
- `src\api\service\index_service.rs`: 6 warnings
- `src\query\executor\aggregation.rs`: 5 warnings
- `src\storage\redb_storage.rs`: 4 warnings
- `src\query\parser\lexer\lexer.rs`: 3 warnings
- `src\storage\memory_storage.rs`: 3 warnings
- `src\query\optimizer\optimizer.rs`: 3 warnings
- `src\query\visitor\deduce_type_visitor.rs`: 3 warnings
- `src\query\optimizer\plan_validator.rs`: 2 warnings
- `src\query\scheduler\async_scheduler.rs`: 2 warnings

## Detailed Warning Categorization

### warning: fields `unwind_expr` and `variable` are never read

**Total Occurrences**: 130  
**Unique Files**: 89

#### `src\query\executor\result_processing\sort.rs`: 8 occurrences

- Line 688: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 712: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 736: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- ... 5 more occurrences in this file

#### `src\api\service\index_service.rs`: 6 occurrences

- Line 419: unused import: `crate::core::Tag`
- Line 462: unused `std::result::Result` that must be used
- Line 479: unused `std::result::Result` that must be used
- ... 3 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 5 occurrences

- Line 530: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 558: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 559: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- ... 2 more occurrences in this file

#### `src\storage\redb_storage.rs`: 4 occurrences

- Line 286: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`
- Line 336: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`
- Line 45: constant `SCHEMA_TABLE` is never used
- ... 1 more occurrences in this file

#### `src\query\parser\lexer\lexer.rs`: 3 occurrences

- Line 1041: variable does not need to be mutable
- Line 1055: variable does not need to be mutable
- Line 292: method `read_raw_string` is never used

#### `src\query\visitor\deduce_type_visitor.rs`: 3 occurrences

- Line 256: methods `visit_property` and `visit_set` are never used
- Line 1220: fields `storage`, `validate_context`, `inputs`, `space`, and `vid_type` are never read
- Line 1240: multiple methods are never used

#### `src\query\optimizer\optimizer.rs`: 3 occurrences

- Line 307: struct `DummyPlanNode` is never constructed
- Line 316: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used
- Line 1168: associated function `find_group_by_id_mut` is never used

#### `src\storage\memory_storage.rs`: 3 occurrences

- Line 5: unused imports: `EdgeId` and `TagId`
- Line 175: variable does not need to be mutable
- Line 18: fields `vertex_props` and `memory_pool` are never read

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 31: field `thread_pool` is never read
- Line 99: multiple methods are never used

#### `src\query\context\request_context.rs`: 2 occurrences

- Line 1071: variable does not need to be mutable
- Line 1080: unused `std::result::Result` that must be used

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 456: unused import: `OptGroup`
- Line 459: function `create_test_context` is never used

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\context\managers\impl\storage_client_impl.rs`: 2 occurrences

- Line 20: field `storage_path` is never read
- Line 211: associated functions `vertex_key` and `edge_key_string` are never used

#### `src\query\planner\statements\path_planner.rs`: 2 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`
- Line 25: field `query_context` is never read

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 48: fields `storage` and `execution_context` are never read
- Line 76: multiple methods are never used

#### `src\common\network.rs`: 2 occurrences

- Line 445: unused import: `std::net::TcpStream`
- Line 576: unused variable: `byte`: help: if this is intentional, prefix it with an underscore: `_byte`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 424: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- Line 27: field `props` is never read

#### `src\query\executor\data_processing\join\hash_table.rs`: 2 occurrences

- Line 170: method `clear` is never used
- Line 335: field `config` is never read

#### `src\core\context\manager.rs`: 2 occurrences

- Line 96: field `created_at` is never read
- Line 306: method `is_max_contexts_exceeded` is never used

#### `src\query\validator\strategies\expression_operations.rs`: 2 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`
- Line 384: method `validate_pattern_comprehension` is never used

#### `src\query\context\managers\impl\index_manager_impl.rs`: 2 occurrences

- Line 103: methods `lookup_vertex_by_id` and `lookup_edge_by_id` are never used
- Line 1355: associated functions `vertex_matches_values`, `edge_matches_values`, `extract_vertex_field_value`, and `extract_edge_field_value` are never used

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 13: fields `unwind_expr` and `variable` are never read

#### `src\common\thread.rs`: 1 occurrences

- Line 92: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 11: field `with_items` is never read

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\core\result\result_core.rs`: 1 occurrences

- Line 252: method `update_iterator_and_value` is never used

#### `src\index\binary.rs`: 1 occurrences

- Line 315: unused import: `TimeValue`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 1 occurrences

- Line 158: method `set_error` is never used

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 527: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\visitor\deduce_alias_type_visitor.rs`: 1 occurrences

- Line 97: method `set_error` is never used

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\validator\set_validator.rs`: 1 occurrences

- Line 28: field `base` is never read

#### `src\query\visitor\extract_group_suite_visitor.rs`: 1 occurrences

- Line 120: methods `set_error` and `is_aggregate_function` are never used

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 457: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 26: field `query_context` is never read

#### `src\query\validator\sequential_validator.rs`: 1 occurrences

- Line 18: field `base` is never read

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 549: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\core\match_clause_planner.rs`: 1 occurrences

- Line 35: field `paths` is never read

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 1 occurrences

- Line 76: method `set_error` is never used

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\core\result\result_iterator.rs`: 1 occurrences

- Line 47: field `data` is never read

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\visitor\vid_extract_visitor.rs`: 1 occurrences

- Line 171: method `set_error` is never used

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 478: function `is_filter_expression` is never used

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\visitor\validate_pattern_expression_visitor.rs`: 1 occurrences

- Line 83: method `and_all` is never used

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\core\type_utils.rs`: 1 occurrences

- Line 136: associated functions `test_check_compatibility`, `test_check_compatibility_batch`, `test_literal_type`, `test_binary_operation_result_type`, and `test_should_cache_expression` are never used

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 24: field `return_items` is never read

#### `src\core\result\memory_manager.rs`: 1 occurrences

- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 243: field `planners` is never read

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 13: field `yield_items` is never read

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 204: methods `validate_expression_cycles` and `calculate_expression_depth` are never used

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 460: unused import: `super::*`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 368: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\visitor\property_tracker_visitor.rs`: 1 occurrences

- Line 183: method `set_error` is never used

#### `src\query\validator\pipe_validator.rs`: 1 occurrences

- Line 11: field `base` is never read

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 321: unused import: `crate::storage::StorageEngine`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 13: field `order_items` is never read

#### `src\query\planner\statements\go_planner.rs`: 1 occurrences

- Line 26: field `query_context` is never read

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 13: fields `skip` and `limit` are never read

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\validator\find_path_validator.rs`: 1 occurrences

- Line 41: field `base` is never read

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\core\signal_handler.rs`: 1 occurrences

- Line 52: fields `signals` and `signal_info` are never read

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_access.rs`: 1 occurrences

- Line 272: field `edge_types` is never read

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 334: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 152: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\query\planner\statements\clauses\projection_planner.rs`: 1 occurrences

- Line 9: field `projection_items` is never read

#### `src\query\validator\order_by_validator.rs`: 1 occurrences

- Line 13: field `base` is never read

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 287: multiple methods are never used

