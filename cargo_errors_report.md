# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 83
- **Total Issues**: 83
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 37
- **Files with Issues**: 49

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 83

### Warning Type Breakdown

- **warning**: 83 warnings

### Files with Warnings (Top 10)

- `src\query\validator\utility\update_config_validator.rs`: 4 warnings
- `src\query\validator\strategies\helpers\type_checker.rs`: 4 warnings
- `src\query\validator\statements\match_validator.rs`: 4 warnings
- `src\query\validator\strategies\aggregate_strategy.rs`: 4 warnings
- `src\query\validator\helpers\type_checker.rs`: 4 warnings
- `src\query\planner\statements\clauses\with_clause_planner.rs`: 4 warnings
- `src\query\validator\clauses\return_validator.rs`: 3 warnings
- `src\query\validator\clauses\with_validator.rs`: 3 warnings
- `src\query\planner\plan\core\nodes\control_flow_node.rs`: 2 warnings
- `src\query\validator\helpers\expression_checker.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused import: `crate::core::Expression`

**Total Occurrences**: 83  
**Unique Files**: 49

#### `src\query\planner\statements\clauses\with_clause_planner.rs`: 4 occurrences

- Line 478: unused variable: `collection`
- Line 544: unused variable: `left`
- Line 544: unused variable: `right`
- ... 1 more occurrences in this file

#### `src\query\validator\utility\update_config_validator.rs`: 4 occurrences

- Line 9: unused import: `crate::core::types::expression::Expression`
- Line 11: unused import: `crate::core::types::expression::ExpressionContext`
- Line 12: unused import: `crate::core::Value`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\helpers\type_checker.rs`: 4 occurrences

- Line 6: unused import: `crate::core::AggregateFunction`
- Line 7: unused import: `crate::core::BinaryOperator`
- Line 8: unused import: `crate::core::UnaryOperator`
- ... 1 more occurrences in this file

#### `src\query\validator\strategies\aggregate_strategy.rs`: 4 occurrences

- Line 6: unused import: `crate::core::types::expression::ExpressionMeta`
- Line 7: unused import: `crate::core::types::expression::ExpressionContext`
- Line 8: unused import: `crate::core::types::expression::ExpressionId`
- ... 1 more occurrences in this file

#### `src\query\validator\statements\match_validator.rs`: 4 occurrences

- Line 10: unused import: `crate::expression::functions::global_registry`
- Line 25: unused import: `crate::core::Expression`
- Line 63: field `expr_context` is never read
- ... 1 more occurrences in this file

#### `src\query\validator\helpers\type_checker.rs`: 4 occurrences

- Line 6: unused import: `crate::core::AggregateFunction`
- Line 7: unused import: `crate::core::BinaryOperator`
- Line 8: unused import: `crate::core::UnaryOperator`
- ... 1 more occurrences in this file

#### `src\query\validator\clauses\with_validator.rs`: 3 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 416: unused import: `crate::core::types::expression::Expression`
- Line 133: method `validate_function_call` is never used

#### `src\query\validator\clauses\return_validator.rs`: 3 occurrences

- Line 8: unused import: `crate::core::Expression`
- Line 379: unused import: `crate::core::types::expression::Expression`
- Line 130: method `validate_function_call` is never used

#### `src\query\parser\parser\parser.rs`: 2 occurrences

- Line 7: unused imports: `ExpressionMeta` and `Expression`
- Line 120: unused variable: `cache`: help: if this is intentional, prefix it with an underscore: `_cache`

#### `src\query\planner\plan\core\nodes\control_flow_node.rs`: 2 occurrences

- Line 10: unused import: `ExpressionMeta`
- Line 11: unused import: `crate::core::Expression`

#### `src\query\validator\statements\remove_validator.rs`: 2 occurrences

- Line 9: unused import: `crate::core::Expression`
- Line 245: unused import: `crate::core::Value`

#### `src\query\validator\strategies\alias_strategy.rs`: 2 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`
- Line 85: method `validate_subexpressions_aliases` is never used

#### `src\query\validator\helpers\expression_checker.rs`: 2 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`
- Line 10: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\validator\statements\merge_validator.rs`: 2 occurrences

- Line 203: unused variable: `props`: help: if this is intentional, prefix it with an underscore: `_props`
- Line 225: unused variable: `value`: help: if this is intentional, prefix it with an underscore: `_value`

#### `src\query\parser\parser\util_stmt_parser.rs`: 2 occurrences

- Line 5: unused import: `std::sync::Arc`
- Line 8: unused import: `crate::core::types::expression::Expression as CoreExpression`

#### `src\query\validator\strategies\helpers\expression_checker.rs`: 2 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`
- Line 10: unused import: `crate::query::validator::strategies::helpers::type_checker::TypeDeduceValidator`

