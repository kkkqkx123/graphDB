# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 188
- **Total Issues**: 188
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 115
- **Files with Issues**: 103

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 188

### Warning Type Breakdown

- **warning**: 188 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 8 warnings
- `src\query\validator\go_validator.rs`: 8 warnings
- `src\query\parser\lexer\lexer.rs`: 6 warnings
- `src\query\visitor\deduce_type_visitor.rs`: 6 warnings
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 6 warnings
- `src\api\service\index_service.rs`: 6 warnings
- `src\query\parser\parser\stmt_parser.rs`: 5 warnings
- `src\query\executor\aggregation.rs`: 5 warnings
- `src\query\validator\order_by_validator.rs`: 5 warnings
- `src\storage\redb_storage.rs`: 4 warnings

## Detailed Warning Categorization

### warning: unused import: `DataType`

**Total Occurrences**: 188  
**Unique Files**: 103

#### `src\query\validator\go_validator.rs`: 8 occurrences

- Line 386: unreachable pattern: no value can reach this
- Line 392: unreachable pattern: no value can reach this
- Line 334: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`
- ... 5 more occurrences in this file

#### `src\query\executor\result_processing\sort.rs`: 8 occurrences

- Line 688: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 712: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 736: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- ... 5 more occurrences in this file

#### `src\api\service\index_service.rs`: 6 occurrences

- Line 419: unused import: `crate::core::vertex_edge_path::Tag`
- Line 462: unused `std::result::Result` that must be used
- Line 479: unused `std::result::Result` that must be used
- ... 3 more occurrences in this file

#### `src\query\parser\lexer\lexer.rs`: 6 occurrences

- Line 6: unused import: `crate::query::parser::core::TokenKind`
- Line 10: unused import: `std::str::Chars`
- Line 719: unused variable: `end_col`: help: if this is intentional, prefix it with an underscore: `_end_col`
- ... 3 more occurrences in this file

#### `src\query\visitor\deduce_type_visitor.rs`: 6 occurrences

- Line 324: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 481: function cannot return without recursing: cannot return without recursing
- Line 36: fields `validate_context` and `inputs` are never read
- ... 3 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 6 occurrences

- Line 507: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- Line 551: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- Line 604: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- ... 3 more occurrences in this file

#### `src\query\parser\parser\stmt_parser.rs`: 5 occurrences

- Line 7: unused import: `crate::query::parser::ast::expr::*`
- Line 9: unused import: `crate::query::parser::ast::stmt::*`
- Line 11: unused import: `crate::query::parser::core::position::Position`
- ... 2 more occurrences in this file

#### `src\query\validator\order_by_validator.rs`: 5 occurrences

- Line 263: unreachable pattern: no value can reach this
- Line 276: unreachable pattern: no value can reach this
- Line 221: unreachable pattern: no value can reach this
- ... 2 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 5 occurrences

- Line 531: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 559: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 560: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- ... 2 more occurrences in this file

#### `src\storage\redb_storage.rs`: 4 occurrences

- Line 286: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`
- Line 336: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`
- Line 45: constant `SCHEMA_TABLE` is never used
- ... 1 more occurrences in this file

#### `src\expression\visitor.rs`: 3 occurrences

- Line 5: unused import: `DataType`
- Line 6: unused imports: `AggregateFunction`, `BinaryOperator`, and `UnaryOperator`
- Line 7: unused import: `crate::core::Value`

#### `src\query\executor\data_access.rs`: 3 occurrences

- Line 650: unused variable: `last_vertex_box`: help: if this is intentional, prefix it with an underscore: `_last_vertex_box`
- Line 661: unused variable: `end_vertex`: help: if this is intentional, prefix it with an underscore: `_end_vertex`
- Line 272: field `edge_types` is never read

#### `src\query\optimizer\plan_validator.rs`: 3 occurrences

- Line 438: unused import: `crate::api::session::session_manager::SessionInfo`
- Line 440: unused import: `OptGroup`
- Line 443: function `create_test_context` is never used

#### `src\storage\memory_storage.rs`: 3 occurrences

- Line 5: unused imports: `EdgeId` and `TagId`
- Line 175: variable does not need to be mutable
- Line 18: fields `vertex_props` and `memory_pool` are never read

#### `src\query\optimizer\optimizer.rs`: 3 occurrences

- Line 306: struct `DummyPlanNode` is never constructed
- Line 315: methods `id`, `type_name`, `dependencies`, `output_var`, `col_names`, and `cost` are never used
- Line 1110: associated function `find_group_by_id_mut` is never used

#### `src\query\visitor\property_tracker_visitor.rs`: 3 occurrences

- Line 15: unused import: `crate::query::parser::ast::expr::*`
- Line 136: field `entity_alias` is never read
- Line 184: method `set_error` is never used

#### `src\services\session.rs`: 2 occurrences

- Line 53: unused variable: `client_info`: help: if this is intentional, prefix it with an underscore: `_client_info`
- Line 53: unused variable: `connection_info`: help: if this is intentional, prefix it with an underscore: `_connection_info`

#### `src\query\context\managers\impl\index_manager_impl.rs`: 2 occurrences

- Line 103: methods `lookup_vertex_by_id` and `lookup_edge_by_id` are never used
- Line 1355: associated functions `vertex_matches_values`, `edge_matches_values`, `extract_vertex_field_value`, and `extract_edge_field_value` are never used

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 2 occurrences

- Line 408: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- Line 27: field `props` is never read

#### `src\query\planner\statements\path_planner.rs`: 2 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`
- Line 25: field `query_context` is never read

