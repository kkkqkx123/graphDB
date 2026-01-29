# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 46
- **Total Warnings**: 110
- **Total Issues**: 156
- **Unique Error Patterns**: 32
- **Unique Warning Patterns**: 68
- **Files with Issues**: 65

## Error Statistics

**Total Errors**: 46

### Error Type Breakdown

- **error[E0308]**: 10 errors
- **error[E0659]**: 9 errors
- **error[E0277]**: 5 errors
- **error[E0599]**: 4 errors
- **error[E0432]**: 3 errors
- **error[E0034]**: 3 errors
- **error[E0369]**: 2 errors
- **error**: 2 errors
- **error[E0614]**: 1 errors
- **error[E0252]**: 1 errors
- **error[E0716]**: 1 errors
- **error[E0425]**: 1 errors
- **error[E0412]**: 1 errors
- **error[E0255]**: 1 errors
- **error[E0433]**: 1 errors
- **error[E0521]**: 1 errors

### Files with Errors (Top 10)

- `src\storage\transaction\lock.rs`: 6 errors
- `src\api\service\graph_service.rs`: 6 errors
- `src\storage\transaction\snapshot.rs`: 6 errors
- `src\storage\iterator\predicate.rs`: 6 errors
- `src\storage\transaction\log.rs`: 5 errors
- `src\storage\memory_storage.rs`: 5 errors
- `src\storage\transaction\mvcc.rs`: 3 errors
- `src\storage\transaction\traits.rs`: 2 errors
- `src\storage\iterator\composite.rs`: 2 errors
- `src\expression\storage\row_reader.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 110

### Warning Type Breakdown

- **warning**: 110 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\projection.rs`: 8 warnings
- `src\query\optimizer\elimination_rules.rs`: 7 warnings
- `src\storage\iterator\composite.rs`: 4 warnings
- `src\query\validator\insert_vertices_validator.rs`: 3 warnings
- `src\storage\iterator\predicate.rs`: 3 warnings
- `src\storage\plan\executors\mod.rs`: 3 warnings
- `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 warnings
- `src\storage\transaction\log.rs`: 3 warnings
- `src\query\planner\statements\match_planner.rs`: 3 warnings
- `src\query\planner\statements\paths\match_path_planner.rs`: 3 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `Option<Arc<T>>`, found `Option<Arc<&T>>`

**Total Occurrences**: 10  
**Unique Files**: 4

#### `src\storage\transaction\lock.rs`: 4 occurrences

- Line 113: mismatched types: expected `&str`, found `String`
- Line 117: mismatched types: expected `&str`, found `String`
- Line 121: mismatched types: expected `&str`, found `String`
- ... 1 more occurrences in this file

#### `src\storage\transaction\log.rs`: 3 occurrences

- Line 365: mismatched types: expected `Option<Result<u8, Error>>`, found `Result<_, _>`
- Line 366: mismatched types: expected an array with a size of 4, found one with a size of 2
- Line 370: mismatched types: expected `Option<Result<u8, Error>>`, found `Result<_, _>`

#### `src\storage\transaction\traits.rs`: 2 occurrences

- Line 179: mismatched types: expected `Option<Arc<T>>`, found `Option<Arc<&T>>`
- Line 344: mismatched types: expected `TransactionId`, found `&TransactionId`

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 76: mismatched types: expected `&Version`, found `Version`

### error[E0659]: `TransactionId` is ambiguous: ambiguous name

**Total Occurrences**: 9  
**Unique Files**: 4

#### `src\api\service\graph_service.rs`: 6 occurrences

- Line 261: `TransactionId` is ambiguous: ambiguous name
- Line 271: `TransactionId` is ambiguous: ambiguous name
- Line 281: `TransactionId` is ambiguous: ambiguous name
- ... 3 more occurrences in this file

#### `src\storage\memory_storage.rs`: 1 occurrences

- Line 1: `TransactionId` is ambiguous: ambiguous name

#### `src\storage\redb_storage.rs`: 1 occurrences

- Line 1: `TransactionId` is ambiguous: ambiguous name

#### `src\graph\transaction.rs`: 1 occurrences

- Line 2: `TransactionId` is ambiguous: ambiguous name

### error[E0277]: the trait bound `dyn predicate::Predicate: Clone` is not satisfied: the trait `Clone` is not implemented for `dyn predicate::Predicate`

**Total Occurrences**: 5  
**Unique Files**: 2

#### `src\storage\iterator\predicate.rs`: 3 occurrences

- Line 231: the trait bound `dyn predicate::Predicate: Clone` is not satisfied: the trait `Clone` is not implemented for `dyn predicate::Predicate`
- Line 367: the trait bound `dyn predicate::Predicate: Clone` is not satisfied: the trait `Clone` is not implemented for `dyn predicate::Predicate`
- Line 368: the trait bound `dyn predicate::Predicate: Clone` is not satisfied: the trait `Clone` is not implemented for `dyn predicate::Predicate`

#### `src\storage\transaction\log.rs`: 2 occurrences

- Line 320: the trait bound `storage::transaction::log::LogRecord: Encode` is not satisfied: the trait `Encode` is not implemented for `storage::transaction::log::LogRecord`
- Line 377: the trait bound `storage::transaction::log::LogRecord: Decode<()>` is not satisfied: the trait `Decode<()>` is not implemented for `storage::transaction::log::LogRecord`

### error[E0599]: no variant or associated item named `Int` found for enum `FieldType` in the current scope: variant or associated item not found in `FieldType`

**Total Occurrences**: 4  
**Unique Files**: 4

#### `src\expression\storage\row_reader.rs`: 1 occurrences

- Line 392: no variant or associated item named `Int` found for enum `FieldType` in the current scope: variant or associated item not found in `FieldType`

#### `src\storage\transaction\lock.rs`: 1 occurrences

- Line 626: no method named `is_failure` found for enum `storage::transaction::lock::LockResult` in the current scope: method not found in `LockResult`

#### `src\storage\iterator\composite.rs`: 1 occurrences

- Line 748: `composite::FilterIter<composite::CompositeIter<sequential_iter::SequentialIter>>` is not an iterator: `composite::FilterIter<composite::CompositeIter<sequential_iter::SequentialIter>>` is not an iterator

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 434: no method named `uses_snapshot` found for struct `snapshot::Snapshot` in the current scope: method not found in `Snapshot`

### error[E0432]: unresolved import `super::IterError`: no `IterError` in `storage::iterator`, help: a similar name exists in the module: `Iterator`

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\storage\iterator\composite.rs`: 1 occurrences

