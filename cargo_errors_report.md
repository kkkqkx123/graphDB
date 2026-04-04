# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 279
- **Total Warnings**: 47
- **Total Issues**: 326
- **Unique Error Patterns**: 108
- **Unique Warning Patterns**: 23
- **Files with Issues**: 36

## Error Statistics

**Total Errors**: 279

### Error Type Breakdown

- **error[E0599]**: 86 errors
- **error[E0659]**: 42 errors
- **error[E0425]**: 32 errors
- **error[E0277]**: 21 errors
- **error[E0308]**: 15 errors
- **error[E0432]**: 11 errors
- **error[E0433]**: 11 errors
- **error[E0407]**: 10 errors
- **error[E0046]**: 8 errors
- **error[E0609]**: 8 errors
- **error[E0369]**: 8 errors
- **error[E0004]**: 8 errors
- **error[E0061]**: 5 errors
- **error[E0107]**: 4 errors
- **error[E0560]**: 3 errors
- **error[E0220]**: 2 errors
- **error[E0437]**: 2 errors
- **error[E0106]**: 1 errors
- **error[E0026]**: 1 errors
- **error[E0063]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\validator_enum.rs`: 66 errors
- `src\query\parser\parsing\fulltext_parser.rs`: 54 errors
- `src\query\planning\plan\core\nodes\management\fulltext_nodes.rs`: 38 errors
- `src\query\planning\plan\core\nodes\base\plan_node_enum.rs`: 26 errors
- `src\query\executor\data_access\fulltext_search.rs`: 18 errors
- `src\query\parser\ast\stmt.rs`: 17 errors
- `src\query\executor\factory\executor_factory.rs`: 16 errors
- `src\query\executor\expression\functions\fulltext.rs`: 10 errors
- `src\query\executor\executor_enum.rs`: 4 errors
- `src\query\planning\template_extractor.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 47

### Warning Type Breakdown

- **warning**: 47 warnings

### Files with Warnings (Top 10)

- `crates\inversearch\src\config\mod.rs`: 18 warnings
- `crates\inversearch\src\storage\mod.rs`: 6 warnings
- `crates\inversearch\src\lib.rs`: 5 warnings
- `src\query\parser\ast\mod.rs`: 4 warnings
- `src\search\adapters\bm25_adapter.rs`: 3 warnings
- `src\sync\scheduler.rs`: 2 warnings
- `src\query\executor\data_access\fulltext_search.rs`: 2 warnings
- `src\sync\recovery.rs`: 1 warnings
- `build.rs`: 1 warnings
- `src\query\validator\fulltext_validator.rs`: 1 warnings

## Detailed Error Categorization

### error[E0599]: no variant or associated item named `CreateFulltextIndex` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`

**Total Occurrences**: 86  
**Unique Files**: 5

#### `src\query\validator\validator_enum.rs`: 56 occurrences

- Line 255: no variant or associated item named `CreateFulltextIndex` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- Line 256: no variant or associated item named `DropFulltextIndex` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- Line 257: no variant or associated item named `AlterFulltextIndex` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- ... 53 more occurrences in this file

#### `src\query\parser\parsing\fulltext_parser.rs`: 24 occurrences

- Line 49: no function or associated item named `error` found for struct `ParserResult` in the current scope: function or associated item not found in `ParserResult`
- Line 108: no function or associated item named `error` found for struct `ParserResult` in the current scope: function or associated item not found in `ParserResult`
- Line 112: no function or associated item named `default` found for struct `ast::fulltext::IndexOptions` in the current scope: function or associated item not found in `ast::fulltext::IndexOptions`
- ... 21 more occurrences in this file

#### `src\query\executor\factory\executor_factory.rs`: 2 occurrences

- Line 618: no method named `search_engine` found for reference `&ExecutionContext` in the current scope: method not found in `&ExecutionContext`
- Line 635: no method named `search_engine` found for reference `&ExecutionContext` in the current scope: method not found in `&ExecutionContext`

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 52: no variant or associated item named `new` found for enum `core::error::query::QueryError` in the current scope: variant or associated item not found in `core::error::query::QueryError`
- Line 327: no variant or associated item named `new` found for enum `core::error::query::QueryError` in the current scope: variant or associated item not found in `core::error::query::QueryError`

