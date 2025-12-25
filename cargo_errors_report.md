# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 8
- **Total Warnings**: 0
- **Total Issues**: 8
- **Unique Error Patterns**: 5
- **Unique Warning Patterns**: 0
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 8

### Error Type Breakdown

- **error[E0308]**: 6 errors
- **error[E0119]**: 1 errors
- **error[E0382]**: 1 errors

### Files with Errors (Top 10)

- `src\query\planner\match_planning\clauses\projection_planner.rs`: 4 errors
- `src\query\optimizer\scan_optimization.rs`: 2 errors
- `src\query\context\runtime_context.rs`: 1 errors
- `src\cache\manager.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: arguments to this method are incorrect

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\planner\match_planning\clauses\projection_planner.rs`: 4 occurrences

- Line 324: arguments to this method are incorrect
- Line 340: arguments to this method are incorrect
- Line 390: arguments to this function are incorrect
- ... 1 more occurrences in this file

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 127: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`
- Line 145: mismatched types: expected `PlanNodeEnum`, found `Arc<PlanNodeEnum>`

### error[E0119]: conflicting implementations of trait `cache::traits::Cache<_, _>` for type `std::sync::Arc<adaptive::AdaptiveCache<_, _>>`: conflicting implementation for `std::sync::Arc<adaptive::AdaptiveCache<_, _>>`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\cache\manager.rs`: 1 occurrences

- Line 233: conflicting implementations of trait `cache::traits::Cache<_, _>` for type `std::sync::Arc<adaptive::AdaptiveCache<_, _>>`: conflicting implementation for `std::sync::Arc<adaptive::AdaptiveCache<_, _>>`

### error[E0382]: borrow of partially moved value: `runtime_ctx`: value borrowed here after partial move

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\context\runtime_context.rs`: 1 occurrences

- Line 448: borrow of partially moved value: `runtime_ctx`: value borrowed here after partial move

