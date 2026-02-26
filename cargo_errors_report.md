# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 13
- **Total Warnings**: 0
- **Total Issues**: 13
- **Unique Error Patterns**: 6
- **Unique Warning Patterns**: 0
- **Files with Issues**: 6

## Error Statistics

**Total Errors**: 13

### Error Type Breakdown

- **error[E0433]**: 13 errors

### Files with Errors (Top 10)

- `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 4 errors
- `src\query\planner\rewrite\merge\merge_get_nbrs_and_dedup.rs`: 2 errors
- `src\query\planner\rewrite\merge\merge_get_vertices_and_dedup.rs`: 2 errors
- `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 2 errors
- `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 2 errors
- `src\query\validator\strategies\aggregate_strategy.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0433]: failed to resolve: use of undeclared type `ProjectNode`: use of undeclared type `ProjectNode`

**Total Occurrences**: 13  
**Unique Files**: 6

#### `src\query\planner\rewrite\merge\collapse_consecutive_project.rs`: 4 occurrences

- Line 178: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 183: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- Line 193: failed to resolve: use of undeclared type `Expression`: use of undeclared type `Expression`
- ... 1 more occurrences in this file

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_project.rs`: 2 occurrences

- Line 159: failed to resolve: use of undeclared type `ProjectNode`: use of undeclared type `ProjectNode`
- Line 163: failed to resolve: use of undeclared type `GetNeighborsNode`: use of undeclared type `GetNeighborsNode`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_project.rs`: 2 occurrences

- Line 156: failed to resolve: use of undeclared type `ProjectNode`: use of undeclared type `ProjectNode`
- Line 160: failed to resolve: use of undeclared type `GetVerticesNode`: use of undeclared type `GetVerticesNode`

#### `src\query\planner\rewrite\merge\merge_get_nbrs_and_dedup.rs`: 2 occurrences

- Line 145: failed to resolve: use of undeclared type `DedupNode`: use of undeclared type `DedupNode`
- Line 149: failed to resolve: use of undeclared type `GetNeighborsNode`: use of undeclared type `GetNeighborsNode`

#### `src\query\planner\rewrite\merge\merge_get_vertices_and_dedup.rs`: 2 occurrences

- Line 145: failed to resolve: use of undeclared type `DedupNode`: use of undeclared type `DedupNode`
- Line 149: failed to resolve: use of undeclared type `GetVerticesNode`: use of undeclared type `GetVerticesNode`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 576: failed to resolve: use of undeclared type `DataType`: use of undeclared type `DataType`

