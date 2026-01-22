# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 65
- **Total Warnings**: 103
- **Total Issues**: 168
- **Unique Error Patterns**: 22
- **Unique Warning Patterns**: 59
- **Files with Issues**: 65

## Error Statistics

**Total Errors**: 65

### Error Type Breakdown

- **error[E0061]**: 44 errors
- **error[E0609]**: 4 errors
- **error[E0726]**: 3 errors
- **error[E0433]**: 2 errors
- **error[E0369]**: 2 errors
- **error[E0599]**: 2 errors
- **error[E0308]**: 2 errors
- **error[E0277]**: 2 errors
- **error[E0621]**: 1 errors
- **error[E0106]**: 1 errors
- **error[E0596]**: 1 errors
- **error[E0432]**: 1 errors

### Files with Errors (Top 10)

- `src\query\parser\parser\stmt_parser.rs`: 25 errors
- `src\query\parser\ast\tests.rs`: 14 errors
- `src\query\parser\parser\clause_parser.rs`: 6 errors
- `src\query\parser\parser\pattern_parser.rs`: 5 errors
- `src\query\parser\parser\expr_parser.rs`: 4 errors
- `src\query\parser\core\error.rs`: 4 errors
- `src\query\parser\parser\mod.rs`: 3 errors
- `src\query\parser\parser\utils.rs`: 2 errors
- `src\query\parser\lexer\lexer.rs`: 1 errors
- `src\core\query_pipeline_manager.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 103

### Warning Type Breakdown

- **warning**: 103 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 7 warnings
- `src\query\validator\go_validator.rs`: 7 warnings
- `src\query\parser\lexer\lexer.rs`: 5 warnings
- `src\query\parser\parser\stmt_parser.rs`: 5 warnings
- `src\query\executor\aggregation.rs`: 4 warnings
- `src\query\executor\result_processing\transformations\rollup_apply.rs`: 4 warnings
- `src\query\validator\order_by_validator.rs`: 4 warnings
- `src\query\optimizer\projection_pushdown.rs`: 2 warnings
- `src\query\optimizer\scan_optimization.rs`: 2 warnings
- `src\storage\redb_storage.rs`: 2 warnings

## Detailed Error Categorization

### error[E0061]: this function takes 2 arguments but 3 arguments were supplied

**Total Occurrences**: 44  
**Unique Files**: 6

#### `src\query\parser\parser\stmt_parser.rs`: 24 occurrences

- Line 37: this method takes 0 arguments but 1 argument was supplied
- Line 39: this method takes 0 arguments but 1 argument was supplied
- Line 40: this method takes 0 arguments but 1 argument was supplied
- ... 21 more occurrences in this file

#### `src\query\parser\ast\tests.rs`: 8 occurrences

- Line 451: this function takes 3 arguments but 4 arguments were supplied
- Line 468: this function takes 3 arguments but 4 arguments were supplied
- Line 485: this function takes 2 arguments but 3 arguments were supplied
- ... 5 more occurrences in this file

#### `src\query\parser\parser\clause_parser.rs`: 5 occurrences

- Line 328: this function takes 2 arguments but 3 arguments were supplied
- Line 335: this function takes 2 arguments but 3 arguments were supplied
- Line 350: this function takes 3 arguments but 4 arguments were supplied
- ... 2 more occurrences in this file

#### `src\query\parser\parser\pattern_parser.rs`: 4 occurrences

- Line 16: this function takes 2 arguments but 3 arguments were supplied
- Line 101: this function takes 2 arguments but 3 arguments were supplied
- Line 137: this function takes 2 arguments but 3 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\parser\parser\expr_parser.rs`: 2 occurrences

- Line 259: this enum variant takes 1 argument but 2 arguments were supplied
- Line 340: this function takes 4 arguments but 3 arguments were supplied

#### `src\query\parser\parser\utils.rs`: 1 occurrences

- Line 31: this function takes 3 arguments but 4 arguments were supplied

### error[E0609]: no field `line` on type `query::parser::core::error::ParseError`: unknown field

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\query\parser\ast\tests.rs`: 4 occurrences

- Line 460: no field `line` on type `query::parser::core::error::ParseError`: unknown field
- Line 461: no field `column` on type `query::parser::core::error::ParseError`: unknown field
- Line 554: no field `line` on type `query::parser::core::error::ParseError`: unknown field
- ... 1 more occurrences in this file

### error[E0726]: implicit elided lifetime not allowed here: expected lifetime parameter

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\query\parser\parser\pattern_parser.rs`: 1 occurrences