#### `src\query\executor\expression\functions\fulltext.rs`: 2 occurrences

- Line 146: no variant or associated item named `InvalidArgumentType` found for enum `ExpressionErrorType` in the current scope: variant or associated item not found in `ExpressionErrorType`
- Line 246: no variant or associated item named `InvalidArgumentType` found for enum `ExpressionErrorType` in the current scope: variant or associated item not found in `ExpressionErrorType`

### error[E0659]: `OrderDirection` is ambiguous: ambiguous name

**Total Occurrences**: 42  
**Unique Files**: 10

#### `src\query\parser\parsing\fulltext_parser.rs`: 12 occurrences

- Line 9: `OrderDirection` is ambiguous: ambiguous name
- Line 10: `WhereClause` is ambiguous: ambiguous name
- Line 10: `YieldClause` is ambiguous: ambiguous name
- ... 9 more occurrences in this file

#### `src\query\planning\plan\core\nodes\management\fulltext_nodes.rs`: 10 occurrences

- Line 8: `WhereClause` is ambiguous: ambiguous name
- Line 8: `YieldClause` is ambiguous: ambiguous name
- Line 190: `YieldClause` is ambiguous: ambiguous name
- ... 7 more occurrences in this file

#### `src\query\planning\template_extractor.rs`: 4 occurrences

- Line 10: `YieldClause` is ambiguous: ambiguous name
- Line 353: `OrderDirection` is ambiguous: ambiguous name
- Line 354: `OrderDirection` is ambiguous: ambiguous name
- ... 1 more occurrences in this file

#### `src\query\planning\statements\dql\yield_planner.rs`: 4 occurrences

- Line 6: `YieldItem` is ambiguous: ambiguous name
- Line 40: `YieldItem` is ambiguous: ambiguous name
- Line 125: `OrderDirection` is ambiguous: ambiguous name
- ... 1 more occurrences in this file

#### `src\query\parser\mod.rs`: 3 occurrences

- Line 17: `OrderDirection` is ambiguous: ambiguous name
- Line 18: `YieldClause` is ambiguous: ambiguous name
- Line 18: `YieldItem` is ambiguous: ambiguous name

#### `src\query\planning\statements\dql\with_planner.rs`: 2 occurrences

- Line 127: `OrderDirection` is ambiguous: ambiguous name
- Line 130: `OrderDirection` is ambiguous: ambiguous name

#### `src\query\planning\statements\dql\return_planner.rs`: 2 occurrences

- Line 115: `OrderDirection` is ambiguous: ambiguous name
- Line 118: `OrderDirection` is ambiguous: ambiguous name

#### `src\query\validator\statements\lookup_validator.rs`: 2 occurrences

- Line 12: `YieldItem` is ambiguous: ambiguous name
- Line 143: `YieldItem` is ambiguous: ambiguous name

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 11: `YieldClause` is ambiguous: ambiguous name
- Line 11: `YieldItem` is ambiguous: ambiguous name

#### `src\query\parser\ast\stmt.rs`: 1 occurrences

- Line 660: `OrderDirection` is ambiguous: ambiguous name

### error[E0425]: cannot find type `CreateFulltextIndexNode` in this scope

**Total Occurrences**: 32  
**Unique Files**: 2

#### `src\query\planning\plan\core\nodes\base\plan_node_enum.rs`: 24 occurrences

- Line 173: cannot find type `CreateFulltextIndexNode` in this scope
- Line 174: cannot find type `DropFulltextIndexNode` in this scope
- Line 175: cannot find type `AlterFulltextIndexNode` in this scope: not found in this scope
- ... 21 more occurrences in this file

#### `src\query\executor\factory\executor_factory.rs`: 8 occurrences

- Line 521: cannot find type `CreateFulltextIndexNode` in module `crate::query::planning::plan::core::nodes`
- Line 540: cannot find type `DropFulltextIndexNode` in module `crate::query::planning::plan::core::nodes`
- Line 555: cannot find type `AlterFulltextIndexNode` in module `crate::query::planning::plan::core::nodes`: not found in `crate::query::planning::plan::core::nodes`
- ... 5 more occurrences in this file

