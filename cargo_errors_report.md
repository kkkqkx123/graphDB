# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 201
- **Total Warnings**: 64
- **Total Issues**: 265
- **Unique Error Patterns**: 49
- **Unique Warning Patterns**: 27
- **Files with Issues**: 63

## Error Statistics

**Total Errors**: 201

### Error Type Breakdown

- **error[E0308]**: 159 errors
- **error[E0433]**: 10 errors
- **error[E0061]**: 8 errors
- **error[E0599]**: 7 errors
- **error[E0560]**: 5 errors
- **error[E0515]**: 5 errors
- **error[E0277]**: 1 errors
- **error[E0382]**: 1 errors
- **error[E0624]**: 1 errors
- **error[E0422]**: 1 errors
- **error[E0507]**: 1 errors
- **error[E0614]**: 1 errors
- **error[E0603]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\graph_query_executor.rs`: 20 errors
- `src\api\embedded\statement.rs`: 19 errors
- `src\query\validator\statements\delete_validator.rs`: 17 errors
- `src\query\validator\statements\update_validator.rs`: 14 errors
- `src\query\validator\statements\match_validator.rs`: 14 errors
- `src\query\validator\strategies\expression_strategy_test.rs`: 12 errors
- `src\query\validator\statements\set_validator.rs`: 9 errors
- `src\query\validator\statements\go_validator.rs`: 9 errors
- `src\query\validator\statements\fetch_edges_validator.rs`: 9 errors
- `src\query\validator\statements\unwind_validator.rs`: 7 errors

## Warning Statistics

**Total Warnings**: 64

### Warning Type Breakdown

- **warning**: 64 warnings

### Files with Warnings (Top 10)

- `src\query\planner\statements\clauses\with_clause_planner.rs`: 7 warnings
- `src\query\validator\strategies\aggregate_strategy.rs`: 4 warnings
- `src\query\validator\strategies\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\utility\update_config_validator.rs`: 4 warnings
- `src\query\planner\plan\core\nodes\project_node.rs`: 3 warnings
- `src\query\parser\parser\util_stmt_parser.rs`: 2 warnings
- `src\query\validator\clauses\return_validator.rs`: 2 warnings
- `src\query\validator\clauses\yield_validator.rs`: 2 warnings
- `src\query\parser\parser\parser.rs`: 2 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`

**Total Occurrences**: 159  
**Unique Files**: 29

#### `src\api\embedded\statement.rs`: 18 occurrences

- Line 456: mismatched types: expected `Box<Expression>`, found `Option<_>`
- Line 459: mismatched types: expected `Box<Expression>`, found `Option<_>`
- Line 464: mismatched types: expected `Box<Expression>`, found `Option<_>`
- ... 15 more occurrences in this file

#### `src\query\validator\statements\delete_validator.rs`: 16 occurrences

- Line 197: mismatched types: expected `&ContextualExpression`, found `Arc<ExpressionMeta>`
- Line 214: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- Line 223: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- ... 13 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 15 occurrences

- Line 553: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 616: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 617: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 12 more occurrences in this file

#### `src\query\validator\statements\update_validator.rs`: 14 occurrences

- Line 327: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 374: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 441: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 11 more occurrences in this file

#### `src\query\validator\statements\match_validator.rs`: 14 occurrences

- Line 257: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 320: mismatched types: expected `&_`, found `String`
- Line 347: mismatched types: expected `&str`, found `String`
- ... 11 more occurrences in this file

#### `src\query\validator\statements\set_validator.rs`: 9 occurrences

- Line 187: arguments to this method are incorrect
- Line 190: arguments to this method are incorrect
- Line 193: arguments to this method are incorrect
- ... 6 more occurrences in this file

#### `src\query\validator\statements\fetch_edges_validator.rs`: 9 occurrences

- Line 131: arguments to this method are incorrect
- Line 297: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 298: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 6 more occurrences in this file

#### `src\query\validator\statements\go_validator.rs`: 9 occurrences

- Line 149: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 210: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 427: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- ... 6 more occurrences in this file

#### `src\query\validator\statements\unwind_validator.rs`: 7 occurrences

- Line 67: arguments to this function are incorrect
- Line 144: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 296: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 4 more occurrences in this file

#### `src\query\validator\statements\insert_edges_validator.rs`: 6 occurrences

- Line 121: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 379: mismatched types: expected `&Option<Expression>`, found `&Option<ContextualExpression>`
- Line 475: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 5 occurrences

- Line 103: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- Line 139: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 262: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 2 more occurrences in this file

#### `src\query\optimizer\strategy\traversal_start.rs`: 4 occurrences

- Line 299: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 306: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 486: arguments to this function are incorrect
- Line 508: arguments to this function are incorrect
- Line 530: arguments to this function are incorrect
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\clause_strategy.rs`: 4 occurrences

- Line 295: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 335: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 375: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\expression_operations.rs`: 3 occurrences