- Line 8: implicit elided lifetime not allowed here: expected lifetime parameter

#### `src\query\parser\parser\utils.rs`: 1 occurrences

- Line 14: implicit elided lifetime not allowed here: expected lifetime parameter

#### `src\query\parser\parser\clause_parser.rs`: 1 occurrences

- Line 25: implicit elided lifetime not allowed here: expected lifetime parameter

### error[E0277]: the trait bound `query::parser::core::error::ParseError: Clone` is not satisfied: the trait `Clone` is not implemented for `query::parser::core::error::ParseError`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\parser\core\error.rs`: 1 occurrences

- Line 191: the trait bound `query::parser::core::error::ParseError: Clone` is not satisfied: the trait `Clone` is not implemented for `query::parser::core::error::ParseError`

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 474: the trait bound `str: StdError` is not satisfied: the trait `StdError` is not implemented for `str`

### error[E0308]: mismatched types: types differ in mutability

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\parser\parser\mod.rs`: 2 occurrences

- Line 250: mismatched types: types differ in mutability
- Line 262: mismatched types: types differ in mutability

### error[E0433]: failed to resolve: could not find `lexer` in `super`: could not find `lexer` in `super`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\parser\core\error.rs`: 2 occurrences

- Line 173: failed to resolve: could not find `lexer` in `super`: could not find `lexer` in `super`
- Line 174: failed to resolve: could not find `lexer` in `super`: could not find `lexer` in `super`

### error[E0599]: no variant or associated item named `Regex` found for enum `operators::BinaryOperator` in the current scope: variant or associated item not found in `BinaryOperator`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\parser\parser\expr_parser.rs`: 2 occurrences

- Line 106: no variant or associated item named `Regex` found for enum `operators::BinaryOperator` in the current scope: variant or associated item not found in `BinaryOperator`
- Line 251: no variant or associated item named `Grouped` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`

### error[E0369]: binary operation `==` cannot be applied to type `Vec<query::parser::core::error::ParseError>`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\parser\core\error.rs`: 1 occurrences

- Line 191: binary operation `==` cannot be applied to type `Vec<query::parser::core::error::ParseError>`

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 478: binary operation `==` cannot be applied to type `std::option::Option<Box<dyn StdError + std::marker::Send + Sync>>`: std::option::Option<Box<dyn StdError + std::marker::Send + Sync>>, std::option::Option<std::string::String>

### error[E0432]: unresolved import `crate::query::parser::ast::stmt::Statement`: no `Statement` in `query::parser::ast::stmt`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\parser\mod.rs`: 1 occurrences

- Line 301: unresolved import `crate::query::parser::ast::stmt::Statement`: no `Statement` in `query::parser::ast::stmt`

### error[E0106]: missing lifetime specifier: expected named lifetime parameter

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\core\query_pipeline_manager.rs`: 1 occurrences

- Line 23: missing lifetime specifier: expected named lifetime parameter

### error[E0621]: explicit lifetime required in the type of `ctx`: lifetime `'a` required

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 27: explicit lifetime required in the type of `ctx`: lifetime `'a` required

### error[E0596]: cannot borrow `self.chars` as mutable, as it is behind a `&` reference: `self` is a `&` reference, so the data it refers to cannot be borrowed as mutable

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\lexer\lexer.rs`: 1 occurrences

- Line 871: cannot borrow `self.chars` as mutable, as it is behind a `&` reference: `self` is a `&` reference, so the data it refers to cannot be borrowed as mutable

## Detailed Warning Categorization

### warning: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`

**Total Occurrences**: 103  
**Unique Files**: 60

#### `src\query\executor\result_processing\sort.rs`: 7 occurrences

- Line 688: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 712: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 736: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- ... 4 more occurrences in this file

#### `src\query\validator\go_validator.rs`: 7 occurrences

- Line 386: unreachable pattern: no value can reach this
- Line 392: unreachable pattern: no value can reach this
- Line 334: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`
- ... 4 more occurrences in this file

#### `src\query\parser\parser\stmt_parser.rs`: 5 occurrences

- Line 6: unused imports: `BinaryOp` and `UnaryOp`
- Line 8: unused import: `crate::query::parser::ast::expr::*`
- Line 9: unused import: `crate::query::parser::ast::pattern::*`
- ... 2 more occurrences in this file

#### `src\query\parser\lexer\lexer.rs`: 5 occurrences

- Line 6: unused import: `crate::query::parser::core::TokenKind`
- Line 10: unused import: `std::str::Chars`
- Line 720: unused variable: `end_col`: help: if this is intentional, prefix it with an underscore: `_end_col`
- ... 2 more occurrences in this file

#### `src\query\executor\result_processing\transformations\rollup_apply.rs`: 4 occurrences

- Line 507: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- Line 551: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- Line 660: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`
- ... 1 more occurrences in this file