### error[E0277]: a value of type `std::vec::Vec<(usize, graph_schema::OrderDirection)>` cannot be built from an iterator over elements of type `(usize, ast::fulltext::OrderDirection)`: value of type `std::vec::Vec<(usize, graph_schema::OrderDirection)>` cannot be built from `std::iter::Iterator<Item=(usize, ast::fulltext::OrderDirection)>`

**Total Occurrences**: 21  
**Unique Files**: 4

#### `src\query\planning\plan\core\nodes\management\fulltext_nodes.rs`: 12 occurrences

- Line 186: the trait bound `stmt::YieldClause: serde::Serialize` is not satisfied: unsatisfied trait bound
- Line 186: the trait bound `stmt::WhereClause: serde::Serialize` is not satisfied: unsatisfied trait bound
- Line 190: the trait bound `stmt::YieldClause: serde::Deserialize<'de>` is not satisfied: unsatisfied trait bound
- ... 9 more occurrences in this file

#### `src\query\parser\parsing\fulltext_parser.rs`: 7 occurrences

- Line 314: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ParserResult`
- Line 314: the `?` operator can only be used in a method that returns `Result` or `Option` (or another type that implements `FromResidual`): cannot use the `?` operator in a method that returns `ParserResult`
- Line 321: the `?` operator can only be used in a method that returns `Result` or `Option` (or another type that implements `FromResidual`): cannot use the `?` operator in a method that returns `ParserResult`
- ... 4 more occurrences in this file

#### `src\query\planning\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 311: a value of type `std::vec::Vec<(usize, graph_schema::OrderDirection)>` cannot be built from an iterator over elements of type `(usize, ast::fulltext::OrderDirection)`: value of type `std::vec::Vec<(usize, graph_schema::OrderDirection)>` cannot be built from `std::iter::Iterator<Item=(usize, ast::fulltext::OrderDirection)>`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 193: `FulltextValidator` doesn't implement `std::fmt::Debug`: the trait `std::fmt::Debug` is not implemented for `FulltextValidator`

### error[E0308]: mismatched types: expected `Value`, found enum constructor

**Total Occurrences**: 15  
**Unique Files**: 8

#### `src\query\parser\parsing\fulltext_parser.rs`: 4 occurrences

- Line 322: mismatched types: expected `ast::fulltext::YieldClause`, found `stmt::YieldClause`
- Line 329: mismatched types: expected `ast::fulltext::WhereClause`, found `stmt::WhereClause`
- Line 525: `?` operator has incompatible types: expected `ast::fulltext::YieldClause`, found `stmt::YieldClause`
- ... 1 more occurrences in this file

#### `src\query\executor\expression\functions\fulltext.rs`: 3 occurrences

- Line 204: mismatched types: expected `Value`, found enum constructor
- Line 226: mismatched types: expected `List`, found `Vec<Value>`
- Line 281: mismatched types: expected `Value`, found enum constructor

#### `src\query\executor\data_access\fulltext_search.rs`: 3 occurrences

- Line 50: mismatched types: expected `usize`, found `FulltextQuery`
- Line 325: mismatched types: expected `usize`, found `FulltextQuery`
- Line 233: mismatched types: expected `List`, found `Vec<Value>`

#### `src\query\parser\parsing\clause_parser.rs`: 1 occurrences

- Line 296: mismatched types: expected `ast::fulltext::OrderDirection`, found `graph_schema::OrderDirection`

#### `src\query\parser\parsing\util_stmt_parser.rs`: 1 occurrences

- Line 466: mismatched types: expected `ast::fulltext::OrderDirection`, found `graph_schema::OrderDirection`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 923: `match` arms have incompatible types: expected `&ExpressionProps`, found `ExpressionProps`

#### `src\query\planning\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 80: mismatched types: expected `graph_schema::OrderDirection`, found `ast::fulltext::OrderDirection`

#### `src\query\planning\statements\match_statement_planner.rs`: 1 occurrences

- Line 900: mismatched types: expected `graph_schema::OrderDirection`, found `ast::fulltext::OrderDirection`

### error[E0432]: unresolved import `crate::core::error::QueryErrorType`: no `QueryErrorType` in `core::error`, help: a similar name exists in the module: `QueryError`

**Total Occurrences**: 11  
**Unique Files**: 5

#### `src\query\executor\factory\executor_factory.rs`: 6 occurrences

