# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 144
- **Total Warnings**: 0
- **Total Issues**: 144
- **Unique Error Patterns**: 39
- **Unique Warning Patterns**: 0
- **Files with Issues**: 28

## Error Statistics

**Total Errors**: 144

### Error Type Breakdown

- **error[E0308]**: 113 errors
- **error[E0599]**: 8 errors
- **error[E0061]**: 6 errors
- **error[E0560]**: 5 errors
- **error[E0515]**: 5 errors
- **error[E0614]**: 3 errors
- **error[E0277]**: 1 errors
- **error[E0624]**: 1 errors
- **error[E0422]**: 1 errors
- **error[E0603]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\graph_query_executor.rs`: 20 errors
- `src\api\embedded\statement.rs`: 19 errors
- `src\query\validator\strategies\expression_strategy_test.rs`: 12 errors
- `src\query\validator\statements\delete_validator.rs`: 8 errors
- `src\query\validator\statements\set_validator.rs`: 8 errors
- `src\query\validator\statements\unwind_validator.rs`: 6 errors
- `src\query\validator\statements\fetch_edges_validator.rs`: 6 errors
- `src\query\validator\statements\insert_edges_validator.rs`: 5 errors
- `src\query\validator\statements\fetch_vertices_validator.rs`: 5 errors
- `src\query\validator\statements\go_validator.rs`: 5 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `&Stmt`, found `&ParserResult`

**Total Occurrences**: 113  
**Unique Files**: 24

#### `src\api\embedded\statement.rs`: 18 occurrences

- Line 456: mismatched types: expected `Box<Expression>`, found `Option<_>`
- Line 459: mismatched types: expected `Box<Expression>`, found `Option<_>`
- Line 464: mismatched types: expected `Box<Expression>`, found `Option<_>`
- ... 15 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 15 occurrences

- Line 553: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 616: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 617: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 12 more occurrences in this file

#### `src\query\validator\statements\delete_validator.rs`: 8 occurrences

- Line 575: mismatched types: expected `Option<ContextualExpression>`, found `Option<Expression>`
- Line 620: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 638: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 5 more occurrences in this file

#### `src\query\validator\statements\set_validator.rs`: 8 occurrences

- Line 190: arguments to this method are incorrect
- Line 193: arguments to this method are incorrect
- Line 196: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 5 more occurrences in this file

#### `src\query\validator\statements\unwind_validator.rs`: 6 occurrences

- Line 240: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 252: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 408: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 5 occurrences

- Line 273: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 372: mismatched types: expected `Vec<ContextualExpression>`, found `Vec<Expression>`
- Line 394: mismatched types: expected `&[ContextualExpression]`, found `&Vec<Expression>`
- ... 2 more occurrences in this file

#### `src\query\validator\statements\go_validator.rs`: 5 occurrences

- Line 294: mismatched types: expected `&String`, found `String`
- Line 460: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 515: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 2 more occurrences in this file

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 486: arguments to this function are incorrect
- Line 508: arguments to this function are incorrect
- Line 530: arguments to this function are incorrect
- ... 1 more occurrences in this file

#### `src\query\validator\statements\insert_edges_validator.rs`: 4 occurrences

- Line 458: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 458: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 458: mismatched types: expected `Option<ContextualExpression>`, found `Option<Expression>`
- ... 1 more occurrences in this file

#### `src\query\validator\statements\fetch_edges_validator.rs`: 4 occurrences

- Line 497: arguments to this method are incorrect
- Line 507: arguments to this method are incorrect
- Line 514: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\statements\update_validator.rs`: 4 occurrences

- Line 783: mismatched types: expected `Option<ContextualExpression>`, found `Option<Expression>`
- Line 871: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 875: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\statements\match_validator.rs`: 4 occurrences

- Line 883: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 888: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 898: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

#### `src\query\optimizer\strategy\traversal_start.rs`: 4 occurrences

- Line 299: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 306: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\clause_strategy.rs`: 4 occurrences

