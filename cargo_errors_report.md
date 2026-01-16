# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 97
- **Total Issues**: 98
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 50
- **Files with Issues**: 64

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0433]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\predicate_pushdown.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 97

### Warning Type Breakdown

- **warning**: 97 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 7 warnings
- `src\query\executor\aggregation.rs`: 7 warnings
- `src\query\parser\mod.rs`: 4 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 3 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\planner\statements\seeks\seek_strategy.rs`: 2 warnings
- `src\query\planner\statements\utils\connection_builder.rs`: 2 warnings
- `src\query\optimizer\plan_validator.rs`: 2 warnings
- `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 warnings
- `src\query\parser\parser\utils.rs`: 2 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: could not find `HashLeftJoinNode` in `nodes`: could not find `HashLeftJoinNode` in `nodes`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 1502: failed to resolve: could not find `HashLeftJoinNode` in `nodes`: could not find `HashLeftJoinNode` in `nodes`

## Detailed Warning Categorization

### warning: unused import: `crate::query::planner::plan::SubPlan`

**Total Occurrences**: 97  
**Unique Files**: 64

#### `src\query\executor\aggregation.rs`: 7 occurrences

- Line 487: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 519: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 535: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- ... 4 more occurrences in this file

#### `src\query\executor\result_processing\sort.rs`: 7 occurrences

- Line 689: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- Line 714: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- Line 739: unused variable: `db_path`: help: if this is intentional, prefix it with an underscore: `_db_path`
- ... 4 more occurrences in this file

#### `src\query\parser\mod.rs`: 4 occurrences

- Line 18: ambiguous glob re-exports: the name `SetParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `ReturnParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `WithParser` in the type namespace is first re-exported here
- ... 1 more occurrences in this file

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`
- Line 520: unexpected `cfg` condition value: `system_monitor`
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\optimizer\predicate_pushdown.rs`: 3 occurrences

- Line 10: unused import: `crate::core::types::EdgeDirection`
- Line 62: variable does not need to be mutable
- Line 131: variable does not need to be mutable

#### `src\query\planner\statements\utils\finder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\context\managers\impl\storage_client_impl.rs`: 2 occurrences

- Line 86: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`
- Line 115: unused variable: `e`: help: if this is intentional, prefix it with an underscore: `_e`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 459: unused import: `OptGroup`
- Line 445: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\planner\statements\utils\connection_builder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\index_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\parser\parser\utils.rs`: 2 occurrences

- Line 7: unused imports: `OrderByClause`, `OrderByItem`, `ReturnClause`, `ReturnItem`, `YieldClause`, and `YieldItem`
- Line 9: unused import: `OrderDirection`

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 23: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 25: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\api\session\session_manager.rs`: 1 occurrences

- Line 1: unused import: `error`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\query\parser\clauses\skip_limit_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 468: unused import: `DedupNode as Dedup`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 1071: variable does not need to be mutable

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 457: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 321: unused import: `crate::storage::StorageEngine`

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 5: unused imports: `PropertyDef` and `PropertyType`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 10: unused import: `AtomicU64`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 549: unused import: `crate::core::value::NullType`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 368: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 527: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 424: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 8: unused import: `std::collections::HashSet`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 317: unused import: `std::path::Path`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused imports: `CharsetInfo` and `SchemaChangeType`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 8: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