- Line 525: unresolved import `crate::query::executor::admin::CreateFulltextIndexExecutor`: no `CreateFulltextIndexExecutor` in `query::executor::admin`
- Line 544: unresolved import `crate::query::executor::admin::DropFulltextIndexExecutor`: no `DropFulltextIndexExecutor` in `query::executor::admin`
- Line 559: unresolved import `crate::query::executor::admin::AlterFulltextIndexExecutor`: no `AlterFulltextIndexExecutor` in `query::executor::admin`
- ... 3 more occurrences in this file

#### `src\query\executor\executor_enum.rs`: 2 occurrences

- Line 12: unresolved imports `super::admin::AlterFulltextIndexExecutor`, `super::admin::CreateFulltextIndexExecutor`, `super::admin::DescribeFulltextIndexExecutor`, `super::admin::DropFulltextIndexExecutor`, `super::admin::ShowFulltextIndexExecutor`: no `AlterFulltextIndexExecutor` in `query::executor::admin`, no `CreateFulltextIndexExecutor` in `query::executor::admin`, no `DescribeFulltextIndexExecutor` in `query::executor::admin`, no `DropFulltextIndexExecutor` in `query::executor::admin`, no `ShowFulltextIndexExecutor` in `query::executor::admin`
- Line 28: unresolved import `super::data_access::MatchFulltextExecutor`: no `MatchFulltextExecutor` in `query::executor::data_access`

#### `src\query\executor\data_access\fulltext_search.rs`: 1 occurrences

- Line 7: unresolved import `crate::core::error::QueryErrorType`: no `QueryErrorType` in `core::error`, help: a similar name exists in the module: `QueryError`

#### `src\query\planning\planner.rs`: 1 occurrences

- Line 41: unresolved import `crate::query::planning::planner::fulltext`: could not find `fulltext` in `planner`

#### `src\query\executor\expression\functions\fulltext.rs`: 1 occurrences

- Line 11: unresolved import `crate::query::executor::expression::functions::signature::FunctionSignature`: no `FunctionSignature` in `query::executor::expression::functions::signature`

### error[E0433]: failed to resolve: could not find `ParseErrorType` in `parser`: could not find `ParseErrorType` in `parser`

**Total Occurrences**: 11  
**Unique Files**: 2

#### `src\query\validator\validator_enum.rs`: 8 occurrences

- Line 761: failed to resolve: use of undeclared type `CreateFulltextIndexValidator`: use of undeclared type `CreateFulltextIndexValidator`
- Line 762: failed to resolve: use of undeclared type `DropFulltextIndexValidator`: use of undeclared type `DropFulltextIndexValidator`
- Line 763: failed to resolve: use of undeclared type `AlterFulltextIndexValidator`: use of undeclared type `AlterFulltextIndexValidator`
- ... 5 more occurrences in this file

#### `src\query\parser\parsing\fulltext_parser.rs`: 3 occurrences

- Line 471: failed to resolve: could not find `ParseErrorType` in `parser`: could not find `ParseErrorType` in `parser`
- Line 431: failed to resolve: could not find `ContextualExpression` in `ast`: could not find `ContextualExpression` in `ast`
- Line 432: failed to resolve: could not find `Expression` in `ast`: could not find `Expression` in `ast`

### error[E0407]: method `category` is not a member of trait `PlanNode`: not a member of trait `PlanNode`

**Total Occurrences**: 10  
**Unique Files**: 2

#### `src\query\planning\plan\core\nodes\management\fulltext_nodes.rs`: 8 occurrences

- Line 54: method `category` is not a member of trait `PlanNode`: not a member of trait `PlanNode`
- Line 86: method `category` is not a member of trait `PlanNode`: not a member of trait `PlanNode`
- Line 118: method `category` is not a member of trait `PlanNode`: not a member of trait `PlanNode`
- ... 5 more occurrences in this file

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 277: method `reset` is not a member of trait `Executor`: not a member of trait `Executor`
- Line 351: method `reset` is not a member of trait `Executor`: not a member of trait `Executor`

### error[E0046]: not all trait items implemented, missing: `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum`: missing `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum` in implementation

**Total Occurrences**: 8  
**Unique Files**: 1

#### `src\query\planning\plan\core\nodes\management\fulltext_nodes.rs`: 8 occurrences