#### `src\query\visitor\deduce_alias_type_visitor.rs`: 2 occurrences

- Line 16: unused import: `crate::query::parser::ast::expr::*`
- Line 98: method `set_error` is never used

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 111: unreachable pattern: no value can reach this
- Line 112: unreachable pattern: no value can reach this

#### `src\query\context\managers\impl\storage_client_impl.rs`: 2 occurrences

- Line 21: field `storage_path` is never read
- Line 212: associated functions `vertex_key` and `edge_key_string` are never used

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\core\expression_visitor.rs`: 2 occurrences

- Line 9: unused import: `crate::core::type_system::TypeUtils`
- Line 12: unused import: `crate::query::parser::ast::expr::*`

#### `src\query\visitor\find_visitor.rs`: 2 occurrences

- Line 9: unused import: `crate::query::parser::ast::expr::*`
- Line 10: unused import: `std::collections::HashSet`

#### `src\query\validator\strategies\expression_operations.rs`: 2 occurrences

- Line 510: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`
- Line 357: methods `validate_pattern_comprehension` and `validate_reduce_expression` are never used

#### `src\query\visitor\vid_extract_visitor.rs`: 2 occurrences

- Line 16: unused import: `crate::query::parser::ast::expr::*`
- Line 172: method `set_error` is never used

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`
- Line 104: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 2 occurrences

- Line 15: unused import: `crate::query::parser::ast::expr::*`
- Line 159: method `set_error` is never used

#### `src\query\visitor\validate_pattern_expression_visitor.rs`: 2 occurrences

- Line 15: unused import: `crate::query::parser::ast::expr::*`
- Line 65: methods `add_local_variable`, `remove_local_variable`, and `and_all` are never used

#### `src\query\scheduler\async_scheduler.rs`: 2 occurrences

- Line 48: fields `storage` and `execution_context` are never read
- Line 76: multiple methods are never used

#### `src\query\parser\expressions\mod.rs`: 2 occurrences

- Line 5: unused import: `crate::query::parser::ast::*`
- Line 6: unused imports: `ParseError`, `TokenKind`, and `Token`

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`
- Line 124: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 2 occurrences

- Line 14: unused import: `crate::query::parser::ast::expr::*`
- Line 77: method `set_error` is never used

#### `src\query\visitor\extract_group_suite_visitor.rs`: 2 occurrences

