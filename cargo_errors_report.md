# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 81
- **Total Issues**: 83
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 45
- **Files with Issues**: 67

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0596]**: 2 errors

### Files with Errors (Top 10)

- `src\query\executor\result_processing\sort.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 81

### Warning Type Breakdown

- **warning**: 81 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 2 warnings
- `src\services\session.rs`: 2 warnings
- `src\query\optimizer\plan_validator.rs`: 2 warnings
- `src\query\validator\go_validator.rs`: 2 warnings
- `src\query\validator\strategies\variable_validator.rs`: 2 warnings
- `src\query\optimizer\scan_optimization.rs`: 2 warnings
- `src\query\parser\expressions\mod.rs`: 2 warnings
- `src\query\validator\strategies\alias_strategy.rs`: 2 warnings
- `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 warnings
- `src\query\executor\data_access.rs`: 2 warnings

## Detailed Error Categorization

### error[E0596]: cannot borrow `executor` as mutable, as it is not declared as mutable: cannot borrow as mutable

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 739: cannot borrow `executor` as mutable, as it is not declared as mutable: cannot borrow as mutable
- Line 791: cannot borrow `executor` as mutable, as it is not declared as mutable: cannot borrow as mutable

## Detailed Warning Categorization

### warning: unreachable pattern: no value can reach this

**Total Occurrences**: 81  
**Unique Files**: 67

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`
- Line 104: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\result_processing\sort.rs`: 2 occurrences

- Line 614: unused import: `crate::config::test_config::test_config`
- Line 801: variable does not need to be mutable

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\optimizer\join_optimization.rs`: 2 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`
- Line 114: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 650: unused variable: `last_vertex_box`: help: if this is intentional, prefix it with an underscore: `_last_vertex_box`
- Line 661: unused variable: `end_vertex`: help: if this is intentional, prefix it with an underscore: `_end_vertex`

#### `src\core\expression_visitor.rs`: 2 occurrences

- Line 9: unused import: `crate::core::type_system::TypeUtils`
- Line 12: unused import: `crate::query::parser::ast::expression::*`

#### `src\services\session.rs`: 2 occurrences

- Line 53: unused variable: `client_info`: help: if this is intentional, prefix it with an underscore: `_client_info`
- Line 53: unused variable: `connection_info`: help: if this is intentional, prefix it with an underscore: `_connection_info`

#### `src\query\visitor\find_visitor.rs`: 2 occurrences

- Line 9: unused import: `crate::query::parser::ast::expression::*`
- Line 10: unused import: `std::collections::HashSet`

#### `src\query\validator\go_validator.rs`: 2 occurrences

- Line 617: unreachable pattern: no value can reach this
- Line 563: unreachable pattern: no value can reach this

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 111: unreachable pattern: no value can reach this
- Line 112: unreachable pattern: no value can reach this

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\parser\expressions\mod.rs`: 2 occurrences

- Line 5: unused import: `crate::query::parser::ast::*`
- Line 6: unused imports: `ParseError`, `TokenKind`, and `Token`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 437: unused import: `crate::query::context::execution::QueryContext`
- Line 439: unused import: `OptContext`

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`
- Line 124: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\validator\order_by_validator.rs`: 1 occurrences

- Line 220: unreachable pattern: no value can reach this

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 375: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 657: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 510: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 361: unused imports: `ListExpression`, `MapExpression`, `PathExpression`, `PropertyAccessExpression`, `RangeExpression`, and `SubscriptExpression`

#### `src\query\visitor\deduce_alias_type_visitor.rs`: 1 occurrences

- Line 16: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 1017: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\query\visitor\rewrite_visitor.rs`: 1 occurrences

- Line 15: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 9: unused import: `crate::query::parser::core::position::Position`

#### `src\query\visitor\property_tracker_visitor.rs`: 1 occurrences

- Line 151: unused variable: `alias`: help: if this is intentional, prefix it with an underscore: `_alias`

#### `src\query\visitor\deduce_props_visitor.rs`: 1 occurrences

- Line 8: unused import: `crate::query::parser::ast::expression::*`

#### `src\index\binary.rs`: 1 occurrences

- Line 329: unused import: `TimeValue`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 107: variable does not need to be mutable

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\query\visitor\validate_pattern_expression_visitor.rs`: 1 occurrences

- Line 15: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 617: unused import: `super::super::schema::SchemaValidationError`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\visitor\vid_extract_visitor.rs`: 1 occurrences

- Line 16: unused import: `crate::query::parser::ast::expression::*`

#### `src\storage\redb_storage.rs`: 1 occurrences

- Line 334: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 408: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 1 occurrences

- Line 14: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\planner\statements\path_planner.rs`: 1 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 12: unused import: `crate::core::Expression`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 1080: variable does not need to be mutable

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\executor\aggregation.rs`: 1 occurrences

- Line 568: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 7: unused import: `crate::core::Value`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 634: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`

#### `src\storage\memory_storage.rs`: 1 occurrences

- Line 175: variable does not need to be mutable

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 1 occurrences

- Line 15: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 726: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 524: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 322: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 675: variable does not need to be mutable

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 267: unused import: `UnaryOperator`

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 7: unused import: `BinaryOperator`

#### `src\query\visitor\extract_group_suite_visitor.rs`: 1 occurrences

- Line 15: unused import: `crate::query::parser::ast::expression::*`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 91: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 109: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\common\thread.rs`: 1 occurrences

- Line 89: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 128: unreachable pattern: no value can reach this

#### `src\query\parser\lexer\lexer.rs`: 1 occurrences

- Line 907: variable does not need to be mutable

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 655: unreachable pattern: no value can reach this

