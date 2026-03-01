# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 769
- **Total Warnings**: 30
- **Total Issues**: 799
- **Unique Error Patterns**: 198
- **Unique Warning Patterns**: 15
- **Files with Issues**: 86

## Error Statistics

**Total Errors**: 769

### Error Type Breakdown

- **error[E0308]**: 523 errors
- **error[E0599]**: 214 errors
- **error[E0277]**: 10 errors
- **error[E0616]**: 10 errors
- **error[E0061]**: 6 errors
- **error[E0432]**: 2 errors
- **error[E0063]**: 2 errors
- **error[E0425]**: 1 errors
- **error[E0624]**: 1 errors

### Files with Errors (Top 10)

- `src\query\validator\validator_enum.rs`: 149 errors
- `src\query\planner\template_extractor.rs`: 31 errors
- `src\api\embedded\statement.rs`: 30 errors
- `src\query\validator\strategies\expression_strategy_test.rs`: 29 errors
- `src\query\parser\ast\stmt.rs`: 28 errors
- `src\query\validator\statements\update_validator.rs`: 27 errors
- `src\query\executor\graph_query_executor.rs`: 26 errors
- `src\query\parser\ast\utils.rs`: 23 errors
- `src\query\validator\statements\delete_validator.rs`: 22 errors
- `src\query\validator\strategies\aggregate_strategy.rs`: 21 errors

## Warning Statistics

**Total Warnings**: 30

### Warning Type Breakdown

- **warning**: 30 warnings

### Files with Warnings (Top 10)

- `src\query\validator\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\strategies\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\validator_enum.rs`: 2 warnings
- `src\query\validator\clauses\yield_validator.rs`: 2 warnings
- `src\query\parser\parser\util_stmt_parser.rs`: 2 warnings
- `src\query\parser\parser\clause_parser.rs`: 1 warnings
- `src\query\parser\parser\parser.rs`: 1 warnings
- `src\query\validator\statements\merge_validator.rs`: 1 warnings
- `src\query\validator\statements\unwind_validator.rs`: 1 warnings
- `src\query\parser\parser\traversal_parser.rs`: 1 warnings

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `ContextualExpression`, found `Expression`

**Total Occurrences**: 523  
**Unique Files**: 68

#### `src\query\planner\template_extractor.rs`: 31 occurrences

- Line 316: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 333: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 395: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 28 more occurrences in this file

#### `src\api\embedded\statement.rs`: 30 occurrences

- Line 255: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 263: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 268: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 27 more occurrences in this file

#### `src\query\parser\ast\stmt.rs`: 28 occurrences

- Line 906: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 912: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 921: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 25 more occurrences in this file

#### `src\query\validator\strategies\expression_strategy_test.rs`: 27 occurrences

- Line 33: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 38: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 63: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 24 more occurrences in this file

#### `src\query\executor\graph_query_executor.rs`: 26 occurrences

- Line 317: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 336: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 340: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 23 more occurrences in this file

#### `src\query\validator\statements\update_validator.rs`: 25 occurrences

- Line 225: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 228: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 229: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 22 more occurrences in this file

#### `src\query\validator\statements\delete_validator.rs`: 22 occurrences

- Line 197: mismatched types: expected `&ContextualExpression`, found `&Arc<ExpressionMeta>`
- Line 214: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- Line 223: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- ... 19 more occurrences in this file

#### `src\query\validator\strategies\aggregate_strategy.rs`: 21 occurrences

- Line 19: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 83: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 268: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 18 more occurrences in this file

#### `src\query\validator\statements\match_validator.rs`: 17 occurrences

- Line 142: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 264: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 288: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 14 more occurrences in this file

#### `src\query\validator\strategies\expression_operations.rs`: 14 occurrences

- Line 24: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 195: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 249: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 11 more occurrences in this file

#### `src\query\planner\statements\insert_planner.rs`: 14 occurrences

- Line 98: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 146: mismatched types: expected `Vec<(Expression, Expression, ..., ...)>`, found `Vec<(..., ..., ..., ...)>`
- Line 226: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 11 more occurrences in this file