- Line 15: unused import: `crate::query::parser::ast::expr::*`
- Line 121: methods `set_error` and `is_aggregate_function` are never used

#### `src\query\context\request_context.rs`: 2 occurrences

- Line 1080: variable does not need to be mutable
- Line 1089: unused `std::result::Result` that must be used

#### `src\query\optimizer\join_optimization.rs`: 2 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`
- Line 114: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 31: field `thread_pool` is never read
- Line 99: multiple methods are never used

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 9: unused import: `crate::query::parser::core::position::Position`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 11: field `with_items` is never read

#### `src\index\cache.rs`: 1 occurrences

- Line 140: method `access_count` is never used

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 13: fields `skip` and `limit` are never read

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 655: unreachable pattern: no value can reach this

#### `src\common\thread.rs`: 1 occurrences

- Line 89: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\index\storage.rs`: 1 occurrences

- Line 376: fields `space_id`, `index_id`, and `index_name` are never read

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 1017: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 267: unused import: `UnaryOperator`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 8: unused import: `crate::query::parser::ast::expr::*`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 109: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 524: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_processing\join\cross_join.rs`: 1 occurrences

- Line 82: method `execute_multi_way_cartesian_product` is never used

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 243: field `planners` is never read

#### `src\query\executor\traits.rs`: 1 occurrences

- Line 169: fields `id`, `name`, `description`, and `is_open` are never read

#### `src\query\validator\set_validator.rs`: 1 occurrences

- Line 28: field `base` is never read

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 7: unused import: `Value`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 6: unused import: `BinaryOperator`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 1 occurrences

- Line 9: field `projection_items` is never read

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 13: field `order_items` is never read

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 375: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\validator\find_path_validator.rs`: 1 occurrences

- Line 41: field `base` is never read

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 129: unreachable pattern: no value can reach this

#### `src\index\binary.rs`: 1 occurrences

- Line 329: unused import: `TimeValue`

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 322: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\data_processing\join\hash_table.rs`: 1 occurrences

- Line 170: method `clear` is never used

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\visitor\rewrite_visitor.rs`: 1 occurrences

- Line 15: unused import: `crate::query::parser::ast::expr::*`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 12: unused import: `crate::core::Expression`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 91: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\data_processing\join\left_join.rs`: 1 occurrences

- Line 204: method `create_null_right_row` is never used

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 13: fields `unwind_expr` and `variable` are never read

#### `src\query\validator\pipe_validator.rs`: 1 occurrences

- Line 11: field `base` is never read

#### `src\query\validator\sequential_validator.rs`: 1 occurrences

- Line 18: field `base` is never read

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 726: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 412: fields `index_name`, `index_type`, `properties`, and `tag_name` are never read

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 107: variable does not need to be mutable

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 13: field `yield_items` is never read

#### `src\storage\iterator\get_neighbors_iter.rs`: 1 occurrences

- Line 290: method `col_valid` is never used

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 617: unused import: `super::super::schema::SchemaValidationError`

#### `src\core\value\comparison.rs`: 1 occurrences

- Line 403: associated functions `cmp_coordinate_list` and `cmp_polygon_list` are never used

#### `src\query\planner\statements\go_planner.rs`: 1 occurrences

- Line 26: field `query_context` is never read

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 24: field `return_items` is never read

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 205: methods `validate_expression_cycles` and `calculate_expression_depth` are never used

#### `src\query\planner\plan\core\nodes\start_node.rs`: 1 occurrences

- Line 18: field `dependencies_vec` is never read

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 26: field `query_context` is never read

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 1 occurrences

- Line 213: function `is_filter_expression` is never used

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 675: variable does not need to be mutable

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\executor\result_processing\topn.rs`: 1 occurrences

- Line 287: multiple methods are never used

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 361: unused imports: `ListExpr`, `MapExpr`, `PathExpr`, `PropertyAccessExpr`, `RangeExpr`, and `SubscriptExpr`

#### `src\query\planner\statements\core\match_clause_planner.rs`: 1 occurrences

- Line 35: field `paths` is never read