- Line 10: unresolved import `super::IterError`: no `IterError` in `storage::iterator`, help: a similar name exists in the module: `Iterator`

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 9: unresolved import `super::LockKey`: no `LockKey` in `storage::transaction`

#### `src\storage\iterator\mod.rs`: 1 occurrences

- Line 28: unresolved imports `storage_iter::StorageIterator`, `storage_iter::VertexIter`, `storage_iter::EdgeIter`, `storage_iter::PropIter`: no `StorageIterator` in `storage::iterator::storage_iter`, no `VertexIter` in `storage::iterator::storage_iter`, no `EdgeIter` in `storage::iterator::storage_iter`, no `PropIter` in `storage::iterator::storage_iter`, help: a similar name exists in the module: `StorageError`

### error[E0034]: multiple applicable items in scope: multiple `insert_edge` found

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\storage\memory_storage.rs`: 3 occurrences

- Line 1032: multiple applicable items in scope: multiple `insert_edge` found
- Line 1034: multiple applicable items in scope: multiple `get_edge` found
- Line 1054: multiple applicable items in scope: multiple `scan_vertices_by_tag` found

### error: lifetime may not live long enough: method was supposed to return data with lifetime `'2` but it is returning data with lifetime `'1`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\expression\storage\row_reader.rs`: 1 occurrences

- Line 67: lifetime may not live long enough: method was supposed to return data with lifetime `'2` but it is returning data with lifetime `'1`

#### `src\storage\memory_storage.rs`: 1 occurrences

- Line 933: lifetime may not live long enough: returning this value requires that `'1` must outlive `'static`

### error[E0369]: binary operation `==` cannot be applied to type `Vec<Box<dyn predicate::Predicate>>`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\storage\iterator\predicate.rs`: 1 occurrences