- Line 295: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 335: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 375: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\statements\lookup_validator.rs`: 3 occurrences

- Line 128: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 139: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `Option<ContextualExpression>`, found `Option<Expression>`

#### `src\query\validator\strategies\expression_operations.rs`: 3 occurrences

- Line 197: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 251: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 252: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\validator\statements\remove_validator.rs`: 3 occurrences

- Line 258: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 261: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 265: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\query_pipeline_manager.rs`: 2 occurrences

- Line 192: mismatched types: expected `&Stmt`, found `&ParserResult`
- Line 209: mismatched types: expected `Stmt`, found `ParserResult`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 2 occurrences

- Line 64: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 348: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 2 occurrences

- Line 92: mismatched types: expected `Option<OrderByClauseContext>`, found `Vec<_>`
- Line 123: mismatched types: expected `HashMap<String, AliasType>`, found `HashMap<String, DataType>`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 2 occurrences

- Line 392: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 392: mismatched types: expected `Vec<Vec<ContextualExpression>>`, found `Vec<Vec<Expression>>`

#### `src\query\validator\clauses\limit_validator.rs`: 1 occurrences

- Line 469: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 53: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\optimizer\cost\node_estimators\data_processing.rs`: 1 occurrences

- Line 73: mismatched types: expected `&str`, found `&ContextualExpression`

### error[E0599]: no variant or associated item named `ShowSpaces` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`

**Total Occurrences**: 8  
**Unique Files**: 4

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

#### `src\core\types\expression\contextual.rs`: 1 occurrences

- Line 124: no method named `is_property` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope: method not found in `Arc<ExpressionMeta>`

### error[E0061]: this function takes 2 arguments but 1 argument was supplied

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\validator\strategies\helpers\type_checker.rs`: 3 occurrences

- Line 615: this function takes 2 arguments but 1 argument was supplied
- Line 623: this function takes 2 arguments but 1 argument was supplied
- Line 631: this function takes 2 arguments but 1 argument was supplied

#### `src\query\validator\helpers\type_checker.rs`: 3 occurrences

- Line 615: this function takes 2 arguments but 1 argument was supplied
- Line 623: this function takes 2 arguments but 1 argument was supplied
- Line 631: this function takes 2 arguments but 1 argument was supplied

### error[E0515]: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 5 occurrences

- Line 317: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- Line 339: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- Line 346: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- ... 2 more occurrences in this file

### error[E0560]: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 5 occurrences

- Line 90: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field
- Line 91: struct `clause_structs::ReturnClauseContext` has no field named `return_items`: `clause_structs::ReturnClauseContext` does not have this field
- Line 93: struct `clause_structs::ReturnClauseContext` has no field named `skip`: `clause_structs::ReturnClauseContext` does not have this field
- ... 2 more occurrences in this file

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\validator\statements\fetch_edges_validator.rs`: 2 occurrences

- Line 218: type `i64` cannot be dereferenced: can't be dereferenced
- Line 280: type `i64` cannot be dereferenced: can't be dereferenced

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 288: type `i64` cannot be dereferenced: can't be dereferenced

### error[E0603]: module `test_helpers` is private: private module

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 11: module `test_helpers` is private: private module

### error[E0422]: cannot find struct, variant or union type `PropertyAccessContext` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 147: cannot find struct, variant or union type `PropertyAccessContext` in this scope: not found in this scope

### error[E0624]: method `validate_group_key_type_internal` is private: private method

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 219: method `validate_group_key_type_internal` is private: private method

### error[E0277]: the trait bound `std::string::String: Borrow<&std::string::String>` is not satisfied: the trait `Borrow<&std::string::String>` is not implemented for `std::string::String`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\statement.rs`: 1 occurrences

- Line 436: the trait bound `std::string::String: Borrow<&std::string::String>` is not satisfied: the trait `Borrow<&std::string::String>` is not implemented for `std::string::String`

