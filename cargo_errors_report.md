# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 0
- **Total Issues**: 4
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 1

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0599]**: 4 errors

### Files with Errors (Top 10)

- `src\query\executor\data_processing\join\inner_join.rs`: 4 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0599]: no method named `execute` found for struct `inner_join::InnerJoinExecutor` in the current scope

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\query\executor\data_processing\join\inner_join.rs`: 4 occurrences

- Line 337: no method named `execute` found for struct `inner_join::InnerJoinExecutor` in the current scope
- Line 415: no method named `execute` found for struct `inner_join::InnerJoinExecutor` in the current scope
- Line 467: no method named `execute` found for struct `inner_join::InnerJoinExecutor` in the current scope
- ... 1 more occurrences in this file