- Line 45: not all trait items implemented, missing: `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum`: missing `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum` in implementation
- Line 77: not all trait items implemented, missing: `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum`: missing `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum` in implementation
- Line 109: not all trait items implemented, missing: `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum`: missing `output_var`, `col_names`, `set_output_var`, `set_col_names`, `into_enum` in implementation
- ... 5 more occurrences in this file

### error[E0609]: no field `span` on type `&ast::fulltext::CreateFulltextIndex`: unknown field

**Total Occurrences**: 8  
**Unique Files**: 1

#### `src\query\parser\ast\stmt.rs`: 8 occurrences

- Line 156: no field `span` on type `&ast::fulltext::CreateFulltextIndex`: unknown field
- Line 157: no field `span` on type `&ast::fulltext::DropFulltextIndex`: unknown field
- Line 158: no field `span` on type `&ast::fulltext::AlterFulltextIndex`: unknown field
- ... 5 more occurrences in this file

### error[E0004]: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered

**Total Occurrences**: 8  
**Unique Files**: 4

#### `src\query\planning\plan\core\nodes\base\plan_node_traits_impl.rs`: 4 occurrences

- Line 11: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered
- Line 102: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered
- Line 193: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered
- ... 1 more occurrences in this file

#### `src\query\planning\plan\core\nodes\base\plan_node_enum.rs`: 2 occurrences

- Line 1079: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered
- Line 1320: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered

#### `src\query\planning\plan\core\nodes\base\macros.rs`: 1 occurrences

- Line 161: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered

#### `src\query\planning\plan\core\nodes\base\plan_node_children.rs`: 1 occurrences

- Line 10: non-exhaustive patterns: `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered: patterns `&plan_node_enum::PlanNodeEnum::CreateFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::DropFulltextIndex(_)`, `&plan_node_enum::PlanNodeEnum::AlterFulltextIndex(_)` and 5 more not covered

### error[E0369]: binary operation `==` cannot be applied to type `&ast::fulltext::CreateFulltextIndex`

**Total Occurrences**: 8  
**Unique Files**: 1

#### `src\query\parser\ast\stmt.rs`: 8 occurrences

- Line 96: binary operation `==` cannot be applied to type `&ast::fulltext::CreateFulltextIndex`
- Line 97: binary operation `==` cannot be applied to type `&ast::fulltext::DropFulltextIndex`
- Line 98: binary operation `==` cannot be applied to type `&ast::fulltext::AlterFulltextIndex`
- ... 5 more occurrences in this file

### error[E0061]: this method takes 1 argument but 2 arguments were supplied

**Total Occurrences**: 5  
**Unique Files**: 2

#### `src\query\executor\expression\functions\fulltext.rs`: 4 occurrences

- Line 354: this method takes 1 argument but 2 arguments were supplied
- Line 363: this method takes 1 argument but 2 arguments were supplied
- Line 372: this method takes 1 argument but 2 arguments were supplied
- ... 1 more occurrences in this file

#### `src\query\parser\parsing\fulltext_parser.rs`: 1 occurrences

- Line 470: this function takes 3 arguments but 2 arguments were supplied

### error[E0107]: struct takes 0 generic arguments but 1 generic argument was supplied: expected 0 generic arguments

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\executor\executor_enum.rs`: 2 occurrences

- Line 156: struct takes 0 generic arguments but 1 generic argument was supplied: expected 0 generic arguments
- Line 157: struct takes 0 generic arguments but 1 generic argument was supplied: expected 0 generic arguments

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 270: missing generics for trait `executor_base::Executor`: expected 1 generic argument
- Line 315: missing generics for trait `executor_base::Executor`: expected 1 generic argument

### error[E0560]: struct `search::result::FulltextSearchResult` has no field named `shards`: `search::result::FulltextSearchResult` does not have this field

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 341: struct `search::result::FulltextSearchResult` has no field named `shards`: `search::result::FulltextSearchResult` does not have this field
- Line 183: struct `search::result::FulltextSearchResult` has no field named `shards`: `search::result::FulltextSearchResult` does not have this field

#### `src\query\parser\parsing\fulltext_parser.rs`: 1 occurrences

- Line 418: struct `stmt::YieldItem` has no field named `expr`: `stmt::YieldItem` does not have this field

