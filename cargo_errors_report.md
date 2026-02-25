# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 1
- **Total Warnings**: 0
- **Total Issues**: 1
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 1

### Error Type Breakdown

- **error[E0425]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\statements\match_statement_planner.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0425]: cannot find value `_space_id` in this scope: help: a local variable with a similar name exists: `space_id`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\planner\statements\match_statement_planner.rs`: 1 occurrences

- Line 641: cannot find value `_space_id` in this scope: help: a local variable with a similar name exists: `space_id`