#### `src\query\validator\strategies\clause_strategy.rs`: 12 occurrences

- Line 94: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 101: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 109: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 9 more occurrences in this file

#### `src\query\validator\clauses\limit_validator.rs`: 11 occurrences

- Line 153: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 173: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 313: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 8 more occurrences in this file

#### `src\query\validator\statements\unwind_validator.rs`: 10 occurrences

- Line 67: arguments to this function are incorrect
- Line 144: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 159: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 7 more occurrences in this file

#### `src\query\validator\statements\insert_edges_validator.rs`: 10 occurrences

- Line 98: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 121: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 165: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 7 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\push_project_down.rs`: 10 occurrences

- Line 404: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 409: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 445: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 7 more occurrences in this file

#### `src\query\validator\statements\fetch_edges_validator.rs`: 9 occurrences

- Line 131: arguments to this method are incorrect
- Line 297: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 298: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 6 more occurrences in this file

#### `src\query\validator\statements\set_validator.rs`: 9 occurrences

- Line 187: arguments to this method are incorrect
- Line 190: arguments to this method are incorrect
- Line 193: arguments to this method are incorrect
- ... 6 more occurrences in this file

#### `src\query\validator\statements\go_validator.rs`: 9 occurrences

- Line 149: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 222: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 425: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- ... 6 more occurrences in this file

#### `src\query\validator\strategies\expression_strategy.rs`: 9 occurrences

- Line 33: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 48: mismatched types: expected `&ContextualExpression`, found `&Arc<ExpressionMeta>`
- Line 64: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 6 more occurrences in this file

#### `src\query\validator\clauses\with_validator.rs`: 9 occurrences

- Line 80: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 183: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- Line 184: mismatched types: expected `Arc<ExpressionMeta>`, found `Expression`
- ... 6 more occurrences in this file

#### `src\query\parser\ast\pattern.rs`: 8 occurrences

- Line 204: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 207: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 215: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 5 more occurrences in this file

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 8 occurrences

- Line 261: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 268: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 279: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 5 more occurrences in this file

#### `src\query\planner\statements\match_statement_planner.rs`: 8 occurrences

- Line 317: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 328: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 366: mismatched types: expected `Expression`, found `ContextualExpression`
- ... 5 more occurrences in this file

#### `src\query\validator\strategies\pagination_strategy.rs`: 7 occurrences

- Line 59: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 188: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 188: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 4 more occurrences in this file

#### `src\query\planner\statements\create_planner.rs`: 6 occurrences

- Line 114: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 141: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 160: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 3 more occurrences in this file

#### `src\query\validator\helpers\variable_checker.rs`: 6 occurrences

- Line 307: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 308: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 311: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\strategies\alias_strategy.rs`: 6 occurrences

- Line 38: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 48: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 57: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 6 occurrences

- Line 80: arguments to this function are incorrect
- Line 80: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 179: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\plan\core\nodes\project_node.rs`: 6 occurrences

- Line 53: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 87: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 106: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 3 more occurrences in this file

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 6 occurrences

- Line 147: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 167: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 178: arguments to this function are incorrect
- ... 3 more occurrences in this file

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 6 occurrences

- Line 307: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 308: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 311: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\statements\fetch_vertices_validator.rs`: 5 occurrences

- Line 103: mismatched types: expected `&[Expression]`, found `&Vec<ContextualExpression>`
- Line 139: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 262: mismatched types: expected `&Expression`, found `&ContextualExpression`
- ... 2 more occurrences in this file

#### `src\query\validator\clauses\order_by_validator.rs`: 5 occurrences

- Line 188: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 228: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 398: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 2 more occurrences in this file

#### `src\query\validator\statements\remove_validator.rs`: 5 occurrences

- Line 44: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 83: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 260: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 2 more occurrences in this file

#### `src\query\planner\rewrite\projection_pushdown\projection_pushdown.rs`: 4 occurrences