### error[E0220]: associated type `Output` not found for `Self`: associated type `Output` not found

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 273: associated type `Output` not found for `Self`: associated type `Output` not found
- Line 318: associated type `Output` not found for `Self`: associated type `Output` not found

### error[E0437]: type `Output` is not a member of trait `Executor`: not a member of trait `Executor`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 271: type `Output` is not a member of trait `Executor`: not a member of trait `Executor`
- Line 316: type `Output` is not a member of trait `Executor`: not a member of trait `Executor`

### error[E0106]: missing lifetime specifier: expected named lifetime parameter

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\parsing\fulltext_parser.rs`: 1 occurrences

- Line 20: missing lifetime specifier: expected named lifetime parameter

### error[E0026]: struct `ParserResult` does not have a field named `errors`: struct `ParserResult` does not have this field

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\parsing\stmt_parser.rs`: 1 occurrences

- Line 532: struct `ParserResult` does not have a field named `errors`: struct `ParserResult` does not have this field

### error[E0063]: missing field `span` in initializer of `stmt::WhereClause`: missing `span`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\parser\parsing\fulltext_parser.rs`: 1 occurrences

- Line 431: missing field `span` in initializer of `stmt::WhereClause`: missing `span`

## Detailed Warning Categorization

### warning: unused import: `std::path::PathBuf`

**Total Occurrences**: 47  
**Unique Files**: 14

#### `crates\inversearch\src\config\mod.rs`: 18 occurrences

- Line 108: unexpected `cfg` condition value: `store-redis`
- Line 115: unexpected `cfg` condition value: `store-redis`
- Line 125: unexpected `cfg` condition value: `store-file`
- ... 15 more occurrences in this file

#### `crates\inversearch\src\storage\mod.rs`: 6 occurrences

- Line 46: unexpected `cfg` condition value: `store-memory`
- Line 49: unexpected `cfg` condition value: `store-file`
- Line 52: unexpected `cfg` condition value: `store-redis`
- ... 3 more occurrences in this file

#### `crates\inversearch\src\lib.rs`: 5 occurrences

- Line 76: unexpected `cfg` condition value: `store-memory`
- Line 79: unexpected `cfg` condition value: `store-file`
- Line 82: unexpected `cfg` condition value: `store-wal`
- ... 2 more occurrences in this file

#### `src\query\parser\ast\mod.rs`: 4 occurrences

- Line 11: ambiguous glob re-exports: the name `YieldClause` in the type namespace is first re-exported here
- Line 11: ambiguous glob re-exports: the name `YieldItem` in the type namespace is first re-exported here
- Line 11: ambiguous glob re-exports: the name `WhereClause` in the type namespace is first re-exported here
- ... 1 more occurrences in this file

#### `src\search\adapters\bm25_adapter.rs`: 3 occurrences

- Line 3: unused import: `add_document`
- Line 4: unused import: `delete_document`
- Line 149: unused variable: `batch_count`: help: if this is intentional, prefix it with an underscore: `_batch_count`

#### `src\query\executor\data_access\fulltext_search.rs`: 2 occurrences

- Line 11: unused imports: `YieldClause` and `YieldItem`
- Line 14: unused import: `crate::core::types::FulltextSearchResult as CoreFulltextSearchResult`

#### `src\sync\scheduler.rs`: 2 occurrences

- Line 4: unused import: `BatchConfig`
- Line 6: unused import: `crate::sync::task::SyncTask`

#### `build.rs`: 1 occurrences

- Line 6: unused import: `std::path::PathBuf`

#### `src\query\executor\expression\functions\mod.rs`: 1 occurrences

- Line 244: unused variable: `f`: help: if this is intentional, prefix it with an underscore: `_f`

#### `src\query\parser\parsing\fulltext_parser.rs`: 1 occurrences

- Line 14: unused import: `Parser`

#### `src\query\parser\mod.rs`: 1 occurrences

- Line 17: unused imports: `OrderDirection`, `YieldClause`, and `YieldItem`

#### `src\sync\queue.rs`: 1 occurrences

- Line 2: unused import: `std::collections::VecDeque`

#### `src\sync\recovery.rs`: 1 occurrences

- Line 2: unused import: `SyncState`

#### `src\query\validator\fulltext_validator.rs`: 1 occurrences

- Line 11: unused imports: `FulltextMatchCondition` and `ShowFulltextIndex`

