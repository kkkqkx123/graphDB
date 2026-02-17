# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0063]**: 2 errors

### Files with Errors (Top 10)

- `src\query\validator\strategies\expression_strategy_test.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0063]: missing fields `filter_condition`, `limit` and `skip` in initializer of `clause_structs::YieldClauseContext`: missing `filter_condition`, `limit` and `skip`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\validator\strategies\expression_strategy_test.rs`: 2 occurrences

- Line 434: missing fields `filter_condition`, `limit` and `skip` in initializer of `clause_structs::YieldClauseContext`: missing `filter_condition`, `limit` and `skip`
- Line 503: missing fields `filter_condition`, `limit` and `skip` in initializer of `clause_structs::YieldClauseContext`: missing `filter_condition`, `limit` and `skip`