- Line 233: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 238: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 274: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\clauses\return_validator.rs`: 4 occurrences

- Line 77: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 179: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 206: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- ... 1 more occurrences in this file

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

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 4 occurrences

- Line 496: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 509: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 517: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\statements\insert_vertices_validator.rs`: 4 occurrences

- Line 146: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 188: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 392: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\utility\update_config_validator.rs`: 4 occurrences

- Line 80: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 256: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`
- Line 260: mismatched types: expected `Arc<ExpressionContext>`, found `ExpressionContext`
- ... 1 more occurrences in this file

#### `src\query\planner\statements\go_planner.rs`: 4 occurrences

- Line 76: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 130: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 136: mismatched types: expected `ContextualExpression`, found `Expression`
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\expression_checker.rs`: 4 occurrences

- Line 496: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 509: mismatched types: expected `&ContextualExpression`, found `&Expression`
- Line 517: mismatched types: expected `&ContextualExpression`, found `&Expression`
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\elimination\remove_append_vertices_below_join.rs`: 3 occurrences

- Line 121: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 127: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 296: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\statements\lookup_validator.rs`: 3 occurrences

- Line 128: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 139: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 387: mismatched types: expected `Option<ContextualExpression>`, found `Option<Expression>`

#### `src\query\planner\statements\clauses\yield_planner.rs`: 3 occurrences

- Line 156: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 164: mismatched types: expected `Option<Expression>`, found `Option<ContextualExpression>`
- Line 188: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\statements\fetch_edges_planner.rs`: 3 occurrences

- Line 57: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 58: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 59: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\validator\clauses\group_by_validator.rs`: 3 occurrences

- Line 100: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 157: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 247: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`

#### `src\query\validator\helpers\schema_validator.rs`: 3 occurrences

- Line 304: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 409: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 436: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 2 occurrences

- Line 97: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 154: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\statements\create_validator.rs`: 2 occurrences

- Line 479: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 517: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 2 occurrences

- Line 77: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 89: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\query_pipeline_manager.rs`: 2 occurrences

- Line 186: mismatched types: expected `&Stmt`, found `&ParserResult`
- Line 203: mismatched types: expected `Stmt`, found `ParserResult`

#### `src\query\planner\statements\group_by_planner.rs`: 2 occurrences

- Line 182: mismatched types: expected `&Expression`, found `&ContextualExpression`
- Line 204: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 2 occurrences

- Line 98: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 157: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 229: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 433: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\lookup_planner.rs`: 2 occurrences

- Line 93: mismatched types: expected `Expression`, found `ContextualExpression`
- Line 138: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\elimination\remove_noop_project.rs`: 2 occurrences

- Line 107: mismatched types: expected `ContextualExpression`, found `Expression`
- Line 113: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\validator\statements\merge_validator.rs`: 2 occurrences

- Line 192: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`
- Line 222: mismatched types: expected `&Expression`, found `&Arc<ExpressionMeta>`

#### `src\query\planner\statements\update_planner.rs`: 1 occurrences

- Line 62: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\rewrite\elimination\eliminate_append_vertices.rs`: 1 occurrences

- Line 81: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 658: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\planner\statements\use_planner.rs`: 1 occurrences

- Line 55: mismatched types: expected `ContextualExpression`, found `Expression`

#### `src\query\planner\statements\clauses\where_clause_planner.rs`: 1 occurrences

- Line 47: mismatched types: expected `Expression`, found `ContextualExpression`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 249: mismatched types: expected `ValidationResult`, found `Result<ValidationResult, ...>`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 69: mismatched types: expected `&Expression`, found `&ContextualExpression`

#### `src\query\planner\statements\maintain_planner.rs`: 1 occurrences

- Line 38: mismatched types: expected `ContextualExpression`, found `Expression`

### error[E0599]: no method named `deduce_type` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope: method not found in `Arc<ExpressionMeta>`

**Total Occurrences**: 214  
**Unique Files**: 16

