# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 3
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\query\optimizer\strategy\join_order.rs`: 3 warnings

## Detailed Warning Categorization

### warning: unused variable: `hash_join_cost`: help: if this is intentional, prefix it with an underscore: `_hash_join_cost`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\optimizer\strategy\join_order.rs`: 3 occurrences

- Line 485: unused variable: `hash_join_cost`: help: if this is intentional, prefix it with an underscore: `_hash_join_cost`
- Line 488: unused variable: `nested_loop_cost`: help: if this is intentional, prefix it with an underscore: `_nested_loop_cost`
- Line 564: unused variable: `conditions`: help: if this is intentional, prefix it with an underscore: `_conditions`

