# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 14
- **Total Warnings**: 0
- **Total Issues**: 14
- **Unique Error Patterns**: 6
- **Unique Warning Patterns**: 0
- **Files with Issues**: 3

## Error Statistics

**Total Errors**: 14

### Error Type Breakdown

- **error[E0277]**: 14 errors

### Files with Errors (Top 10)

- `src\query\validator\strategies\expression_strategy.rs`: 7 errors
- `src\query\validator\strategies\expression_strategy_test.rs`: 5 errors
- `src\query\validator\strategies\type_inference.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0277]: the trait bound `common_structs::ValidationContextImpl: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `common_structs::ValidationContextImpl`

**Total Occurrences**: 14  
**Unique Files**: 3

#### `src\query\validator\strategies\expression_strategy.rs`: 7 occurrences

- Line 30: the trait bound `clause_structs::WhereClauseContext: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `clause_structs::WhereClauseContext`
- Line 58: the trait bound `clause_structs::MatchClauseContext: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `clause_structs::MatchClauseContext`
- Line 84: the trait bound `clause_structs::ReturnClauseContext: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `clause_structs::ReturnClauseContext`
- ... 4 more occurrences in this file

#### `src\query\validator\strategies\expression_strategy_test.rs`: 5 occurrences

- Line 424: the trait bound `common_structs::ValidationContextImpl: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `common_structs::ValidationContextImpl`
- Line 427: the trait bound `common_structs::ValidationContextImpl: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `common_structs::ValidationContextImpl`
- Line 462: the trait bound `clause_structs::YieldClauseContext: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `clause_structs::YieldClauseContext`
- ... 2 more occurrences in this file

#### `src\query\validator\strategies\type_inference.rs`: 2 occurrences

- Line 930: the trait bound `common_structs::ValidationContextImpl: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `common_structs::ValidationContextImpl`
- Line 933: the trait bound `common_structs::ValidationContextImpl: ExpressionValidationContext` is not satisfied: the trait `ExpressionValidationContext` is not implemented for `common_structs::ValidationContextImpl`

