# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 347
- **Total Warnings**: 38
- **Total Issues**: 385
- **Unique Error Patterns**: 54
- **Unique Warning Patterns**: 17
- **Files with Issues**: 70

## Error Statistics

**Total Errors**: 347

### Error Type Breakdown

- **error[E0308]**: 270 errors
- **error[E0599]**: 51 errors
- **error[E0425]**: 7 errors
- **error[E0061]**: 7 errors
- **error[E0560]**: 5 errors
- **error[E0277]**: 2 errors
- **error[E0603]**: 1 errors
- **error[E0433]**: 1 errors
- **error[E0422]**: 1 errors
- **error[E0624]**: 1 errors
- **error[E0614]**: 1 errors

### Files with Errors (Top 10)

- `src\api\embedded\statement.rs`: 32 errors
- `src\query\executor\graph_query_executor.rs`: 26 errors
- `src\query\validator\statements\delete_validator.rs`: 18 errors
- `src\query\validator\statements\update_validator.rs`: 17 errors
- `src\query\validator\strategies\aggregate_strategy.rs`: 15 errors
- `src\query\validator\statements\match_validator.rs`: 12 errors
- `src\query\validator\strategies\expression_strategy_test.rs`: 12 errors
- `src\query\planner\plan\core\nodes\join_node.rs`: 10 errors
- `src\query\validator\helpers\variable_checker.rs`: 10 errors
- `src\query\validator\statements\go_validator.rs`: 9 errors

## Warning Statistics

**Total Warnings**: 38

### Warning Type Breakdown

- **warning**: 38 warnings

### Files with Warnings (Top 10)

- `src\query\validator\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\strategies\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\clauses\with_validator.rs`: 2 warnings
- `src\query\parser\parser\util_stmt_parser.rs`: 2 warnings
- `src\query\parser\parser\parser.rs`: 2 warnings
- `src\query\validator\clauses\yield_validator.rs`: 2 warnings
- `src\query\validator\clauses\return_validator.rs`: 2 warnings
- `src\query\planner\plan\core\nodes\project_node.rs`: 2 warnings
- `src\query\validator\strategies\expression_strategy.rs`: 1 warnings
- `src\query\parser\parser\dml_parser.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `ContextualExpression`, found `Expression`

**Total Occurrences**: 270  
**Unique Files**: 47

#### `src\api\embedded\statement.rs`: 31 occurrences

- Line 456: mismatched types: expected `Expression`, found `Option<_>`
- Line 459: mismatched types: expected `Expression`, found `Option<_>`
- Line 464: mismatched types: expected `Expression`, found `Option<_>`
- ... 28 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 21 occurrences

- Line 431: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 437: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 464: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 18 more occurrences in this file

#### `src\query\validator\statements\delete_validator.rs`: 16 occurrences

- Line 197: mismatched types: expected `&ContextualExpression`, found `&Arc<ExpressionMeta>`
- Line 214: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- Line 223: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- ... 13 more occurrences in this file

#### `src\query\validator\statements\update_validator.rs`: 13 occurrences

- Line 279: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 423: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 613: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 10 more occurrences in this file

#### `src\query\validator\strategies\aggregate_strategy.rs`: 13 occurrences

- Line 381: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 397: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 440: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 10 more occurrences in this file

#### `src\query\validator\statements\match_validator.rs`: 12 occurrences

- Line 257: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 288: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 388: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 9 more occurrences in this file

#### `src\query\planner\plan\core\nodes\join_node.rs`: 10 occurrences

- Line 31: mismatched types: expected `Vec<Expression>`, found `Vec<ContextualExpression>`
- Line 32: mismatched types: expected `Vec<Expression>`, found `Vec<ContextualExpression>`
- Line 62: mismatched types: expected `Vec<Expression>`, found `Vec<ContextualExpression>`
- ... 7 more occurrences in this file

#### `src\query\validator\statements\fetch_edges_validator.rs`: 9 occurrences

- Line 131: arguments to this method are incorrect
- Line 297: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 298: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 6 more occurrences in this file

#### `src\query\validator\statements\go_validator.rs`: 9 occurrences

- Line 149: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 222: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 425: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- ... 6 more occurrences in this file

#### `src\query\validator\statements\set_validator.rs`: 9 occurrences

- Line 187: arguments to this method are incorrect
- Line 190: arguments to this method are incorrect
- Line 193: arguments to this method are incorrect
- ... 6 more occurrences in this file

#### `src\query\validator\clauses\limit_validator.rs`: 9 occurrences

- Line 313: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 330: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 331: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 6 more occurrences in this file

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 8 occurrences

- Line 261: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 268: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 279: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 5 more occurrences in this file

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

#### `src\query\validator\helpers\variable_checker.rs`: 6 occurrences

- Line 307: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 308: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 311: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 6 occurrences

- Line 147: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 167: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 178: arguments to this function are incorrect
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 6 occurrences

- Line 80: arguments to this function are incorrect
- Line 80: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 179: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 5 occurrences

- Line 103: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- Line 139: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 262: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 2 more occurrences in this file

#### `src\query\planner\statements\match_statement_planner.rs`: 5 occurrences

- Line 306: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 316: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 327: mismatched types: expected `Expression`, found `ContextualExpression`
- ... 2 more occurrences in this file

#### `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 5 occurrences