#### `src\query\validator\validator_enum.rs`: 148 occurrences

- Line 190: no method named `get_type` found for reference `&ShowValidator` in the current scope: method not found in `&ShowValidator`
- Line 191: no method named `get_type` found for reference `&DescValidator` in the current scope: method not found in `&DescValidator`
- Line 192: no method named `get_type` found for reference `&ShowCreateValidator` in the current scope: method not found in `&ShowCreateValidator`
- ... 145 more occurrences in this file

#### `src\query\parser\ast\utils.rs`: 13 occurrences

- Line 33: no method named `expression` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope
- Line 34: no method named `expression` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope
- Line 44: no method named `expression` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope
- ... 10 more occurrences in this file

#### `src\query\validator\dml\set_operation_validator.rs`: 8 occurrences

- Line 59: no variant or associated item named `from_stmt` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- Line 68: no variant or associated item named `from_stmt` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- Line 196: no method named `outputs` found for reference `&Box<Validator>` in the current scope
- ... 5 more occurrences in this file

#### `src\query\validator\utility\explain_validator.rs`: 6 occurrences

- Line 62: no variant or associated item named `from_stmt` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- Line 86: no method named `statement_type` found for reference `&Box<Validator>` in the current scope
- Line 141: no method named `is_global_statement` found for reference `&Box<Validator>` in the current scope: method not found in `&Box<Validator>`
- ... 3 more occurrences in this file

#### `src\query\validator\helpers\type_checker.rs`: 5 occurrences

- Line 30: no method named `deduce_type` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope: method not found in `Arc<ExpressionMeta>`
- Line 507: no variant or associated item named `Any` found for enum `validator_trait::ValueType` in the current scope: variant or associated item not found in `ValueType`
- Line 508: no method named `to_data_type` found for reference `&validator_trait::ValueType` in the current scope: method not found in `&ValueType`
- ... 2 more occurrences in this file

#### `src\query\validator\strategies\helpers\type_checker.rs`: 5 occurrences

- Line 30: no method named `deduce_type` found for struct `std::sync::Arc<core::types::expression::expression::ExpressionMeta>` in the current scope: method not found in `Arc<ExpressionMeta>`
- Line 507: no variant or associated item named `Any` found for enum `validator_trait::ValueType` in the current scope: variant or associated item not found in `ValueType`
- Line 508: no method named `to_data_type` found for reference `&validator_trait::ValueType` in the current scope: method not found in `&ValueType`
- ... 2 more occurrences in this file

#### `src\query\validator\helpers\variable_checker.rs`: 4 occurrences

- Line 22: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 101: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 175: no method named `expression` found for reference `&def::Expression` in the current scope
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\variable_checker.rs`: 4 occurrences

- Line 22: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 101: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 175: no method named `expression` found for reference `&def::Expression` in the current scope
- ... 1 more occurrences in this file

#### `src\query\validator\utility\update_config_validator.rs`: 3 occurrences

- Line 255: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope
- Line 259: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope
- Line 264: no method named `add_expression` found for struct `core::types::expression::context::ExpressionContext` in the current scope

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 3 occurrences

- Line 19: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 334: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 391: no method named `expression` found for reference `&def::Expression` in the current scope

#### `src\query\validator\strategies\expression_operations.rs`: 3 occurrences

- Line 28: no variant or associated item named `ExpressionError` found for enum `ValidationErrorType` in the current scope: variant or associated item not found in `ValidationErrorType`
- Line 391: no variant or associated item named `ExpressionError` found for enum `ValidationErrorType` in the current scope: variant or associated item not found in `ValidationErrorType`
- Line 526: no variant or associated item named `ExpressionError` found for enum `ValidationErrorType` in the current scope: variant or associated item not found in `ValidationErrorType`

#### `src\query\validator\assignment_validator.rs`: 3 occurrences

- Line 58: no variant or associated item named `from_stmt` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- Line 113: no method named `statement_type` found for reference `&Box<Validator>` in the current scope
- Line 177: no method named `is_global_statement` found for reference `&Box<Validator>` in the current scope: method not found in `&Box<Validator>`

#### `src\query\validator\helpers\expression_checker.rs`: 3 occurrences

- Line 19: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 334: no method named `expression` found for reference `&def::Expression` in the current scope
- Line 391: no method named `expression` found for reference `&def::Expression` in the current scope

#### `src\query\validator\dml\query_validator.rs`: 3 occurrences

- Line 49: no variant or associated item named `from_stmt` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`
- Line 68: no method named `outputs` found for reference `&Box<Validator>` in the current scope
- Line 137: no method named `is_global_statement` found for reference `&Box<Validator>` in the current scope: method not found in `&Box<Validator>`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 2 occurrences