#### `src\query\planner\statements\use_planner.rs`: 2 occurrences

- Line 17: unused import: `crate::core::types::expression::ExpressionContext`
- Line 19: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\planner\statements\maintain_planner.rs`: 2 occurrences

- Line 11: unused import: `crate::core::types::expression::ExpressionContext`
- Line 13: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\statements\unwind_validator.rs`: 2 occurrences

- Line 17: unused import: `crate::core::Expression`
- Line 18: unused imports: `NullType` and `Value`

#### `src\query\validator\clauses\yield_validator.rs`: 2 occurrences

- Line 18: unused import: `crate::core::Expression`
- Line 331: unused import: `ExpressionId`

#### `src\query\planner\statements\clauses\order_by_planner.rs`: 1 occurrences

- Line 6: unused import: `crate::core::Expression`

#### `src\query\parser\parser\stmt_parser.rs`: 1 occurrences

- Line 6: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\project_node.rs`: 1 occurrences

- Line 84: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\strategies\expression_strategy.rs`: 1 occurrences

- Line 2: unused import: `crate::core::types::expression::Expression`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 9: unused import: `crate::core::DataType`

#### `src\query\validator\clauses\order_by_validator.rs`: 1 occurrences

- Line 17: unused import: `crate::core::Expression`

#### `src\query\validator\statements\go_validator.rs`: 1 occurrences

- Line 10: unused import: `crate::core::Expression`

#### `src\query\validator\helpers\variable_checker.rs`: 1 occurrences

- Line 7: unused import: `crate::core::types::expression::ExpressionId`

#### `src\core\types\expression\utils.rs`: 1 occurrences

- Line 567: unused import: `ExpressionId`

#### `src\query\validator\statements\create_validator.rs`: 1 occurrences

- Line 512: method `evaluate_expression` is never used

#### `src\query\parser\parser\clause_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\statements\lookup_planner.rs`: 1 occurrences

- Line 11: unused imports: `ContextualExpression` and `ExpressionContext`

#### `src\query\optimizer\strategy\traversal_start.rs`: 1 occurrences

- Line 321: unused import: `crate::core::types::BinaryOperator`

#### `src\query\planner\plan\core\nodes\plan_node_traits.rs`: 1 occurrences

- Line 8: unused import: `crate::core::Expression`

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 12: unused import: `ExpressionContext`

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 1 occurrences

- Line 180: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\planner\statements\create_planner.rs`: 1 occurrences

- Line 18: unused import: `ExpressionContext`

#### `src\query\planner\statements\delete_planner.rs`: 1 occurrences

- Line 16: unused import: `ExpressionContext`

#### `src\query\planner\rewrite\expression_utils.rs`: 1 occurrences

- Line 10: unused import: `crate::core::types::expression::ExpressionId`

#### `src\query\validator\statements\fetch_edges_validator.rs`: 1 occurrences

- Line 21: unused import: `crate::core::Expression`

#### `src\query\validator\strategies\mod.rs`: 1 occurrences

- Line 18: unused import: `agg_functions::*`

#### `src\query\validator\validator_enum.rs`: 1 occurrences

- Line 15: unused import: `crate::core::error::ValidationError`

#### `src\query\planner\statements\clauses\return_clause_planner.rs`: 1 occurrences

- Line 5: unused import: `ExpressionContext`

#### `src\query\parser\parser\dml_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\plan\core\nodes\filter_node.rs`: 1 occurrences

- Line 9: unused import: `crate::core::Expression`

#### `src\query\planner\statements\update_planner.rs`: 1 occurrences

- Line 16: unused import: `ExpressionContext`

#### `src\query\validator\statements\insert_edges_validator.rs`: 1 occurrences

- Line 142: associated function `basic_validate_vertex_id_format_internal` is never used

#### `src\query\parser\parser\traversal_parser.rs`: 1 occurrences

- Line 5: unused import: `std::sync::Arc`

#### `src\query\planner\rewrite\merge\collapse_project.rs`: 1 occurrences

- Line 280: unused import: `crate::core::types::expression::ExpressionId`

