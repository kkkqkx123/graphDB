# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 60
- **Total Warnings**: 76
- **Total Issues**: 136
- **Unique Error Patterns**: 18
- **Unique Warning Patterns**: 44
- **Files with Issues**: 76

## Error Statistics

**Total Errors**: 60

### Error Type Breakdown

- **error[E0308]**: 16 errors
- **error[E0412]**: 11 errors
- **error[E0433]**: 10 errors
- **error[E0432]**: 8 errors
- **error[E0369]**: 8 errors
- **error[E0560]**: 5 errors
- **error[E0277]**: 1 errors
- **error[E0004]**: 1 errors

### Files with Errors (Top 10)

- `src\storage\memory_storage.rs`: 12 errors
- `src\api\service\stats_manager.rs`: 10 errors
- `src\query\executor\data_processing\set_operations\base.rs`: 7 errors
- `src\query\executor\aggregation.rs`: 5 errors
- `src\api\service\schema_manager.rs`: 4 errors
- `src\api\service\index_manager.rs`: 4 errors
- `src\query\executor\data_processing\set_operations\minus.rs`: 2 errors
- `src\query\executor\data_processing\set_operations\union_all.rs`: 2 errors
- `src\query\executor\data_processing\set_operations\intersect.rs`: 2 errors
- `src\query\executor\data_processing\set_operations\union.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 76

### Warning Type Breakdown

- **warning**: 76 warnings

### Files with Warnings (Top 10)

- `src\query\parser\mod.rs`: 4 warnings
- `src\core\result\memory_manager.rs`: 3 warnings
- `src\query\optimizer\plan_validator.rs`: 2 warnings
- `src\query\planner\statements\seeks\vertex_seek.rs`: 2 warnings
- `src\query\planner\statements\utils\connection_builder.rs`: 2 warnings
- `src\query\planner\statements\seeks\seek_strategy.rs`: 2 warnings
- `src\query\optimizer\predicate_pushdown.rs`: 2 warnings
- `src\api\session\session_manager.rs`: 2 warnings
- `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 warnings
- `src\query\planner\statements\utils\finder.rs`: 2 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `i64`, found `Value`

**Total Occurrences**: 16  
**Unique Files**: 2

#### `src\api\service\stats_manager.rs`: 10 occurrences

- Line 210: mismatched types: expected `&str`, found `String`
- Line 216: mismatched types: expected `&str`, found `String`
- Line 222: mismatched types: expected `&str`, found `String`
- ... 7 more occurrences in this file

#### `src\storage\memory_storage.rs`: 6 occurrences

- Line 322: mismatched types: expected `i64`, found `Value`
- Line 338: mismatched types: expected `Box<Value>`, found `Value`
- Line 339: mismatched types: expected `Box<Value>`, found `Value`
- ... 3 more occurrences in this file

### error[E0412]: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`

**Total Occurrences**: 11  
**Unique Files**: 5

#### `src\query\executor\data_processing\set_operations\base.rs`: 7 occurrences

- Line 259: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`
- Line 260: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`
- Line 263: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`
- ... 4 more occurrences in this file

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 133: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 155: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 123: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 147: cannot find type `RocksDBStorage` in module `crate::storage`: help: a struct with a similar name exists: `MockStorage`

### error[E0433]: failed to resolve: could not find `RocksDBStorage` in `storage`: could not find `RocksDBStorage` in `storage`, help: a struct with a similar name exists: `MockStorage`

**Total Occurrences**: 10  
**Unique Files**: 6

#### `src\query\executor\aggregation.rs`: 5 occurrences

- Line 488: failed to resolve: could not find `rocksdb_storage` in `storage`: could not find `rocksdb_storage` in `storage`
- Line 521: failed to resolve: could not find `rocksdb_storage` in `storage`: could not find `rocksdb_storage` in `storage`
- Line 537: failed to resolve: could not find `rocksdb_storage` in `storage`: could not find `rocksdb_storage` in `storage`
- ... 2 more occurrences in this file

#### `src\query\executor\data_processing\set_operations\union.rs`: 1 occurrences

- Line 148: failed to resolve: could not find `RocksDBStorage` in `storage`: could not find `RocksDBStorage` in `storage`, help: a struct with a similar name exists: `MockStorage`

#### `src\query\executor\data_processing\set_operations\union_all.rs`: 1 occurrences

- Line 138: failed to resolve: could not find `RocksDBStorage` in `storage`: could not find `RocksDBStorage` in `storage`, help: a struct with a similar name exists: `MockStorage`

#### `src\api\session\client_session.rs`: 1 occurrences

- Line 234: failed to resolve: use of undeclared type `QueryError`: use of undeclared type `QueryError`

#### `src\query\executor\data_processing\set_operations\intersect.rs`: 1 occurrences

- Line 162: failed to resolve: could not find `RocksDBStorage` in `storage`: could not find `RocksDBStorage` in `storage`, help: a struct with a similar name exists: `MockStorage`

#### `src\query\executor\data_processing\set_operations\minus.rs`: 1 occurrences

- Line 170: failed to resolve: could not find `RocksDBStorage` in `storage`: could not find `RocksDBStorage` in `storage`, help: a struct with a similar name exists: `MockStorage`

### error[E0432]: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`