- Line 506: no method named `has_aggregate_expression` found for struct `strategies::helpers::type_checker::TypeValidator` in the current scope
- Line 511: no method named `has_aggregate_expression` found for struct `strategies::helpers::type_checker::TypeValidator` in the current scope

#### `src\query\query_pipeline_manager.rs`: 1 occurrences

- Line 303: no variant or associated item named `from_stmt` found for enum `Validator` in the current scope: variant or associated item not found in `Validator`

### error[E0277]: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ValidationResult`

**Total Occurrences**: 10  
**Unique Files**: 7

#### `src\query\validator\utility\explain_validator.rs`: 2 occurrences

- Line 117: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ValidationResult`
- Line 250: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ValidationResult`

#### `src\query\planner\statements\subgraph_planner.rs`: 2 occurrences

- Line 74: a value of type `Vec<def::Expression>` cannot be built from an iterator over elements of type `contextual::ContextualExpression`: value of type `Vec<def::Expression>` cannot be built from `std::iter::Iterator<Item=contextual::ContextualExpression>`
- Line 119: a value of type `Vec<def::Expression>` cannot be built from an iterator over elements of type `contextual::ContextualExpression`: value of type `Vec<def::Expression>` cannot be built from `std::iter::Iterator<Item=contextual::ContextualExpression>`

#### `src\query\validator\dml\set_operation_validator.rs`: 2 occurrences

- Line 230: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ValidationResult`
- Line 237: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ValidationResult`

#### `src\query\planner\statements\insert_planner.rs`: 1 occurrences

- Line 70: a value of type `Vec<(def::Expression, Vec<Vec<def::Expression>>)>` cannot be built from an iterator over elements of type `(contextual::ContextualExpression, Vec<Vec<contextual::ContextualExpression>>)`: value of type `Vec<(def::Expression, Vec<Vec<def::Expression>>)>` cannot be built from `std::iter::Iterator<Item=(contextual::ContextualExpression, Vec<Vec<contextual::ContextualExpression>>)>`

#### `src\query\validator\statements\go_validator.rs`: 1 occurrences

- Line 441: a value of type `Vec<(def::Expression, std::option::Option<std::string::String>)>` cannot be built from an iterator over elements of type `(contextual::ContextualExpression, std::option::Option<std::string::String>)`: value of type `Vec<(def::Expression, std::option::Option<std::string::String>)>` cannot be built from `std::iter::Iterator<Item=(contextual::ContextualExpression, std::option::Option<std::string::String>)>`

#### `src\query\validator\assignment_validator.rs`: 1 occurrences

- Line 144: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ValidationResult`

#### `src\query\validator\dml\query_validator.rs`: 1 occurrences

- Line 109: the `?` operator can only be applied to values that implement `Try`: the `?` operator cannot be applied to type `ValidationResult`

### error[E0616]: field `context` of struct `contextual::ContextualExpression` is private: private field

**Total Occurrences**: 10  
**Unique Files**: 1

#### `src\query\parser\ast\utils.rs`: 10 occurrences

- Line 32: field `context` of struct `contextual::ContextualExpression` is private: private field
- Line 43: field `context` of struct `contextual::ContextualExpression` is private: private field
- Line 56: field `context` of struct `contextual::ContextualExpression` is private: private field
- ... 7 more occurrences in this file

### error[E0061]: this function takes 2 arguments but 1 argument was supplied

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\validator\helpers\type_checker.rs`: 3 occurrences

