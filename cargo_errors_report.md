# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 376
- **Total Warnings**: 0
- **Total Issues**: 376
- **Unique Error Patterns**: 53
- **Unique Warning Patterns**: 0
- **Files with Issues**: 60

## Error Statistics

**Total Errors**: 376

### Error Type Breakdown

- **error[E0308]**: 300 errors
- **error[E0599]**: 51 errors
- **error[E0425]**: 7 errors
- **error[E0061]**: 7 errors
- **error[E0560]**: 5 errors
- **error[E0624]**: 1 errors
- **error[E0603]**: 1 errors
- **error[E0614]**: 1 errors
- **error[E0422]**: 1 errors
- **error[E0277]**: 1 errors
- **error[E0433]**: 1 errors

### Files with Errors (Top 10)

- `src\api\embedded\statement.rs`: 32 errors
- `src\query\executor\graph_query_executor.rs`: 26 errors
- `src\query\validator\statements\delete_validator.rs`: 18 errors
- `src\query\validator\statements\update_validator.rs`: 17 errors
- `src\query\validator\strategies\aggregate_strategy.rs`: 15 errors
- `src\query\planner\statements\create_planner.rs`: 14 errors
- `src\query\validator\statements\match_validator.rs`: 12 errors
- `src\query\validator\strategies\expression_strategy_test.rs`: 12 errors
- `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 10 errors
- `src\query\validator\helpers\variable_checker.rs`: 10 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `ContextualExpression`, found `Expression`

**Total Occurrences**: 300  
**Unique Files**: 53

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

#### `src\query\validator\strategies\aggregate_strategy.rs`: 13 occurrences

- Line 381: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 397: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 440: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 10 more occurrences in this file

#### `src\query\validator\statements\update_validator.rs`: 13 occurrences

- Line 279: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 423: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 613: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 10 more occurrences in this file

#### `src\query\planner\statements\create_planner.rs`: 13 occurrences

- Line 87: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 87: mismatched types: expected `Vec<ContextualExpression>`, found `Vec<Expression>`
- Line 107: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 10 more occurrences in this file

#### `src\query\validator\statements\match_validator.rs`: 12 occurrences

- Line 257: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 288: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 388: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 9 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 10 occurrences

- Line 404: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 409: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 445: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 7 more occurrences in this file

#### `src\query\validator\clauses\limit_validator.rs`: 9 occurrences

- Line 313: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 330: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 331: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 6 more occurrences in this file

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
- Line 222: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 425: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- ... 6 more occurrences in this file

#### `src\query\planner\statements\match_statement_planner.rs`: 8 occurrences

- Line 317: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 328: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 366: mismatched types: expected `Expression`, found `ContextualExpression`
- ... 5 more occurrences in this file

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

#### `src\query\validator\helpers\variable_checker.rs`: 6 occurrences

- Line 307: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 308: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 311: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\statements\insert_edges_validator.rs`: 6 occurrences

- Line 121: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 379: mismatched types: expected `&Option<Expression>`, found `&Option<ContextualExpression>`
- Line 475: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 6 occurrences

- Line 80: arguments to this function are incorrect
- Line 80: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 179: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 6 occurrences

- Line 147: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 167: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 178: arguments to this function are incorrect
- ... 3 more occurrences in this file

#### `src\query\planner\plan\core\nodes\project_node.rs`: 6 occurrences

- Line 53: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 87: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 106: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 5 occurrences

- Line 103: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- Line 139: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 262: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 2 more occurrences in this file

#### `src\query\planner\statements\delete_planner.rs`: 4 occurrences

- Line 56: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 63: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 70: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\optimizer\strategy\traversal_start.rs`: 4 occurrences

- Line 299: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 306: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\go_planner.rs`: 4 occurrences

- Line 76: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 130: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 136: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 4 occurrences

- Line 496: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 509: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 517: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\expression_checker.rs`: 4 occurrences

- Line 496: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 509: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 517: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

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

#### `src\query\planner\statements\fetch_edges_planner.rs`: 3 occurrences

- Line 57: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 58: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 59: mismatched types: expected `&Expression`, found `&ContextualExpression`

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

#### `src\query\validator\utility\update_config_validator.rs`: 3 occurrences

- Line 256: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`
- Line 260: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`
- Line 265: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 3 occurrences

- Line 156: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 164: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 188: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 3 occurrences

- Line 121: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 127: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 296: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\group_by_planner.rs`: 2 occurrences

- Line 182: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 204: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 2 occurrences

- Line 97: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 154: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 2 occurrences

- Line 92: mismatched types: expected `Option<OrderByClauseContext>`, found `Vec<_>`
- Line 123: mismatched types: expected `HashMap<String, AliasType>`, found `HashMap<String, DataType>`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 2 occurrences

- Line 77: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 89: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 278: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 284: mismatched types: expected `&ContextualExpression`, found `&Expression`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 2 occurrences

- Line 98: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 157: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\query_pipeline_manager.rs`: 2 occurrences

- Line 191: mismatched types: expected `&Stmt`, found `&ParserResult`
- Line 208: mismatched types: expected `Stmt`, found `ParserResult`

#### `src\query\validator\statements\insert_vertices_validator.rs`: 2 occurrences

- Line 392: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 392: mismatched types: expected `Vec<Vec<ContextualExpression>>`, found `Vec<Vec<Expression>>`

#### `src\query\planner\statements\lookup_planner.rs`: 2 occurrences

- Line 93: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 138: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\elimination\remove_noop_project.rs`: 2 occurrences

- Line 107: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 113: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 647: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 69: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\rewrite\elimination\eliminate_append_vertices.rs`: 1 occurrences

- Line 81: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\statements\maintain_planner.rs`: 1 occurrences

