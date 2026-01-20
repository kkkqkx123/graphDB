# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 30
- **Total Warnings**: 64
- **Total Issues**: 94
- **Unique Error Patterns**: 9
- **Unique Warning Patterns**: 37
- **Files with Issues**: 48

## Error Statistics

**Total Errors**: 30

### Error Type Breakdown

- **error[E0308]**: 9 errors
- **error[E0433]**: 9 errors
- **error[E0061]**: 5 errors
- **error[E0609]**: 4 errors
- **error[E0599]**: 3 errors

### Files with Errors (Top 10)

- `src\query\context\request_context.rs`: 10 errors
- `src\services\session.rs`: 5 errors
- `src\query\optimizer\plan_validator.rs`: 3 errors
- `src\query\optimizer\predicate_pushdown.rs`: 3 errors
- `src\query\optimizer\elimination_rules.rs`: 3 errors
- `src\query\optimizer\operation_merge.rs`: 3 errors
- `src\query\optimizer\limit_pushdown.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 64

### Warning Type Breakdown

- **warning**: 64 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 7 warnings
- `src\query\executor\aggregation.rs`: 4 warnings
- `src\services\session.rs`: 2 warnings
- `src\query\optimizer\scan_optimization.rs`: 2 warnings
- `src\query\validator\strategies\variable_validator.rs`: 2 warnings
- `src\query\optimizer\projection_pushdown.rs`: 2 warnings
- `src\query\optimizer\join_optimization.rs`: 2 warnings
- `src\storage\memory_storage.rs`: 2 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings
- `src\storage\redb_storage.rs`: 2 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `Option<i64>`, found `Option<&str>`

**Total Occurrences**: 9  
**Unique Files**: 1

#### `src\query\context\request_context.rs`: 9 occurrences

- Line 926: mismatched types: expected `Option<i64>`, found `Option<&str>`
- Line 944: mismatched types: expected `Option<i64>`, found `Option<&str>`
- Line 958: mismatched types: expected `Option<i64>`, found `Option<&str>`
- ... 6 more occurrences in this file

### error[E0433]: failed to resolve: could not find `session` in `context`: could not find `session` in `context`

**Total Occurrences**: 9  
**Unique Files**: 5

#### `src\query\optimizer\elimination_rules.rs`: 2 occurrences

- Line 578: failed to resolve: could not find `session` in `context`: could not find `session` in `context`
- Line 589: failed to resolve: could not find `query` in `context`: could not find `query` in `context`

#### `src\query\optimizer\limit_pushdown.rs`: 2 occurrences

- Line 897: failed to resolve: could not find `session` in `context`: could not find `session` in `context`
- Line 908: failed to resolve: could not find `query` in `context`: could not find `query` in `context`

#### `src\query\optimizer\predicate_pushdown.rs`: 2 occurrences

- Line 1311: failed to resolve: could not find `session` in `context`: could not find `session` in `context`
- Line 1322: failed to resolve: could not find `query` in `context`: could not find `query` in `context`

#### `src\query\optimizer\operation_merge.rs`: 2 occurrences

- Line 473: failed to resolve: could not find `session` in `context`: could not find `session` in `context`
- Line 484: failed to resolve: could not find `query` in `context`: could not find `query` in `context`

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 471: failed to resolve: could not find `query` in `context`: could not find `query` in `context`

### error[E0061]: this function takes 0 arguments but 4 arguments were supplied

**Total Occurrences**: 5  
**Unique Files**: 5

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 906: this function takes 0 arguments but 4 arguments were supplied

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 587: this function takes 0 arguments but 4 arguments were supplied

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 1320: this function takes 0 arguments but 4 arguments were supplied

#### `src\query\optimizer\operation_merge.rs`: 1 occurrences

- Line 482: this function takes 0 arguments but 4 arguments were supplied

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 469: this function takes 0 arguments but 4 arguments were supplied

### error[E0609]: no field `username` on type `session_manager::SessionInfo`: unknown field

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\services\session.rs`: 4 occurrences

- Line 327: no field `username` on type `session_manager::SessionInfo`: unknown field
- Line 330: no field `status` on type `session_manager::SessionInfo`: unknown field
- Line 385: no field `username` on type `session_manager::SessionInfo`: unknown field
- ... 1 more occurrences in this file

### error[E0599]: no function or associated item named `new` found for struct `session_manager::SessionInfo` in the current scope: function or associated item not found in `SessionInfo`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 460: no function or associated item named `new` found for struct `session_manager::SessionInfo` in the current scope: function or associated item not found in `SessionInfo`

#### `src\services\session.rs`: 1 occurrences

- Line 326: no method named `len` found for type `i64` in the current scope

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 914: no function or associated item named `new` found for struct `session_manager::SessionInfo` in the current scope: function or associated item not found in `SessionInfo`

## Detailed Warning Categorization

### warning: unused import: `SortNode`

**Total Occurrences**: 64  
**Unique Files**: 46

#### `src\query\executor\result_processing\sort.rs`: 7 occurrences

- Line 688: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 712: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 736: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- ... 4 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 4 occurrences

- Line 530: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 558: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 559: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- ... 1 more occurrences in this file

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 1041: variable does not need to be mutable
- Line 1055: variable does not need to be mutable

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 286: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`
- Line 336: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`

#### `src\services\session.rs`: 2 occurrences

- Line 50: unused variable: `client_info`: help: if this is intentional, prefix it with an underscore: `_client_info`
- Line 50: unused variable: `connection_info`: help: if this is intentional, prefix it with an underscore: `_connection_info`

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`
- Line 104: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\optimizer\join_optimization.rs`: 2 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`
- Line 114: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`
- Line 124: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\storage\memory_storage.rs`: 2 occurrences

- Line 5: unused imports: `EdgeId` and `TagId`
- Line 175: variable does not need to be mutable

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 375: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\api\service\index_service.rs`: 1 occurrences

- Line 419: unused import: `crate::core::Tag`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 1079: variable does not need to be mutable

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 460: unused import: `super::*`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 424: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 322: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 91: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 334: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 457: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 550: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 527: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 1017: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\planner\statements\path_planner.rs`: 1 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\common\thread.rs`: 1 occurrences

- Line 89: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\core\mod.rs`: 1 occurrences

- Line 57: unused import: `context::*`

#### `src\index\binary.rs`: 1 occurrences

- Line 329: unused import: `TimeValue`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListComprehensionExpr`, `ListExpr`, `MapExpr`, `PathExpr`, `PredicateExpr`, `PropertyAccessExpr`, `RangeExpr`, `ReduceExpr`, and `SubscriptExpr`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 230: unused variable: `query_text`: help: if this is intentional, prefix it with an underscore: `_query_text`

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 726: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 456: unused import: `OptGroup`