- Line 231: binary operation `==` cannot be applied to type `Vec<Box<dyn predicate::Predicate>>`

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 491: binary operation `==` cannot be applied to type `std::option::Option<&std::sync::Arc<mvcc::VersionRecord>>`: std::option::Option<&std::sync::Arc<mvcc::VersionRecord>>, std::option::Option<&std::sync::Arc<mvcc::VersionRecord>>

### error[E0433]: failed to resolve: use of undeclared type `Mutex`: use of undeclared type `Mutex`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 203: failed to resolve: use of undeclared type `Mutex`: use of undeclared type `Mutex`

### error[E0716]: temporary value dropped while borrowed: creates a temporary value which is freed while still in use

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\iterator\predicate.rs`: 1 occurrences

- Line 157: temporary value dropped while borrowed: creates a temporary value which is freed while still in use

### error[E0521]: borrowed data escapes outside of method: `predicate` escapes the method body here, argument requires that `'1` must outlive `'static`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\iterator\predicate.rs`: 1 occurrences

- Line 342: borrowed data escapes outside of method: `predicate` escapes the method body here, argument requires that `'1` must outlive `'static`

### error[E0252]: the name `HashMap` is defined multiple times: `HashMap` reimported here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 389: the name `HashMap` is defined multiple times: `HashMap` reimported here

### error[E0425]: cannot find value `locks` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\transaction\lock.rs`: 1 occurrences

- Line 401: cannot find value `locks` in this scope: not found in this scope

### error[E0255]: the name `VersionVec` is defined multiple times: `VersionVec` redefined here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 49: the name `VersionVec` is defined multiple times: `VersionVec` redefined here

### error[E0614]: type `mvcc::Version` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 267: type `mvcc::Version` cannot be dereferenced: can't be dereferenced

### error[E0412]: cannot find type `Mutex` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\storage\transaction\snapshot.rs`: 1 occurrences

- Line 189: cannot find type `Mutex` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused import: `SchemaManager`

**Total Occurrences**: 110  
**Unique Files**: 61

#### `src\query\executor\result_processing\projection.rs`: 8 occurrences

- Line 319: unused import: `DataSet`
- Line 321: unused import: `crate::query::executor::executor_enum::ExecutorEnum`
- Line 322: unused import: `crate::query::executor::base::BaseExecutor`
- ... 5 more occurrences in this file

#### `src\query\optimizer\elimination_rules.rs`: 7 occurrences

- Line 87: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 171: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- Line 316: unused variable: `output_var`: help: if this is intentional, prefix it with an underscore: `_output_var`
- ... 4 more occurrences in this file

#### `src\storage\iterator\composite.rs`: 4 occurrences

- Line 120: unused variable: `idx`: help: if this is intentional, prefix it with an underscore: `_idx`
- Line 141: unused variable: `row`: help: if this is intentional, prefix it with an underscore: `_row`
- Line 120: variable does not need to be mutable
- ... 1 more occurrences in this file

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 3 occurrences

- Line 23: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`
- Line 477: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 483: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\storage\iterator\predicate.rs`: 3 occurrences

- Line 10: unused import: `std::any::Any`
- Line 11: unused import: `std::collections::HashMap`
- Line 469: unused variable: `pred2`: help: if this is intentional, prefix it with an underscore: `_pred2`

#### `src\query\validator\insert_vertices_validator.rs`: 3 occurrences

- Line 204: unused import: `crate::core::Value`
- Line 48: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 79: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 3 occurrences

- Line 431: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`
- Line 437: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`
- Line 459: unused variable: `pattern`: help: if this is intentional, prefix it with an underscore: `_pattern`

#### `src\query\planner\statements\match_planner.rs`: 3 occurrences

- Line 75: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 296: unreachable pattern: no value can reach this
- Line 470: unused variable: `planner`: help: if this is intentional, prefix it with an underscore: `_planner`

#### `src\storage\transaction\log.rs`: 3 occurrences

- Line 15: unused import: `self`
- Line 456: unused variable: `flushed`: help: if this is intentional, prefix it with an underscore: `_flushed`
- Line 454: unused variable: `min_lsn`: help: if this is intentional, prefix it with an underscore: `_min_lsn`