- Line 121: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 127: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 296: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 2 more occurrences in this file

#### `src\query\validator\strategies\clause_strategy.rs`: 4 occurrences

- Line 295: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 335: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 375: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 4 occurrences

- Line 233: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 238: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 274: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 4 occurrences

- Line 496: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 509: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 517: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

#### `src\query\optimizer\strategy\traversal_start.rs`: 4 occurrences

- Line 299: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 306: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\expression_checker.rs`: 4 occurrences

- Line 496: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 509: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 517: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\utility\update_config_validator.rs`: 3 occurrences

- Line 256: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`
- Line 260: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`
- Line 265: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`

#### `src\query\planner\statements\fetch_edges_planner.rs`: 3 occurrences

- Line 57: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 58: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 59: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\validator\statements\remove_validator.rs`: 3 occurrences

- Line 258: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 261: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 265: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 3 occurrences

- Line 156: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 164: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 188: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\validator\statements\lookup_validator.rs`: 3 occurrences

- Line 128: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 139: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `Option<ContextualExpression>`, found `Option<Expression>`

#### `src\query\planner\plan\core\nodes\project_node.rs`: 3 occurrences

- Line 93: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 115: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 120: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\strategies\expression_operations.rs`: 3 occurrences

- Line 197: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 251: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 252: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 2 occurrences

- Line 392: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 392: mismatched types: expected `Vec<Vec<ContextualExpression>>`, found `Vec<Vec<Expression>>`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 2 occurrences

- Line 98: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 157: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\elimination\remove_noop_project.rs`: 2 occurrences

- Line 107: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 113: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 2 occurrences

- Line 92: mismatched types: expected `Option<OrderByClauseContext>`, found `Vec<_>`
- Line 123: mismatched types: expected `HashMap<String, AliasType>`, found `HashMap<String, DataType>`

#### `src\query\planner\statements\group_by_planner.rs`: 2 occurrences

- Line 182: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 204: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 278: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 284: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 2 occurrences

- Line 97: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 154: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\query_pipeline_manager.rs`: 2 occurrences

- Line 192: mismatched types: expected `&Stmt`, found `&ParserResult`
- Line 209: mismatched types: expected `Stmt`, found `ParserResult`

#### `src\query\optimizer\cost\node_estimators\data_processing.rs`: 1 occurrences

- Line 73: mismatched types: expected `&str`, found `&ContextualExpression`

#### `src\query\planner\plan\core\nodes\factory.rs`: 1 occurrences

- Line 61: arguments to this function are incorrect

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 647: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 137: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\use_planner.rs`: 1 occurrences

- Line 55: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\elimination\eliminate_append_vertices.rs`: 1 occurrences