#### `src\query\validator\order_by_validator.rs`: 4 occurrences

- Line 263: unreachable pattern: no value can reach this
- Line 276: unreachable pattern: no value can reach this
- Line 221: unreachable pattern: no value can reach this
- ... 1 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 4 occurrences

- Line 530: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 558: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 559: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- ... 1 more occurrences in this file

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`
- Line 104: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\visitor\deduce_type_visitor.rs`: 2 occurrences

- Line 324: unused variable: `property`: help: if this is intentional, prefix it with an underscore: `_property`
- Line 481: function cannot return without recursing: cannot return without recursing

#### `src\query\executor\data_access.rs`: 2 occurrences

- Line 655: unused variable: `last_vertex_box`: help: if this is intentional, prefix it with an underscore: `_last_vertex_box`
- Line 670: unused variable: `end_vertex`: help: if this is intentional, prefix it with an underscore: `_end_vertex`

#### `src\services\session.rs`: 2 occurrences

- Line 53: unused variable: `client_info`: help: if this is intentional, prefix it with an underscore: `_client_info`
- Line 53: unused variable: `connection_info`: help: if this is intentional, prefix it with an underscore: `_connection_info`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 286: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`
- Line 336: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`

#### `src\query\parser\parser\clause_parser.rs`: 2 occurrences

- Line 6: unused imports: `FromClause` and `OverClause`
- Line 22: unused import: `Span`

#### `src\query\optimizer\join_optimization.rs`: 2 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`
- Line 114: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 111: unreachable pattern: no value can reach this
- Line 112: unreachable pattern: no value can reach this

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`
- Line 124: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\storage\memory_storage.rs`: 2 occurrences

- Line 5: unused imports: `EdgeId` and `TagId`
- Line 175: variable does not need to be mutable

#### `src\query\parser\expressions\mod.rs`: 2 occurrences

- Line 5: unused import: `crate::query::parser::ast::*`
- Line 6: unused imports: `ParseError`, `TokenKind`, and `Token`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 438: unused import: `crate::api::session::session_manager::SessionInfo`
- Line 440: unused import: `OptGroup`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 91: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 107: variable does not need to be mutable

#### `src\query\context\ast\query_types\go.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 361: unused imports: `ListExpr`, `MapExpr`, `PathExpr`, `PropertyAccessExpr`, `RangeExpr`, and `SubscriptExpr`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 408: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 10: unused import: `std::collections::HashSet`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 1017: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\core\expression_utils.rs`: 1 occurrences

- Line 655: unreachable pattern: no value can reach this

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 438: unused import: `super::*`

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 510: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\api\service\index_service.rs`: 1 occurrences

- Line 419: unused import: `crate::core::Tag`

#### `src\query\validator\strategies\type_inference.rs`: 1 occurrences

- Line 841: unreachable pattern: no value can reach this

#### `src\query\visitor\evaluable_expr_visitor.rs`: 1 occurrences

- Line 6: unused import: `BinaryOperator`

#### `src\index\binary.rs`: 1 occurrences

- Line 329: unused import: `TimeValue`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 1080: variable does not need to be mutable

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 675: variable does not need to be mutable

#### `src\query\planner\statements\path_planner.rs`: 1 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`

#### `src\query\parser\parser\expr_parser.rs`: 1 occurrences

- Line 9: unused import: `crate::query::parser::core::position::Position`

#### `src\query\optimizer\prune_properties_visitor.rs`: 1 occurrences

- Line 128: unreachable pattern: no value can reach this

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\query\executor\result_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 12: unused import: `crate::core::Expression`

#### `src\query\context\ast\query_types\fetch_vertices.rs`: 1 occurrences

- Line 4: unused imports: `FromType` and `Starts`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\logic\loops.rs`: 1 occurrences

- Line 524: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 322: unused import: `crate::storage::StorageEngine`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\query\executor\result_processing\transformations\unwind.rs`: 1 occurrences

- Line 375: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 267: unused import: `UnaryOperator`

#### `src\common\thread.rs`: 1 occurrences

- Line 89: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 726: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\context\validate\context.rs`: 1 occurrences

- Line 617: unused import: `super::super::schema::SchemaValidationError`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