- Line 197: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 251: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 252: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\validator\statements\remove_validator.rs`: 3 occurrences

- Line 258: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 261: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 265: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\validator\statements\lookup_validator.rs`: 3 occurrences

- Line 128: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 139: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `Option<ContextualExpression>`, found `Option<Expression>`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 2 occurrences

- Line 156: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 164: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 2 occurrences

- Line 392: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 392: mismatched types: expected `Vec<Vec<ContextualExpression>>`, found `Vec<Vec<Expression>>`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 2 occurrences

- Line 92: mismatched types: expected `Option<OrderByClauseContext>`, found `Vec<_>`
- Line 123: mismatched types: expected `HashMap<String, AliasType>`, found `HashMap<String, DataType>`

#### `src\query\query_pipeline_manager.rs`: 2 occurrences

- Line 192: mismatched types: expected `&Stmt`, found `&ParserResult`
- Line 209: mismatched types: expected `Stmt`, found `ParserResult`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 137: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 53: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\optimizer\cost\node_estimators\data_processing.rs`: 1 occurrences

- Line 73: mismatched types: expected `&str`, found `&ContextualExpression`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 1 occurrences

- Line 349: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\validator\clauses\limit_validator.rs`: 1 occurrences

- Line 469: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 98: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\plan\core\nodes\factory.rs`: 1 occurrences

- Line 61: arguments to this function are incorrect

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 98: mismatched types: expected `ContextualExpression`, found `Expression`

### error[E0433]: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

**Total Occurrences**: 10  
**Unique Files**: 5

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 4 occurrences

- Line 241: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 246: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 300: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 3 occurrences

- Line 191: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 196: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`
- Line 217: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 1 occurrences

- Line 165: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 101: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 1 occurrences

- Line 166: failed to resolve: use of undeclared type `ContextualExpression`: use of undeclared type `ContextualExpression`

### error[E0061]: this method takes 3 arguments but 2 arguments were supplied

**Total Occurrences**: 8  
**Unique Files**: 4

#### `src\query\validator\strategies\helpers\type_checker.rs`: 3 occurrences

- Line 615: this function takes 2 arguments but 1 argument was supplied
- Line 623: this function takes 2 arguments but 1 argument was supplied
- Line 631: this function takes 2 arguments but 1 argument was supplied

#### `src\query\validator\helpers\type_checker.rs`: 3 occurrences

- Line 615: this function takes 2 arguments but 1 argument was supplied
- Line 623: this function takes 2 arguments but 1 argument was supplied
- Line 631: this function takes 2 arguments but 1 argument was supplied

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 129: this method takes 3 arguments but 2 arguments were supplied

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 354: this method takes 2 arguments but 1 argument was supplied

### error[E0599]: no variant or associated item named `ShowSpaces` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`

**Total Occurrences**: 7  
**Unique Files**: 3

#### `src\query\validator\validator_enum.rs`: 3 occurrences

- Line 460: no variant or associated item named `ShowSpaces` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`
- Line 461: no variant or associated item named `ShowTags` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`
- Line 462: no variant or associated item named `ShowEdges` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 3 occurrences

- Line 102: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 111: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 161: no method named `validate_property_access` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 319: no method named `map_err` found for struct `ValidationResult` in the current scope: method not found in `ValidationResult`

### error[E0560]: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 5 occurrences

- Line 90: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field
- Line 91: struct `clause_structs::ReturnClauseContext` has no field named `return_items`: `clause_structs::ReturnClauseContext` does not have this field
- Line 93: struct `clause_structs::ReturnClauseContext` has no field named `skip`: `clause_structs::ReturnClauseContext` does not have this field
- ... 2 more occurrences in this file

### error[E0515]: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 5 occurrences

- Line 317: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- Line 339: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- Line 346: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- ... 2 more occurrences in this file

### error[E0603]: module `test_helpers` is private: private module

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 11: module `test_helpers` is private: private module

### error[E0277]: the trait bound `std::string::String: Borrow<&std::string::String>` is not satisfied: the trait `Borrow<&std::string::String>` is not implemented for `std::string::String`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\statement.rs`: 1 occurrences

- Line 436: the trait bound `std::string::String: Borrow<&std::string::String>` is not satisfied: the trait `Borrow<&std::string::String>` is not implemented for `std::string::String`

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 423: type `i64` cannot be dereferenced: can't be dereferenced