- Line 81: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\statements\maintain_planner.rs`: 1 occurrences

- Line 38: mismatched types: expected `ContextualExpression`, found `Expression`

### error[E0599]: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

**Total Occurrences**: 51  
**Unique Files**: 20

#### `src\query\validator\strategies\expression_strategy.rs`: 7 occurrences

- Line 34: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 69: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 102: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- ... 4 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 5 occurrences

- Line 317: no method named `get_expression` found for reference `&def::Expression` in the current scope: method not found in `&Expression`
- Line 339: no method named `get_expression` found for reference `&def::Expression` in the current scope: method not found in `&Expression`
- Line 346: no method named `get_expression` found for reference `&def::Expression` in the current scope: method not found in `&Expression`
- ... 2 more occurrences in this file

#### `src\query\validator\helpers\variable_checker.rs`: 4 occurrences

- Line 22: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 101: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 175: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\statements\update_validator.rs`: 4 occurrences

- Line 323: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 371: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 404: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 3 occurrences

- Line 19: no method named `get_expression` found for reference `&def::Expression` in the current scope: method not found in `&Expression`
- Line 334: no method named `get_expression` found for reference `&def::Expression` in the current scope: method not found in `&Expression`
- Line 391: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`

#### `src\query\validator\helpers\expression_checker.rs`: 3 occurrences

- Line 19: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 334: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 391: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`

#### `src\query\validator\validator_enum.rs`: 3 occurrences

- Line 460: no variant or associated item named `ShowSpaces` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`
- Line 461: no variant or associated item named `ShowTags` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`
- Line 462: no variant or associated item named `ShowEdges` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 3 occurrences

- Line 102: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 111: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 161: no method named `validate_property_access` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope

#### `src\query\validator\utility\update_config_validator.rs`: 3 occurrences

- Line 255: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope
- Line 259: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope
- Line 264: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope

#### `src\query\validator\strategies\alias_strategy.rs`: 3 occurrences

- Line 40: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 64: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 90: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\aggregate_strategy.rs`: 2 occurrences

- Line 22: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 86: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 2 occurrences

- Line 25: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 103: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\statements\remove_validator.rs`: 2 occurrences

- Line 48: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 86: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 28: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\pagination_strategy.rs`: 1 occurrences

- Line 65: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 319: no method named `map_err` found for struct `ValidationResult` in the current scope: method not found in `ValidationResult`

#### `src\query\validator\helpers\type_checker.rs`: 1 occurrences

- Line 596: no function or associated item named `default` found for struct `clause_structs::YieldClauseContext` in the current scope: function or associated item not found in `YieldClauseContext`

#### `src\query\validator\strategies\helpers\type_checker.rs`: 1 occurrences

- Line 596: no function or associated item named `default` found for struct `clause_structs::YieldClauseContext` in the current scope: function or associated item not found in `YieldClauseContext`

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 280: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\api\embedded\statement.rs`: 1 occurrences

- Line 431: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

### error[E0061]: this function takes 2 arguments but 1 argument was supplied

**Total Occurrences**: 7  
**Unique Files**: 3

#### `src\query\validator\strategies\helpers\type_checker.rs`: 3 occurrences

- Line 597: this function takes 2 arguments but 1 argument was supplied
- Line 605: this function takes 2 arguments but 1 argument was supplied
- Line 613: this function takes 2 arguments but 1 argument was supplied

#### `src\query\validator\helpers\type_checker.rs`: 3 occurrences

- Line 597: this function takes 2 arguments but 1 argument was supplied
- Line 605: this function takes 2 arguments but 1 argument was supplied
- Line 613: this function takes 2 arguments but 1 argument was supplied

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 354: this method takes 2 arguments but 1 argument was supplied

### error[E0425]: cannot find function `parse_expression_safe` in this scope: not found in this scope

**Total Occurrences**: 7  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 7 occurrences

- Line 584: cannot find function `parse_expression_safe` in this scope: not found in this scope
- Line 587: cannot find function `parse_expression_safe` in this scope: not found in this scope
- Line 598: cannot find function `parse_expression_safe` in this scope: not found in this scope
- ... 4 more occurrences in this file

