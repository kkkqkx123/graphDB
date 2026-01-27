# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 0
- **Total Issues**: 3
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0308]**: 3 errors

### Files with Errors (Top 10)

- `src\query\planner\operation_kind_support.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `CoreOperationKind`, found `PlanNodeEnum`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\planner\operation_kind_support.rs`: 3 occurrences

- Line 93: mismatched types: expected `CoreOperationKind`, found `PlanNodeEnum`
- Line 101: mismatched types: expected `CoreOperationKind`, found `PlanNodeEnum`
- Line 109: mismatched types: expected `CoreOperationKind`, found `PlanNodeEnum`