- Line 38: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\use_planner.rs`: 1 occurrences

- Line 55: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\optimizer\cost\node_estimators\data_processing.rs`: 1 occurrences

- Line 73: mismatched types: expected `&str`, found `&ContextualExpression`

#### `src\query\planner\statements\update_planner.rs`: 1 occurrences

- Line 62: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 47: mismatched types: expected `Expression`, found `ContextualExpression`

### error[E0599]: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope

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

#### `src\query\validator\statements\update_validator.rs`: 4 occurrences

- Line 323: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 371: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 404: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\variable_checker.rs`: 4 occurrences

- Line 22: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 101: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 175: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\expression_strategy_test.rs`: 3 occurrences

- Line 102: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 111: no method named `validate_return_item` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope
- Line 161: no method named `validate_property_access` found for struct `expression_strategy::ExpressionValidationStrategy` in the current scope

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 3 occurrences

- Line 19: no method named `get_expression` found for reference `&def::Expression` in the current scope: method not found in `&Expression`
- Line 334: no method named `get_expression` found for reference `&def::Expression` in the current scope: method not found in `&Expression`
- Line 391: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`

#### `src\query\validator\validator_enum.rs`: 3 occurrences

- Line 460: no variant or associated item named `ShowSpaces` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`
- Line 461: no variant or associated item named `ShowTags` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`
- Line 462: no variant or associated item named `ShowEdges` found for enum `stmt::Stmt` in the current scope: variant or associated item not found in `stmt::Stmt`

#### `src\query\validator\utility\update_config_validator.rs`: 3 occurrences

- Line 255: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope
- Line 259: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope
- Line 264: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope

#### `src\query\validator\helpers\expression_checker.rs`: 3 occurrences

- Line 19: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 334: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`
- Line 391: no method named `inner` found for enum `def::Expression` in the current scope: method not found in `Expression`

#### `src\query\validator\strategies\alias_strategy.rs`: 3 occurrences

- Line 40: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 64: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 90: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\aggregate_strategy.rs`: 2 occurrences

- Line 22: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 86: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\statements\remove_validator.rs`: 2 occurrences

- Line 48: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 86: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 2 occurrences

- Line 25: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied
- Line 103: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 28: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\pagination_strategy.rs`: 1 occurrences

- Line 65: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\strategies\helpers\type_checker.rs`: 1 occurrences

- Line 596: no function or associated item named `default` found for struct `clause_structs::YieldClauseContext` in the current scope: function or associated item not found in `YieldClauseContext`

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 318: no method named `map_err` found for struct `ValidationResult` in the current scope: method not found in `ValidationResult`

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 280: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

#### `src\query\validator\helpers\type_checker.rs`: 1 occurrences

- Line 596: no function or associated item named `default` found for struct `clause_structs::YieldClauseContext` in the current scope: function or associated item not found in `YieldClauseContext`

#### `src\api\embedded\statement.rs`: 1 occurrences

- Line 431: the method `as_ref` exists for reference `&Expression`, but its trait bounds were not satisfied

### error[E0425]: cannot find function `parse_expression_safe` in this scope: not found in this scope

**Total Occurrences**: 7  
**Unique Files**: 1

#### `src\query\executor\factory.rs`: 7 occurrences

- Line 584: cannot find function `parse_expression_safe` in this scope: not found in this scope
- Line 587: cannot find function `parse_expression_safe` in this scope: not found in this scope
- Line 598: cannot find function `parse_expression_safe` in this scope: not found in this scope
- ... 4 more occurrences in this file

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

### error[E0560]: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field

**Total Occurrences**: 5  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 5 occurrences

- Line 90: struct `clause_structs::ReturnClauseContext` has no field named `aliases`: `clause_structs::ReturnClauseContext` does not have this field
- Line 91: struct `clause_structs::ReturnClauseContext` has no field named `return_items`: `clause_structs::ReturnClauseContext` does not have this field
- Line 93: struct `clause_structs::ReturnClauseContext` has no field named `skip`: `clause_structs::ReturnClauseContext` does not have this field
- ... 2 more occurrences in this file

### error[E0433]: failed to resolve: use of undeclared type `QueryRequestContext`: use of undeclared type `QueryRequestContext`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 99: failed to resolve: use of undeclared type `QueryRequestContext`: use of undeclared type `QueryRequestContext`

### error[E0277]: a value of type `Vec<contextual::ContextualExpression>` cannot be built from an iterator over elements of type `def::Expression`: value of type `Vec<contextual::ContextualExpression>` cannot be built from `std::iter::Iterator<Item=def::Expression>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\create_planner.rs`: 1 occurrences

- Line 342: a value of type `Vec<contextual::ContextualExpression>` cannot be built from an iterator over elements of type `def::Expression`: value of type `Vec<contextual::ContextualExpression>` cannot be built from `std::iter::Iterator<Item=def::Expression>`

### error[E0614]: type `i64` cannot be dereferenced: can't be dereferenced

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\statements\delete_validator.rs`: 1 occurrences

- Line 423: type `i64` cannot be dereferenced: can't be dereferenced

### error[E0422]: cannot find struct, variant or union type `PropertyAccessContext` in this scope: not found in this scope

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 147: cannot find struct, variant or union type `PropertyAccessContext` in this scope: not found in this scope

### error[E0603]: module `test_helpers` is private: private module

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 11: module `test_helpers` is private: private module

### error[E0624]: method `validate_group_key_type_internal` is private: private method

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 219: method `validate_group_key_type_internal` is private: private method