- Line 597: this function takes 2 arguments but 1 argument was supplied
- Line 605: this function takes 2 arguments but 1 argument was supplied
- Line 613: this function takes 2 arguments but 1 argument was supplied

#### `src\query\validator\strategies\helpers\type_checker.rs`: 3 occurrences

- Line 597: this function takes 2 arguments but 1 argument was supplied
- Line 605: this function takes 2 arguments but 1 argument was supplied
- Line 613: this function takes 2 arguments but 1 argument was supplied

### error[E0063]: missing field `expression` in initializer of `update_validator::ValidatedAssignment`: missing `expression`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\statements\update_validator.rs`: 2 occurrences

- Line 483: missing field `expression` in initializer of `update_validator::ValidatedAssignment`: missing `expression`
- Line 515: missing field `expression` in initializer of `update_validator::ValidatedAssignment`: missing `expression`

### error[E0432]: unresolved import `crate::query::parser::parser::parse_expression_meta_from_string`: no `parse_expression_meta_from_string` in `query::parser::parser`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\executor\data_modification.rs`: 1 occurrences

- Line 11: unresolved import `crate::query::parser::parser::parse_expression_meta_from_string`: no `parse_expression_meta_from_string` in `query::parser::parser`

#### `src\query\parser\parser\mod.rs`: 1 occurrences

- Line 24: unresolved imports `parser::parse_expression_meta_from_string`, `parser::parse_expression_meta_from_string_with_cache`: no `parse_expression_meta_from_string` in `query::parser::parser::parser`, no `parse_expression_meta_from_string_with_cache` in `query::parser::parser::parser`

### error[E0425]: cannot find function `parse_expression_meta_from_string` in module `crate::query::parser`: not found in `crate::query::parser`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\executor\tag_filter.rs`: 1 occurrences

- Line 95: cannot find function `parse_expression_meta_from_string` in module `crate::query::parser`: not found in `crate::query::parser`

### error[E0624]: method `validate_group_key_type_internal` is private: private method

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 196: method `validate_group_key_type_internal` is private: private method

## Detailed Warning Categorization

### warning: unused import: `crate::core::Expression`

**Total Occurrences**: 30  
**Unique Files**: 21

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

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 18: unused import: `crate::core::Expression`
- Line 331: unused import: `ExpressionId`

#### `src\query\validator\validator_enum.rs`: 2 occurrences

- Line 15: unused import: `crate::core::error::ValidationError`
- Line 19: unused import: `ExpressionProps`

#### `src\query\parser\parser\util_stmt_parser.rs`: 2 occurrences

- Line 5: unused import: `std::sync::Arc`
- Line 8: unused import: `crate::core::types::expression::Expression as CoreExpression`

#### `src\query\validator\clauses\with_validator.rs`: 1 occurrences

- Line 8: unused import: `crate::core::Expression`

#### `src\query\validator\statements\create_validator.rs`: 1 occurrences

- Line 21: unused import: `crate::core::Expression`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 113: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 1 occurrences

- Line 7: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\validator\statements\remove_validator.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Expression`

#### `src\query\validator\statements\merge_validator.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Expression`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 2: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\statements\unwind_validator.rs`: 1 occurrences

- Line 18: unused import: `NullType`

#### `src\query\validator\clauses\return_validator.rs`: 1 occurrences

- Line 8: unused import: `crate::core::Expression`

#### `src\query\validator\helpers\expression_checker.rs`: 1 occurrences

- Line 7: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\parser\parser\traversal_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\parser\parser\clause_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\parser\parser\parser.rs`: 1 occurrences

- Line 7: unused imports: `ExpressionMeta` and `Expression`

#### `src\query\parser\parser\dml_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\validator\strategies\mod.rs`: 1 occurrences

- Line 18: unused import: `agg_functions::*`

