# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 4
- **Total Warnings**: 0
- **Total Issues**: 4
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 4

### Error Type Breakdown

- **error[E0407]**: 4 errors

### Files with Errors (Top 10)

- `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 errors
- `src\query\planner\statements\paths\match_path_planner.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0407]: method `backup` is not a member of trait `StorageClient`: not a member of trait `StorageClient`

**Total Occurrences**: 4  
**Unique Files**: 2

#### `src\query\planner\statements\paths\shortest_path_planner.rs`: 2 occurrences

- Line 561: method `backup` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- Line 565: method `restore` is not a member of trait `StorageClient`: not a member of trait `StorageClient`

#### `src\query\planner\statements\paths\match_path_planner.rs`: 2 occurrences

- Line 647: method `backup` is not a member of trait `StorageClient`: not a member of trait `StorageClient`
- Line 651: method `restore` is not a member of trait `StorageClient`: not a member of trait `StorageClient`

