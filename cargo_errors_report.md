# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 2
- **Total Warnings**: 0
- **Total Issues**: 2
- **Unique Error Patterns**: 1
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 2

### Error Type Breakdown

- **error[E0046]**: 2 errors

### Files with Errors (Top 10)

- `src\query\planner\statements\paths\shortest_path_planner.rs`: 1 errors
- `src\query\planner\statements\paths\match_path_planner.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0046]: not all trait items implemented, missing: `get_space_by_id`: missing `get_space_by_id` in implementation

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 1 occurrences

- Line 69: not all trait items implemented, missing: `get_space_by_id`: missing `get_space_by_id` in implementation

#### `src\query\planner\statements\paths\match_path_planner.rs`: 1 occurrences

- Line 160: not all trait items implemented, missing: `get_space_by_id`: missing `get_space_by_id` in implementation

