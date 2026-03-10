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

- **error[E0277]**: 1 errors

### Files with Errors (Top 10)

- `src\query\cache\mod.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0277]: `plan_cache::QueryPlanCache` doesn't implement `std::fmt::Debug`: `plan_cache::QueryPlanCache` cannot be formatted using `{:?}`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\cache\mod.rs`: 1 occurrences

- Line 39: `plan_cache::QueryPlanCache` doesn't implement `std::fmt::Debug`: `plan_cache::QueryPlanCache` cannot be formatted using `{:?}`