### error[E0560]: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 5 occurrences

- Line 90: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field
- Line 91: struct `clause_structs::ReturnClauseContext` has no field named `return_items`: `clause_structs::ReturnClauseContext` does not have this field
- Line 93: struct `clause_structs::ReturnClauseContext` has no field named `skip`: `clause_structs::ReturnClauseContext` does not have this field
- ... 2 more occurrences in this file

### error[E0277]: a value of type `Vec<contextual::ContextualExpression>` cannot be built from an iterator over elements of type `def::Expression`: value of type `Vec<contextual::ContextualExpression>` cannot be built from `std::iter::Iterator<Item=def::Expression>`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 2 occurrences

- Line 318: a value of type `Vec<contextual::ContextualExpression>` cannot be built from an iterator over elements of type `def::Expression`: value of type `Vec<contextual::ContextualExpression>` cannot be built from `std::iter::Iterator<Item=def::Expression>`
- Line 328: a value of type `Vec<contextual::ContextualExpression>` cannot be built from an iterator over elements of type `def::Expression`: value of type `Vec<contextual::ContextualExpression>` cannot be built from `std::iter::Iterator<Item=def::Expression>`

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

### error[E0433]: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 101: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`

### error[E0624]: method `validate_group_key_type_internal` is private: private method

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 219: method `validate_group_key_type_internal` is private: private method

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 423: type `i64` cannot be dereferenced: can't be dereferenced

## Detailed Warning Categorization

### warning: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

**Total Occurrences**: 38  
**Unique Files**: 26

#### `src\query\validator\strategies\helpers\type_checker.rs`: 4 occurrences

- Line 6: unused import: `crate::core::AggregateFunction`
- Line 7: unused import: `crate::core::BinaryOperator`
- Line 8: unused import: `crate::core::UnaryOperator`
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\type_checker.rs`: 4 occurrences

- Line 6: unused import: `crate::core::AggregateFunction`
- Line 7: unused import: `crate::core::BinaryOperator`
- Line 8: unused import: `crate::core::UnaryOperator`
- ... 1 more occurrences in this file

#### `src\query\parser\parser\parser.rs`: 2 occurrences

- Line 7: unused imports: `ExpressionMeta` and `Expression`
- Line 120: unused variable: `cache`: help: if this is intentional, prefix it with an underscore: `_cache`

#### `src\query\planner\plan\core\nodes\project_node.rs`: 2 occurrences

- Line 9: unused imports: `ContextualExpression` and `ExpressionMeta`
- Line 10: unused import: `crate::core::Expression`

#### `src\query\parser\parser\util_stmt_parser.rs`: 2 occurrences

- Line 5: unused import: `std::sync::Arc`
- Line 8: unused import: `crate::core::types::expression::Expression as CoreExpression`

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 18: unused import: `crate::core::Expression`
- Line 331: unused import: `ExpressionId`

#### `src\query\validator\clauses\return_validator.rs`: 2 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 379: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\clauses\with_validator.rs`: 2 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 416: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\helpers\expression_checker.rs`: 1 occurrences

- Line 7: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\validator\statements\merge_validator.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Expression`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 567: unused import: `ExpressionId`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 1 occurrences

- Line 7: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 113: unused import: `crate::core::types::expression::Expression`

#### `src\query\parser\parser\clause_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\parser\parser\traversal_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\validator\statements\create_validator.rs`: 1 occurrences

- Line 21: unused import: `crate::core::Expression`

#### `src\query\parser\parser\dml_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\validator\strategies\mod.rs`: 1 occurrences

- Line 18: unused import: `agg_functions::*`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::core::Expression`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 11: unused imports: `ContextualExpression` and `ExpressionContext`

#### `src\query\validator\clauses\order_by_validator.rs`: 1 occurrences

- Line 17: unused import: `crate::core::Expression`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 15: unused import: `crate::core::error::ValidationError`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 2: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\statements\remove_validator.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Expression`

#### `src\query\validator\statements\unwind_validator.rs`: 1 occurrences

- Line 18: unused import: `NullType`