**Total Occurrences**: 8  
**Unique Files**: 8

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 419: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 452: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 616: unresolved import `crate::storage::rocksdb_storage`: could not find `rocksdb_storage` in `storage`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 363: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`

#### `src\services\context.rs`: 1 occurrences

- Line 233: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 522: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 11: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`, help: a similar name exists in the module: `MockStorage`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 163: unresolved import `crate::storage::RocksDBStorage`: no `RocksDBStorage` in `storage`

### error[E0369]: binary operation `==` cannot be applied to type `std::option::Option<service::index_manager::TagIndex>`: std::option::Option<service::index_manager::TagIndex>, std::option::Option<service::index_manager::TagIndex>

**Total Occurrences**: 8  
**Unique Files**: 2

#### `src\api\service\index_manager.rs`: 4 occurrences

- Line 249: binary operation `==` cannot be applied to type `std::option::Option<service::index_manager::TagIndex>`: std::option::Option<service::index_manager::TagIndex>, std::option::Option<service::index_manager::TagIndex>
- Line 252: binary operation `==` cannot be applied to type `std::option::Option<service::index_manager::TagIndex>`: std::option::Option<service::index_manager::TagIndex>, std::option::Option<service::index_manager::TagIndex>
- Line 271: binary operation `==` cannot be applied to type `std::option::Option<service::index_manager::EdgeIndex>`: std::option::Option<service::index_manager::EdgeIndex>, std::option::Option<service::index_manager::EdgeIndex>
- ... 1 more occurrences in this file

#### `src\api\service\schema_manager.rs`: 4 occurrences

- Line 219: binary operation `==` cannot be applied to type `std::option::Option<service::schema_manager::TagSchema>`: std::option::Option<service::schema_manager::TagSchema>, std::option::Option<service::schema_manager::TagSchema>
- Line 222: binary operation `==` cannot be applied to type `std::option::Option<service::schema_manager::TagSchema>`: std::option::Option<service::schema_manager::TagSchema>, std::option::Option<service::schema_manager::TagSchema>
- Line 240: binary operation `==` cannot be applied to type `std::option::Option<service::schema_manager::EdgeTypeSchema>`: std::option::Option<service::schema_manager::EdgeTypeSchema>, std::option::Option<service::schema_manager::EdgeTypeSchema>
- ... 1 more occurrences in this file

### error[E0560]: struct `vertex_edge_path::Vertex` has no field named `tag`: unknown field

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\storage\memory_storage.rs`: 5 occurrences

- Line 323: struct `vertex_edge_path::Vertex` has no field named `tag`: unknown field
- Line 341: struct `vertex_edge_path::Edge` has no field named `properties`: `vertex_edge_path::Edge` does not have this field
- Line 356: struct `vertex_edge_path::Vertex` has no field named `tag`: unknown field
- ... 2 more occurrences in this file

### error[E0277]: can't compare `i64` with `core::value::types::Value`: no implementation for `i64 == core::value::types::Value`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\memory_storage.rs`: 1 occurrences

- Line 368: can't compare `i64` with `core::value::types::Value`: no implementation for `i64 == core::value::types::Value`