### error[E0382]: use of moved value: `expr_context`: value used here after move

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\rewrite\expression_utils.rs`: 1 occurrences

- Line 175: use of moved value: `expr_context`: value used here after move

### error[E0624]: method `validate_group_key_type_internal` is private: private method

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 219: method `validate_group_key_type_internal` is private: private method

### error[E0507]: cannot move out of `expr_context`, a captured variable in an `FnMut` closure: move occurs because `expr_context` has type `std::sync::Arc<core::types::expression::context::ExpressionContext>`, which does not implement the `Copy` trait

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\rewrite\expression_utils.rs`: 1 occurrences

- Line 169: cannot move out of `expr_context`, a captured variable in an `FnMut` closure: move occurs because `expr_context` has type `std::sync::Arc<core::types::expression::context::ExpressionContext>`, which does not implement the `Copy` trait

### error[E0422]: cannot find struct, variant or union type `PropertyAccessContext` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 147: cannot find struct, variant or union type `PropertyAccessContext` in this scope: not found in this scope

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::expression::ExpressionId`

**Total Occurrences**: 64  
**Unique Files**: 37

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 7 occurrences

- Line 13: unused import: `crate::core::types::expression::utils::extract_group_suite`
- Line 453: unused variable: `object`
- Line 481: unused variable: `collection`
- ... 4 more occurrences in this file

#### `src\query\validator\strategies\aggregate_strategy.rs`: 4 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionMeta`
- Line 7: unused import: `crate::core::types::expression::ExpressionContext`
- Line 8: unused import: `crate::core::types::expression::ExpressionId`
- ... 1 more occurrences in this file

#### `src\query\validator\utility\update_config_validator.rs`: 4 occurrences

- Line 9: unused import: `crate::core::types::expression::Expression`
- Line 11: unused import: `crate::core::types::expression::ExpressionContext`
- Line 12: unused import: `crate::core::Value`
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\type_checker.rs`: 4 occurrences

- Line 6: unused import: `crate::core::AggregateFunction`
- Line 7: unused import: `crate::core::BinaryOperator`
- Line 8: unused import: `crate::core::UnaryOperator`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\type_checker.rs`: 4 occurrences

- Line 6: unused import: `crate::core::AggregateFunction`
- Line 7: unused import: `crate::core::BinaryOperator`
- Line 8: unused import: `crate::core::UnaryOperator`
- ... 1 more occurrences in this file

#### `src\query\planner\plan\core\nodes\project_node.rs`: 3 occurrences

- Line 9: unused import: `ExpressionMeta`
- Line 10: unused import: `crate::core::Expression`
- Line 85: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\parser\parser\util_stmt_parser.rs`: 2 occurrences

- Line 5: unused import: `std::sync::Arc`
- Line 8: unused import: `crate::core::types::expression::Expression as CoreExpression`

#### `src\query\validator\clauses\return_validator.rs`: 2 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 379: unused import: `crate::core::types::expression::Expression`

#### `src\query\parser\parser\parser.rs`: 2 occurrences

- Line 7: unused imports: `ExpressionMeta` and `Expression`
- Line 120: unused variable: `cache`: help: if this is intentional, prefix it with an underscore: `_cache`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 2 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`
- Line 10: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 18: unused import: `crate::core::Expression`
- Line 331: unused import: `ExpressionId`

#### `src\query\validator\clauses\with_validator.rs`: 2 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 416: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\helpers\expression_checker.rs`: 2 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`
- Line 10: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\planner\statements\maintain_planner.rs`: 1 occurrences

- Line 13: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\helpers\variable_checker.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 15: unused import: `crate::core::error::ValidationError`

#### `src\query\validator\statements\remove_validator.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Expression`

#### `src\query\validator\strategies\alias_strategy.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::core::Expression`

#### `src\query\parser\parser\traversal_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 113: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\statements\create_validator.rs`: 1 occurrences

- Line 21: unused import: `crate::core::Expression`

#### `src\query\planner\plan\core\nodes\plan_node_traits.rs`: 1 occurrences

- Line 8: unused import: `crate::core::Expression`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 11: unused imports: `ContextualExpression` and `ExpressionContext`

#### `src\query\planner\rewrite\expression_utils.rs`: 1 occurrences

- Line 10: unused import: `crate::core::types::expression::ExpressionId`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 567: unused import: `ExpressionId`

#### `src\query\parser\parser\clause_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\validator\statements\merge_validator.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Expression`

#### `src\query\validator\statements\unwind_validator.rs`: 1 occurrences

- Line 18: unused import: `NullType`

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 1 occurrences

- Line 280: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 2: unused import: `crate::core::types::expression::Expression`

#### `src\query\parser\parser\dml_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\validator\clauses\order_by_validator.rs`: 1 occurrences

- Line 17: unused import: `crate::core::Expression`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 179: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\strategies\mod.rs`: 1 occurrences

- Line 18: unused import: `agg_functions::*`

#### `src\query\planner\statements\use_planner.rs`: 1 occurrences

- Line 19: unused import: `crate::core::types::expression::ExpressionId`

