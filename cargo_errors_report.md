# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 126
- **Total Issues**: 127
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 73
- **Files with Issues**: 80

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0599]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\lookup_validator.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 126

### Warning Type Breakdown

- **warning**: 126 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 7 warnings
- `src\query\executor\aggregation.rs`: 7 warnings
- `src\query\scheduler\async_scheduler.rs`: 6 warnings
- `src\query\parser\mod.rs`: 4 warnings
- `src\index\binary.rs`: 3 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\validator\sequential_validator.rs`: 3 warnings
- `src\query\parser\lexer\lexer.rs`: 3 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 3 warnings
- `src\query\planner\statements\path_planner.rs`: 3 warnings

## Detailed Error Categorization

### error[E0599]: no method named `has_aggregate_expr` found for struct `Validator` in the current scope: method not found in `Validator`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\lookup_validator.rs`: 1 occurrences

- Line 111: no method named `has_aggregate_expr` found for struct `Validator` in the current scope: method not found in `Validator`

## Detailed Warning Categorization

### warning: unused import: `AtomicU64`

**Total Occurrences**: 126  
**Unique Files**: 79

#### `src\query\executor\result_processing\sort.rs`: 7 occurrences

- Line 689: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- Line 714: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- Line 739: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- ... 4 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 7 occurrences

- Line 487: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 519: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 535: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- ... 4 more occurrences in this file

#### `src\query\scheduler\async_scheduler.rs`: 6 occurrences

- Line 110: variable does not need to be mutable
- Line 132: variable does not need to be mutable
- Line 272: variable does not need to be mutable
- ... 3 more occurrences in this file

#### `src\query\parser\mod.rs`: 4 occurrences

- Line 18: ambiguous glob re-exports: the name `SetParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `ReturnParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `WithParser` in the type namespace is first re-exported here
- ... 1 more occurrences in this file

#### `src\query\planner\statements\path_planner.rs`: 3 occurrences

- Line 83: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`
- Line 84: unused variable: `max_hops`: help: if this is intentional, prefix it with an underscore: `_max_hops`
- Line 173: unused variable: `path_ctx`: help: if this is intentional, prefix it with an underscore: `_path_ctx`

#### `src\query\parser\lexer\lexer.rs`: 3 occurrences

- Line 126: unused variable: `start_column`: help: if this is intentional, prefix it with an underscore: `_start_column`
- Line 1042: variable does not need to be mutable
- Line 1056: variable does not need to be mutable

#### `src\query\validator\sequential_validator.rs`: 3 occurrences

- Line 5: unused import: `ValueType`
- Line 65: variable `has_ddl` is assigned to, but never used
- Line 80: value assigned to `has_ddl` is never read

#### `src\index\binary.rs`: 3 occurrences

- Line 14: unused imports: `DurationValue`, `Edge`, `GeographyValue`, and `Vertex`
- Line 315: unused import: `TimeValue`
- Line 285: variable does not need to be mutable

#### `src\query\optimizer\predicate_pushdown.rs`: 3 occurrences

- Line 10: unused import: `crate::core::types::EdgeDirection`
- Line 62: variable does not need to be mutable
- Line 131: variable does not need to be mutable

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`
- Line 520: unexpected `cfg` condition value: `system_monitor`
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 23: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`
- Line 25: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 459: unused import: `OptGroup`
- Line 445: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\index\storage.rs`: 2 occurrences

- Line 215: unused variable: `field_name`: help: if this is intentional, prefix it with an underscore: `_field_name`
- Line 252: unused variable: `field_name`: help: if this is intentional, prefix it with an underscore: `_field_name`

#### `src\query\validator\go_validator.rs`: 2 occurrences

- Line 165: unused variable: `filter`: help: if this is intentional, prefix it with an underscore: `_filter`
- Line 183: unused variable: `existing`: help: if this is intentional, prefix it with an underscore: `_existing`

#### `src\query\planner\statements\seeks\index_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\parser\parser\utils.rs`: 2 occurrences

- Line 7: unused imports: `OrderByClause`, `OrderByItem`, `ReturnClause`, `ReturnItem`, `YieldClause`, and `YieldItem`
- Line 9: unused import: `OrderDirection`

#### `src\query\validator\set_validator.rs`: 2 occurrences

- Line 5: unused import: `ValueType`
- Line 123: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\context\managers\impl\storage_client_impl.rs`: 2 occurrences

- Line 86: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`
- Line 115: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 10: unused import: `AtomicU64`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 317: unused import: `std::path::Path`

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\validator\find_path_validator.rs`: 1 occurrences

- Line 5: unused import: `ValueType`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 457: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 8: unused import: `std::collections::HashSet`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 321: unused import: `crate::storage::StorageEngine`

#### `src\query\parser\clauses\skip_limit_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 549: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 1071: variable does not need to be mutable

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 527: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 1088: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 8: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\validator\order_by_validator.rs`: 1 occurrences

- Line 117: unused variable: `expr`: help: if this is intentional, prefix it with an underscore: `_expr`

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\index\mod.rs`: 1 occurrences

- Line 18: ambiguous glob re-exports: the name `IndexStatus` in the type namespace is first re-exported here

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 5: unused imports: `PropertyDef` and `PropertyType`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 14: unused import: `crate::query::planner::plan::core::nodes::join_node::JoinConnector`

#### `src\api\service\graph_service.rs`: 1 occurrences

- Line 4: unused import: `DEFAULT_SESSION_IDLE_TIMEOUT`

#### `src\query\validator\mod.rs`: 1 occurrences

- Line 81: ambiguous glob re-exports: the name `PathType` in the type namespace is first re-exported here

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 368: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\validator\fetch_vertices_validator.rs`: 1 occurrences

- Line 187: unused variable: `prop_name`: help: if this is intentional, prefix it with an underscore: `_prop_name`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused imports: `CharsetInfo` and `SchemaChangeType`

#### `src\expression\context\basic_context.rs`: 1 occurrences

- Line 12: unused import: `std::sync::Arc`

#### `src\api\session\session_manager.rs`: 1 occurrences

- Line 1: unused import: `error`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 460: unused import: `super::*`

#### `src\query\validator\fetch_edges_validator.rs`: 1 occurrences

- Line 172: unused variable: `rank_expr`: help: if this is intentional, prefix it with an underscore: `_rank_expr`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 424: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\validator\pipe_validator.rs`: 1 occurrences

- Line 7: unused import: `crate::core::Expression`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