### error[E0004]: non-exhaustive patterns: `core::error::DBError::Session(_)` and `core::error::DBError::Permission(_)` not covered: patterns `core::error::DBError::Session(_)` and `core::error::DBError::Permission(_)` not covered

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\scheduler\async_scheduler.rs`: 1 occurrences

- Line 90: non-exhaustive patterns: `core::error::DBError::Session(_)` and `core::error::DBError::Permission(_)` not covered: patterns `core::error::DBError::Session(_)` and `core::error::DBError::Permission(_)` not covered

## Detailed Warning Categorization

### warning: variable does not need to be mutable

**Total Occurrences**: 76  
**Unique Files**: 57

#### `src\query\parser\mod.rs`: 4 occurrences

- Line 18: ambiguous glob re-exports: the name `SetParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `ReturnParser` in the type namespace is first re-exported here
- Line 18: ambiguous glob re-exports: the name `WithParser` in the type namespace is first re-exported here
- ... 1 more occurrences in this file

#### `src\core\result\memory_manager.rs`: 3 occurrences

- Line 444: unexpected `cfg` condition value: `system_monitor`
- Line 520: unexpected `cfg` condition value: `system_monitor`
- Line 413: unused variable: `guard`: help: if this is intentional, prefix it with an underscore: `_guard`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 247: unused import: `std::collections::HashMap`
- Line 251: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\optimizer\predicate_pushdown.rs`: 2 occurrences

- Line 61: variable does not need to be mutable
- Line 130: variable does not need to be mutable

#### `src\query\parser\parser\utils.rs`: 2 occurrences

- Line 7: unused imports: `OrderByClause`, `OrderByItem`, `ReturnClause`, `ReturnItem`, `YieldClause`, and `YieldItem`
- Line 9: unused import: `OrderDirection`

#### `src\query\planner\statements\utils\finder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\seek_strategy.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\vertex_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\utils\connection_builder.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\api\session\session_manager.rs`: 2 occurrences

- Line 1: unused import: `error`
- Line 8: unused imports: `PermissionError` and `PermissionResult`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 2 occurrences

- Line 23: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`
- Line 25: unused import: `crate::query::planner::plan::factory::PlanNodeFactory`

#### `src\query\planner\statements\seeks\scan_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\planner\statements\seeks\index_seek.rs`: 2 occurrences

- Line 2: unused import: `crate::query::planner::plan::SubPlan`
- Line 3: unused import: `crate::query::planner::planner::PlannerError`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 457: unused import: `OptGroup`
- Line 443: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 568: unused variable: `type_inference`: help: if this is intentional, prefix it with an underscore: `_type_inference`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\core\result\result_builder.rs`: 1 occurrences

- Line 188: variable does not need to be mutable

#### `src\query\parser\clauses\where_clause_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 549: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\clauses\projection_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 572: unused import: `SortNode`

#### `src\query\context\managers\schema_manager.rs`: 1 occurrences

- Line 5: unused imports: `CharsetInfo` and `SchemaChangeType`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 247: variable does not need to be mutable

#### `src\api\service\graph_service.rs`: 1 occurrences

- Line 7: unused imports: `PermissionError` and `PermissionResult`

#### `src\query\context\managers\impl\meta_client_impl.rs`: 1 occurrences

- Line 317: unused import: `std::path::Path`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 486: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 496: unused import: `crate::core::value::NullType`

#### `src\core\context\session.rs`: 1 occurrences

- Line 6: unused import: `ContextExt`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 10: unused import: `crate::expression::ExpressionContext`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 1071: variable does not need to be mutable

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 8: unused import: `crate::query::validator::structs::CypherClauseKind`

#### `src\query\planner\statements\clauses\unwind_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 321: unused import: `crate::storage::StorageEngine`

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 468: unused import: `DedupNode as Dedup`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 1 occurrences

- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 10: unused import: `AtomicU64`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 967: unused import: `crate::core::value::NullType`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 4: unused import: `QueryInfo`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 7: unused import: `std::collections::HashSet`

#### `src\query\context\managers\meta_client.rs`: 1 occurrences

- Line 5: unused imports: `PropertyDef` and `PropertyType`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 360: unused import: `crate::core::value::NullType`

#### `src\query\parser\clauses\skip_limit_impl.rs`: 1 occurrences

- Line 3: unused import: `crate::query::parser::ast::*`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::query::planner::statements::utils::connection_strategy::UnifiedConnector`

#### `src\query\optimizer\scan_optimization.rs`: 1 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\optimizer\projection_pushdown.rs`: 1 occurrences

- Line 119: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

