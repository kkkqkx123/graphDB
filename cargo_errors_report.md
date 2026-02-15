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

- **error[E0412]**: 2 errors

### Files with Errors (Top 10)

- `src\query\optimizer\plan_node_visitor.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0412]: cannot find type `FullOuterJoinNode` in this scope: not found in this scope

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\plan_node_visitor.rs`: 2 occurrences

- Line 42: cannot find type `FullOuterJoinNode` in this scope: not found in this scope
- Line 220: cannot find type `FullOuterJoinNode` in this scope: not found in this scope

