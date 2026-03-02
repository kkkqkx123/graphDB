# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 67
- **Total Warnings**: 65
- **Total Issues**: 132
- **Unique Error Patterns**: 25
- **Unique Warning Patterns**: 28
- **Files with Issues**: 47

## Error Statistics

**Total Errors**: 67

### Error Type Breakdown

- **error[E0308]**: 48 errors
- **error[E0515]**: 5 errors
- **error[E0599]**: 5 errors
- **error[E0560]**: 5 errors
- **error[E0603]**: 1 errors
- **error[E0422]**: 1 errors
- **error[E0277]**: 1 errors
- **error[E0624]**: 1 errors

### Files with Errors (Top 10)

- `src\query\executor\graph_query_executor.rs`: 20 errors
- `src\api\embedded\statement.rs`: 19 errors
- `src\query\validator\strategies\expression_strategy_test.rs`: 12 errors
- `src\query\executor\factory.rs`: 4 errors
- `src\query\optimizer\strategy\traversal_start.rs`: 4 errors
- `src\query\query_pipeline_manager.rs`: 3 errors
- `src\query\planner\statements\clauses\with_clause_planner.rs`: 2 errors
- `src\core\types\expression\contextual.rs`: 1 errors
- `src\query\optimizer\cost\node_estimators\data_processing.rs`: 1 errors
- `src\query\validator\strategies\expression_strategy.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 65

### Warning Type Breakdown

- **warning**: 65 warnings

### Files with Warnings (Top 10)

- `src\query\validator\utility\update_config_validator.rs`: 4 warnings
- `src\query\validator\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\strategies\aggregate_strategy.rs`: 4 warnings
- `src\query\validator\strategies\helpers\type_checker.rs`: 4 warnings
- `src\query\planner\statements\clauses\with_clause_planner.rs`: 4 warnings
- `src\query\validator\helpers\expression_checker.rs`: 2 warnings
- `src\query\validator\strategies\helpers\expression_checker.rs`: 2 warnings
- `src\query\validator\clauses\return_validator.rs`: 2 warnings
- `src\query\validator\statements\unwind_validator.rs`: 2 warnings
- `src\query\validator\statements\remove_validator.rs`: 2 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `Option<OrderByClauseContext>`, found `Vec<_>`

**Total Occurrences**: 48  
**Unique Files**: 8

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

#### `src\query\executor\factory.rs`: 4 occurrences

- Line 486: arguments to this function are incorrect
- Line 508: arguments to this function are incorrect
- Line 530: arguments to this function are incorrect
- ... 1 more occurrences in this file

#### `src\query\optimizer\strategy\traversal_start.rs`: 4 occurrences

- Line 299: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 306: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\expression_strategy_test.rs`: 2 occurrences

- Line 92: mismatched types: expected `Option<OrderByClauseContext>`, found `Vec<_>`
- Line 123: mismatched types: expected `HashMap<String, AliasType>`, found `HashMap<String, DataType>`

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 2 occurrences

- Line 64: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 348: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\query_pipeline_manager.rs`: 2 occurrences

- Line 192: mismatched types: expected `&Stmt`, found `&ParserResult`
- Line 209: mismatched types: expected `Stmt`, found `ParserResult`

#### `src\query\optimizer\cost\node_estimators\data_processing.rs`: 1 occurrences

- Line 73: mismatched types: expected `&str`, found `&ContextualExpression`

### error[E0515]: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\executor\graph_query_executor.rs`: 5 occurrences

- Line 317: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- Line 339: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- Line 346: cannot return value referencing function parameter `meta`: returns a value referencing data owned by the current function
- ... 2 more occurrences in this file

### error[E0599]: no method named `is_property` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope: method not found in `Arc<ExpressionMeta>`

**Total Occurrences**: 5  
**Unique Files**: 3

#### `src\query\validator\strategies\expression_strategy_test.rs`: 3 occurrences

- Line 102: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 111: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 161: no method named `validate_property_access` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope

#### `src\core\types\expression\contextual.rs`: 1 occurrences

- Line 124: no method named `is_property` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope: method not found in `Arc<ExpressionMeta>`

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

### error[E0277]: the trait bound `std::string::String: Borrow<&std::string::String>` is not satisfied: the trait `Borrow<&std::string::String>` is not implemented for `std::string::String`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\statement.rs`: 1 occurrences