#### `src\storage\plan\executors\mod.rs`: 3 occurrences

- Line 1: unused import: `ColumnSchema`
- Line 2: unused imports: `StorageError` and `Vertex`
- Line 3: unused import: `ScanResult`

#### `src\storage\memory_storage.rs`: 2 occurrences

- Line 1: unused import: `SchemaManager`
- Line 9: unused import: `RowReaderWrapper`

#### `src\query\executor\search_executors.rs`: 2 occurrences

- Line 13: unused import: `crate::expression::evaluator::traits::ExpressionContext`
- Line 358: value assigned to `vertices` is never read

#### `src\query\parser\ast\utils.rs`: 2 occurrences

- Line 14: unused variable: `span`: help: if this is intentional, prefix it with an underscore: `_span`
- Line 55: unused variable: `match_expression`: help: if this is intentional, prefix it with an underscore: `_match_expression`

#### `src\expression\context\row_context.rs`: 2 occurrences

- Line 249: function cannot return without recursing: cannot return without recursing
- Line 268: function cannot return without recursing: cannot return without recursing

#### `src\query\validator\update_validator.rs`: 2 occurrences

- Line 34: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 168: unused variable: `op`: help: try ignoring the field: `op: _`

#### `src\query\executor\graph_query_executor.rs`: 2 occurrences

- Line 138: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`
- Line 152: variable does not need to be mutable

#### `src\storage\transaction\traits.rs`: 2 occurrences

- Line 10: unused import: `Value`
- Line 11: unused imports: `LockManager`, `LogRecord`, `TransactionLog`, and `VersionVec`

#### `src\query\planner\statements\match_statement_planner.rs`: 2 occurrences

- Line 86: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 353: unreachable pattern: no value can reach this

#### `src\expression\context\basic_context.rs`: 2 occurrences

- Line 592: function cannot return without recursing: cannot return without recursing
- Line 611: function cannot return without recursing: cannot return without recursing

#### `src\expression\storage\row_reader.rs`: 2 occurrences

- Line 313: unreachable pattern: no value can reach this
- Line 326: unreachable pattern: no value can reach this

#### `src\query\executor\result_processing\dedup.rs`: 2 occurrences

- Line 494: unused import: `crate::query::executor::base::BaseExecutor`
- Line 495: unused import: `crate::query::executor::executor_enum::ExecutorEnum`

#### `src\expression\context\query_expression_context.rs`: 2 occurrences

- Line 444: function cannot return without recursing: cannot return without recursing
- Line 463: function cannot return without recursing: cannot return without recursing

#### `src\query\validator\insert_edges_validator.rs`: 2 occurrences

- Line 53: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`
- Line 84: unused variable: `edge_name`: help: if this is intentional, prefix it with an underscore: `_edge_name`

#### `src\expression\context\default_context.rs`: 2 occurrences

- Line 524: function cannot return without recursing: cannot return without recursing
- Line 543: function cannot return without recursing: cannot return without recursing

#### `src\storage\transaction\snapshot.rs`: 2 occurrences

- Line 9: unused import: `LockType`
- Line 389: unused import: `std::collections::HashMap`

#### `src\core\types\expression\visitor.rs`: 2 occurrences

- Line 149: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 177: unused variable: `variable`: help: if this is intentional, prefix it with an underscore: `_variable`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 961: variable does not need to be mutable
- Line 1009: variable does not need to be mutable

#### `src\query\optimizer\optimizer_config.rs`: 2 occurrences

- Line 134: unused import: `std::io::Write`
- Line 135: unused import: `tempfile::NamedTempFile`

#### `src\storage\iterator\storage_iter.rs`: 2 occurrences

- Line 10: unused imports: `Edge`, `Value`, and `Vertex`
- Line 11: unused import: `std::sync::Arc`

#### `src\expression\evaluator\expression_evaluator.rs`: 1 occurrences

- Line 437: unreachable pattern: no value can reach this

#### `src\expression\context\traits.rs`: 1 occurrences

- Line 5: unused import: `crate::core::error::ExpressionError`

#### `src\core\result\iterator.rs`: 1 occurrences

- Line 2: unused import: `crate::core::DBResult`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 45: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 50: unused import: `SpaceManageInfo`

