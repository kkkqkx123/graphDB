# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 0
- **Total Issues**: 4
- **Unique Error Patterns**: 3
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0599]**: 3 errors
- **error[E0061]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\rules\join\join_optimization.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0599]: no associated item named `InvalidPlan` found for struct `OptimizerError` in the current scope: associated item not found in `OptimizerError`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\optimizer\rules\join\join_optimization.rs`: 3 occurrences

- Line 100: no associated item named `InvalidPlan` found for struct `OptimizerError` in the current scope: associated item not found in `OptimizerError`
- Line 114: no associated item named `InvalidPlan` found for struct `OptimizerError` in the current scope: associated item not found in `OptimizerError`
- Line 295: the method `left_input` exists for reference `&LimitNode`, but its trait bounds were not satisfied

### error[E0061]: this function takes 1 argument but 0 arguments were supplied

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\rules\join\join_optimization.rs`: 1 occurrences

- Line 358: this function takes 1 argument but 0 arguments were supplied