- Line 436: the trait bound `std::string::String: Borrow<&std::string::String>` is not satisfied: the trait `Borrow<&std::string::String>` is not implemented for `std::string::String`

### error[E0624]: method `validate_group_key_type_internal` is private: private method

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 219: method `validate_group_key_type_internal` is private: private method

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

## Detailed Warning Categorization

### warning: unused import: `crate::core::types::expression::ExpressionId`

**Total Occurrences**: 65  
**Unique Files**: 39

#### `src\query\validator\strategies\helpers\type_checker.rs`: 4 occurrences

- Line 6: unused import: `crate::core::AggregateFunction`
- Line 7: unused import: `crate::core::BinaryOperator`
- Line 8: unused import: `crate::core::UnaryOperator`
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

#### `src\query\validator\strategies\aggregate_strategy.rs`: 4 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionMeta`
- Line 7: unused import: `crate::core::types::expression::ExpressionContext`
- Line 8: unused import: `crate::core::types::expression::ExpressionId`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 4 occurrences

- Line 480: unused variable: `collection`
- Line 546: unused variable: `left`
- Line 546: unused variable: `right`
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\expression_checker.rs`: 2 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`
- Line 10: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\validator\clauses\with_validator.rs`: 2 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 416: unused import: `crate::core::types::expression::Expression`

#### `src\query\parser\parser\util_stmt_parser.rs`: 2 occurrences

- Line 5: unused import: `std::sync::Arc`
- Line 8: unused import: `crate::core::types::expression::Expression as CoreExpression`

#### `src\query\validator\statements\unwind_validator.rs`: 2 occurrences

- Line 17: unused import: `crate::core::Expression`
- Line 18: unused imports: `NullType` and `Value`

#### `src\query\validator\clauses\return_validator.rs`: 2 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 379: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\statements\remove_validator.rs`: 2 occurrences

- Line 9: unused import: `crate::core::Expression`
- Line 245: unused import: `crate::core::Value`

#### `src\query\validator\statements\merge_validator.rs`: 2 occurrences

- Line 203: unused variable: `props`: help: if this is intentional, prefix it with an underscore: `_props`
- Line 225: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 2 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`
- Line 10: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\validator\statements\match_validator.rs`: 2 occurrences

- Line 10: unused import: `crate::expression::functions::global_registry`
- Line 25: unused import: `crate::core::Expression`

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 18: unused import: `crate::core::Expression`
- Line 331: unused import: `ExpressionId`

#### `src\query\parser\parser\parser.rs`: 2 occurrences

- Line 7: unused imports: `ExpressionMeta` and `Expression`
- Line 120: unused variable: `cache`: help: if this is intentional, prefix it with an underscore: `_cache`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::core::Expression`

#### `src\query\planner\statements\maintain_planner.rs`: 1 occurrences

- Line 13: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\clauses\order_by_validator.rs`: 1 occurrences

- Line 17: unused import: `crate::core::Expression`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 180: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\parser\parser\traversal_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 11: unused imports: `ContextualExpression` and `ExpressionContext`

#### `src\query\parser\parser\dml_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\validator\statements\fetch_edges_validator.rs`: 1 occurrences

- Line 21: unused import: `crate::core::Expression`

#### `src\query\validator\strategies\mod.rs`: 1 occurrences

- Line 18: unused import: `agg_functions::*`

#### `src\query\planner\plan\core\nodes\plan_node_traits.rs`: 1 occurrences

- Line 8: unused import: `crate::core::Expression`

#### `src\query\validator\helpers\variable_checker.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\parser\parser\clause_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 567: unused import: `ExpressionId`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 2: unused import: `crate::core::types::expression::Expression`

#### `src\query\planner\statements\use_planner.rs`: 1 occurrences

- Line 19: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\strategies\alias_strategy.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\planner\rewrite\expression_utils.rs`: 1 occurrences

- Line 10: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 15: unused import: `crate::core::error::ValidationError`

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 1 occurrences

- Line 280: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\planner\plan\core\nodes\project_node.rs`: 1 occurrences

- Line 84: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\planner\plan\core\nodes\factory.rs`: 1 occurrences

- Line 21: unused import: `ExpressionMeta`

#### `src\query\validator\statements\go_validator.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Expression`