#### `src\query\scheduler\execution_plan_analyzer.rs`: 1 occurrences

- Line 110: unused import: `crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 184: value assigned to `last_changes` is never read

#### `src\query\planner\statements\seeks\scan_seek.rs`: 1 occurrences

- Line 82: unused variable: `seek`: help: if this is intentional, prefix it with an underscore: `_seek`

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 71: variable does not need to be mutable

#### `src\storage\plan\nodes\mod.rs`: 1 occurrences

- Line 2: unused imports: `Edge` and `Vertex`

#### `src\query\executor\operation_kind_support.rs`: 1 occurrences

- Line 100: unused variable: `storage`: help: if this is intentional, prefix it with an underscore: `_storage`

#### `src\storage\storage_engine.rs`: 1 occurrences

- Line 7: unused import: `RowReaderWrapper`

#### `src\expression\evaluator\traits.rs`: 1 occurrences

- Line 55: unused variable: `name`: help: if this is intentional, prefix it with an underscore: `_name`

#### `src\storage\metadata\schema_manager.rs`: 1 occurrences

- Line 6: unused import: `IndexInfo`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 305: unused variable: `tag_name`: help: if this is intentional, prefix it with an underscore: `_tag_name`

#### `src\core\result\builder.rs`: 1 occurrences

- Line 2: unused import: `ResultMeta`

#### `src\query\executor\data_processing\join\base_join.rs`: 1 occurrences

- Line 365: unused variable: `col_name`: help: if this is intentional, prefix it with an underscore: `_col_name`

#### `src\query\optimizer\predicate_pushdown.rs`: 1 occurrences

- Line 180: unused variable: `ctx`: help: if this is intentional, prefix it with an underscore: `_ctx`

#### `src\query\planner\plan\execution_plan.rs`: 1 occurrences

- Line 68: unused variable: `n`: help: if this is intentional, prefix it with an underscore: `_n`

#### `src\query\context\managers\schema_traits.rs`: 1 occurrences

- Line 247: unexpected `cfg` condition value: `schema-manager-default`

#### `src\query\planner\statements\clauses\pagination_planner.rs`: 1 occurrences

- Line 36: unused variable: `ast_ctx`: help: if this is intentional, prefix it with an underscore: `_ast_ctx`

#### `src\query\visitor\ast_transformer.rs`: 1 occurrences

- Line 8: unused imports: `AlterStmt`, `Assignment`, `ChangePasswordStmt`, `CreateStmt`, `DeleteStmt`, `DescStmt`, `DropStmt`, `ExplainStmt`, `FetchStmt`, `FindPathStmt`, `GoStmt`, `InsertStmt`, `LookupStmt`, `MatchStmt`, `MergeStmt`, `PipeStmt`, `QueryStmt`, `RemoveStmt`, `ReturnStmt`, `SetStmt`, `ShowStmt`, `Stmt`, `SubgraphStmt`, `UnwindStmt`, `UpdateStmt`, `UseStmt`, and `WithStmt`

#### `src\query\planner\plan\core\nodes\plan_node_operations.rs`: 1 occurrences

- Line 348: unnecessary parentheses around function argument

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\storage\transaction\lock.rs`: 1 occurrences

- Line 11: unused import: `crate::core::StorageError`

#### `src\storage\mod.rs`: 1 occurrences

- Line 21: ambiguous glob re-exports: the name `TransactionId` in the type namespace is first re-exported here

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 450: unused variable: `test_expr`: help: if this is intentional, prefix it with an underscore: `_test_expr`

#### `src\storage\transaction\mvcc.rs`: 1 occurrences

- Line 11: unused imports: `TransactionState` and `VersionVec`

#### `src\query\planner\planner.rs`: 1 occurrences

- Line 191: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\query\validator\delete_validator.rs`: 1 occurrences

- Line 32: unused variable: `query_context`: help: if this is intentional, prefix it with an underscore: `_query_context`

#### `src\core\types\expression\expression.rs`: 1 occurrences

- Line 279: unused variable: `meta2`: help: if this is intentional, prefix it with an underscore: `_meta2`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 43: unused variable: `input_id`: help: if this is intentional, prefix it with an underscore: `_input_id`

